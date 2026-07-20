/**
 * generic receive Offload (GRO) for virtio-net.
 * 
 * GRO coalesces multiple small TCP segments into one large segment before
 * delivering them to the guest. This is the single most important
 * optimization for network throughput in an emulator: instead of
 * delivering 32 separate 1500-byte frames (each triggering a virtio
 * interrupt + TCP softirq + descriptor ring operation), GRO delivers
 * ONE ~44KB segment with a GSO header. The guest processes it as a
 * single unit, cutting per-byte CPU overhead by up to 32x.
 * 
 * without GRO, a guest running in an
 * emulator maxes out at ~200 pps × 1500 bytes = 300 KB/s because the
 * CPU can't process interrupts fast enough. with GRO, the same CPU can
 * deliver 32× more data per interrupt, approaching the emulator's raw
 * instruction-throughput ceiling.
 * 
 * the coalescing rules (from the Linux GRO implementation):
 *  1. segments must be from the same TCP flow (same 5-tuple).
 *  2. segments must be consecutive in sequence number.
 *  3. TCP flags must be compatible (no FIN/RST/SYN in the middle).
 *  4. total coalesced size must not exceed MAX_GSO_SIZE.
 * 
 * the output segment carries a virtio-net header with:
 *  gso_type  = VIRTIO_NET_HDR_GSO_TCPV4
 *  hdr_len   = sizeof(ethhdr) + ip_hdr_len + tcp_hdr_len
 *  gso_size  = MSS (original segment payload size)
 * flags     = VIRTIO_NET_HDR_F_DATA_VALID (checksum is valid)
 */


/// maximum number of frames to coalesce into one GSO segment. 32 is the
/// same limit Linux's GRO uses — beyond this, diminishing returns.
pub const MAX_GRO_SEGMENTS: usize = 32;

/// maximum size of a coalesced segment. 16KB is the safe limit for the
/// guest's virtio-net descriptor chain: the guest provides ~4 descriptors
/// of 4KB each (one page per descriptor). 16KB fits exactly.
///
/// with the proxy, the guest's RTT drops from 100ms to 3ms, so ACKs come
/// back 33x faster. The proxy sends data much faster, and GRO coalesces
/// up to 47 frames into 64KB — which exceeds the descriptor chain and
/// gets silently truncated, causing "error getting response". 16KB
/// prevents this.
///
/// 16KB × 11 frames per segment still gives 11x interrupt reduction.
pub const MAX_GSO_SIZE: usize = 16384;

/// Minimum frame size for a TCP segment (eth + ip + tcp = 14+20+20 = 54).
const MIN_TCP_FRAME: usize = 54;

/// A parsed TCP segment, extracted from a raw Ethernet frame.
#[derive(Clone, Copy, Debug)]
struct TcpSegment {
    /// TCP sequence number (host byte order).
    seq: u32,
    /// Length of the TCP payload (not including headers).
    payload_len: usize,
    /// Total header length: ethernet + IP + TCP.
    header_len: usize,
    /// IP header length (typically 20, can be more with options).
    ip_hdr_len: usize,
    /// TCP header length (typically 20, can be more with options).
    tcp_hdr_len: usize,
    /// TCP flags (offset 14 + ip_hdr_len + 13 in the frame).
    flags: u8,
}

/// The result of GRO coalescing — either a coalesced GSO segment or a
/// plain frame that couldn't be coalesced.
pub struct GroOutput {
    /// The raw Ethernet frame (coalesced or not).
    pub frame: Vec<u8>,
    /// GSO metadata for the virtio-net header, if the segment was coalesced.
    pub gso: Option<GsoMeta>,
}

/// Metadata needed to set the virtio-net header for a GSO segment.
#[derive(Clone, Copy, Debug)]
pub struct GsoMeta {
    /// Total header length (eth + IP + TCP). The guest uses this to
    /// know where the payload starts when splitting the GSO segment
    /// back into MSS-sized chunks.
    pub hdr_len: u16,
    /// The original segment's payload size (MSS). The guest uses this
    /// to know how to split the coalesced payload.
    pub mss: u16,
    /// Offset from packet start (after virtio-net header) to the TCP
    /// header. This is `eth_hdr_len + ip_hdr_len` = typically 14 + 20 = 34.
    /// The guest's virtio-net driver uses this as `csum_start` when
    /// computing the TCP checksum for each split segment.
    pub csum_start: u16,
}

