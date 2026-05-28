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
@group(1) @binding(7) var<storage, read> hdr_sky: array<vec4f>;

@group(2) @binding(0) var<storage, read_write> accum: array<vec4f>;
@group(2) @binding(1) var<storage, read_write> output: array<u32>;
fn hit_sphere(ray_origin: vec3f, ray_dir: vec3f, sph: GpuSphere, t_max: f32) -> HitResult {
    var result: HitResult;
    result.hit = 0u;

    let center = vec3f(sph.center[0], sph.center[1], sph.center[2]);
    let oc = ray_origin - center;
    let a = dot(ray_dir, ray_dir);
    let b = 2.0 * dot(oc, ray_dir);
    let c = dot(oc, oc) - sph.radius * sph.radius;
    let disc = b * b - 4.0 * a * c;

    if disc < 0.0 {
        return result;
    }

    let t = (-b - sqrt(disc)) / (2.0 * a);
    if t < 0.001 || t >= t_max {
        return result;
    }

    result.hit = 1u;
    result.t = t;
    result.point = ray_origin + ray_dir * t;
    let n = normalize(result.point - center);
    result.normal = n;
    result.geo_normal = n;
    return result;
}

fn hit_plane(ray_origin: vec3f, ray_dir: vec3f, pl: GpuPlane, t_max: f32) -> HitResult {
    var result: HitResult;
    result.hit = 0u;

    let point  = vec3f(pl.point[0],  pl.point[1],  pl.point[2]);
    let normal = vec3f(pl.normal[0], pl.normal[1], pl.normal[2]);
    let denom = dot(normal, ray_dir);
    if abs(denom) < 1e-6 {
        return result;
    }

    let t = dot(point - ray_origin, normal) / denom;
    if t < 0.001 || t >= t_max {
        return result;
    }

    result.hit = 1u;
    result.t = t;
    result.point = ray_origin + ray_dir * t;
    result.normal = normal;
    result.geo_normal = normal;
    return result;
}

fn hit_triangle(ray_origin: vec3f, ray_dir: vec3f, tri: GpuTriangle, t_max: f32) -> HitResult {
    var result: HitResult;
    result.hit = 0u;

    let v0v1 = tri.e01.xyz;
    let v0v2 = tri.e02.xyz;
    let pvec = cross(ray_dir, v0v2);
    let det = dot(v0v1, pvec);
    if abs(det) < 1e-8 {
        return result;
    }
    let inv_det = 1.0 / det;

    let tvec = ray_origin - tri.v0.xyz;
    let u_bary = dot(tvec, pvec) * inv_det;
    if u_bary < 0.0 || u_bary > 1.0 {
        return result;
    }

    let qvec = cross(tvec, v0v1);
    let v_bary = dot(ray_dir, qvec) * inv_det;
    if v_bary < 0.0 || u_bary + v_bary > 1.0 {
        return result;
    }

    let t = dot(v0v2, qvec) * inv_det;
    if t < 0.001 || t >= t_max {
        return result;
    }

    let w_bary = 1.0 - u_bary - v_bary;

    let geo_cross = cross(v0v1, v0v2);
    if dot(geo_cross, geo_cross) < 1e-12 {
        return result;
    }
    var geo_normal = normalize(geo_cross);

    var normal = normalize(
        tri.n0.xyz * w_bary + tri.n1.xyz * u_bary + tri.n2.xyz * v_bary
    );

    // Ensure interpolated normal is on same side as geometric normal
    if dot(normal, geo_normal) < 0.0 {
        normal = -normal;
    }
    // Flip both if ray hits from the back
    if dot(ray_dir, geo_normal) > 0.0 {
        normal = -normal;
        geo_normal = -geo_normal;
    }

    result.hit = 1u;
    result.t = t;
    result.point = ray_origin + ray_dir * t;
    result.normal = normal;
    result.geo_normal = geo_normal;
    return result;
}
const BVH_STACK_SIZE: u32 = 32u;

fn aabb_hit(bmin: vec3f, bmax: vec3f, ray_origin: vec3f, ray_dir: vec3f, t_max: f32) -> bool {
    let inv_dir = 1.0 / ray_dir;
    let t0s = (bmin - ray_origin) * inv_dir;
    let t1s = (bmax - ray_origin) * inv_dir;
    let tsmall = min(t0s, t1s);
    let tbig   = max(t0s, t1s);
    let t_enter = max(max(tsmall.x, tsmall.y), max(tsmall.z, 0.001));
    let t_exit  = min(min(tbig.x,   tbig.y),   min(tbig.z,   t_max));
    return t_exit >= t_enter;
}

