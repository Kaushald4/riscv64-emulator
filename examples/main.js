const term = new Terminal({ cols: 80, rows: 30, fontSize: 14, cursorBlink: true });
const fitAddon = new FitAddon.FitAddon();
term.loadAddon(fitAddon);
term.open(document.getElementById("terminal"));
fitAddon.fit();
term.write("\x1b[2J\x1b[3J");

setTimeout(() => {
  const ta = document.querySelector(".xterm-helper-textarea");
  if (ta) { ta.focus(); ta.addEventListener("blur", () => setTimeout(() => ta.focus(), 50)); }
}, 100);

const worker = new Worker("worker.js");
const statusEl = document.getElementById("status");
const bootBtn = document.getElementById("boot");

worker.onmessage = (e) => {
  const msg = e.data;
  switch (msg.type) {
    case "ready":
      statusEl.textContent = "Running";
      break;
    case "uart":
      term.write(msg.data);
      break;
    case "trapped":
      statusEl.textContent = "Trapped";
      break;
  }
};

bootBtn.addEventListener("click", async () => {
  bootBtn.disabled = true;
  statusEl.textContent = "Loading...";

  const wasmResp = await fetch("glasshart_emulator.wasm");
  const wasmBuffer = await wasmResp.arrayBuffer();

  const [fwBuf, kBuf, dtbBuf, rfsBuf] = await Promise.all(
    ["fw_jump.bin", "Image", "virt.dtb", "rootfs.img"].map(n => fetch(n).then(r => r.arrayBuffer()))
  );

  term.reset();
  statusEl.textContent = "Booting...";
  worker.postMessage({
    type: "boot",
    wasm: wasmBuffer,
    images: { fw: new Uint8Array(fwBuf), kernel: new Uint8Array(kBuf), dtb: new Uint8Array(dtbBuf), rootfs: new Uint8Array(rfsBuf) },
  });
});

// send keys instantly to the worker
term.onData((data) => worker.postMessage({ type: "input", data }));

window.addEventListener("resize", () => fitAddon.fit());