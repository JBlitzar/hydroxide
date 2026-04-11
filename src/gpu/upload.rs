use crate::{
    bluenoise::{BLUE_NOISE_LEN, BLUE_NOISE_U8, BLUE_NOISE2_U8},
    bvh::FlatBVHNode,
    camera::Camera,
    geometry::{
        mesh::{FlatMeshBVHNode, GpuTriangle, MeshBVH},
        plane::Plane,
        sphere::Sphere,
    },
    gpu::ObjectKey,
    material::{Checkerboard, Dielectric, DiffuseLight, Lambertian, Metal},
    sky::{GradientSky, HDRSky, SolidColorSky},
    world::World,
};

// mat_type constants: 0=Lambertian, 1=Metal, 2=Dielectric, 3=DiffuseLight, 4=Checkerboard

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSphere {
    pub center: [f32; 3],
    pub radius: f32,
    pub mat_type: u32,
    pub albedo: [f32; 3],
    pub fuzz: f32,
    pub ri: f32,
    pub scale: f32,
    pub _pad: f32,
    pub color_a: [f32; 3],
    pub _pad2: f32,
    pub color_b: [f32; 3],
    pub _pad3: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuMesh {
    pub bvh_offset: u32,
    pub bvh_count: u32,
    pub tri_offset: u32,
    pub tri_count: u32,
    pub mat_type: u32,
    pub albedo: [f32; 3],
    pub fuzz: f32,
    pub ri: f32,
    pub scale: f32,
    pub _pad: f32,
    pub color_a: [f32; 3],
    pub _pad2: f32,
    pub color_b: [f32; 3],
    pub _pad3: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuPlane {
    pub point: [f32; 3],
    pub _pad0: f32,
    pub normal: [f32; 3],
    pub _pad1: f32,
    pub mat_type: u32,
    pub albedo: [f32; 3],
    pub fuzz: f32,
    pub ri: f32,
    pub scale: f32,
    pub _pad2: f32,
    pub color_a: [f32; 3],
    pub _pad3: f32,
    pub color_b: [f32; 3],
    pub _pad4: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuLight {
    pub center: [f32; 3],
    pub radius: f32,
    pub le: [f32; 3],
    pub _pad: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuUniforms {
    pub width: u32,
    pub height: u32,
    pub spp: u32,
    pub current_sample: u32,
    pub cam_pos: [f32; 3],
    pub _pad1: f32,
    pub cam_forward: [f32; 3],
    pub _pad2: f32,
    pub cam_right: [f32; 3],
    pub _pad3: f32,
    pub cam_up: [f32; 3],
    pub _pad4: f32,
    pub half_tan_fov_x: f32,
    pub half_tan_fov_y: f32,
    pub focus_distance: f32,
    pub aperture: f32,
    pub sky_type: u32,
    pub _pad5: [u32; 3],
    pub sky_top: [f32; 4],
    pub sky_bottom: [f32; 4],
    pub hdr_width: u32,
    pub hdr_height: u32,
    pub hdr_exposure: f32,
    pub num_lights: u32,
    pub num_spheres: u32,
    pub num_planes: u32,
    pub num_meshes: u32,
    pub num_mesh_bvh_nodes: u32,
    pub num_triangles: u32,
    pub top_bvh_root: u32,
    pub max_depth: u32,
    pub termination_prob: f32,
}

pub struct PackedScene {
    pub uniforms: GpuUniforms,
    pub top_bvh: Vec<FlatBVHNode>,
    pub spheres: Vec<GpuSphere>,
    pub planes: Vec<GpuPlane>,
    pub mesh_bvh: Vec<FlatMeshBVHNode>,
    pub triangles: Vec<GpuTriangle>,
    pub mesh_info: Vec<GpuMesh>,
    pub lights: Vec<GpuLight>,
    pub blue_noise: Vec<[f32; 2]>,
    pub hdr_sky: Vec<[f32; 4]>,
}

struct MaterialParams {
    mat_type: u32,
    albedo: [f32; 3],
    fuzz: f32,
    ri: f32,
    scale: f32,
    color_a: [f32; 3],
    color_b: [f32; 3],
}

fn extract_material(mat: &dyn crate::material::Material) -> MaterialParams {
    let a = mat.as_any();
    if let Some(m) = a.downcast_ref::<Lambertian>() {
        MaterialParams {
            mat_type: 0,
            albedo: m.albedo.to_f32_array(),
            fuzz: 0.0,
            ri: 0.0,
            scale: 0.0,
            color_a: [0.0; 3],
            color_b: [0.0; 3],
        }
    } else if let Some(m) = a.downcast_ref::<Metal>() {
        MaterialParams {
            mat_type: 1,
            albedo: m.albedo.to_f32_array(),
            fuzz: m.fuzz as f32,
            ri: 0.0,
            scale: 0.0,
            color_a: [0.0; 3],
            color_b: [0.0; 3],
        }
    } else if let Some(m) = a.downcast_ref::<Dielectric>() {
        MaterialParams {
            mat_type: 2,
            albedo: m.albedo.to_f32_array(),
            fuzz: 0.0,
            ri: m.refractive_index as f32,
            scale: 0.0,
            color_a: [0.0; 3],
            color_b: [0.0; 3],
        }
    } else if let Some(m) = a.downcast_ref::<DiffuseLight>() {
        MaterialParams {
            mat_type: 3,
            albedo: m.albedo.to_f32_array(),
            fuzz: 0.0,
            ri: 0.0,
            scale: 0.0,
            color_a: [0.0; 3],
            color_b: [0.0; 3],
        }
    } else if let Some(m) = a.downcast_ref::<Checkerboard>() {
        MaterialParams {
            mat_type: 4,
            albedo: [0.0; 3],
            fuzz: 0.0,
            ri: 0.0,
            scale: m.scale as f32,
            color_a: m.color_a.to_f32_array(),
            color_b: m.color_b.to_f32_array(),
        }
    } else {
        panic!("Unknown material type for GPU upload");
    }
}

pub fn pack_scene(world: &World, camera: &Camera, spp: u32, max_depth: u32, termination_prob: f64) -> PackedScene {
    // 1. Flatten top-level BVH
    let (mut top_bvh, object_index_map) = world.objects().flatten();

    // 2. Walk scene_objects and build type-specific arrays.
    //    object_index in FlatBVHNode currently holds a "global" index from the flatten pass.
    //    We need to remap it to a type-specific index (sphere_idx, mesh_idx, plane_idx).
    let mut spheres: Vec<GpuSphere> = Vec::new();
    let mut planes: Vec<GpuPlane> = Vec::new();
    let mut mesh_info: Vec<GpuMesh> = Vec::new();
    let mut all_mesh_bvh: Vec<FlatMeshBVHNode> = Vec::new();
    let mut all_triangles: Vec<GpuTriangle> = Vec::new();

    // Map from global object_index -> (object_type, type_specific_index)
    let mut remap: std::collections::HashMap<u32, (u32, u32)> = std::collections::HashMap::new();

    for obj in world.scene_objects.iter() {
        let key = ObjectKey::of(obj);
        let global_idx = match object_index_map.get(&key) {
            Some(&idx) => idx,
            None => continue,
        };

        let any = obj.as_any();
        if let Some(sph) = any.downcast_ref::<Sphere>() {
            let type_idx = spheres.len() as u32;
            let mp = extract_material(sph.material.as_ref());
            spheres.push(GpuSphere {
                center: sph.center.to_f32_array(),
                radius: sph.radius as f32,
                mat_type: mp.mat_type,
                albedo: mp.albedo,
                fuzz: mp.fuzz,
                ri: mp.ri,
                scale: mp.scale,
                _pad: 0.0,
                color_a: mp.color_a,
                _pad2: 0.0,
                color_b: mp.color_b,
                _pad3: 0.0,
            });
            remap.insert(global_idx, (0, type_idx));
        } else if let Some(mesh) = any.downcast_ref::<MeshBVH>() {
            let type_idx = mesh_info.len() as u32;
            let bvh_offset = all_mesh_bvh.len() as u32;
            let tri_offset = all_triangles.len() as u32;

            let (flat_nodes, _tri_map, gpu_tris) = mesh.flatten();
            let bvh_count = flat_nodes.len() as u32;
            let tri_count = gpu_tris.len() as u32;

            all_mesh_bvh.extend_from_slice(&flat_nodes);
            all_triangles.extend_from_slice(&gpu_tris);

            let mp = extract_material(mesh.material.as_ref());
            mesh_info.push(GpuMesh {
                bvh_offset,
                bvh_count,
                tri_offset,
                tri_count,
                mat_type: mp.mat_type,
                albedo: mp.albedo,
                fuzz: mp.fuzz,
                ri: mp.ri,
                scale: mp.scale,
                _pad: 0.0,
                color_a: mp.color_a,
                _pad2: 0.0,
                color_b: mp.color_b,
                _pad3: 0.0,
            });
            remap.insert(global_idx, (1, type_idx));
        } else if let Some(plane) = any.downcast_ref::<Plane>() {
            let type_idx = planes.len() as u32;
            let mp = extract_material(plane.material.as_ref());
            planes.push(GpuPlane {
                point: plane.point.to_f32_array(),
                _pad0: 0.0,
                normal: plane.normal.to_f32_array(),
                _pad1: 0.0,
                mat_type: mp.mat_type,
                albedo: mp.albedo,
                fuzz: mp.fuzz,
                ri: mp.ri,
                scale: mp.scale,
                _pad2: 0.0,
                color_a: mp.color_a,
                _pad3: 0.0,
                color_b: mp.color_b,
                _pad4: 0.0,
            });
            remap.insert(global_idx, (2, type_idx));
        }
    }

    // Patch flat BVH leaf nodes with type-specific indices
    for node in top_bvh.iter_mut() {
        if node.is_leaf == 1 {
            if let Some(&(obj_type, type_idx)) = remap.get(&node.object_index) {
                node.object_type = obj_type;
                node.object_index = type_idx;
            }
        }
    }

    // 3. Lights
    let lights: Vec<GpuLight> = world
        .lights()
        .iter()
        .map(|l| GpuLight {
            center: l.center.to_f32_array(),
            radius: l.radius as f32,
            le: l.Le.to_f32_array(),
            _pad: 0.0,
        })
        .collect();

    // 4. Blue noise -> vec2f normalized to [0,1]
    let blue_noise: Vec<[f32; 2]> = (0..BLUE_NOISE_LEN)
        .map(|i| {
            let a = (BLUE_NOISE_U8[i] as f32 + 0.5) / 256.0;
            let b = (BLUE_NOISE2_U8[i] as f32 + 0.5) / 256.0;
            [a, b]
        })
        .collect();

    // 5. Sky 
    let sky = world.sky();
    let sky_any = sky.as_any();
    let (sky_type, sky_top, sky_bottom, hdr_sky, hdr_w, hdr_h, hdr_exp) =
        if let Some(gs) = sky_any.downcast_ref::<GradientSky>() {
            let mut top = [0.0f32; 4];
            let mut bot = [0.0f32; 4];
            let t = gs.top_color.to_f32_array();
            let b = gs.bottom_color.to_f32_array();
            top[..3].copy_from_slice(&t);
            bot[..3].copy_from_slice(&b);
            (1u32, top, bot, Vec::new(), 0u32, 0u32, 1.0f32)
        } else if let Some(hdr) = sky_any.downcast_ref::<HDRSky>() {
            let data: Vec<[f32; 4]> = hdr
                .data
                .iter()
                .map(|c| {
                    let a = c.to_f32_array();
                    [a[0], a[1], a[2], 0.0]
                })
                .collect();
            (
                2u32,
                [0.0; 4],
                [0.0; 4],
                data,
                hdr.width as u32,
                hdr.height as u32,
                hdr.exposure as f32,
            )
        } else if let Some(sc) = sky_any.downcast_ref::<SolidColorSky>() {
            let mut top = [0.0f32; 4];
            let c = sc.color.to_f32_array();
            top[..3].copy_from_slice(&c);
            (0u32, top, top, Vec::new(), 0u32, 0u32, 1.0f32)
        } else {
            (0u32, [0.0; 4], [0.0; 4], Vec::new(), 0u32, 0u32, 1.0f32)
        };

    // 6. Uniforms
    let uniforms = GpuUniforms {
        width: camera.width_px as u32,
        height: camera.height_px as u32,
        spp,
        current_sample: 0,
        cam_pos: camera.position.to_f32_array(),
        _pad1: 0.0,
        cam_forward: camera.forward.to_f32_array(),
        _pad2: 0.0,
        cam_right: camera.right.to_f32_array(),
        _pad3: 0.0,
        cam_up: camera.up.to_f32_array(),
        _pad4: 0.0,
        half_tan_fov_x: camera.half_tan_fov_x as f32,
        half_tan_fov_y: camera.half_tan_fov_y as f32,
        focus_distance: camera.focus_distance as f32,
        aperture: camera.aperture as f32,
        sky_type,
        _pad5: [0; 3],
        sky_top,
        sky_bottom,
        hdr_width: hdr_w,
        hdr_height: hdr_h,
        hdr_exposure: hdr_exp,
        num_lights: lights.len() as u32,
        num_spheres: spheres.len() as u32,
        num_planes: planes.len() as u32,
        num_meshes: mesh_info.len() as u32,
        num_mesh_bvh_nodes: all_mesh_bvh.len() as u32,
        num_triangles: all_triangles.len() as u32,
        top_bvh_root: 0,
        max_depth,
        termination_prob: termination_prob as f32,
    };

    PackedScene {
        uniforms,
        top_bvh,
        spheres,
        planes,
        mesh_bvh: all_mesh_bvh,
        triangles: all_triangles,
        mesh_info,
        lights,
        blue_noise,
        hdr_sky,
    }
}
