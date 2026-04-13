use crate::camera::Camera;
use crate::geometry::Hittable;
use crate::geometry::mesh::MeshBVH;
use crate::renderer::Renderer;
use crate::scene::{MaterialDesc, ObjectDesc, SceneDescription, SkyDesc};
use crate::sky::{HDRSky, Sky};
use crate::vec3::Vec3;
use crate::world::World;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use wasm_bindgen::prelude::*;

pub use wasm_bindgen_rayon::init_thread_pool;

static TEAPOT_STL: &[u8] = include_bytes!("../teapot_fixed.stl");

struct SkyEntry {
    name: &'static str,
    build: fn() -> SkyDesc,
}

const SKY_TABLE: &[SkyEntry] = &[
    SkyEntry {
        name: "Gradient (default)",
        build: || SkyDesc::Gradient {
            top: Vec3::new(0.87, 0.92, 1.0),
            bottom: Vec3::new(1.0, 1.0, 1.0),
        },
    },
    SkyEntry {
        name: "Sunset Gradient",
        build: || SkyDesc::Gradient {
            top: Vec3::new(0.1, 0.1, 0.4),
            bottom: Vec3::new(1.0, 0.4, 0.1),
        },
    },
    SkyEntry {
        name: "Solid Black",
        build: || SkyDesc::Solid {
            color: Vec3::new(0.0, 0.0, 0.0),
        },
    },
    SkyEntry {
        name: "Solid White",
        build: || SkyDesc::Solid {
            color: Vec3::new(1.0, 1.0, 1.0),
        },
    },
];

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum ObjectKind {
    Sphere,
    Cube,
    Mesh,
    Ground,
}

struct ArcSky(Arc<HDRSky>);
impl Sky for ArcSky {
    fn color(&self, ray: &crate::vec3::Ray) -> Vec3 {
        self.0.color(ray)
    }
}

#[wasm_bindgen]
pub struct WasmRenderer {
    objects: Vec<ObjectDesc>,
    kinds: Vec<ObjectKind>,
    rotations: Vec<Vec3>,
    base_verts: Vec<Option<Vec<Vec3>>>,
    sky: SkyDesc,
    hdr_sky: Option<Arc<HDRSky>>,
}

