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
