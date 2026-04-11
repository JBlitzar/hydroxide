struct Ray { origin: vec3f, dir: vec3f }

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
    // material params copied from whichever array the leaf pointed to
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
    new_ray: Ray,
    attenuation: vec3f,
    emitted: vec3f,
    f_diffuse: vec3f, // zero if specular
}



fn pcg_hash(seed: u32) -> u32 { // https://www.reedbeta.com/blog/hash-functions-for-gpu-rendering/
    var state = seed * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rand_f32(state: ptr<function, u32>) -> f32 {
    let u = pcg_hash(*state);
    *state = u;
    return f32(u) / 4294967296.0;
}



struct Sphere {
    center: vec3f,
    radius: f32,
    mat_type: u32,
    albedo: vec3f,
    fuzz: f32,
    ri: f32,
    scale: f32,
    color_a: vec3f,
    color_b: vec3f,
}

struct Plane {
    point: vec3f,
    normal: vec3f,
    mat_type: u32,
    albedo: vec3f,
    fuzz: f32,
    ri: f32,
    scale: f32,
    color_a: vec3f,
    color_b: vec3f,
}

struct Triangle {
    v0: vec3f,
    e01: vec3f,
    e02: vec3f,
    n0: vec3f,
    n1: vec3f,
    n2: vec3f,
    mat_type: u32,
    albedo: vec3f,
    fuzz: f32,
    ri: f32,
    scale: f32,
    color_a: vec3f,
    color_b: vec3f,
}

fn hit_sphere(ray: Ray, sph: Sphere, t_max: f32) -> HitResult {
    let oc = ray.origin - sph.center;
    let a = dot(ray.dir, ray.dir);
    let b = 2.0 * dot(oc, ray.dir);
    let c = dot(oc, oc) - sph.radius * sph.radius;
    let discriminant = b * b - 4.0 * a * c;

    if (discriminant < 0.0) {
        return HitResult(0u, 0.0, vec3f(0.0), vec3f(0.0), vec3f(0.0));
    }

    let sqrt_disc = sqrt(discriminant);
    var t = (-b - sqrt_disc) / (2.0 * a);
    if (t < 0.001 || t > t_max) {
        t = (-b + sqrt_disc) / (2.0 * a);
        if (t < 0.001 || t > t_max) {
            return HitResult(0u, 0.0, vec3f(0.0), vec3f(0.0), vec3f(0.0));
        }
    }

    let hit_point = ray.origin + t * ray.dir;
    let normal = normalize(hit_point - sph.center);
    return HitResult(1u, t, hit_point, normal, normal);
}


fn hit_plane(ray: Ray, plane: Plane, t_max: f32) -> HitResult {
    let denom = dot(plane.normal, ray.dir);
    if (abs(denom) > 0.0001) {
        let t = dot(plane.point - ray.origin, plane.normal) / denom;
        if (t >= 0.001 && t <= t_max) {
            let hit_point = ray.origin + t * ray.dir;
            return HitResult(1u, t, hit_point, plane.normal, plane.normal);
        }
    }
    
    return HitResult(0u, 0.0, vec3f(0.0), vec3f(0.0), vec3f(0.0));

}


fn hit_triangle(ray: Ray, tri: Triangle, t_max: f32) -> HitResult {
    let v0v1 = tri.e01;
    let v0v2 = tri.e02;
    let pvec = cross(ray.dir, v0v2);
    let det = dot(v0v1, pvec);

    if (abs(det) < 0.0001) {
        return HitResult(0u, 0.0, vec3f(0.0), vec3f(0.0), vec3f(0.0));
    }

    let inv_det = 1.0 / det;
    let tvec = ray.origin - tri.v0;
    let u = dot(tvec, pvec) * inv_det;
    if (u < 0.0 || u > 1.0) {
        return HitResult(0u, 0.0, vec3f(0.0), vec3f(0.0), vec3f(0.0));
    }

    let qvec = cross(tvec, v0v1);
    let v = dot(ray.dir, qvec) * inv_det;
    if (v < 0.0 || (u + v) > 1.0) {
        return HitResult(0u, 0.0, vec3f(0.0), vec3f(0.0), vec3f(0.0));
    }

    let t = dot(v0v2, qvec) * inv_det;
    if (t < 0.001 || t > t_max) {
        return HitResult(0u, 0.0, vec3f(0.0), vec3f(0.0), vec3f(0.0));
    }

    let w = 1.0 - u - v;
    let face_cross = cross(tri.e01, tri.e02);
    if (length(face_cross) < 0.0001) {
        return HitResult(0u, 0.0, vec3f(0.0), vec3f(0.0), vec3f(0.0));
    }

    let geo_normal = normalize(face_cross);
    let normal = normalize(tri.n0 * w + tri.n1 * u + tri.n2 * v);
    let hit_point = ray.origin + t * ray.dir;

    return HitResult(1u, t, hit_point, normal, geo_normal);
}

fn eval_material(mat_type, ray, hit, albedo, fuzz, ri, scale, color_a, color_b, rngstate) -> MaterialResult {
    if(mat_type == 0u){
        // lambertian. 
       
        
    }
}