#[wasm_bindgen]
impl WasmRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        fastrand::seed(42);
        let objects = vec![
            {
                let (vertices, faces) = MeshBVH::load_stl_bytes_indexed(
                    TEAPOT_STL,
                    Some(2.0),
                    Some(Vec3::new(-2.0, 0.0, -5.0)),
                    None,
                );
                ObjectDesc::Mesh {
                    vertices,
                    faces,
                    material: MaterialDesc::Dielectric {
                        albedo: Vec3::new(1.0, 1.0, 1.0),
                        refractive_index: 1.7,
                    },
                }
            },
            ObjectDesc::Sphere {
                center: Vec3::new(2.0, 5.0, -5.0),
                radius: 2.0,
                material: MaterialDesc::DiffuseLight {
                    albedo: Vec3::new(3.0, 0.3, 0.3),
                },
            },
            ObjectDesc::Sphere {
                center: Vec3::new(-2.0, 5.0, -5.0),
                radius: 2.0,
                material: MaterialDesc::DiffuseLight {
                    albedo: Vec3::new(0.05, 3.0, 0.3),
                },
            },
            ObjectDesc::Sphere {
                center: Vec3::new(0.0, 0.7, -5.0),
                radius: 0.7,
                material: MaterialDesc::Metal {
                    albedo: Vec3::new(0.8, 0.8, 0.8),
                    fuzz: 0.0,
                },
            },
            {
                let (vertices, faces) = MeshBVH::cube_indexed(Vec3::new(2.0, 0.5, -5.0), 1.0);
                ObjectDesc::Mesh {
                    vertices,
                    faces,
                    material: MaterialDesc::Lambertian {
                        albedo: Vec3::new(0.2, 0.5, 0.5),
                    },
                }
            },
            ObjectDesc::Sphere {
                center: Vec3::new(0.0, -1000.0, 0.0),
                radius: 1000.0,
                material: MaterialDesc::Checkerboard {
                    color_a: Vec3::new(0.0, 0.0, 0.0),
                    color_b: Vec3::new(1.0, 1.0, 1.0),
                    scale: 1.0,
                },
            },
        ];
        let kinds = vec![
            ObjectKind::Mesh,   // teapot
            ObjectKind::Sphere, // red light
            ObjectKind::Sphere, // green light
            ObjectKind::Sphere, // metal sphere
            ObjectKind::Cube,   // cube
            ObjectKind::Ground, // ground checkerboard
        ];
        let rotations = vec![Vec3::ZERO; kinds.len()];
        let base_verts: Vec<Option<Vec<Vec3>>> = objects
            .iter()
            .map(|o| match o {
                ObjectDesc::Mesh { vertices, .. } => Some(Self::compute_base_verts(vertices)),
                _ => None,
            })
            .collect();
        WasmRenderer {
            objects,
            kinds,
            rotations,
            base_verts,
            sky: (SKY_TABLE[0].build)(),
            hdr_sky: None,
        }
    }

    pub fn sky_count(&self) -> u32 {
        SKY_TABLE.len() as u32
    }

    pub fn sky_name(&self, index: u32) -> String {
        SKY_TABLE
            .get(index as usize)
            .map(|e| e.name.to_string())
            .unwrap_or_default()
    }

    pub fn set_sky(&mut self, index: u32) {
        if (index as usize) < SKY_TABLE.len() {
            self.sky = (SKY_TABLE[index as usize].build)();
            self.hdr_sky = None;
        }
    }

    pub fn set_sky_hdr_bytes(&mut self, bytes: &[u8]) {
        self.hdr_sky = Some(Arc::new(HDRSky::from_hdr_bytes(bytes)));
    }

    pub fn set_sky_hdr(&mut self, hdr_index: u32, bytes: &[u8]) {
        use crate::scene::HdrSkyId;
        let id = match hdr_index {
            0 => HdrSkyId::CitrusOrchard,
            1 => HdrSkyId::QwantaniMoonrise,
            _ => return,
        };
        self.sky = SkyDesc::Hdr { id, exposure: 1.0 };
        self.hdr_sky = Some(Arc::new(HDRSky::from_hdr_bytes(bytes)));
    }

    fn build_sky_box(&self) -> Box<dyn Sky> {
        if let Some(ref hdr) = self.hdr_sky {
            return Box::new(ArcSky(Arc::clone(hdr)));
        }
        self.sky.build_sky()
    }

    fn build_world(&self, camera: Camera) -> World {
        let objects: Vec<Arc<dyn Hittable>> = self.objects.iter().map(|o| o.build()).collect();
        World::new(camera, objects, Some(self.build_sky_box()))
    }

    pub fn pick(
        &self,
        pixel_x: u32,
        pixel_y: u32,
        width: u32,
        height: u32,
        fov: f64,
        cam_x: f64,
        cam_y: f64,
        cam_z: f64,
        target_x: f64,
        target_y: f64,
        target_z: f64,
        focus_distance: f64,
        _aperture: f64,
    ) -> i32 {
        let camera = Camera::look_at(
            width as usize,
            height as usize,
            fov,
            Vec3::new(cam_x, cam_y, cam_z),
            Vec3::new(target_x, target_y, target_z),
            focus_distance,
            0.0,
        );
        let world = self.build_world(camera);
        match world.pick_index(pixel_x as usize, pixel_y as usize) {
            Some(i) if i < self.kinds.len() && self.kinds[i] != ObjectKind::Ground => i as i32,
            _ => -1,
        }
    }

    pub fn pick_distance(
        &self,
        pixel_x: u32,
        pixel_y: u32,
        width: u32,
        height: u32,
        fov: f64,
        cam_x: f64,
        cam_y: f64,
        cam_z: f64,
        target_x: f64,
        target_y: f64,
        target_z: f64,
    ) -> f64 {
        let camera = Camera::look_at(
            width as usize,
            height as usize,
            fov,
            Vec3::new(cam_x, cam_y, cam_z),
            Vec3::new(target_x, target_y, target_z),
            1.0,
            0.0,
        );
        let world = self.build_world(camera);
        world
            .pick(pixel_x as usize, pixel_y as usize)
            .map(|h| h.t)
            .unwrap_or(-1.0)
    }

    pub fn outline(
        &self,
        object_index: u32,
        width: u32,
        height: u32,
        fov: f64,
        cam_x: f64,
        cam_y: f64,
        cam_z: f64,
        target_x: f64,
        target_y: f64,
        target_z: f64,
        focus_distance: f64,
        _aperture: f64,
        radius: u32,
    ) -> Vec<u8> {
        let idx = object_index as usize;
        if idx >= self.objects.len() {
            return vec![];
        }
        let camera = Camera::look_at(
            width as usize,
            height as usize,
            fov,
            Vec3::new(cam_x, cam_y, cam_z),
            Vec3::new(target_x, target_y, target_z),
            focus_distance,
            0.0,
        );
        let mut world = self.build_world(camera);
        let obj = world.scene_object(idx).cloned();
        match obj {
            Some(o) => world.outline(&o, radius as usize),
            None => vec![],
        }
    }

    pub fn object_count(&self) -> u32 {
        self.objects.len() as u32
    }

    pub fn get_object_info(&self, index: u32) -> Vec<f64> {
        let idx = index as usize;
        if idx >= self.objects.len() {
            return vec![];
        }
        let kind = self.kinds[idx];
        let rot = self.rotations[idx];
        let obj_type_f = match kind {
            ObjectKind::Sphere | ObjectKind::Ground => 0.0,
            ObjectKind::Cube => 1.0,
            ObjectKind::Mesh => 2.0,
        };

        match &self.objects[idx] {
            ObjectDesc::Sphere {
                center,
                radius,
                material,
            } => {
                let (mt, albedo, fuzz, ri) = Self::read_material_desc(material);
                vec![
                    obj_type_f, center.x, center.y, center.z, *radius, mt, albedo.x, albedo.y,
                    albedo.z, fuzz, ri, rot.x, rot.y, rot.z,
                ]
            }
            ObjectDesc::Mesh {
                vertices, material, ..
            } => {
                let (small, big) = Self::mesh_bounds(vertices);
                let cx = (small.x + big.x) * 0.5;
                let cy = (small.y + big.y) * 0.5;
                let cz = (small.z + big.z) * 0.5;
                let size = (big.x - small.x).max(big.y - small.y).max(big.z - small.z);
                let (mt, albedo, fuzz, ri) = Self::read_material_desc(material);
                vec![
                    obj_type_f, cx, cy, cz, size, mt, albedo.x, albedo.y, albedo.z, fuzz, ri,
                    rot.x, rot.y, rot.z,
                ]
            }
            ObjectDesc::Plane { material, .. } => {
                let (mt, albedo, fuzz, ri) = Self::read_material_desc(material);
                vec![
                    obj_type_f, 0.0, 0.0, 0.0, 0.0, mt, albedo.x, albedo.y, albedo.z, fuzz, ri,
                    rot.x, rot.y, rot.z,
                ]
            }
        }
    }

    pub fn add_sphere(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        radius: f64,
        mat_type: u32,
        r: f64,
        g: f64,
        b: f64,
        fuzz: f64,
        refractive_index: f64,
    ) -> u32 {
        self.objects.push(ObjectDesc::Sphere {
            center: Vec3::new(x, y, z),
            radius,
            material: Self::make_material_desc(mat_type, r, g, b, fuzz, refractive_index),
        });
        self.kinds.push(ObjectKind::Sphere);
        self.rotations.push(Vec3::ZERO);
        self.base_verts.push(None);
        (self.objects.len() - 1) as u32
    }

    pub fn add_cube(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        size: f64,
        mat_type: u32,
        r: f64,
        g: f64,
        b: f64,
        fuzz: f64,
        refractive_index: f64,
    ) -> u32 {
        let (vertices, faces) = MeshBVH::cube_indexed(Vec3::new(x, y, z), size);
        let base = Self::compute_base_verts(&vertices);
        self.objects.push(ObjectDesc::Mesh {
            vertices,
            faces,
            material: Self::make_material_desc(mat_type, r, g, b, fuzz, refractive_index),
        });
        self.kinds.push(ObjectKind::Cube);
        self.rotations.push(Vec3::ZERO);
        self.base_verts.push(Some(base));
        (self.objects.len() - 1) as u32
    }

    pub fn remove_object(&mut self, index: u32) {
        let idx = index as usize;
        if idx < self.objects.len() {
            self.objects.remove(idx);
            self.kinds.remove(idx);
            self.rotations.remove(idx);
            self.base_verts.remove(idx);
        }
    }

    pub fn update_sphere(
        &mut self,
        index: u32,
        x: f64,
        y: f64,
        z: f64,
        radius: f64,
        mat_type: u32,
        r: f64,
        g: f64,
        b: f64,
        fuzz: f64,
        refractive_index: f64,
    ) {
        let idx = index as usize;
        if idx >= self.objects.len() {
            return;
        }
        self.objects[idx] = ObjectDesc::Sphere {
            center: Vec3::new(x, y, z),
            radius,
            material: Self::make_material_desc(mat_type, r, g, b, fuzz, refractive_index),
        };
    }

    pub fn update_cube(
        &mut self,
        index: u32,
        x: f64,
        y: f64,
        z: f64,
        size: f64,
        mat_type: u32,
        r: f64,
        g: f64,
        b: f64,
        fuzz: f64,
        refractive_index: f64,
    ) {
        let idx = index as usize;
        if idx >= self.objects.len() {
            return;
        }
        let (vertices, faces) = MeshBVH::cube_indexed(Vec3::new(x, y, z), size);
        self.objects[idx] = ObjectDesc::Mesh {
            vertices,
            faces,
            material: Self::make_material_desc(mat_type, r, g, b, fuzz, refractive_index),
        };
    }

    pub fn update_mesh_material(
        &mut self,
        index: u32,
        mat_type: u32,
        r: f64,
        g: f64,
        b: f64,
        fuzz: f64,
        refractive_index: f64,
    ) {
        let idx = index as usize;
        if idx >= self.objects.len() {
            return;
        }
        if let ObjectDesc::Mesh {
            ref mut material, ..
        } = self.objects[idx]
        {
            *material = Self::make_material_desc(mat_type, r, g, b, fuzz, refractive_index);
        }
    }

    pub fn update_mesh(
        &mut self,
        index: u32,
        new_cx: f64,
        new_cy: f64,
        new_cz: f64,
        new_size: f64,
        rot_x: f64,
        rot_y: f64,
        rot_z: f64,
        mat_type: u32,
        r: f64,
        g: f64,
        b: f64,
        fuzz: f64,
        refractive_index: f64,
    ) {
        let idx = index as usize;
        if idx >= self.objects.len() {
            return;
        }
        let new_rot = Vec3::new(rot_x, rot_y, rot_z);
        let center = Vec3::new(new_cx, new_cy, new_cz);
        let base = self.base_verts[idx].clone();
        if let ObjectDesc::Mesh {
            ref mut vertices,
            ref mut material,
            ..
        } = self.objects[idx]
        {
            if let Some(ref base) = base {
                for (v, bv) in vertices.iter_mut().zip(base.iter()) {
                    let scaled = bv.scalar_mul(new_size);
                    let rotated = scaled.rotate(&new_rot);
                    *v = rotated.add(&center);
                }
            }
            *material = Self::make_material_desc(mat_type, r, g, b, fuzz, refractive_index);
        }
        self.rotations[idx] = new_rot;
    }

    pub fn add_mesh_stl(
        &mut self,
        stl_bytes: &[u8],
        x: f64,
        y: f64,
        z: f64,
        size: f64,
        mat_type: u32,
        r: f64,
        g: f64,
        b: f64,
        fuzz: f64,
        refractive_index: f64,
    ) -> u32 {
        let (vertices, faces) =
            MeshBVH::load_stl_bytes_indexed(stl_bytes, Some(size), Some(Vec3::new(x, y, z)), None);
        let base = Self::compute_base_verts(&vertices);
        self.objects.push(ObjectDesc::Mesh {
            vertices,
            faces,
            material: Self::make_material_desc(mat_type, r, g, b, fuzz, refractive_index),
        });
        self.kinds.push(ObjectKind::Mesh);
        self.rotations.push(Vec3::ZERO);
        self.base_verts.push(Some(base));
        (self.objects.len() - 1) as u32
    }

    pub fn render(
        &self,
        width: u32,
        height: u32,
        fov: f64,
        cam_x: f64,
        cam_y: f64,
        cam_z: f64,
        target_x: f64,
        target_y: f64,
        target_z: f64,
        samples: u32,
        termination_prob: f64,
        focus_distance: f64,
        aperture: f64,
    ) -> Vec<u8> {
        let camera = Camera::look_at(
            width as usize,
            height as usize,
            fov,
            Vec3::new(cam_x, cam_y, cam_z),
            Vec3::new(target_x, target_y, target_z),
            focus_distance,
            aperture,
        );
        let world = self.build_world(camera);
        let mut renderer = Renderer::new(
            width as usize,
            height as usize,
            Some(samples as usize),
            Some(termination_prob),
        );
        renderer.set_adaptive(true);
        renderer.render(&world);
        renderer.take_buffer_rgba()
    }

    pub fn snapshot(&self) -> Vec<u8> {
        bincode::serialize(&(
            &self.objects,
            &self.kinds,
            &self.rotations,
            &self.base_verts,
            &self.sky,
        ))
        .expect("snapshot serialize")
    }

    pub fn restore(&mut self, bytes: &[u8]) {
        let (objects, kinds, rotations, base_verts, sky): (
            Vec<ObjectDesc>,
            Vec<ObjectKind>,
            Vec<Vec3>,
            Vec<Option<Vec<Vec3>>>,
            SkyDesc,
        ) = bincode::deserialize(bytes).expect("snapshot deserialize");
        self.objects = objects;
        self.kinds = kinds;
        self.rotations = rotations;
        self.base_verts = base_verts;
        self.sky = sky;
        self.hdr_sky = None;
    }

    pub fn export_scene(
        &self,
        width: u32,
        height: u32,
        fov: f64,
        cam_x: f64,
        cam_y: f64,
        cam_z: f64,
        target_x: f64,
        target_y: f64,
        target_z: f64,
        focus_distance: f64,
        aperture: f64,
        samples: u32,
        termination_prob: f64,
    ) -> Vec<u8> {
        let camera = Camera::look_at(
            width as usize,
            height as usize,
            fov,
            Vec3::new(cam_x, cam_y, cam_z),
            Vec3::new(target_x, target_y, target_z),
            focus_distance,
            aperture,
        );
        let scene = SceneDescription {
            camera,
            objects: self.objects.clone(),
            sky: self.sky.clone(),
            samples: samples as usize,
            termination_prob,
        };
        scene.to_bytes()
    }
}

