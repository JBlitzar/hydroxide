wgpu Path Tracer Port
Architecture Overview
The entire path tracing kernel (cast_ray, BVH traversal, material evaluation, direct light estimation, tonemapping, despeckle) moves into a single wgpu compute pipeline. The Rust host code remains responsible for scene building, BVH construction, and orchestrating the GPU dispatch. The web target uses wgpu over WebGPU (wasm32 + web-sys); native target uses Vulkan/Metal/DX12 via wgpu's native backends.

Rust Host
Compute Shader - WGSL
SceneDescription
BVH Builder
GPU Buffer Upload
wgpu Dispatch
Readback RGBA
Per-pixel invocation
cast_ray loop
BVH Traversal
Material Eval
Direct Light NEE
Sky Eval
Accumulate + Tonemap
Despeckle
Key Design Decisions
f32 everywhere on GPU -- WGSL has no f64. All Vec3/Ray/AABB/Triangle data is converted to f32 before upload. Rust host keeps f64 for scene description and BVH build, then truncates to f32 for GPU buffers. This means a GpuVec3 (3xf32) and GpuRay struct in WGSL.

Flat BVH (stackless or explicit-stack traversal) -- The recursive BVHNode / MeshBVH trees are linearized into flat arrays of GpuBVHNode structs suitable for iterative traversal in WGSL. Both the top-level scene BVH and per-mesh BVHs become flat arrays.

RNG -- fastrand can't run on GPU. Use PCG hash seeded with (pixel_x, pixel_y, sample_index, frame) in WGSL.

Blue noise -- Upload the existing blue noise table as a storage buffer; index into it from the shader.

Progressive rendering -- Dispatch one sample-per-pixel per frame, accumulate in an f32 accumulation buffer, then tonemap+convert to u8 in a second pass. This replaces the adaptive convergence check (which is inherently serial/per-pixel branchy). Convergence can be re-added later as a compute pass that marks pixels as "done".

Data Layout (GPU Buffers)
Uniforms (bind group 0)
rust

#[repr(C)]
struct GpuUniforms {
width: u32,
height: u32,
samples: u32,
current_sample: u32,
max_depth: u32,
termination_prob: f32,
// camera
cam_pos: [f32; 4], // xyz + pad
cam_forward: [f32; 4],
cam_right: [f32; 4],
cam_up: [f32; 4],
half_tan_fov_x: f32,
half_tan_fov_y: f32,
focus_distance: f32,
aperture: f32,
// sky
sky_type: u32, // 0=gradient, 1=solid, 2=hdr
sky_top: [f32; 4],
sky_bottom: [f32; 4],
hdr_width: u32,
hdr_height: u32,
hdr_exposure: f32,
num_lights: u32,
num_spheres: u32,
num_mesh_bvh_nodes: u32,
num_triangles: u32,
top_bvh_root: u32,
}

Show more
Storage Buffers (bind group 1)
Buffer Contents Struct
spheres Packed sphere data { center: vec3f, radius: f32, mat_type: u32, albedo: vec3f, fuzz: f32, ri: f32 }
top_bvh Linearized top-level BVH { min: vec3f, max: vec3f, left: u32, right: u32, is_leaf: u32, object_type: u32, object_index: u32 }
mesh_bvh All per-mesh BVH nodes concatenated { min: vec3f, max: vec3f, left: u32, right: u32, is_leaf: u32, tri_index: u32 }
triangles All triangles concatenated { v0: vec3f, v1: vec3f, v2: vec3f, n0: vec3f, n1: vec3f, n2: vec3f, e01: vec3f, e02: vec3f }
mesh_info Per-mesh metadata { bvh_offset: u32, bvh_count: u32, tri_offset: u32, mat_type: u32, albedo: vec3f, fuzz: f32, ri: f32 }
lights Sphere lights { center: vec3f, radius: f32, Le: vec3f }
blue_noise Blue noise table array<vec2f>
hdr_sky HDR env map array<vec4f> (rgb + pad)
Output Buffers (bind group 2)
Buffer Purpose
accum vec4f per pixel -- running color sum (rgb) + sample count (w)
output u32 per pixel -- final packed RGBA after tonemap
Files Changed / Created
New files
File Purpose
src/gpu/mod.rs GpuRenderer struct, buffer creation, dispatch, readback
src/gpu/upload.rs flatten_bvh(), pack_spheres(), pack_triangles(), pack_lights(), pack_uniforms()
src/gpu/shaders/pathtrace.wgsl Main compute shader -- ray gen, BVH traversal, material eval, NEE, sky, accumulate
src/gpu/shaders/tonemap.wgsl Second compute pass -- reads accum, writes tonemapped u8 RGBA to output
Modified files
File Change
Cargo.toml Add wgpu = { version = "24", features = [...] }, bytemuck, pollster (native), feature flags
src/lib.rs Add pub mod gpu;
src/main.rs Add --gpu flag; call GpuRenderer instead of Renderer
src/wasm.rs WasmRenderer::render() calls GpuRenderer (async via wasm-bindgen-futures)
src/bvh.rs Add pub fn flatten(&self) -> Vec<FlatBVHNode> method
src/geometry/mesh.rs Add pub fn flatten_nodes(&self) -> (Vec<FlatMeshBVHNode>, &[Triangle]) method
Method Sketches
GpuRenderer (src/gpu/mod.rs)
rust

