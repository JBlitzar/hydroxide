use crate::camera::Camera;
use crate::geometry::Hittable;
use crate::geometry::mesh::MeshBVH;
use crate::geometry::*;
use crate::material::DiffuseLight;
use crate::material::{Checkerboard, Dielectric, Lambertian, Material, Metal};
use crate::renderer::Renderer;
use crate::sky::{GradientSky, HDRSky, Sky, SolidColorSky};
use crate::vec3::Vec3;
use crate::world::World;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

pub use wasm_bindgen_rayon::init_thread_pool;

static TEAPOT_STL: &[u8] = include_bytes!("../teapot_fixed.stl");

struct ArcSky(Arc<HDRSky>);
impl Sky for ArcSky {
    fn color(&self, ray: &crate::vec3::Ray) -> Vec3 {
        self.0.color(ray)
    }
}

struct SkyEntry {
    name: &'static str,
    build: fn() -> Box<dyn Sky>,
}

const SKY_TABLE: &[SkyEntry] = &[
    SkyEntry {
        name: "Gradient (default)",
        build: || {
            Box::new(GradientSky {
                top_color: Vec3::new(0.87, 0.92, 1.0),
                bottom_color: Vec3::new(1.0, 1.0, 1.0),
            })
        },
    },
    SkyEntry {
        name: "Sunset Gradient",
        build: || {
            Box::new(GradientSky {
                top_color: Vec3::new(0.1, 0.1, 0.4),
                bottom_color: Vec3::new(1.0, 0.4, 0.1),
            })
        },
    },
    SkyEntry {
        name: "Solid Black",
        build: || {
            Box::new(SolidColorSky {
                color: Vec3::new(0.0, 0.0, 0.0),
            })
        },
    },
    SkyEntry {
        name: "Solid White",
        build: || {
            Box::new(SolidColorSky {
                color: Vec3::new(1.0, 1.0, 1.0),
            })
        },
    },
];

#[derive(Clone, Copy, PartialEq)]
enum ObjectKind {
    Sphere,
    Cube,
    Mesh,
    Ground,
}

#[wasm_bindgen]
pub struct WasmRenderer {
    scene: Vec<Arc<dyn Hittable>>,
    kinds: Vec<ObjectKind>,
    sky_index: usize,
    hdr_sky: Option<Arc<HDRSky>>,
}

