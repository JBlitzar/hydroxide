use crate::bvh::BVHNode;
use crate::camera::Camera;
use crate::geometry::Hittable;
use crate::geometry::mesh::MeshBVH;
use crate::material::{Checkerboard, Dielectric, Lambertian, Material, Metal};
use crate::vec3::Vec3;
use crate::world::World;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

pub use wasm_bindgen_rayon::init_thread_pool;

static TEAPOT_STL: &[u8] = include_bytes!("../teapot_fixed.stl");
// static DRAGON_STL: &[u8] = include_bytes!("../dragon.stl");

#[wasm_bindgen]
pub struct WasmRenderer {
    scene: BVHNode,
}

#[wasm_bindgen]
impl WasmRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        fastrand::seed(42);
        let mut objects: Vec<Arc<dyn Hittable>> = vec![
            Arc::new(MeshBVH::from_stl_bytes(
                TEAPOT_STL,
                Box::new(Dielectric {
                    albedo: Vec3::new(1.0, 1.0, 1.0),
                    refractive_index: 1.7,
                }),
                Some(2.0),
                Some(Vec3::new(0.0, 0.0, -5.0)),
                None,
            )),
            // Arc::new(MeshBVH::from_stl_bytes(
            //     DRAGON_STL,
            //     Box::new(Metal {
            //         albedo: Vec3::new(0.7, 1.0, 1.0),
            //         fuzz: 0.9,
            //     }),
            //     Some(2.0),
            //     Some(Vec3::new(1.0, -0.5, -5.0)),
            //     Some(Vec3::new(0.0, 0.0, 0.0)),
            // )),
            Arc::new(crate::geometry::sphere::Sphere {
                center: Vec3::new(-2.0, 0.7, -7.0),
                radius: 0.7,
                material: Box::new(Metal {
                    albedo: Vec3::new(0.7, 0.6, 0.5),
                    fuzz: 0.0,
                }),
            }),
            Arc::new(crate::geometry::sphere::Sphere {
                center: Vec3::new(2.0, 0.7, -7.0),
                radius: 0.7,
                material: Box::new(Dielectric {
                    albedo: Vec3::new(1.0, 1.0, 1.0),
                    refractive_index: 1.5,
                }),
            }),
            Arc::new(crate::geometry::sphere::Sphere {
                center: Vec3::new(0.0, 0.7, -7.0),
                radius: 0.7,
                material: Box::new(Lambertian {
                    albedo: Vec3::new(0.1, 0.1, 0.9),
                }),
            }),
            Arc::new(crate::geometry::sphere::Sphere {
                center: Vec3::new(0.0, -1000.0, 0.0),
                radius: 1000.0,
                material: Box::new(Checkerboard {
                    color_a: Vec3::new(0.0, 0.0, 0.0),
                    color_b: Vec3::new(1.0, 1.0, 1.0),
                    scale: 1.0,
                }),
            }),
        ];
        WasmRenderer {
            scene: BVHNode::of_objects_and_endpoints(&mut objects),
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
        let mut world = World::new(
            camera,
            self.scene.clone(),
            Some(samples as usize),
            Some(termination_prob),
            None,
        );
        world.render();
        world.take_buffer_rgba()
    }
}
