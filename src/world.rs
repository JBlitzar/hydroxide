use std::sync::Arc;

use crate::bvh::BVHNode;
use crate::bvh::PickHit;
use crate::camera::Camera;
use crate::geometry::Hittable;
use crate::light::SphereLight;
use crate::material::Dielectric;
use crate::material::Lambertian;
use crate::material::Material;
use crate::material::Metal;
use crate::sky::{GradientSky, Sky};
use crate::vec3::Vec3;

pub struct World {
    camera: Camera,
    objects: BVHNode,
    scene_objects: Vec<Arc<dyn Hittable>>,
    lights: Vec<SphereLight>,
    sky: Box<dyn Sky>,
    selection_mask: Vec<u8>,
}

impl World {
    pub fn new(camera: Camera, objects: Vec<Arc<dyn Hittable>>, sky: Option<Box<dyn Sky>>) -> Self {
        let width_px = camera.width_px;
        let height_px = camera.height_px;
        let extra_lights = sky.as_ref().map_or_else(Vec::new, |s| s.lights());
        let mut all_lights = SphereLight::of_mixed_objects(objects.clone());
        all_lights.extend(extra_lights);
        World {
            camera,
            objects: BVHNode::of_objects_and_endpoints(&mut objects.clone()),
            scene_objects: objects,
            lights: all_lights,
            sky: sky.unwrap_or_else(|| {
                Box::new(GradientSky {
                    top_color: Vec3::new(0.87, 0.92, 1.0),
                    bottom_color: Vec3::new(1.0, 1.0, 1.0),
                })
            }),
            selection_mask: vec![0; width_px * height_px],
        }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn objects(&self) -> &BVHNode {
        &self.objects
    }

    pub fn lights(&self) -> &[SphereLight] {
        &self.lights
    }

    pub fn sky(&self) -> &dyn Sky {
        &*self.sky
    }

    pub fn pick(&self, x: usize, y: usize) -> Option<PickHit> {
        self.objects
            .pick(&self.camera.get_ray_direction(x, y), f64::INFINITY)
    }

    pub fn pick_index(&self, x: usize, y: usize) -> Option<usize> {
        let hit = self.pick(x, y)?;
        self.scene_objects
            .iter()
            .position(|obj| Arc::ptr_eq(&hit.object, obj))
    }

    pub fn scene_object(&self, index: usize) -> Option<&Arc<dyn Hittable>> {
        self.scene_objects.get(index)
    }

    pub fn outline(&mut self, object: &Arc<dyn Hittable>, radius: usize) -> Vec<u8> {
        let w = self.camera.width_px;
        let h = self.camera.height_px;

        self.selection_mask.fill(0);
        for y in 0..h {
            for x in 0..w {
                if let Some(hit) = self.pick(x, y) {
                    if Arc::ptr_eq(&hit.object, object) {
                        self.selection_mask[y * w + x] = 255;
                    }
                }
            }
        }

        let mut outline = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                if self.selection_mask[y * w + x] != 0 {
                    continue;
                }
                'search: for ny in y.saturating_sub(radius)..=(y + radius).min(h - 1) {
                    for nx in x.saturating_sub(radius)..=(x + radius).min(w - 1) {
                        if self.selection_mask[ny * w + nx] != 0 {
                            outline[y * w + x] = 255;
                            break 'search;
                        }
                    }
                }
            }
        }
        outline
    }

    pub fn new_random_spheres(camera: Camera, num_spheres: usize) -> Self {
        let mut objects_vec: Vec<Arc<dyn Hittable>> = Vec::new();
        for _ in 0..num_spheres {
            let radius = fastrand::f64() * 0.5 + 0.1;
            let center = Vec3::new(
                fastrand::f64() * 20.0 - 10.0,
                -1.0 + radius,
                fastrand::f64() * -20.0 - 5.0,
            );

            let rand_type = fastrand::u8(0..3_u8);
            let mat: Box<dyn Material>;
            match rand_type {
                0 => {
                    mat = Box::new(Lambertian {
                        albedo: Vec3::new(fastrand::f64(), fastrand::f64(), fastrand::f64()),
                    });
                }
                1 => {
                    mat = Box::new(Metal {
                        albedo: Vec3::new(fastrand::f64(), fastrand::f64(), fastrand::f64()),
                        fuzz: fastrand::f64() * 0.5,
                    });
                }
                2 => {
                    mat = Box::new(Dielectric {
                        albedo: Vec3::new(1.0, 1.0, 1.0),
                        refractive_index: fastrand::f64() * 2.0 + 1.0,
                    });
                }
                _ => unreachable!(),
            }
            objects_vec.push(Arc::new(crate::geometry::sphere::Sphere {
                center,
                radius,
                material: mat,
            }));
        }

        let ground_material = Box::new(Lambertian {
            albedo: Vec3::new(0.5, 0.5, 0.5),
        });
        objects_vec.push(Arc::new(crate::geometry::sphere::Sphere {
            center: Vec3::new(0.0, -1001.0, -5.0),
            radius: 1000.0,
            material: ground_material,
        }));

        World::new(camera, objects_vec, None)
    }
}
