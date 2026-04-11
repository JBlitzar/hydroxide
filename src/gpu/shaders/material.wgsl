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
    loop {
        let p = vec3f(
            rand_f32(rng) * 2.0 - 1.0,
            rand_f32(rng) * 2.0 - 1.0,
            rand_f32(rng) * 2.0 - 1.0,
        );
        if dot(p, p) < 1.0 {
            return p;
        }
    }
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