pub struct GpuRenderer {
device: wgpu::Device,
queue: wgpu::Queue,
pathtrace_pipeline: wgpu::ComputePipeline,
tonemap_pipeline: wgpu::ComputePipeline,
bind_groups: Vec<wgpu::BindGroup>,
accum_buffer: wgpu::Buffer,
output_buffer: wgpu::Buffer,
staging_buffer: wgpu::Buffer, // for readback
width: u32,
height: u32,
}

impl GpuRenderer {
/// Create device, compile shaders, create pipelines.
/// On web: adapter from navigator.gpu; native: pollster::block_on.
pub async fn new(width: u32, height: u32) -> Self { ... }

    /// Upload scene data to GPU buffers, create bind groups.
    pub fn upload_scene(&mut self, world: &World, camera: &Camera, settings: &RenderSettings) {
        // 1. Flatten top-level BVH -> Vec<FlatBVHNode>
        // 2. For each mesh object, flatten its MeshBVH, concatenate
        // 3. Pack all spheres, triangles, lights, blue noise, HDR sky
        // 4. Create/update wgpu::Buffers with bytemuck::cast_slice
        // 5. Rebuild bind groups
    }

    /// Dispatch N samples, accumulating into accum_buffer.
    pub fn render(&self, num_samples: u32) {
        for sample in 0..num_samples {
            // Update uniform.current_sample
            self.queue.write_buffer(&self.uniform_buf, offset, &[sample]);
            let mut encoder = self.device.create_command_encoder(&Default::default());
            {
                let mut pass = encoder.begin_compute_pass(&Default::default());
                pass.set_pipeline(&self.pathtrace_pipeline);
                pass.set_bind_group(0, &self.bind_groups[0], &[]);
                pass.set_bind_group(1, &self.bind_groups[1], &[]);
                pass.set_bind_group(2, &self.bind_groups[2], &[]);
                let wg_x = (self.width + 7) / 8;
                let wg_y = (self.height + 7) / 8;
                pass.dispatch_workgroups(wg_x, wg_y, 1);
            }
            self.queue.submit(Some(encoder.finish()));
        }
        // Final tonemap pass
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.tonemap_pipeline);
            // ...
            pass.dispatch_workgroups((self.width + 7) / 8, (self.height + 7) / 8, 1);
        }
        // Copy output -> staging for readback
        encoder.copy_buffer_to_buffer(&self.output_buffer, 0, &self.staging_buffer, 0, size);
        self.queue.submit(Some(encoder.finish()));
    }

    /// Map staging buffer and return RGBA pixels.
    pub async fn read_pixels(&self) -> Vec<u8> {
        let slice = self.staging_buffer.slice(..);
        let (tx, rx) = futures::channel::oneshot::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| { tx.send(result).ok(); });
        self.device.poll(wgpu::Maintain::Wait);
        rx.await.unwrap().unwrap();
        let data = slice.get_mapped_range();
        // Unpack u32 -> [u8; 4] RGBA
        data.chunks(4).flat_map(|c| c.to_vec()).collect()
    }

}

Show more
BVH Flattening (src/bvh.rs addition)
rust

#[repr(C)] #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FlatBVHNode {
pub min: [f32; 3],
pub \_pad0: f32,
pub max: [f32; 3],
pub \_pad1: f32,
pub left: u32,
pub right: u32,
pub is_leaf: u32,
pub object_type: u32, // 0=sphere, 1=mesh
pub object_index: u32,
pub \_pad2: [u32; 3],
}