impl WasmRenderer {
    fn compute_base_verts(vertices: &[Vec3]) -> Vec<Vec3> {
        let (small, big) = Self::mesh_bounds(vertices);
        let cx = (small.x + big.x) * 0.5;
        let cy = (small.y + big.y) * 0.5;
        let cz = (small.z + big.z) * 0.5;
        let size = (big.x - small.x).max(big.y - small.y).max(big.z - small.z);
        let inv = if size > 1e-12 { 1.0 / size } else { 1.0 };
        vertices
            .iter()
            .map(|v| Vec3::new((v.x - cx) * inv, (v.y - cy) * inv, (v.z - cz) * inv))
            .collect()
    }

    fn mesh_bounds(vertices: &[Vec3]) -> (Vec3, Vec3) {
        let mut small = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut big = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for v in vertices {
            small.x = small.x.min(v.x);
            small.y = small.y.min(v.y);
            small.z = small.z.min(v.z);
            big.x = big.x.max(v.x);
            big.y = big.y.max(v.y);
            big.z = big.z.max(v.z);
        }
        (small, big)
    }

    fn read_material_desc(mat: &MaterialDesc) -> (f64, Vec3, f64, f64) {
        match mat {
            MaterialDesc::Lambertian { albedo } => (0.0, *albedo, 0.0, 0.0),
            MaterialDesc::Metal { albedo, fuzz } => (1.0, *albedo, *fuzz, 0.0),
            MaterialDesc::Dielectric {
                albedo,
                refractive_index,
            } => (2.0, *albedo, 0.0, *refractive_index),
            MaterialDesc::DiffuseLight { albedo } => (3.0, *albedo, 0.0, 0.0),
            MaterialDesc::Checkerboard { color_b, .. } => (4.0, *color_b, 0.0, 0.0),
        }
    }

    fn make_material_desc(
        mat_type: u32,
        r: f64,
        g: f64,
        b: f64,
        fuzz: f64,
        refractive_index: f64,
    ) -> MaterialDesc {
        let albedo = Vec3::new(r, g, b);
        match mat_type {
            1 => MaterialDesc::Metal { albedo, fuzz },
            2 => MaterialDesc::Dielectric {
                albedo,
                refractive_index,
            },
            3 => MaterialDesc::DiffuseLight { albedo },
            4 => MaterialDesc::Checkerboard {
                color_a: Vec3::new(0.0, 0.0, 0.0),
                color_b: albedo,
                scale: 1.0,
            },
            _ => MaterialDesc::Lambertian { albedo },
        }
    }
}