fn traverse_mesh_bvh(
    ray_origin: vec3f,
    ray_dir: vec3f,
    bvh_offset: u32,
    bvh_count: u32,
    tri_offset: u32,
    t_max_in: f32,
) -> HitResult {
    var result: HitResult;
    result.hit = 0u;
    result.t = t_max_in;

    var stack: array<u32, BVH_STACK_SIZE>;
    var sp: u32 = 0u;

    // Push root (index 0 relative to this mesh's BVH slice)
    stack[sp] = 0u;
    sp += 1u;

    while sp > 0u {
        sp -= 1u;
        let rel_idx = stack[sp];

        if rel_idx >= bvh_count {
            continue;
        }

        let node = mesh_bvh[bvh_offset + rel_idx];
        let node_min = vec3f(node.min[0], node.min[1], node.min[2]);
        let node_max = vec3f(node.max[0], node.max[1], node.max[2]);

        if !aabb_hit(node_min, node_max, ray_origin, ray_dir, result.t) {
            continue;
        }

        if node.is_leaf == 1u {
            let tri = tris[tri_offset + node.tri_index];
            let tri_hit = hit_triangle(ray_origin, ray_dir, tri, result.t);
            if tri_hit.hit == 1u && tri_hit.t < result.t {
                result = tri_hit;
            }
        } else {
            // Push children -- push far child first so near child is popped first
            if sp + 2u <= BVH_STACK_SIZE {
                stack[sp] = node.right;
                sp += 1u;
                stack[sp] = node.left;
                sp += 1u;
            }
        }
    }
    return result;
}

fn traverse_top_bvh(
    ray_origin: vec3f,
    ray_dir: vec3f,
    t_max_in: f32,
) -> SceneHit {
    var result: SceneHit;
    result.hit = 0u;
    result.t = t_max_in;

    var stack: array<u32, BVH_STACK_SIZE>;
    var sp: u32 = 0u;

    stack[sp] = u.top_bvh_root;
    sp += 1u;

    while sp > 0u {
        sp -= 1u;
        let idx = stack[sp];

        let node = top_bvh[idx];
        let node_min = vec3f(node.min[0], node.min[1], node.min[2]);
        let node_max = vec3f(node.max[0], node.max[1], node.max[2]);

        if !aabb_hit(node_min, node_max, ray_origin, ray_dir, result.t) {
            continue;
        }

        if node.is_leaf == 1u {
            let obj_type = node.object_type;
            let obj_idx  = node.object_index;

            switch obj_type {
                // Sphere
                case 0u: {
                    let sph = spheres[obj_idx];
                    let h = hit_sphere(ray_origin, ray_dir, sph, result.t);
                    if h.hit == 1u && h.t < result.t {
                        result.hit = 1u;
                        result.t = h.t;
                        result.point = h.point;
                        result.normal = h.normal;
                        result.geo_normal = h.geo_normal;
                        result.mat_type = sph.mat_type;
                        result.albedo = vec3f(sph.albedo[0], sph.albedo[1], sph.albedo[2]);
                        result.fuzz = sph.fuzz;
                        result.ri = sph.ri;
                        result.scale = sph.scale;
                        result.color_a = vec3f(sph.color_a[0], sph.color_a[1], sph.color_a[2]);
                        result.color_b = vec3f(sph.color_b[0], sph.color_b[1], sph.color_b[2]);
                    }
                }
                // Mesh
                case 1u: {
                    let info = mesh_info[obj_idx];
                    let h = traverse_mesh_bvh(
                        ray_origin, ray_dir,
                        info.bvh_offset, info.bvh_count,
                        info.tri_offset, result.t,
                    );
                    if h.hit == 1u && h.t < result.t {
                        result.hit = 1u;
                        result.t = h.t;
                        result.point = h.point;
                        result.normal = h.normal;
                        result.geo_normal = h.geo_normal;
                        result.mat_type = info.mat_type;
                        result.albedo = vec3f(info.albedo[0], info.albedo[1], info.albedo[2]);
                        result.fuzz = info.fuzz;
                        result.ri = info.ri;
                        result.scale = info.scale;
                        result.color_a = vec3f(info.color_a[0], info.color_a[1], info.color_a[2]);
                        result.color_b = vec3f(info.color_b[0], info.color_b[1], info.color_b[2]);
                    }
                }
                // Plane
                case 2u: {
                    let pl = planes[obj_idx];
                    let h = hit_plane(ray_origin, ray_dir, pl, result.t);
                    if h.hit == 1u && h.t < result.t {
                        result.hit = 1u;
                        result.t = h.t;
                        result.point = h.point;
                        result.normal = h.normal;
                        result.geo_normal = h.geo_normal;
                        result.mat_type = pl.mat_type;
                        result.albedo = vec3f(pl.albedo[0], pl.albedo[1], pl.albedo[2]);
                        result.fuzz = pl.fuzz;
                        result.ri = pl.ri;
                        result.scale = pl.scale;
                        result.color_a = vec3f(pl.color_a[0], pl.color_a[1], pl.color_a[2]);
                        result.color_b = vec3f(pl.color_b[0], pl.color_b[1], pl.color_b[2]);
                    }
                }
                default: {}
            }
        } else {
            if sp + 2u <= BVH_STACK_SIZE {
                stack[sp] = node.right;
                sp += 1u;
                stack[sp] = node.left;
                sp += 1u;
            }
        }
    }
    return result;
}
// mat_type: 0=Lambertian, 1=Metal, 2=Dielectric, 3=DiffuseLight, 4=Checkerboard

