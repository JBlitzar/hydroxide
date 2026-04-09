import init, { WasmRenderer, initThreadPool } from "./pkg/hydroxide.js";

function isSafari(ua, vendor) {
  if (!vendor || !vendor.includes("Apple")) return false;
  if (!ua.includes("Safari")) return false;
  return !/(Chrome|Chromium|Edg|OPR|CriOS|FxiOS)/.test(ua);
}

function isMobileDevice() {
  const ua = navigator.userAgent || "";
  const uad = navigator.userAgentData;
  if (uad && typeof uad.mobile === "boolean") return uad.mobile;
  return /(Android|iPhone|iPad|iPod)/i.test(ua);
}

function blockUnsupported() {
  const ua = navigator.userAgent || "";
  const vendor = navigator.vendor || "";
  const blocked = isMobileDevice() || isSafari(ua, vendor);
  if (!blocked) return false;

  const gh = "https://github.com/jblitzar/hydroxide";
  const info = document.getElementById("info");
  if (info) {
    info.innerHTML = `Unsupported on mobile/Safari. Please use Chrome/Firefox on desktop. <a href="${gh}" target="_blank" rel="noreferrer">[Github]</a>`;
  }

  const hints = document.querySelector(".i2");
  if (hints) hints.style.display = "none";
  const sky = document.getElementById("sky-select");
  if (sky) sky.style.display = "none";
  const add = document.getElementById("add-object");
  if (add) add.style.display = "none";
  const panel = document.getElementById("panel");
  if (panel) panel.style.display = "none";
  return true;
}

if (blockUnsupported()) {
  // Stop before loading WASM/workers.
  throw new Error("Unsupported browser");
}

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");
const info = document.getElementById("info");
ctx.imageSmoothingEnabled = false;

let azimuth = 0;
let elevation = 0.1;
let distance = 8.0;
const target = { x: 0, y: 0.7, z: -7 };

let isDragging = false;
let dragMoved = false;
let lastX = 0,
  lastY = 0;
const FOV = Math.PI / 2;
const FOCUS_DISTANCE = 10.0;
const APERTURE = 0.04;
let focusDistance = FOCUS_DISTANCE;

const PREVIEW_SCALE = 0.125;
const OUTLINE_SCALE = 0.125;
const MAX_RENDER_W = 1920;
const MAX_RENDER_H = 1080;
const PASSES = [
  [0.125, 2, 0.1],
  [0.25, 4, 0.1],
  [0.5, 16, 0.05],
  [1.0, 16, 0.05],
  [1.0, 100, 0.01],
  [1.0, 500, 0.01],
];

let mainRenderer = null;
let selectedIndex = -1;
let cachedOutlineMask = null;
let cachedOutlineW = 0;
let cachedOutlineH = 0;

let qualityWorker = null;
let workerReady = false;
let workerBusy = false;
let currentToken = 0;
let currentPass = 0;
let mutationLog = [];
const workerUrl = new URL("./render-worker.js", import.meta.url);

let spareWorker = null;
let spareReady = false;

const skySelect = document.getElementById("sky-select");
const panel = document.getElementById("panel");
const panelContent = document.getElementById("panel-content");
const objType = document.getElementById("obj-type");
const posX = document.getElementById("pos-x");
const posY = document.getElementById("pos-y");
const posZ = document.getElementById("pos-z");
const objRadius = document.getElementById("obj-radius");
const objSize = document.getElementById("obj-size");
const paramRadius = document.getElementById("param-radius");
const paramSize = document.getElementById("param-size");
const matType = document.getElementById("mat-type");
const matColor = document.getElementById("mat-color");
const matFuzz = document.getElementById("mat-fuzz");
const matRI = document.getElementById("mat-ri");
const fieldFuzz = document.getElementById("field-fuzz");
const fieldRI = document.getElementById("field-ri");
const btnApply = document.getElementById("btn-apply");
const btnDelete = document.getElementById("btn-delete");
const addObjectSelect = document.getElementById("add-object");

function renderBaseSize() {
  return {
    w: Math.min(canvas.width, MAX_RENDER_W),
    h: Math.min(canvas.height, MAX_RENDER_H),
  };
}

function cameraFromOrbit() {
  const x = target.x + distance * Math.cos(elevation) * Math.sin(azimuth);
  const y = target.y + distance * Math.sin(elevation);
  const z = target.z + distance * Math.cos(elevation) * Math.cos(azimuth);
  return { x, y, z };
}