/// Parse a raw Ethernet frame as a TCP segment. Returns `None` if the
/// frame is not IPv4 TCP or is too short.
fn parse_tcp(frame: &[u8]) -> Option<TcpSegment> {
    if frame.len() < MIN_TCP_FRAME {
        return None;
    }

    // Ethernet: check IPv4 ethertype (0x0800, big-endian at offset 12).
    if frame[12] != 0x08 || frame[13] != 0x00 {
        return None;
    }

    // IP header: version + IHL at offset 14. IHL is in 32-bit words.
    let ip_hdr_len = ((frame[14] & 0x0F) as usize) * 4;
    if ip_hdr_len < 20 {
        return None;
    }
    let ip_end = 14 + ip_hdr_len;
    if ip_end + 20 > frame.len() {
        return None;
    }

    // Check protocol = TCP (6).
    let protocol = frame[14 + 9];
    if protocol != 6 {
        return None;
    }

    // TCP header: data offset is the high nibble of byte 12 of the TCP
    // header (offset 14 + ip_hdr_len + 12), in 32-bit words.
    let tcp_start = ip_end;
    let tcp_hdr_len = ((frame[tcp_start + 12] >> 4) as usize) * 4;
    if tcp_hdr_len < 20 {
        return None;
    }

    let header_len = 14 + ip_hdr_len + tcp_hdr_len;
    if header_len > frame.len() {
        return None;
    }

    // TCP sequence number: bytes 4-7 of the TCP header (big-endian).
    let seq = u32::from_be_bytes([
        frame[tcp_start + 4],
        frame[tcp_start + 5],
        frame[tcp_start + 6],
        frame[tcp_start + 7],
    ]);

    // TCP flags: byte 13 of the TCP header.
    let flags = frame[tcp_start + 13];

    let payload_len = frame.len() - header_len;

    Some(TcpSegment {
        seq,
        payload_len,
        header_len,
        ip_hdr_len,
        tcp_hdr_len,
        flags,
    })
}

/// Compare the 5-tuple (src_ip + dst_ip + src_port + dst_port) of two
/// frames to check if they belong to the same TCP flow. We compare the
/// raw bytes from the IP header through the TCP ports — 12 bytes total
/// (4+4+2+2). This is faster than parsing individual fields.
///
/// The 5-tuple bytes in the frame are:
///   offset 14 + 12: src IP (4 bytes)
///   offset 14 + 16: dst IP (4 bytes)
///   offset 14 + ip_hdr_len + 0: src port (2 bytes)
///   offset 14 + ip_hdr_len + 2: dst port (2 bytes)
fn same_flow(a: &[u8], b: &[u8]) -> bool {
    // Both must be IPv4 TCP with the same IP header length.
    if a.len() < MIN_TCP_FRAME || b.len() < MIN_TCP_FRAME {
        return false;
    }
    if a[12..14] != b[12..14] {
        return false; // different ethertype
    }
    let a_ip_hdr_len = ((a[14] & 0x0F) as usize) * 4;
    let b_ip_hdr_len = ((b[14] & 0x0F) as usize) * 4;
    if a_ip_hdr_len != b_ip_hdr_len {
        return false;
    }

    // Compare source + dest IP (8 bytes at offset 14+12).
    let ip_end = 14 + a_ip_hdr_len;
    if a.len() < ip_end + 4 || b.len() < ip_end + 4 {
        return false;
    }
    if a[14 + 12..14 + 20] != b[14 + 12..14 + 20] {
        return false; // different IPs
    }

    // Compare source + dest port (4 bytes at TCP header start).
    if a.len() < ip_end + 4 || b.len() < ip_end + 4 {
        return false;
    }
    a[ip_end..ip_end + 4] == b[ip_end..ip_end + 4]
}

/// TCP flags that prevent coalescing. If any of these are set, the
/// segment is a "boundary" and must be delivered separately.
// These flag constants are used by `coalescable_flags` below. Some
// (PSH, URG, ACK) are intentionally not checked because Linux's GRO
// coalesces through them — they don't break a TCP data stream.
#[allow(dead_code)]
const TCP_PSH: u8 = 0x08;
#[allow(dead_code)]
const TCP_URG: u8 = 0x20;
#[allow(dead_code)]
const TCP_ACK: u8 = 0x10;
const TCP_FIN: u8 = 0x01;
const TCP_SYN: u8 = 0x02;
const TCP_RST: u8 = 0x04;