#[wasm_bindgen]
impl WasmRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        fastrand::seed(42);
        let objects: Vec<Arc<dyn Hittable>> = vec![
            Arc::new(MeshBVH::from_stl_bytes(
                TEAPOT_STL,
                Box::new(Dielectric {
                    albedo: Vec3::new(1.0, 1.0, 1.0),
                    refractive_index: 1.7,
                }),
                Some(2.0),
                Some(Vec3::new(-2.0, 0.0, -5.0)),
                None,
            )),
            Arc::new(sphere::Sphere {
                center: Vec3::new(2.0, 5.0, -5.0),
                radius: 2.0,
                material: Box::new(DiffuseLight {
                    albedo: Vec3::new(3.0, 0.3, 0.3),
                }),
            }),
            Arc::new(sphere::Sphere {
                center: Vec3::new(-2.0, 5.0, -5.0),
                radius: 2.0,
                material: Box::new(DiffuseLight {
                    albedo: Vec3::new(0.05, 3.0, 0.3),
                }),
            }),
            Arc::new(sphere::Sphere {
                center: Vec3::new(0.0, 0.7, -5.0),
                radius: 0.7,
                material: Box::new(Metal {
                    albedo: Vec3::new(0.8, 0.8, 0.8),
                    fuzz: 0.0,
                }),
            }),
            Arc::new(MeshBVH::build_cube(
                Vec3::new(2.0, 0.5, -5.0),
                1.0,
                Box::new(Lambertian {
                    albedo: Vec3::new(0.2, 0.5, 0.5),
                }),
            )),
            Arc::new(sphere::Sphere {
                center: Vec3::new(0.0, -1000.0, 0.0),
                radius: 1000.0,
                material: Box::new(Checkerboard {
                    color_a: Vec3::new(0.0, 0.0, 0.0),
                    color_b: Vec3::new(1.0, 1.0, 1.0),
                    scale: 1.0,
                }),
            }),
        ];
        let kinds = vec![
            ObjectKind::Mesh,   // teapot
            ObjectKind::Sphere, // red light
            ObjectKind::Sphere, // green light
            ObjectKind::Sphere, // metal sphere
            ObjectKind::Cube,   // cube
            ObjectKind::Ground, // ground checkerboard
        ];
        WasmRenderer {
            scene: objects,
            kinds,
            sky_index: 0,
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
            self.sky_index = index as usize;
            self.hdr_sky = None;
        }
    }

    pub fn set_sky_hdr_bytes(&mut self, bytes: &[u8]) {
        self.hdr_sky = Some(Arc::new(HDRSky::from_hdr_bytes(bytes)));
    }

    fn build_sky(&self) -> Box<dyn Sky> {
        if let Some(ref hdr) = self.hdr_sky {
            return Box::new(ArcSky(Arc::clone(hdr)));
        }
        (SKY_TABLE[self.sky_index].build)()
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
        let world = World::new(camera, self.scene.clone(), Some(self.build_sky()));
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
        let world = World::new(camera, self.scene.clone(), Some(self.build_sky()));
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
        if idx >= self.scene.len() {
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
        let mut world = World::new(camera, self.scene.clone(), Some(self.build_sky()));
        let obj = world.scene_object(idx).cloned();
        match obj {
            Some(o) => world.outline(&o, radius as usize),
            None => vec![],
        }
    }

    pub fn object_count(&self) -> u32 {
        self.scene.len() as u32
    }

    pub fn get_object_info(&self, index: u32) -> Vec<f64> {
        let idx = index as usize;
        if idx >= self.scene.len() {
            return vec![];
        }
        let obj = &self.scene[idx];
        let kind = self.kinds[idx];
        let obj_type_f = match kind {
            ObjectKind::Sphere | ObjectKind::Ground => 0.0,
            ObjectKind::Cube => 1.0,
            ObjectKind::Mesh => 2.0,
        };

        if let Some(s) = obj.as_any().downcast_ref::<sphere::Sphere>() {
            let (mt, albedo, fuzz, ri) = Self::read_material(&*s.material);
            return vec![
                obj_type_f, s.center.x, s.center.y, s.center.z, s.radius, mt as f64, albedo.x,
                albedo.y, albedo.z, fuzz, ri,
            ];
        }

        if let Some(m) = obj.as_any().downcast_ref::<MeshBVH>() {
            let bb = obj.bounding_box();
            let cx = (bb.min.x + bb.max.x) * 0.5;
            let cy = (bb.min.y + bb.max.y) * 0.5;
            let cz = (bb.min.z + bb.max.z) * 0.5;
            let size = (bb.max.x - bb.min.x)
                .max(bb.max.y - bb.min.y)
                .max(bb.max.z - bb.min.z);
            let (mt, albedo, fuzz, ri) = Self::read_material(&*m.material);
            return vec![
                obj_type_f, cx, cy, cz, size, mt as f64, albedo.x, albedo.y, albedo.z, fuzz, ri,
            ];
        }

        vec![]
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
        let mat = Self::make_material(mat_type, r, g, b, fuzz, refractive_index);
        self.scene.push(Arc::new(sphere::Sphere {
            center: Vec3::new(x, y, z),
            radius,
            material: mat,
        }));
        self.kinds.push(ObjectKind::Sphere);
        (self.scene.len() - 1) as u32
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
        let mat = Self::make_material(mat_type, r, g, b, fuzz, refractive_index);
        self.scene
            .push(Arc::new(MeshBVH::build_cube(Vec3::new(x, y, z), size, mat)));
        self.kinds.push(ObjectKind::Cube);
        (self.scene.len() - 1) as u32
    }

    pub fn remove_object(&mut self, index: u32) {
        let idx = index as usize;
        if idx < self.scene.len() {
            self.scene.remove(idx);
            self.kinds.remove(idx);
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
        if idx >= self.scene.len() {
            return;
        }
        let mat = Self::make_material(mat_type, r, g, b, fuzz, refractive_index);
        self.scene[idx] = Arc::new(sphere::Sphere {
            center: Vec3::new(x, y, z),
            radius,
            material: mat,
        });
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
        if idx >= self.scene.len() {
            return;
        }
        let mat = Self::make_material(mat_type, r, g, b, fuzz, refractive_index);
        self.scene[idx] = Arc::new(MeshBVH::build_cube(Vec3::new(x, y, z), size, mat));
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
        if idx >= self.scene.len() {
            return;
        }
        let any = self.scene[idx].as_any();
        if let Some(mesh) = any.downcast_ref::<MeshBVH>() {
            let mat = Self::make_material(mat_type, r, g, b, fuzz, refractive_index);
            self.scene[idx] = Arc::new(mesh.with_material(mat));
        }
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
        let world = World::new(camera, self.scene.clone(), Some(self.build_sky()));
        let mut renderer = Renderer::new(
            width as usize,
            height as usize,
            Some(samples as usize),
            Some(termination_prob),
        );
        renderer.render(&world);
        renderer.take_buffer_rgba()
    }
}

impl WasmRenderer {
    fn read_material(mat: &dyn Material) -> (u32, Vec3, f64, f64) {
        let any = mat.as_any();
        if let Some(m) = any.downcast_ref::<Lambertian>() {
            return (0, m.albedo, 0.0, 0.0);
        }
        if let Some(m) = any.downcast_ref::<Metal>() {
            return (1, m.albedo, m.fuzz, 0.0);
        }
        if let Some(m) = any.downcast_ref::<Dielectric>() {
            return (2, m.albedo, 0.0, m.refractive_index);
        }
        if let Some(m) = any.downcast_ref::<DiffuseLight>() {
            return (3, m.albedo, 0.0, 0.0);
        }
        if let Some(m) = any.downcast_ref::<Checkerboard>() {
            return (4, m.color_b, 0.0, 0.0);
        }
        (0, Vec3::new(0.5, 0.5, 0.5), 0.0, 0.0)
    }

    fn make_material(
        mat_type: u32,
        r: f64,
        g: f64,
        b: f64,
        fuzz: f64,
        refractive_index: f64,
    ) -> Box<dyn Material> {
        let albedo = Vec3::new(r, g, b);
        match mat_type {
            1 => Box::new(Metal { albedo, fuzz }),
            2 => Box::new(Dielectric {
                albedo,
                refractive_index,
            }),
            3 => Box::new(DiffuseLight { albedo }),
            4 => Box::new(Checkerboard {
                color_a: Vec3::new(0.0, 0.0, 0.0),
                color_b: albedo,
                scale: 1.0,
            }),
            _ => Box::new(Lambertian { albedo }),
        }
    }
}