function cameraParams() {
  const cam = cameraFromOrbit();
  return {
    fov: FOV,
    cam_x: cam.x,
    cam_y: cam.y,
    cam_z: cam.z,
    target_x: target.x,
    target_y: target.y,
    target_z: target.z,
    focus_distance: focusDistance,
    aperture: APERTURE,
  };
}

function updateAutofocus(w, h) {
  if (!mainRenderer) return;
  const cam = cameraFromOrbit();
  const midX = Math.floor(w * 0.5);
  const midY = Math.floor(h * 0.5);
  let t = -1;
  try {
    t = mainRenderer.pick_distance(
      midX,
      midY,
      w,
      h,
      FOV,
      cam.x,
      cam.y,
      cam.z,
      target.x,
      target.y,
      target.z,
    );
  } catch (_) {
    return;
  }
  if (!(t > 0) || !Number.isFinite(t)) return;
  if (Math.abs(t - focusDistance) < 0.01) return;
  focusDistance = t;
}

function safeComputeOutline() {
  cachedOutlineMask = null;
  if (selectedIndex < 0 || !mainRenderer) return;
  try {
    const base = renderBaseSize();
    const ow = Math.max(1, Math.floor(base.w * OUTLINE_SCALE));
    const oh = Math.max(1, Math.floor(base.h * OUTLINE_SCALE));
    const cp = cameraParams();
    const mask = mainRenderer.outline(
      selectedIndex,
      ow,
      oh,
      cp.fov,
      cp.cam_x,
      cp.cam_y,
      cp.cam_z,
      cp.target_x,
      cp.target_y,
      cp.target_z,
      cp.focus_distance,
      cp.aperture,
      2,
    );
    if (mask && mask.length > 0) {
      cachedOutlineMask = mask;
      cachedOutlineW = ow;
      cachedOutlineH = oh;
    }
  } catch (_) {}
}

function drawOutlineOverlay() {
  if (!cachedOutlineMask) return;
  const ow = cachedOutlineW;
  const oh = cachedOutlineH;
  const outImg = new ImageData(ow, oh);
  const od = outImg.data;
  for (let i = 0; i < cachedOutlineMask.length; i++) {
    if (cachedOutlineMask[i] > 0) {
      const p = i * 4;
      od[p] = 255;
      od[p + 1] = 165;
      od[p + 2] = 0;
      od[p + 3] = 255;
    }
  }
  const offscreen = new OffscreenCanvas(ow, oh);
  const octx = offscreen.getContext("2d");
  octx.putImageData(outImg, 0, 0);
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(offscreen, 0, 0, canvas.width, canvas.height);
}

function displayFrame(rgba, w, h, label) {
  const imgData = new ImageData(new Uint8ClampedArray(rgba), w, h);
  if (w < canvas.width || h < canvas.height) {
    const offscreen = new OffscreenCanvas(w, h);
    const octx = offscreen.getContext("2d");
    octx.putImageData(imgData, 0, 0);
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(offscreen, 0, 0, canvas.width, canvas.height);
  } else {
    ctx.putImageData(imgData, 0, 0);
  }
  if (selectedIndex >= 0) drawOutlineOverlay();
  info.innerHTML =
    "<a href='https://github.com/jblitzar/hydroxide' target='_blank'>[Github]</a> | " +
    label;
}

function renderPreview() {
  if (!mainRenderer) return;
  const base = renderBaseSize();
  const w = Math.max(1, Math.floor(base.w * PREVIEW_SCALE));
  const h = Math.max(1, Math.floor(base.h * PREVIEW_SCALE));
  const cam = cameraFromOrbit();
  updateAutofocus(w, h);

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
    focusDistance,
    APERTURE,
  );
  const dt = performance.now() - t0;
  displayFrame(rgba.buffer, w, h, `${w}x${h} | ${dt.toFixed(0)}ms | preview`);
}

function sendRender() {
  if (!workerReady || isDragging) return;
  const [scale, samples, termProb] = PASSES[currentPass];
  const base = renderBaseSize();
  const w = Math.max(1, Math.floor(base.w * scale));
  const h = Math.max(1, Math.floor(base.h * scale));

  updateAutofocus(w, h);

  const cam = cameraFromOrbit();
  workerBusy = true;
  qualityWorker.postMessage({
    type: "render",
    token: currentToken,
    pass: currentPass,
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
      focus_distance: focusDistance,
      aperture: APERTURE,
    },
  });
}