impl BVHNode {
/// Linearize the tree into a flat array via pre-order traversal.
/// Returns (flat_nodes, mapping from Arc<dyn Hittable> -> object index).
pub fn flatten(&self) -> Vec<FlatBVHNode> { ... }
}

Show more
MeshBVH Flattening (src/geometry/mesh.rs addition)
rust

#[repr(C)] #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FlatMeshBVHNode {
pub min: [f32; 3],
pub \_pad0: f32,
pub max: [f32; 3],
pub \_pad1: f32,
pub left: u32,
pub right: u32,
pub is_leaf: u32,
pub tri_index: u32,
}

#[repr(C)] #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuTriangle {
pub v0: [f32; 4], // xyz + pad
pub v1: [f32; 4],
pub v2: [f32; 4],
pub n0: [f32; 4],
pub n1: [f32; 4],
pub n2: [f32; 4],
pub e01: [f32; 4],
pub e02: [f32; 4],
}

impl MeshBVH {
pub fn flatten_for_gpu(&self, tri_offset: u32) -> (Vec<FlatMeshBVHNode>, Vec<GpuTriangle>) {
// Walk self.nodes, output flat array with tri_index offset by tri_offset
// Convert each Triangle to GpuTriangle (f64->f32)
}
}

Show more
WGSL Compute Shader Sketch (pathtrace.wgsl)
wgsl

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(1) @binding(0) var<storage, read> spheres: array<GpuSphere>;
@group(1) @binding(1) var<storage, read> top_bvh: array<FlatBVHNode>;
@group(1) @binding(2) var<storage, read> mesh_bvh: array<FlatMeshNode>;
@group(1) @binding(3) var<storage, read> tris: array<GpuTriangle>;
@group(1) @binding(4) var<storage, read> mesh_info: array<MeshInfo>;
@group(1) @binding(5) var<storage, read> lights: array<GpuLight>;
@group(1) @binding(6) var<storage, read> blue_noise: array<vec2f>;
@group(1) @binding(7) var<storage, read> hdr_sky: array<vec4f>;
@group(2) @binding(0) var<storage, read_write> accum: array<vec4f>;
@group(2) @binding(1) var<storage, read_write> output: array<u32>;

fn pcg_hash(seed: u32) -> u32 { ... }

fn hit_sphere(ray: Ray, sph: GpuSphere, t_max: f32) -> HitResult { ... }

fn hit_triangle(ray: Ray, tri: GpuTriangle, t_max: f32) -> HitResult { ... }

fn traverse_mesh_bvh(ray: Ray, info: MeshInfo, t_max: f32) -> HitResult {
// Iterative stack-based BVH traversal over mesh_bvh[info.bvh_offset..]
var stack: array<u32, 32>;
...
}

fn traverse_top_bvh(ray: Ray, t_max: f32) -> SceneHit {
// Iterative traversal; at leaf, dispatch to hit_sphere or traverse_mesh_bvh
var stack: array<u32, 32>;
...
}

fn eval_material(mat_type: u32, ray: Ray, hit: HitResult, albedo: vec3f, fuzz: f32, ri: f32)
-> MaterialResult { ... }

fn sample_sky(dir: vec3f) -> vec3f {
// Branch on u.sky_type: gradient, solid, or HDR lookup
}

fn estimate_direct(pos: vec3f, normal: vec3f, geo_normal: vec3f, f_diffuse: vec3f, rng: ptr<function, u32>) -> vec3f {
// Pick random light, sample hemisphere, shadow ray via traverse_top_bvh
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
let x = gid.x;
let y = gid.y;
if x >= u.width || y >= u.height { return; }
let pixel = y \* u.width + x;

    var rng = pcg_hash(pixel ^ (u.current_sample * 1337u));

    // Generate camera ray (with blue noise jitter + DOF)
    var ray = generate_ray(x, y, &rng);

    // Path trace (same logic as cast_ray)
    var beta = vec3f(1.0);
    var L = vec3f(0.0);
    var prev_specular = true;
    for (var depth = 0u; depth < u.max_depth; depth++) {
        let hit = traverse_top_bvh(ray, 1e30);
        if !hit.valid {
            L += beta * sample_sky(ray.dir);
            break;
        }
        let Le = emitted(hit);
        if prev_specular { L += beta * Le; }

        let mat = eval_material(...);
        if mat.is_diffuse {
            L += beta * estimate_direct(...);
            prev_specular = false;
        } else {
            prev_specular = true;
        }
        if !mat.scattered { break; }
        ray = mat.new_ray;
        beta *= mat.attenuation;
        if max3(beta) < 1e-4 { break; }

        // Russian roulette
        if depth >= 5u {
            let p = clamp(max3(beta), u.termination_prob, 0.95);
            if rand_f32(&rng) > p { break; }
            beta /= p;
        }
    }

    // Accumulate
    let prev = accum[pixel];
    accum[pixel] = vec4f(prev.xyz + L, prev.w + 1.0);

}

