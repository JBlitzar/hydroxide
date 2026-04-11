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
