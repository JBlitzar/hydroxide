struct Uniforms {
    width: u32,
    height: u32,
    spp: u32,
    current_sample: u32,
    cam_pos: vec3f,
    _pad1: f32,
    cam_forward: vec3f,
    _pad2: f32,
    cam_right: vec3f,
    _pad3: f32,
    cam_up: vec3f,
    _pad4: f32,
    half_tan_fov_x: f32,
    half_tan_fov_y: f32,
    focus_distance: f32,
    aperture: f32,
    sky_type: u32,
    _pad5_0: u32,
    _pad5_1: u32,
    _pad5_2: u32,
    sky_top: vec4f,
    sky_bottom: vec4f,
    hdr_width: u32,
    hdr_height: u32,
    hdr_exposure: f32,
    num_lights: u32,
    num_spheres: u32,
    num_planes: u32,
    num_meshes: u32,
    num_mesh_bvh_nodes: u32,
    num_triangles: u32,
    top_bvh_root: u32,
    max_depth: u32,
    termination_prob: f32,
}

struct GpuSphere {
    center: array<f32, 3>,
    radius: f32,
    mat_type: u32,
    albedo: array<f32, 3>,
    fuzz: f32,
    ri: f32,
    scale: f32,
    _pad: f32,
    color_a: array<f32, 3>,
    _pad2: f32,
    color_b: array<f32, 3>,
    _pad3: f32,
}

struct GpuMesh {
    bvh_offset: u32,
    bvh_count: u32,
    tri_offset: u32,
    tri_count: u32,
    mat_type: u32,
    albedo: array<f32, 3>,
    fuzz: f32,
    ri: f32,
    scale: f32,
    _pad: f32,
    color_a: array<f32, 3>,
    _pad2: f32,
    color_b: array<f32, 3>,
    _pad3: f32,
}

struct GpuPlane {
    point: array<f32, 3>,
    _pad0: f32,
    normal: array<f32, 3>,
    _pad1: f32,
    mat_type: u32,
    albedo: array<f32, 3>,
    fuzz: f32,
    ri: f32,
    scale: f32,
    _pad2: f32,
    color_a: array<f32, 3>,
    _pad3: f32,
    color_b: array<f32, 3>,
    _pad4: f32,
}

struct FlatBVHNode {
    min: array<f32, 3>,
    _pad0: f32,
    max: array<f32, 3>,
    _pad1: f32,
    left: u32,
    right: u32,
    is_leaf: u32,
    object_type: u32,
    object_index: u32,
    _pad2_0: u32,
    _pad2_1: u32,
    _pad2_2: u32,
}

struct FlatMeshBVHNode {
    min: array<f32, 3>,
    _pad0: f32,
    max: array<f32, 3>,
    _pad1: f32,
    left: u32,
    right: u32,
    is_leaf: u32,
    tri_index: u32,
}

struct GpuTriangle {
    v0: vec4f,
    v1: vec4f,
    v2: vec4f,
    n0: vec4f,
    n1: vec4f,
    n2: vec4f,
    e01: vec4f,
    e02: vec4f,
}

struct GpuLight {
    center: array<f32, 3>,
    radius: f32,
    le: array<f32, 3>,
    _pad: f32,
}

struct HitResult {
    hit: u32,
    t: f32,
    point: vec3f,
    normal: vec3f,
    geo_normal: vec3f,
}

struct SceneHit {
    hit: u32,
    t: f32,
    point: vec3f,
    normal: vec3f,
    geo_normal: vec3f,
    mat_type: u32,
    albedo: vec3f,
    fuzz: f32,
    ri: f32,
    scale: f32,
    color_a: vec3f,
    color_b: vec3f,
}

struct MaterialResult {
    scattered: u32,
    is_diffuse: u32,
    new_ray_origin: vec3f,
    new_ray_dir: vec3f,
    attenuation: vec3f,
    emitted: vec3f,
    f_diffuse: vec3f,
}

// --- Bindings ---

@group(0) @binding(0) var<uniform> u: Uniforms;

@group(1) @binding(0) var<storage, read> spheres: array<GpuSphere>;
@group(1) @binding(1) var<storage, read> top_bvh: array<FlatBVHNode>;
@group(1) @binding(2) var<storage, read> mesh_bvh: array<FlatMeshBVHNode>;
@group(1) @binding(3) var<storage, read> tris: array<GpuTriangle>;
@group(1) @binding(4) var<storage, read> mesh_info: array<GpuMesh>;
@group(1) @binding(5) var<storage, read> planes: array<GpuPlane>;
@group(1) @binding(6) var<storage, read> lights: array<GpuLight>;
@group(1) @binding(7) var<storage, read> blue_noise: array<vec2f>;
@group(1) @binding(8) var<storage, read> hdr_sky: array<vec4f>;

@group(2) @binding(0) var<storage, read_write> accum: array<vec4f>;
@group(2) @binding(1) var<storage, read_write> output: array<u32>;
