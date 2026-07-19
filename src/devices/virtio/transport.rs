use crate::{cpu::memory::Memory, trap::Trap};

use super::descriptor::VirtqDesc;
use super::device::{VirtIODevice, VirtioContext};
use super::queue::VirtQueue;

const RAM_BASE: u64 = 0x8000_0000;

/// Bit 0 of the avail-ring flags field. when the guest sets this, it is
/// telling us "don't bother interrupting me for now" (e.g. it's already
/// busy polling the used ring). we must respect it per the virtio spec.
const VRING_AVAIL_F_NO_INTERRUPT: u16 = 0x0001;

pub fn write_used_ring(mem: &mut Memory, queue: &VirtQueue, desc_head: u16, len: u32) -> Result<(), Trap> {
    let used_ring = queue.used_ring;
    let idx = mem.read16(used_ring - RAM_BASE + 2)?;

    let elem_addr = used_ring + 4 + (idx as u64 % queue.size as u64) * 8;
    mem.write32(elem_addr - RAM_BASE, desc_head as u32)?;
    mem.write32(elem_addr + 4 - RAM_BASE, len)?;

    mem.write16(used_ring - RAM_BASE + 2, idx.wrapping_add(1))?;
    Ok(())
}


pub fn collect_chain(mem: &Memory, queue: &VirtQueue, head_desc: VirtqDesc) -> Result<Vec<VirtqDesc>, Trap> {
    let mut chain = Vec::with_capacity(8);

    if head_desc.flags & 0x04 != 0 {
        let indirect_table = head_desc.addr;
        let mut next_idx = 0u16;

        // fix - prevent infinite loops and out-of-bounds reads in indirect tables.
        // a descriptor is 16 bytes, so length / 16 gives us the max valid index.
        let max_descriptors = (head_desc.len / 16) as u16;
        let mut visited = 0u16;

        loop {
            // safety - Ensure we don't read past the table length or get stuck in a cyclic loop
            if visited >= max_descriptors || next_idx >= max_descriptors {
                break;
            }

            let d = VirtqDesc::read(mem, indirect_table + (next_idx as u64 * 16))?;
            let has_next = (d.flags & 0x01) != 0;
            chain.push(d);

            if !has_next {
                break;
            }
            next_idx = d.next;
            visited += 1;
        }
        return Ok(chain);
    }

    let mut current = head_desc;
    let mut visited: u16 = 0;

    loop {
        chain.push(current);

        if current.flags & 1 == 0 {
            break;
        }

        visited += 1;
        if visited >= queue.size {
            break;
        }

        let next_addr = queue.desc_table + (current.next as u64 * 16);
        current = VirtqDesc::read(mem, next_addr)?;
    }

    Ok(chain)
}

/// decide whether we should fire an interrupt after processing a batch,
/// per virtio 1.1 spec section 2.7 ("Notification restrictions").
///
/// two independent gates — both must be satisfied:
///
/// 1. `VRING_AVAIL_F_NO_INTERRUPT` (bit 0 of the avail-ring flags): if the
///    guest sets this, it asked us not to interrupt. Cheap to check, always
///    available.
///
/// 2. `used_event` (only valid if `VIRTIO_RING_F_EVENT_IDX` was negotiated):
///    the guest publishes a 16-bit index at the end of the avail ring
///    (`avail_ring + 4 + 2 * queue.size`) telling us "don't interrupt until
///    the used ring index reaches this value." We compare against the
///    used-ring index we're about to publish (i.e. the new value after
///    writing all our used entries this batch).
///
/// returning `false` here skips the PLIC poke entirely — no context switch,
/// no guest interrupt handler, no wasted cycles. For a burst of N packets
/// this turns N interrupts into 0-1.
pub fn should_interrupt(mem: &Memory, queue: &VirtQueue, next_used_idx: u16) -> Result<bool, Trap> {
    // Gate 1: NO_INTERRUPT flag. Cheap, always checked.
    let flags = mem.read16(queue.avail_ring - RAM_BASE)?;
    if (flags & VRING_AVAIL_F_NO_INTERRUPT) != 0 {
        return Ok(false);
    }

    // gate 2: used_event (EVENT_IDX). Only valid if the guest actually
    // negotiated the feature. We don't track per-queue feature bits here,
    // so we rely on the queue's `event_idx_enabled` flag (set by the MMIO
    // layer when the guest writes DRIVER_FEATURES). If it wasn't
    // negotiated, the field is uninitialized garbage and we MUST NOT read
    // it — fall through to "always interrupt".
    if !queue.event_idx_enabled {
        return Ok(true);
    }

    // used_event lives at avail_ring + 4 + 2 * queue_size.
    let used_event_addr = queue.avail_ring + 4 + (queue.size as u64 * 2);
    let used_event = mem.read16(used_event_addr - RAM_BASE)?;

    // per spec: interrupt only if `next_used_idx == used_event` (wrapping).
    // the guest tells us the exact index it wants to be woken at.
    Ok(next_used_idx == used_event)
}

pub fn drain_queue<D: VirtIODevice>(device: &mut D, ctx: &mut VirtioContext, queue: &mut VirtQueue, queue_idx: u16) -> Result<bool, Trap> {
    let mut processed = 0u16;
    // capture the used-ring index BEFORE we start writing, so we can compare
    // the final index against the guest's used_event.
    let start_used_idx = mem_read_used_idx(ctx.memory, queue)?;
    let mut last_used_idx = start_used_idx;

    loop {
        let Some((desc_head, head_desc)) = queue.pop_descriptor(ctx.memory)? else {
            break;
        };

        let chain = collect_chain(ctx.memory, queue, head_desc)?;
        let written_len = device.process_descriptor_chain(ctx, &chain, queue_idx)?;

        // fix - virtio Spec dictates that TX queues (Queue 1) MUST report 0 bytes written.
        // if we report > 0, we corrupt the guest Linux kernel's TX completion ring.
        let report_len: u32 = if queue_idx == 1 { 0 } else { written_len };

        write_used_ring(ctx.memory, queue, desc_head, report_len)?;
        last_used_idx = last_used_idx.wrapping_add(1);
        processed += 1;
    }

    if processed == 0 {
        return Ok(false);
    }

    // single interrupt decision for the whole batch — not one per descriptor.
    // this is the "batching" the user asked for, and it's already in place;
    // the new part is the spec-compliant gating via `should_interrupt`.
    if should_interrupt(ctx.memory, queue, last_used_idx)? {
        *ctx.interrupt_status |= 0x1;
        ctx.plic.trigger_interrupt(ctx.irq);
    }

    Ok(true)
}

#[inline]
fn mem_read_used_idx(mem: &Memory, queue: &VirtQueue) -> Result<u16, Trap> {
    mem.read16(queue.used_ring - RAM_BASE + 2)
}