function kick() {
  currentPass = 0;
  if (workerBusy) {
    spawnWorker();
    return;
  }
  sendRender();
}

function onWorkerMessage(e) {
  const msg = e.data;
  if (msg.type === "ready") {
    workerReady = true;
    if (!isDragging) kick();
    return;
  }
  if (msg.type !== "frame") return;

  workerBusy = false;

  if (msg.token !== currentToken) {
    if (!isDragging) kick();
    return;
  }

  displayFrame(
    msg.rgba,
    msg.width,
    msg.height,
    `${msg.width}x${msg.height} | ${msg.dt.toFixed(0)}ms | pass ${msg.pass + 1}/${PASSES.length}`,
  );
  currentPass = (msg.pass + 1) % PASSES.length;
  if (!isDragging) sendRender();
}

function sendWorkerMessage(msg) {
  mutationLog.push(msg);
  if (qualityWorker) qualityWorker.postMessage(msg);
  if (spareWorker) spareWorker.postMessage(msg);
}

function invalidateAndKick() {
  currentToken++;
  renderPreview();
  kick();
}

function fullRerender() {
  currentToken++;
  renderPreview();
  kick();
}

function warmSpare() {
  if (spareWorker) spareWorker.terminate();
  spareReady = false;
  spareWorker = new Worker(workerUrl, { type: "module" });
  spareWorker.onmessage = (e) => {
    if (e.data.type === "ready") spareReady = true;
  };
  for (const msg of mutationLog) spareWorker.postMessage(msg);
}

function spawnWorker() {
  if (qualityWorker) qualityWorker.terminate();
  if (spareReady && spareWorker) {
    qualityWorker = spareWorker;
    workerReady = true;
    workerBusy = false;
    qualityWorker.onmessage = onWorkerMessage;
    spareWorker = null;
    spareReady = false;
    warmSpare();
    if (!isDragging) kick();
  } else {
    workerReady = false;
    workerBusy = false;
    qualityWorker = new Worker(workerUrl, { type: "module" });
    qualityWorker.onmessage = onWorkerMessage;
    for (const msg of mutationLog) qualityWorker.postMessage(msg);
  }
}

const HDR_SKIES = [
  { name: "Citrus Orchard", url: "res/citrus_orchard_road_puresky_4k.hdr" },
  { name: "Qwantani Moonrise", url: "res/qwantani_moonrise_puresky_4k.hdr" },
];
const hdrCache = new Map();

function populateSkys() {
  if (!mainRenderer) return;
  skySelect.innerHTML = "";
  const count = mainRenderer.sky_count();
  for (let i = 0; i < count; i++) {
    const opt = document.createElement("option");
    opt.value = `builtin:${i}`;
    opt.textContent = mainRenderer.sky_name(i);
    skySelect.appendChild(opt);
  }
  for (const hdr of HDR_SKIES) {
    const opt = document.createElement("option");
    opt.value = `hdr:${hdr.url}`;
    opt.textContent = hdr.name;
    skySelect.appendChild(opt);
  }
}

async function loadHdrSky(url) {
  if (hdrCache.has(url)) return hdrCache.get(url);
  info.textContent = "Loading HDR...";
  const resp = await fetch(url);
  const buf = new Uint8Array(await resp.arrayBuffer());
  hdrCache.set(url, buf);
  return buf;
}

skySelect.addEventListener("change", async () => {
  if (!mainRenderer) return;
  const val = skySelect.value;
  if (val.startsWith("builtin:")) {
    const idx = parseInt(val.split(":")[1]);
    mainRenderer.set_sky(idx);
    sendWorkerMessage({ type: "set_sky", index: idx });
    fullRerender();
  } else if (val.startsWith("hdr:")) {
    const url = val.split(":").slice(1).join(":");
    const bytes = await loadHdrSky(url);
    mainRenderer.set_sky_hdr_bytes(bytes);
    sendWorkerMessage({ type: "set_sky_hdr_bytes", bytes });
    fullRerender();
  }
});

