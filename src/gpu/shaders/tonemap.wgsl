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

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(1) @binding(0) var<storage, read_write> accum: array<vec4f>;
@group(1) @binding(1) var<storage, read_write> output: array<u32>;

fn aces_tonemap(color: vec3f) -> vec3f {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp(
        (color * (a * color + b)) / (color * (c * color + d) + e),
        vec3f(0.0),
        vec3f(1.0)
    );
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let x = gid.x;
    let y = gid.y;
    if x >= u.width || y >= u.height {
        return;
    }
    let pixel_idx = y * u.width + x;

    let acc = accum[pixel_idx];
    let n_valid = acc.w;
    
    var color = vec3f(0.0);
    if n_valid > 0.0 {
        color = acc.xyz / n_valid;
    }

    let mapped = aces_tonemap(color);
    
    let r = u32(clamp(pow(mapped.x, 1.0 / 2.2) * 255.0, 0.0, 255.0));
    let g = u32(clamp(pow(mapped.y, 1.0 / 2.2) * 255.0, 0.0, 255.0));
    let b = u32(clamp(pow(mapped.z, 1.0 / 2.2) * 255.0, 0.0, 255.0));

    output[pixel_idx] = r | (g << 8u) | (b << 16u) | (255u << 24u);
}
