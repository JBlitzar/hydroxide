import init, { WasmRenderer, initThreadPool } from "./pkg/oxide.js";

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");
const info = document.getElementById("info");

let azimuth = 0;
let elevation = 0.1;
let distance = 6.0;
const target = { x: 0, y: 0.7, z: -7 };

let isDragging = false;
let lastX = 0,
  lastY = 0;
const FOV = Math.PI / 2;

const PREVIEW_SCALE = 0.125;
const PASSES = [
  [0.125, 2, 0.1],
  [0.25, 4, 0.1],
  [0.5, 16, 0.05],
  [1.0, 16, 0.05],
  [1.0, 100, 0.01],
  [1.0, 500, 0.01],
];

let mainRenderer = null;

let qualityWorker = null;
let qualityWorkerReady = false;
let qualityRenderId = 0;
let progressiveToken = 0;
const pendingRenders = new Map();
const workerUrl = new URL("./render-worker.js", import.meta.url);

function cameraFromOrbit() {
  const x = target.x + distance * Math.cos(elevation) * Math.sin(azimuth);
  const y = target.y + distance * Math.sin(elevation);
  const z = target.z + distance * Math.cos(elevation) * Math.cos(azimuth);
  return { x, y, z };
}

function displayFrame(rgba, w, h, label) {
  const imgData = new ImageData(new Uint8ClampedArray(rgba), w, h);
  if (w < canvas.width || h < canvas.height) {
    const offscreen = new OffscreenCanvas(w, h);
    const octx = offscreen.getContext("2d");
    octx.putImageData(imgData, 0, 0);
    ctx.imageSmoothingEnabled = true;
    ctx.drawImage(offscreen, 0, 0, canvas.width, canvas.height);
  } else {
    ctx.putImageData(imgData, 0, 0);
  }
  info.innerHTML =
    "<a href='https://github.com/jblitzar/oxide' target='_blank'>[Github]</a> | " +
    label;
}

function renderPreview() {
  if (!mainRenderer) return;
  const w = Math.max(1, Math.floor(canvas.width * PREVIEW_SCALE));
  const h = Math.max(1, Math.floor(canvas.height * PREVIEW_SCALE));
  const cam = cameraFromOrbit();
  const t0 = performance.now();
  const rgba = mainRenderer.render(
    w,
    h,
    FOV,
    cam.x,
    cam.y,
    cam.z,
    target.x,
    target.y,
    target.z,
    1,
    0.3,
  );
  const dt = performance.now() - t0;
  displayFrame(rgba.buffer, w, h, `${w}x${h} | ${dt.toFixed(0)}ms | preview`);
}

function sendQualityRender(passIndex, token) {
  if (!qualityWorkerReady || token !== progressiveToken) return;
  const [scale, samples, termProb] = PASSES[passIndex];
  const w = Math.max(1, Math.floor(canvas.width * scale));
  const h = Math.max(1, Math.floor(canvas.height * scale));
  const cam = cameraFromOrbit();
  const id = ++qualityRenderId;

  pendingRenders.set(id, { passIndex, token });

  qualityWorker.postMessage({
    type: "render",
    id,
    params: {
      w,
      h,
      fov: FOV,
      cam_x: cam.x,
      cam_y: cam.y,
      cam_z: cam.z,
      target_x: target.x,
      target_y: target.y,
      target_z: target.z,
      samples,
      termProb,
    },
  });
}

function onQualityFrame(e) {
  const { type, id, width, height, dt, rgba } = e.data;
  if (type === "ready") {
    qualityWorkerReady = true;

    sendQualityRender(0, progressiveToken);
    return;
  }
  if (type !== "frame") return;

  const pass = pendingRenders.get(id);
  pendingRenders.delete(id);

  if (!pass || pass.token !== progressiveToken) return;

  displayFrame(
    rgba,
    width,
    height,
    `${width}x${height} | ${dt.toFixed(0)}ms | pass ${pass.passIndex + 1}/${PASSES.length}`,
  );

  if (pass.passIndex + 1 < PASSES.length) {
    requestAnimationFrame(() => {
      if (pass.token === progressiveToken) {
        sendQualityRender(pass.passIndex + 1, pass.token);
      }
    });
  }
}

function spawnWorker() {
  if (qualityWorker) qualityWorker.terminate();
  qualityWorkerReady = false;
  pendingRenders.clear();
  qualityWorker = new Worker(workerUrl, { type: "module" });
  qualityWorker.onmessage = onQualityFrame;
}

function startProgressive() {
  const token = ++progressiveToken;
  pendingRenders.clear();
  spawnWorker();
}

function onCameraChange() {
  progressiveToken++;
  pendingRenders.clear();
  renderPreview();
}

let refineTimeout = null;
function scheduleRefine() {
  clearTimeout(refineTimeout);
  refineTimeout = setTimeout(() => startProgressive(), 50);
}

function resize() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  onCameraChange();
  scheduleRefine();
}
window.addEventListener("resize", resize);

canvas.addEventListener("mousedown", (e) => {
  isDragging = true;
  lastX = e.clientX;
  lastY = e.clientY;
});

window.addEventListener("mousemove", (e) => {
  if (!isDragging) return;
  const dx = e.clientX - lastX;
  const dy = e.clientY - lastY;
  lastX = e.clientX;
  lastY = e.clientY;
  azimuth -= dx * 0.005;
  elevation = Math.max(
    -Math.PI / 2 + 0.01,
    Math.min(Math.PI / 2 - 0.01, elevation + dy * 0.005),
  );
  onCameraChange();
});

window.addEventListener("mouseup", () => {
  if (isDragging) {
    isDragging = false;
    scheduleRefine();
  }
});

canvas.addEventListener(
  "wheel",
  (e) => {
    e.preventDefault();
    distance = Math.max(1, distance + e.deltaY * 0.01);
    onCameraChange();
    scheduleRefine();
  },
  { passive: false },
);

async function main() {
  await init();
  mainRenderer = new WasmRenderer();
  info.textContent = "Ready";

  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  renderPreview();

  spawnWorker();
}

main();