function selectObject(index) {
  selectedIndex = index;
  cachedOutlineMask = null;
  if (index >= 0) {
    panel.classList.add("open");
    panelContent.classList.add("active");
    safeComputeOutline();
  } else {
    panel.classList.remove("open");
    panelContent.classList.remove("active");
  }
}

function deselect() {
  selectObject(-1);
  fullRerender();
}

objType.addEventListener("change", () => {
  const isSphere = objType.value === "sphere";
  paramRadius.style.display = isSphere ? "block" : "none";
  paramSize.style.display = isSphere ? "none" : "block";
});

matType.addEventListener("change", () => {
  const v = parseInt(matType.value);
  fieldFuzz.classList.toggle("visible", v === 1);
  fieldRI.classList.toggle("visible", v === 2);
});

function hexToRgb(hex) {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) / 255, ((n >> 8) & 0xff) / 255, (n & 0xff) / 255];
}

function rgbToHex(r, g, b) {
  const c = (v) =>
    Math.round(Math.min(1, Math.max(0, v)) * 255)
      .toString(16)
      .padStart(2, "0");
  return "#" + c(r) + c(g) + c(b);
}

let selectedObjType = 0;

btnApply.addEventListener("click", () => {
  if (selectedIndex < 0 || !mainRenderer) return;
  const [r, g, b] = hexToRgb(matColor.value);
  const mt = parseInt(matType.value);
  const fuzz = parseFloat(matFuzz.value) || 0;
  const ri = parseFloat(matRI.value) || 1.5;
  const x = parseFloat(posX.value) || 0;
  const y = parseFloat(posY.value) || 0;
  const z = parseFloat(posZ.value) || 0;

  if (selectedObjType === 0) {
    const radius = parseFloat(objRadius.value) || 0.7;
    mainRenderer.update_sphere(
      selectedIndex,
      x,
      y,
      z,
      radius,
      mt,
      r,
      g,
      b,
      fuzz,
      ri,
    );
    sendWorkerMessage({
      type: "update_sphere",
      index: selectedIndex,
      x,
      y,
      z,
      radius,
      mat_type: mt,
      r,
      g,
      b,
      fuzz,
      ri,
    });
  } else if (selectedObjType === 1) {
    const size = parseFloat(objSize.value) || 1.0;
    mainRenderer.update_cube(
      selectedIndex,
      x,
      y,
      z,
      size,
      mt,
      r,
      g,
      b,
      fuzz,
      ri,
    );
    sendWorkerMessage({
      type: "update_cube",
      index: selectedIndex,
      x,
      y,
      z,
      size,
      mat_type: mt,
      r,
      g,
      b,
      fuzz,
      ri,
    });
  } else {
    mainRenderer.update_mesh_material(selectedIndex, mt, r, g, b, fuzz, ri);
    sendWorkerMessage({
      type: "update_mesh_material",
      index: selectedIndex,
      mat_type: mt,
      r,
      g,
      b,
      fuzz,
      ri,
    });
  }
  safeComputeOutline();
  fullRerender();
});

btnDelete.addEventListener("click", () => {
  if (selectedIndex < 0 || !mainRenderer) return;
  mainRenderer.remove_object(selectedIndex);
  sendWorkerMessage({ type: "remove_object", index: selectedIndex });
  deselect();
});

addObjectSelect.addEventListener("change", () => {
  if (!mainRenderer) return;
  const val = addObjectSelect.value;
  addObjectSelect.selectedIndex = 0;
  let idx;
  if (val === "sphere") {
    let x = Math.random() * 6 - 3;
    let y = 0.7 + Math.random() * 0.3 - 0.15;
    let z = -5 + Math.random() * 6 - 3;
    idx = mainRenderer.add_sphere(x, y, z, y, 0, 0.5, 0.5, 0.8, 0, 1.5);
    sendWorkerMessage({
      type: "add_sphere",
      x: x,
      y: y,
      z: z,
      radius: y,
      mat_type: 0,
      r: 0.5,
      g: 0.5,
      b: 0.8,
      fuzz: 0,
      ri: 1.5,
    });
  } else if (val === "cube") {
    let x = Math.random() * 6 - 3;
    let y = 1.0;
    let z = -5 + Math.random() * 6 - 3;
    idx = mainRenderer.add_cube(x, y, z, 1.0, 0, 0.5, 0.5, 0.8, 0, 1.5);
    sendWorkerMessage({
      type: "add_cube",
      x: x,
      y: y,
      z: z,
      size: 1.0,
      mat_type: 0,
      r: 0.5,
      g: 0.5,
      b: 0.8,
      fuzz: 0,
      ri: 1.5,
    });
  } else {
    return;
  }
  selectedIndex = idx;
  panel.classList.add("open");
  panelContent.classList.add("active");
  loadObjectToPanel(idx);
  fullRerender();
  safeComputeOutline();
});