const PI: f32 = 3.14159265358979323846;

fn reflect(v: vec3f, n: vec3f) -> vec3f {
    return v - n * 2.0 * dot(v, n);
}

fn refract(uv: vec3f, n: vec3f, etai_over_etat: f32) -> vec3f {
    let cos_theta = min(dot(-uv, n), 1.0);
    let r_out_perp = (uv + n * cos_theta) * etai_over_etat;
    let r_out_parallel = n * -sqrt(abs(1.0 - dot(r_out_perp, r_out_perp)));
    return r_out_perp + r_out_parallel;
}

fn schlick(cosine: f32, ri: f32) -> f32 {
    var r0 = (1.0 - ri) / (1.0 + ri);
    r0 = r0 * r0;
    return r0 + (1.0 - r0) * pow(1.0 - cosine, 5.0);
}

fn cosine_weighted_hemisphere(normal: vec3f, rng: ptr<function, u32>) -> vec3f {
    let r1 = rand_f32(rng);
    let r2 = rand_f32(rng);
    let phi = 2.0 * PI * r1;
    let x = cos(phi) * sqrt(r2);
    let y = sin(phi) * sqrt(r2);
    let z = sqrt(1.0 - r2);

    let w = normalize(normal);
    var a: vec3f;
    if abs(w.x) > 0.9 {
        a = vec3f(0.0, 1.0, 0.0);
    } else {
        a = vec3f(1.0, 0.0, 0.0);
    }
    let v = normalize(cross(w, a));
    let u_vec = cross(w, v);
    return u_vec * x + v * y + w * z;
}

fn random_in_unit_sphere(rng: ptr<function, u32>) -> vec3f {
    var result: vec3f = vec3f(0.0);
    loop {
        result = vec3f(
            rand_f32(rng) * 2.0 - 1.0,
            rand_f32(rng) * 2.0 - 1.0,
            rand_f32(rng) * 2.0 - 1.0,
        );
        if dot(result, result) < 1.0 {
            break;
        }
    }
    return result;
}