/// Check if a segment's TCP flags allow it to be coalesced with the
/// previous segment. SYN/FIN/RST are always boundaries. PSH is a boundary
/// (it signals the sender's app wants the data pushed, but Linux's GRO
/// actually coalesces through PSH, so we allow it too — the guest's TCP
/// stack will handle the push correctly when splitting the GSO segment).
fn coalescable_flags(flags: u8) -> bool {
    // SYN, FIN, RST terminate a coalescing run.
    (flags & (TCP_SYN | TCP_FIN | TCP_RST)) == 0
}

/// Coalesce a list of raw Ethernet frames into GSO segments.
///
/// The algorithm:
///   1. Parse each frame as a TCP segment (skip non-TCP frames).
///   2. Walk the list, accumulating consecutive segments that belong
///      to the same flow and have consecutive sequence numbers.
///   3. When a segment can't be coalesced (different flow, non-
///      consecutive seq, flags boundary, or max size reached), flush
///      the current coalescing run and start a new one.
///   4. Return the list of segments (some coalesced, some not).
///
/// Non-TCP frames (ARP, UDP, ICMP) are passed through unmodified —
/// they can't be coalesced and are delivered as-is.
pub fn gro_coalesce(frames: &[Vec<u8>]) -> Vec<GroOutput> {
    if frames.is_empty() {
        return Vec::new();
    }

    // Fast path: if only one frame, no coalescing possible.
    if frames.len() == 1 {
        return vec![GroOutput {
            frame: frames[0].clone(),
            gso: None,
        }];
    }

    let mut output: Vec<GroOutput> = Vec::with_capacity(frames.len());

    // Current coalescing run state.
    let mut run: Vec<usize> = Vec::with_capacity(MAX_GRO_SEGMENTS); // indices into `frames`
    let mut run_size: usize = 0;
    let mut run_mss: u16 = 0; // MSS of the first segment in the run
    let mut run_hdr_len: u16 = 0; // total header length
    let mut run_csum_start: u16 = 0; // csum_start for the virtio-net header
    let mut run_seq: u32 = 0; // expected next sequence number
    let mut run_tcp_info: Option<TcpSegment> = None; // info for the first segment

    /// Flush the current coalescing run into `output`.
    /// If the run has only one segment, output it plain (no GSO).
    /// If the run has multiple segments, coalesce them into one GSO segment.
    fn flush(
        run: &mut Vec<usize>,
        frames: &[Vec<u8>],
        run_size: &mut usize,
        run_mss: &mut u16,
        run_hdr_len: &mut u16,
        run_csum_start: &mut u16,
        run_tcp_info: &mut Option<TcpSegment>,
        output: &mut Vec<GroOutput>,
    ) {
        if run.is_empty() {
            return;
        }
        if run.len() == 1 {
            // Single segment — no coalescing needed.
            output.push(GroOutput {
                frame: frames[run[0]].clone(),
                gso: None,
            });
        } else {
            // Coalesce all segments in the run into one GSO segment.
            let first = &frames[run[0]];
            let info = run_tcp_info.unwrap();
            let header_len = info.header_len;

            // Build the coalesced frame: copy the first frame's headers,
            // then append each segment's payload.
            let total_len = header_len + *run_size;
            let mut coalesced = Vec::with_capacity(total_len);

            // Headers from the first segment.
            coalesced.extend_from_slice(&first[..header_len]);

            // Payloads from all segments (in sequence order — they're
            // already in order because we checked consecutive seqs).
            for &idx in run.iter() {
                let frame = &frames[idx];
                coalesced.extend_from_slice(&frame[header_len..]);
            }

            // Update IP total length field (offset 14+2, big-endian).
            // The IP total length is the size of the IP packet (IP header
            // + IP payload), NOT including the 14-byte Ethernet header.
            // Including the Ethernet header was THE bug that made the
            // Rust GRO produce corrupt packets (89KB/17h) — the guest's
            // IP stack read 14 bytes past the frame boundary.
            let ip_total_len = (info.ip_hdr_len + info.tcp_hdr_len + *run_size) as u16;
            coalesced[16] = (ip_total_len >> 8) as u8;
            coalesced[17] = (ip_total_len & 0xFF) as u8;

            // Recalculate IP header checksum (offset 14+10, 2 bytes).
            coalesced[24] = 0;
            coalesced[25] = 0;
            let ip_hdr_len = info.ip_hdr_len;
            let mut sum: u32 = 0;
            for i in (0..ip_hdr_len).step_by(2) {
                sum += ((coalesced[14 + i] as u32) << 8) | (coalesced[14 + i + 1] as u32);
            }
            while sum >> 16 != 0 {
                sum = (sum & 0xFFFF) + (sum >> 16);
            }
            let csum = !(sum as u16);
            coalesced[24] = (csum >> 8) as u8;
            coalesced[25] = (csum & 0xFF) as u8;

            // Note: we do NOT recalculate the TCP checksum here. The
            // virtio-net header uses VIRTIO_NET_HDR_F_NEEDS_CSUM, which
            // tells the guest to compute the checksum itself for each
            // split segment. The stale checksum in the coalesced frame
            // is ignored — the guest overwrites it during GSO splitting.

            output.push(GroOutput {
                frame: coalesced,
                gso: Some(GsoMeta {
                    hdr_len: *run_hdr_len,
                    mss: *run_mss,
                    csum_start: *run_csum_start,
                }),
            });
        }
        run.clear();
        *run_size = 0;
        *run_mss = 0;
        *run_hdr_len = 0;
        *run_csum_start = 0;
        *run_tcp_info = None;
    }

    for (i, frame) in frames.iter().enumerate() {
        match parse_tcp(frame) {
            Some(info) => {
                // Pure ACKs and other zero-payload segments (window
                // updates, keep-alives) MUST NOT be coalesced — they
                // have no data, so mss would be 0, and the guest's
                // virtio-net driver rejects "bad gso: size: 0". Treat
                // them as boundaries: flush the current run and pass
                // them through individually.
                if info.payload_len == 0 {
                    flush(
                        &mut run,
                        frames,
                        &mut run_size,
                        &mut run_mss,
                        &mut run_hdr_len,
                        &mut run_csum_start,
                        &mut run_tcp_info,
                        &mut output,
                    );
                    output.push(GroOutput {
                        frame: frame.clone(),
                        gso: None,
                    });
                    continue;
                }

                // Check if we can append to the current run.
                let can_append = if run_tcp_info.is_some() {
                    // Must be same flow.
                    if !same_flow(frame, &frames[run[0]]) {
                        false
                    }
                    // Must have consecutive sequence number.
                    else if run_seq != info.seq {
                        false
                    }
                    // Flags must allow coalescing.
                    else if !coalescable_flags(info.flags) {
                        false
                    }
                    // Must not exceed max GSO size.
                    else if run_size + info.payload_len > MAX_GSO_SIZE {
                        false
                    }
                    // Must not exceed max segment count.
                    else if run.len() >= MAX_GRO_SEGMENTS {
                        false
                    } else {
                        true
                    }
                } else {
                    false
                };

                if can_append {
                    // Append to current run.
                    run.push(i);
                    run_size += info.payload_len;
                    // Advance expected sequence number.
                    run_seq = info.seq.wrapping_add(info.payload_len as u32);
                } else {
                    // Flush the current run and start a new one.
                    flush(
                        &mut run,
                        frames,
                        &mut run_size,
                        &mut run_mss,
                        &mut run_hdr_len,
                        &mut run_csum_start,
                        &mut run_tcp_info,
                        &mut output,
                    );

                    // Start new run with this segment.
                    run.push(i);
                    run_size = info.payload_len;
                    run_mss = info.payload_len as u16;
                    run_hdr_len = info.header_len as u16;
                    run_csum_start = (14 + info.ip_hdr_len) as u16;
                    run_seq = info.seq.wrapping_add(info.payload_len as u32);
                    run_tcp_info = Some(info);

                    // If this segment has boundary flags (SYN/FIN/RST),
                    // flush immediately so it's delivered alone.
                    if !coalescable_flags(info.flags) {
                        flush(
                            &mut run,
                            frames,
                            &mut run_size,
                            &mut run_mss,
                            &mut run_hdr_len,
                            &mut run_csum_start,
                            &mut run_tcp_info,
                            &mut output,
                        );
                    }
                }
            }
            None => {
                // Non-TCP frame — flush the current run and pass it through.
                flush(
                    &mut run,
                    frames,
                    &mut run_size,
                    &mut run_mss,
                    &mut run_hdr_len,
                    &mut run_csum_start,
                    &mut run_tcp_info,
                    &mut output,
                );
                output.push(GroOutput {
                    frame: frame.clone(),
                    gso: None,
                });
            }
        }
    }

    // Flush any remaining run.
    flush(
        &mut run,
        frames,
        &mut run_size,
        &mut run_mss,
        &mut run_hdr_len,
        &mut run_csum_start,
        &mut run_tcp_info,
        &mut output,
    );

    output
}