Show more
Tonemap Shader (tonemap.wgsl)
wgsl

fn aces_tonemap(c: vec3f) -> vec3f {
let a = 2.51; let b = 0.03; let c2 = 2.43; let d = 0.59; let e = 0.14;
return clamp((c _ (a _ c + b)) / (c _ (c2 _ c + d) + e), vec3f(0.0), vec3f(1.0));
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
let pixel = gid.y _ u.width + gid.x;
let acc = accum[pixel];
let avg = acc.xyz / max(acc.w, 1.0);
let mapped = aces_tonemap(avg);
let r = u32(pow(mapped.x, 1.0/2.2) _ 255.0);
let g = u32(pow(mapped.y, 1.0/2.2) _ 255.0);
let b = u32(pow(mapped.z, 1.0/2.2) _ 255.0);
output[pixel] = r | (g << 8u) | (b << 16u) | (255u << 24u);
}

Show more
Cargo.toml Changes
toml

[dependencies]
wgpu = { version = "24" }
bytemuck = { version = "1", features = ["derive"] }
pollster = { version = "0.4", optional = true } # native only

[features]
default = ["native"]
native = ["indicatif", "pollster"]
wasm = ["wgpu/webgpu", "wasm-bindgen", "wasm-bindgen-futures", "js-sys", "web-sys"]
Remove rayon and wasm-bindgen-rayon dependencies (no longer needed once GPU path is primary).

wasm.rs Changes
The WasmRenderer::render() method becomes async and internally creates/reuses a GpuRenderer. The scene upload + dispatch + readback flow replaces the current Renderer::render() call.

rust

pub async fn render(&self, /_ same params _/) -> Vec<u8> {
let camera = Camera::look_at(...);
let world = self.build_world(camera);
let mut gpu = GpuRenderer::new(width, height).await;
gpu.upload_scene(&world, &camera, &settings);
gpu.render(samples);
gpu.read_pixels().await
}
Expose via wasm_bindgen_futures so JS gets a Promise<Uint8Array>.

main.rs Changes
rust

fn main() {
// ... existing scene setup ...
let gpu_renderer = pollster::block_on(GpuRenderer::new(width, height));
gpu_renderer.upload_scene(&world, &camera, &settings);
gpu_renderer.render(samples);
let pixels = pollster::block_on(gpu_renderer.read_pixels());
// save image from pixels
}
What Stays in Rust (CPU)
Scene description, serialization (scene.rs)
STL loading and mesh processing (mesh.rs)
BVH construction (both top-level and per-mesh) -- just add flatten() methods
Camera setup
Blue noise table generation
HDR sky loading and decoding
Object picking (World::pick, outline) -- these are single-ray operations, keep on CPU
What Moves to GPU (WGSL)
Ray generation (camera + jitter + DOF)
Path tracing loop (cast_ray)
BVH traversal (both top-level and per-mesh)
Ray-sphere intersection
Ray-triangle intersection (Moller-Trumbore)
Material evaluation (Lambertian, Metal, Dielectric, Checkerboard, DiffuseLight)
Next event estimation (direct light sampling)
Sky evaluation (gradient, solid, HDR lookup)
Tonemapping (ACES) + gamma correction
Despeckle (as optional third compute pass)
Migration Order
Add wgpu, bytemuck, pollster deps; create src/gpu/mod.rs stub
Implement BVH flattening (bvh.rs, mesh.rs)
Implement buffer packing (gpu/upload.rs)
Write WGSL shaders (pathtrace + tonemap)
Wire up GpuRenderer::new(), upload_scene(), render(), read_pixels()
Integrate into main.rs (native path)
Integrate into wasm.rs (async WebGPU path)
Remove rayon/wasm-bindgen-rayon once GPU path is validated
Optional: progressive rendering (render 1 sample per frame, display intermediate)