fn eval_material(
    mat_type: u32,
    ray_dir: vec3f,
    hit_point: vec3f,
    normal: vec3f,
    geo_normal: vec3f,
    albedo: vec3f,
    fuzz: f32,
    ri: f32,
    scale: f32,
    color_a: vec3f,
    color_b: vec3f,
    rng: ptr<function, u32>,
) -> MaterialResult {
    var result: MaterialResult;
    result.scattered = 0u;
    result.is_diffuse = 0u;
    result.emitted = vec3f(0.0);
    result.attenuation = vec3f(0.0);
    result.f_diffuse = vec3f(0.0);
    result.new_ray_origin = vec3f(0.0);
    result.new_ray_dir = vec3f(0.0);

    let eps = 1e-3;

    switch mat_type {
        // Lambertian
        case 0u: {
            let dir = cosine_weighted_hemisphere(normal, rng);
            result.new_ray_origin = hit_point + geo_normal * eps;
            result.new_ray_dir = dir;
            result.attenuation = albedo;
            result.scattered = 1u;
            result.is_diffuse = 1u;
            result.f_diffuse = albedo * (1.0 / PI);
        }
        // Metal
        case 1u: {
            let reflected = reflect(normalize(ray_dir), normal);
            let fuzz_vec = random_in_unit_sphere(rng) * fuzz;
            result.new_ray_origin = hit_point + geo_normal * eps;
            result.new_ray_dir = reflected + fuzz_vec;
            result.attenuation = albedo;
            result.scattered = 1u;
            result.is_diffuse = 0u;
        }
        // Dielectric
        case 2u: {
            var ior = ri;
            if dot(ray_dir, normal) < 0.0 {
                ior = 1.0 / ior;
            }
            let unit_dir = normalize(ray_dir);
            let cos_theta = min(dot(-unit_dir, normal), 1.0);
            let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
            let cannot_refract = ior * sin_theta > 1.0;

            if cannot_refract || schlick(cos_theta, ior) > rand_f32(rng) {
                result.new_ray_dir = reflect(unit_dir, normal);
                result.new_ray_origin = hit_point + geo_normal * eps;
            } else {
                result.new_ray_dir = refract(unit_dir, normal, ior);
                result.new_ray_origin = hit_point - geo_normal * eps;
            }
            result.attenuation = albedo;
            result.scattered = 1u;
            result.is_diffuse = 0u;
        }
        // DiffuseLight
        case 3u: {
            result.emitted = albedo;
            result.scattered = 0u;
        }
        // Checkerboard
        case 4u: {
            let ix = i32(floor(hit_point.x * scale));
            let iz = i32(floor(hit_point.z * scale));
            var color: vec3f;
            if (ix + iz) % 2 == 0 {
                color = color_a;
            } else {
                color = color_b;
            }
            let dir = cosine_weighted_hemisphere(normal, rng);
            result.new_ray_origin = hit_point + geo_normal * eps;
            result.new_ray_dir = dir;
            result.attenuation = color;
            result.scattered = 1u;
            result.is_diffuse = 1u;
            result.f_diffuse = color * (1.0 / PI);
        }
        default: {}
    }
    return result;
}

