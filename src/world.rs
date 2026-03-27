use std::mem;

use crate::camera::Camera;
use crate::geometry::HittableList;
use crate::material::Dielectric;
use crate::material::Lambertian;
use crate::material::Material;
use crate::material::MaterialType;
use crate::material::Metal;
use crate::vec3::Ray;
use crate::vec3::Vec3;
use fastrand;
use rayon::prelude::*;
use stl_io;

pub struct World {
    // one world has one camera
    camera: Camera,
    img_buffer: Vec<u8>,
    objects: HittableList,
    termination_prob: f64,
    samples: usize,
}

impl World {
    pub fn new(camera: Camera, objects: HittableList) -> Self {
        let img_buffer = vec![0; camera.width_px * camera.height_px * 3];
        World {
            camera,
            img_buffer,
            objects,
            termination_prob: 0.1,
            samples: 20,
        }
    }
    pub fn new_random_spheres(camera: Camera, num_spheres: usize) -> Self {
        let mut objects = HittableList::new();
        for _ in 0..num_spheres {
            let radius = fastrand::f64() * 0.5 + 0.1;
            let center = Vec3::new(
                fastrand::f64() * 20.0 - 10.0,
                -1.0 + radius,
                fastrand::f64() * -20.0 - 5.0,
            );

            let rand_type = fastrand::u8(0..3 as u8);
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
            objects.add(Box::new(crate::geometry::sphere::Sphere {
                center: center,
                radius: radius,
                material: mat,
            }));
        }

        let ground_material = Box::new(Lambertian {
            albedo: Vec3::new(0.5, 0.5, 0.5),
        });
        objects.add(Box::new(crate::geometry::sphere::Sphere {
            center: Vec3::new(0.0, -1001.0, -5.0),
            radius: 1000.0,
            material: ground_material,
        }));

        return World::new(camera, objects);
    }

    pub fn render(&mut self) {
        let pixels: Vec<[u8; 3]> = (0..self.camera.height_px * self.camera.width_px)
            .into_par_iter()
            .map(|i| {
                let x = i % self.camera.width_px;
                let y = i / self.camera.width_px;
                self.cast_rays_and_average(x, y, self.samples)
            })
            .collect();

        for (i, pixel) in pixels.iter().enumerate() {
            let x = i % self.camera.width_px;
            let y = i / self.camera.width_px;
            self.write_pixel(x, y, *pixel);
        }
    }

    pub fn cast_rays_and_average(&self, x: usize, y: usize, samples: usize) -> [u8; 3] {
        let mut color_accumulator = Vec3::new(0.0, 0.0, 0.0);
        for _ in 0..samples {
            color_accumulator = color_accumulator.add(&self.cast_ray(x, y));
        }
        [
            ((color_accumulator.x / samples as f64).sqrt() * 255.0).clamp(0.0, 255.0) as u8,
            ((color_accumulator.y / samples as f64).sqrt() * 255.0).clamp(0.0, 255.0) as u8,
            ((color_accumulator.z / samples as f64).sqrt() * 255.0).clamp(0.0, 255.0) as u8,
        ]
    }

    pub fn cast_ray(&self, x: usize, y: usize) -> Vec3 {
        let mut current_ray = self.camera.get_ray_direction(x, y);
        let mut current_color = Vec3::new(1.0, 1.0, 1.0);
        loop {
            if let Some(hit) = self.objects.hit(&current_ray) {
                if let Some((scattered, attenuation)) = hit.material.scatter(&current_ray, &hit) {
                    current_ray = scattered;
                    current_color = current_color.mul(&attenuation);
                    if current_color.max_component() < 0.01 {
                        return Vec3::ZERO;
                    }
                } else {
                    return Vec3::ZERO;
                }
            } else {
                let unit_dir = current_ray.direction.normalize();
                let t = 0.5 * (unit_dir.y + 1.0);
                let sky = Vec3::new(1.0, 1.0, 1.0)
                    .scalar_mul(1.0 - t)
                    .add(&Vec3::new(0.5, 0.7, 1.0).scalar_mul(t));
                return current_color.mul(&sky);
            }
            if fastrand::f64() < self.termination_prob {
                break;
            }
        }
        Vec3::ZERO
    }
    fn write_pixel(&mut self, x: usize, y: usize, color: [u8; 3]) {
        let index = (y * self.camera.width_px + x) * 3;
        self.img_buffer[index] = color[0];
        self.img_buffer[index + 1] = color[1];
        self.img_buffer[index + 2] = color[2];
    }

    pub fn save_image(&self, filename: &str) {
        let img = image::RgbImage::from_raw(
            self.camera.width_px as u32,
            self.camera.height_px as u32,
            self.img_buffer.clone(),
        )
        .expect("invalid image buffer size");

        img.save(filename).expect("failed to save PNG image");
    }

    pub fn hash_buf(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.img_buffer.hash(&mut hasher);
        hasher.finish()
    }
}
