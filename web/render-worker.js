import init, { WasmRenderer, initThreadPool } from "./pkg/hydroxide.js";

let renderer = null;
let pendingMutations = [];
let wasmReady = false;

async function setup() {
  await init();
  await initThreadPool(Math.max(1, navigator.hardwareConcurrency - 2));
  renderer = new WasmRenderer();
  wasmReady = true;
  postMessage({ type: "ready" });
}

function applyMutation(msg) {
  if (!renderer) return;
  switch (msg.type) {
    case "restore":
      renderer.restore(msg.bytes);
      break;
    case "set_sky":
      renderer.set_sky(msg.index);
      break;
    case "add_sphere":
      renderer.add_sphere(msg.x, msg.y, msg.z, msg.radius, msg.mat_type, msg.r, msg.g, msg.b, msg.fuzz, msg.ri);
      break;
    case "add_cube":
      renderer.add_cube(msg.x, msg.y, msg.z, msg.size, msg.mat_type, msg.r, msg.g, msg.b, msg.fuzz, msg.ri);
      break;
    case "update_sphere":
      renderer.update_sphere(msg.index, msg.x, msg.y, msg.z, msg.radius, msg.mat_type, msg.r, msg.g, msg.b, msg.fuzz, msg.ri);
      break;
    case "update_cube":
      renderer.update_cube(msg.index, msg.x, msg.y, msg.z, msg.size, msg.mat_type, msg.r, msg.g, msg.b, msg.fuzz, msg.ri);
      break;
    case "update_mesh":
      renderer.update_mesh(msg.index, msg.x, msg.y, msg.z, msg.size, msg.rx, msg.ry, msg.rz, msg.mat_type, msg.r, msg.g, msg.b, msg.fuzz, msg.ri);
      break;
    case "add_mesh_stl":
      renderer.add_mesh_stl(msg.bytes, msg.x, msg.y, msg.z, msg.size, msg.mat_type, msg.r, msg.g, msg.b, msg.fuzz, msg.ri);
      break;
    case "update_mesh_material":
      renderer.update_mesh_material(msg.index, msg.mat_type, msg.r, msg.g, msg.b, msg.fuzz, msg.ri);
      break;
    case "remove_object":
      renderer.remove_object(msg.index);
      break;
    case "set_sky_hdr_bytes":
      renderer.set_sky_hdr_bytes(msg.bytes);
      break;
    case "set_sky_hdr":
      renderer.set_sky_hdr(msg.hdr_index, msg.bytes);
      break;
  }
}

self.onmessage = (e) => {
  const msg = e.data;

  if (msg.type === "render") {
    if (!renderer) return;
    while (pendingMutations.length > 0) applyMutation(pendingMutations.shift());
    const { token, pass, params } = msg;
    const { w, h, fov, cam_x, cam_y, cam_z, target_x, target_y, target_z, samples, termProb, focus_distance, aperture } = params;
    const t0 = performance.now();
    const rgba = renderer.render(w, h, fov, cam_x, cam_y, cam_z, target_x, target_y, target_z, samples, termProb, focus_distance, aperture);
    const dt = performance.now() - t0;
    postMessage({ type: "frame", token, pass, width: w, height: h, dt, rgba: rgba.buffer }, [rgba.buffer]);
  } else {
    if (wasmReady) {
      applyMutation(msg);
    } else {
      pendingMutations.push(msg);
    }
  }
};

setup();
