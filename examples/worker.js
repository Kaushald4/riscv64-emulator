const fastYieldChannel = new MessageChannel();
fastYieldChannel.port1.onmessage = runEmulatorLoop;

let mem = null;
let exports = null;
let inputQueue = "";
let isRunning = false;

// trackers for preemptive waking
let isSleeping = false;
let sleepTimer = null;

self.onmessage = (e) => {
  const msg = e.data;

  switch (msg.type) {
    case "boot":
      handleBoot(msg);
      break;
      
    case "input":
      inputQueue += msg.data;
      
      // If the emulator is asleep and resting the CPU, wake it up INSTANTLY!
      // This eliminates the 4-10ms browser lag that makes typing feel sluggish.
      if (isSleeping) {
        clearTimeout(sleepTimer);
        isSleeping = false;
        runEmulatorLoop(); 
      }
      break;
  }
};

function runEmulatorLoop() {
  if (!isRunning) return;

  if (inputQueue) {
    const data = inputQueue;
    inputQueue = "";
    for (let i = 0; i < data.length; i++) {
      exports.glasshart_uart_write(data.charCodeAt(i));
    }
  }

  // 500k cycles. It is large enough to boot blazing fast, 
  // but small enough that if a key is pressed mid-batch, the latency is < 1ms.
  const status = exports.glasshart_run(500_000);

  // drain UART
  let out = "";
  for (let i = 0; i < 4096; i++) {
    const v = exports.glasshart_uart_read();
    if (v === 0) break;
    out += String.fromCharCode(v & 0xFF);
  }

  // send output to the screen
  if (out) {
    self.postMessage({ type: "uart", data: out });
  }

  // 5. smart yielding logic
  if (status === -1) {
    self.postMessage({ type: "trapped" });
    return;
  }

  if (status === 0) {
    // 0 = asleep. linux is idle.
    isSleeping = true;
    sleepTimer = setTimeout(runEmulatorLoop, 4);
  } else {
    // 1 = wwake. linux is working hard.
    isSleeping = false;
    // 0ms delay yield!
    fastYieldChannel.port2.postMessage(null); 
  }
}

async function handleBoot(msg) {
  const importObj = {
    env: {
      __webrtc_mac(ptr) {
        new Uint8Array(mem.buffer, ptr, 6).set([0x52, 0x54, 0x00, 0x12, 0x34, 0x56]);
      },
      __webrtc_send(_ptr, _len) { return 0; },
      __webrtc_recv(_ptr, _maxLen) { return 0; },
    },
  };

  const { instance } = await WebAssembly.instantiate(msg.wasm, importObj);
  exports = instance.exports;
  mem = instance.exports.memory;

  const blobs = msg.images;
  const alloc = exports.glasshart_alloc;
  const copy = (data) => {
    const ptr = alloc(data.length);
    new Uint8Array(mem.buffer, ptr, data.length).set(data);
    return ptr;
  };

  exports.glasshart_boot(
    copy(blobs.fw), blobs.fw.length,
    copy(blobs.kernel), blobs.kernel.length,
    copy(blobs.dtb), blobs.dtb.length,
    copy(blobs.rootfs), blobs.rootfs.length,
  );

  self.postMessage({ type: "ready" });
  isRunning = true;
  runEmulatorLoop();
}