fn pcg_hash(seed: u32) -> u32 {
    var state = seed * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rand_f32(state: ptr<function, u32>) -> f32 {
    let u = pcg_hash(*state);
    *state = u;
    return f32(u) / 4294967296.0;
}

fn sample_sky(dir: vec3f) -> vec3f {
    if u.sky_type == 0u {
        return u.sky_top.xyz;
    } else if u.sky_type == 1u {
        let t = 0.5 * (dir.y + 1.0);
        return (1.0 - t) * u.sky_bottom.xyz + t * u.sky_top.xyz;
    } else if u.sky_type == 2u {
        let u_coord = 0.5 + atan2(dir.x, -dir.z) / (2.0 * PI);
        let v_coord = 0.5 - asin(dir.y) / PI;
        let px = clamp(u32(u_coord * f32(u.hdr_width)), 0u, u.hdr_width - 1u);
        let py = clamp(u32(v_coord * f32(u.hdr_height)), 0u, u.hdr_height - 1u);
        let idx = py * u.hdr_width + px;
        return hdr_sky[idx].xyz * u.hdr_exposure;
    }
    return vec3f(0.0);
}

fn max3(v: vec3f) -> f32 {
    return max(v.x, max(v.y, v.z));
}

fn estimate_direct(
    hit_point: vec3f,
    normal: vec3f,
    geo_normal: vec3f,
    f_diffuse: vec3f,
    rng: ptr<function, u32>
) -> vec3f {
    if u.num_lights == 0u {
        return vec3f(0.0);
    }
    let p_sel = 1.0 / f32(u.num_lights);

    // Pick random light
    let l_idx = u32(rand_f32(rng) * f32(u.num_lights));
    let light_idx = min(l_idx, u.num_lights - 1u);
    let light = lights[light_idx];
    let center = vec3f(light.center[0], light.center[1], light.center[2]);
    let le = vec3f(light.le[0], light.le[1], light.le[2]);

    let toward_x = normalize(hit_point - center);
    let n_y = cosine_weighted_hemisphere(toward_x, rng); // same as random_hemisphere conceptually
    let y = center + n_y * light.radius;

    let d = y - hit_point;
    let dist2 = dot(d, d);
    if dist2 <= 1e-12 {
        return vec3f(0.0);
    }
    let dist = sqrt(dist2);
    let wi = d / dist;

    let cos_surf = max(dot(normal, wi), 0.0);
    if cos_surf <= 0.0 {
        return vec3f(0.0);
    }

    let cos_light = max(dot(n_y, -wi), 0.0);
    if cos_light <= 0.0 {
        return vec3f(0.0);
    }

    let eps = 1e-3;
    let shadow_ray_origin = hit_point + geo_normal * eps;
    let t_max = max(dist - eps, 0.0);

    let shadow_hit = traverse_top_bvh(shadow_ray_origin, wi, t_max);
    if shadow_hit.hit == 1u {
        return vec3f(0.0);
    }

    let pdf_area = 1.0 / (2.0 * PI * light.radius * light.radius);
    let pdf_omega = pdf_area * dist2 / cos_light;
    let pdf = p_sel * pdf_omega;

    if pdf <= 1e-20 {
        return vec3f(0.0);
    }

    return f_diffuse * le * (cos_surf / pdf);
}

fn random_in_unit_disk(rng: ptr<function, u32>) -> vec3f {
    let p = vec3f(
        rand_f32(rng) * 2.0 - 1.0,
        rand_f32(rng) * 2.0 - 1.0,
        0.0
    );
    var result = p;
    while dot(result, result) >= 1.0 {
        result = vec3f(
            rand_f32(rng) * 2.0 - 1.0,
            rand_f32(rng) * 2.0 - 1.0,
            0.0
        );
    }
    return result;
}


@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let x = gid.x;
    let y = gid.y;
    if x >= u.width || y >= u.height {
        return;
    }
    let pixel_idx = y * u.width + x;

    var rng = pcg_hash(pixel_idx ^ (u.current_sample * 1337u));

    // camera ray
    // Use a simple random offset for pixel sampling
    let sample_offset = vec2f(rand_f32(&rng), rand_f32(&rng));
    //let sample_offset = vec2f(rand_f32(&rng), rand_f32(&rng)); // fallback

    let x_cmp = ((f32(x) + sample_offset.x) / f32(u.width) - 0.5) * u.half_tan_fov_x;
    let y_cmp = (0.5 - (f32(y) + sample_offset.y) / f32(u.height)) * u.half_tan_fov_y;

    let dir = normalize(u.cam_forward + u.cam_right * x_cmp + u.cam_up * y_cmp);

    let focus_point = u.cam_pos + dir * u.focus_distance;
    let lens_offset = random_in_unit_disk(&rng) * (u.aperture / 2.0);
    let origin = u.cam_pos + u.cam_right * lens_offset.x + u.cam_up * lens_offset.y;

    var ray_origin = origin;
    var ray_dir = normalize(focus_point - origin);

    var beta = vec3f(1.0);
    var L = vec3f(0.0);
    var prev_specular = true;

    for (var depth = 0u; depth < u.max_depth; depth++) {
        let hit = traverse_top_bvh(ray_origin, ray_dir, 1e30);
        if hit.hit == 0u {
            L += beta * sample_sky(ray_dir);
            break;
        }

        let mat = eval_material(
            hit.mat_type, ray_dir, hit.point, hit.normal, hit.geo_normal,
            hit.albedo, hit.fuzz, hit.ri, hit.scale, hit.color_a, hit.color_b,
            &rng
        );

        if prev_specular {
            L += beta * mat.emitted;
        }

        if mat.is_diffuse == 1u {
            L += beta * estimate_direct(hit.point, hit.normal, hit.geo_normal, mat.f_diffuse, &rng);
            prev_specular = false;
        } else {
            prev_specular = true;
        }

        if mat.scattered == 0u {
            break;
        }

        ray_origin = mat.new_ray_origin;
        ray_dir = mat.new_ray_dir;
        beta *= mat.attenuation;

        if max3(beta) < 1e-4 {
            break;
        }

        // Russian roulette
        if depth >= 5u {
            let p = clamp(max3(beta), u.termination_prob, 0.95);
            if rand_f32(&rng) > p {
                break;
            }
            beta /= p;
        }
    }

    let prev = accum[pixel_idx];
    accum[pixel_idx] = vec4f(prev.xyz + L, prev.w + 1.0);
}