function loadObjectToPanel(index) {
  if (!mainRenderer) return;
  const nfo = mainRenderer.get_object_info(index);
  if (!nfo || nfo.length < 11) return;

  selectedObjType = nfo[0];
  const isMesh = selectedObjType === 2;
  const isSphere = selectedObjType === 0;

  posX.value = nfo[1].toFixed(2);
  posY.value = nfo[2].toFixed(2);
  posZ.value = nfo[3].toFixed(2);

  if (isSphere) {
    objType.value = "sphere";
    objRadius.value = nfo[4].toFixed(2);
    paramRadius.style.display = "block";
    paramSize.style.display = "none";
  } else if (isMesh) {
    objType.value = "mesh";
    paramRadius.style.display = "none";
    paramSize.style.display = "none";
  } else {
    objType.value = "cube";
    objSize.value = nfo[4].toFixed(2);
    paramRadius.style.display = "none";
    paramSize.style.display = "block";
  }

  objType.disabled = isMesh;
  posX.disabled = isMesh;
  posY.disabled = isMesh;
  posZ.disabled = isMesh;
  if (!isMesh) {
    objType.disabled = false;
    posX.disabled = false;
    posY.disabled = false;
    posZ.disabled = false;
  }

  matType.value = String(Math.round(nfo[5]));
  matColor.value = rgbToHex(nfo[6], nfo[7], nfo[8]);
  matFuzz.value = nfo[9];
  matRI.value = nfo[10].toFixed(2);

  const mt = Math.round(nfo[5]);
  fieldFuzz.classList.toggle("visible", mt === 1);
  fieldRI.classList.toggle("visible", mt === 2);
}

function resize() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  if (selectedIndex >= 0) safeComputeOutline();
  fullRerender();
}
window.addEventListener("resize", resize);

canvas.addEventListener("mousedown", (e) => {
  isDragging = true;
  dragMoved = false;
  lastX = e.clientX;
  lastY = e.clientY;
  currentToken++;
});

window.addEventListener("mousemove", (e) => {
  if (!isDragging) return;
  const dx = e.clientX - lastX;
  const dy = e.clientY - lastY;
  if (Math.abs(dx) > 2 || Math.abs(dy) > 2) dragMoved = true;
  lastX = e.clientX;
  lastY = e.clientY;
  azimuth -= dx * 0.005;
  elevation = Math.max(
    -Math.PI / 2 + 0.01,
    Math.min(Math.PI / 2 - 0.01, elevation + dy * 0.005),
  );
  currentToken++;
  cachedOutlineMask = null;
  renderPreview();
});

window.addEventListener("mouseup", (e) => {
  if (!isDragging) return;
  if (!dragMoved && mainRenderer) {
    const cp = cameraParams();
    const hit = mainRenderer.pick(
      e.clientX,
      e.clientY,
      canvas.width,
      canvas.height,
      cp.fov,
      cp.cam_x,
      cp.cam_y,
      cp.cam_z,
      cp.target_x,
      cp.target_y,
      cp.target_z,
      cp.focus_distance,
      cp.aperture,
    );
    if (hit >= 0) {
      selectedIndex = hit;
      panel.classList.add("open");
      panelContent.classList.add("active");
      loadObjectToPanel(hit);
    } else {
      deselect();
    }
  }
  isDragging = false;
  kick();
  if (selectedIndex >= 0) safeComputeOutline();
});

canvas.addEventListener(
  "wheel",
  (e) => {
    e.preventDefault();
    distance = Math.max(1, distance + e.deltaY * 0.01);
    if (selectedIndex >= 0) safeComputeOutline();
    fullRerender();
  },
  { passive: false },
);

window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") deselect();
});

async function main() {
  await init();
  mainRenderer = new WasmRenderer();
  info.textContent = "Ready";
  populateSkys();
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  renderPreview();
  spawnWorker();
  warmSpare();
}

main();
