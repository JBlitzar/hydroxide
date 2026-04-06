use std::f64::consts::PI;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

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
use crate::vec3::{Ray, random_unit_vector};
use rayon::prelude::*;

#[cfg(feature = "native")]
use indicatif::{ProgressBar, ProgressStyle};

fn aces_tonemap(color: &Vec3) -> Vec3 {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    Vec3::new(
        (color.x * (a * color.x + b) / (color.x * (c * color.x + d) + e)).clamp(0.0, 1.0),
        (color.y * (a * color.y + b) / (color.y * (c * color.y + d) + e)).clamp(0.0, 1.0),
        (color.z * (a * color.z + b) / (color.z * (c * color.z + d) + e)).clamp(0.0, 1.0),
    )
}

pub struct World {
    camera: Camera,
    img_buffer: Vec<u8>,
    objects: BVHNode,
    scene_objects: Vec<Arc<dyn Hittable>>,
    termination_prob: f64,
    samples: usize,
    lights: Vec<SphereLight>,
    sky: Box<dyn Sky>,
    selection_mask: Vec<u8>,
    sample_chunk_size: usize,
    max_tolerance: f64,
}
impl World {
    pub fn new(
        camera: Camera,
        objects: Vec<Arc<dyn Hittable>>,
        samples: Option<usize>,
        termination_prob: Option<f64>,
        sky: Option<Box<dyn Sky>>,
    ) -> Self {
        let width_px = camera.width_px;
        let height_px = camera.height_px;
        let img_buffer = vec![0; width_px * height_px * 3];
        let extra_lights = sky.as_ref().map_or_else(Vec::new, |s| s.lights());
        let mut all_lights = SphereLight::of_mixed_objects(objects.clone());
        all_lights.extend(extra_lights);
        World {
            camera,
            img_buffer,
            objects: BVHNode::of_objects_and_endpoints(&mut objects.clone()),
            scene_objects: objects,
            termination_prob: termination_prob.unwrap_or(0.01),
            samples: samples.unwrap_or(20),
            lights: all_lights,
            sky: sky.unwrap_or_else(|| {
                Box::new(GradientSky {
                    top_color: Vec3::new(0.87, 0.92, 1.0),
                    bottom_color: Vec3::new(1.0, 1.0, 1.0),
                })
            }),
            selection_mask: vec![0; width_px * height_px],
            sample_chunk_size: 32,
            max_tolerance: 0.05,
        }
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

        World::new(camera, objects_vec, None, None, None)
    }

    pub fn render(&mut self) {
        let width = self.camera.width_px;
        let height = self.camera.height_px;
        let total = width * height;

        #[cfg(feature = "native")]
        let pixels: Vec<[u8; 3]> = {
            let pb = ProgressBar::new(total as u64);
            pb.set_style(
                ProgressStyle::with_template(
                    "{wide_bar} {pos}/{len} ({eta}) | ({elapsed} elapsed)",
                )
                .expect("invalid progress bar template")
                .progress_chars("#987654321-"),
            );
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            let counter = AtomicU64::new(0);
            let mut out = vec![[0u8; 3]; total];
            out.par_chunks_mut(width)
                .enumerate()
                .with_min_len(1)
                .for_each(|(y, row)| {
                    for x in 0..width {
                        row[x] = self.cast_rays_and_average(
                            x,
                            y,
                            self.samples,
                            self.sample_chunk_size,
                            self.max_tolerance,
                        );
                    }
                    let prev = counter.fetch_add(width as u64, Ordering::Relaxed);
                    pb.set_position(prev + width as u64);
                });

            pb.finish_and_clear();
            out
        };

        #[cfg(not(feature = "native"))]
        let pixels: Vec<[u8; 3]> = {
            let mut out = vec![[0u8; 3]; total];
            out.par_chunks_mut(width)
                .enumerate()
                .with_min_len(1)
                .for_each(|(y, row)| {
                    for x in 0..width {
                        row[x] = self.cast_rays_and_average(
                            x,
                            y,
                            self.samples,
                            self.sample_chunk_size,
                            self.max_tolerance,
                        );
                    }
                });
            out
        };

        for (i, pixel) in pixels.iter().enumerate() {
            let x = i % self.camera.width_px;
            let y = i / self.camera.width_px;
            self.write_pixel(x, y, *pixel);
        }
    }

    pub fn render_single_threaded(&mut self) {
        for y in 0..self.camera.height_px {
            for x in 0..self.camera.width_px {
                let color = self.cast_rays_and_average(
                    x,
                    y,
                    self.samples,
                    self.sample_chunk_size,
                    self.max_tolerance,
                );
                self.write_pixel(x, y, color);
            }
        }
    }

    pub fn cast_rays_and_average(
        &self,
        x: usize,
        y: usize,
        samples: usize,
        sample_chunk_size: usize,
        max_tolerance: f64,
    ) -> [u8; 3] {
        // https://cs184.eecs.berkeley.edu/sp21/docs/proj3-1-part-5
        let mut color_accumulator = Vec3::new(0.0, 0.0, 0.0);
        let mut s1 = 0.0;
        let mut s2 = 0.0;
        let mut n_valid: usize = 0;

        for i in 0..samples {
            let sample = self.cast_ray(x, y);
            if sample.x.is_finite()
                && sample.y.is_finite()
                && sample.z.is_finite()
                && sample.x >= 0.0
                && sample.y >= 0.0
                && sample.z >= 0.0
            {
                n_valid += 1;
                color_accumulator = color_accumulator.add(&sample);
                let illum = 0.2126 * sample.x + 0.7152 * sample.y + 0.0722 * sample.z;
                s1 += illum;
                s2 += illum * illum;
            }

            let used = i + 1;
            if sample_chunk_size > 0 && used % sample_chunk_size == 0 && n_valid > 1 {
                let n = n_valid as f64;
                let mu = s1 / n;
                let sigma_squared = (s2 - (s1 * s1) / n) / (n - 1.0);
                if mu > 0.0 && sigma_squared.is_finite() && sigma_squared >= 0.0 {
                    let ci2 = (1.96 * 1.96) * (sigma_squared / n);
                    let tol2 = (max_tolerance * mu) * (max_tolerance * mu);
                    if ci2 < tol2 {
                        break;
                    }
                }
            }
        }

        if n_valid == 0 {
            return [0, 0, 0];
        }

        let inv = 1.0 / (n_valid as f64);
        let avg = Vec3::new(
            color_accumulator.x * inv,
            color_accumulator.y * inv,
            color_accumulator.z * inv,
        );
        // ACES filmic tone mapping then gamma 2.2
        let mapped = aces_tonemap(&avg);
        [
            (mapped.x.powf(1.0 / 2.2) * 255.0).clamp(0.0, 255.0) as u8,
            (mapped.y.powf(1.0 / 2.2) * 255.0).clamp(0.0, 255.0) as u8,
            (mapped.z.powf(1.0 / 2.2) * 255.0).clamp(0.0, 255.0) as u8,
        ]
    }

    #[allow(non_snake_case)]
    pub fn cast_ray(&self, x: usize, y: usize) -> Vec3 {
        let mut beta = Vec3::new(1.0, 1.0, 1.0);
        let mut L = Vec3::ZERO;
        let mut current_ray = self.camera.get_ray_direction(x, y);
        let max_depth: u32 = 100;
        let _sky_color_top = Vec3::new(9.0 / 255.0, 19.0 / 255.0, 84.0 / 255.0);
        let _sky_color_bottom = Vec3::new(27.0 / 255.0, 11.0 / 255.0, 150.0 / 255.0);

        let mut prev_bounce_was_specular = true;

        for depth in 0..max_depth {
            if let Some(hit) = self.objects.hit(&current_ray, f64::INFINITY) {
                let Le = hit.material.emitted(&current_ray, &hit);

                if prev_bounce_was_specular {
                    L = L.add(&beta.mul(&Le));
                }

                if let Some(f_diffuse) = hit.material.eval_diffuse_brdf(&current_ray, &hit) {
                    let direct = self.estimate_direct_sphere_lights(
                        &hit.point,
                        &hit.normal,
                        &hit.geo_normal,
                        &f_diffuse,
                    );
                    L = L.add(&beta.mul(&direct));
                    prev_bounce_was_specular = false;
                } else {
                    prev_bounce_was_specular = true;
                }
                if let Some((scattered, attenuation)) = hit.material.scatter(&current_ray, &hit) {
                    current_ray = scattered;
                    beta = beta.mul(&attenuation);
                    if beta.max_component() < 1e-4 {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                let sky = self.sky.color(&current_ray);
                // return current_color.mul(&sky);
                L = L.add(&beta.mul(&sky));
                break;
            }

            if depth >= 5 {
                let p = beta
                    .max_component()
                    .clamp(self.termination_prob, 0.95)
                    .max(1e-12);
                if fastrand::f64() > p {
                    break;
                }
                beta = beta.scalar_mul(1.0 / p);
            }
        }
        L
    }
    fn write_pixel(&mut self, x: usize, y: usize, color: [u8; 3]) {
        let index = (y * self.camera.width_px + x) * 3;
        self.img_buffer[index] = color[0];
        self.img_buffer[index + 1] = color[1];
        self.img_buffer[index + 2] = color[2];
    }

    pub fn hash_buf(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.img_buffer.hash(&mut hasher);
        hasher.finish()
    }

    pub fn take_buffer_rgba(&mut self) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(self.camera.width_px * self.camera.height_px * 4);
        for chunk in self.img_buffer.chunks(3) {
            rgba.push(chunk[0]);
            rgba.push(chunk[1]);
            rgba.push(chunk[2]);
            rgba.push(255);
        }
        rgba
    }

    #[cfg(feature = "native")]
    pub fn save_image(&self, filename: &str) {
        let img = image::RgbImage::from_raw(
            self.camera.width_px as u32,
            self.camera.height_px as u32,
            self.img_buffer.clone(),
        )
        .expect("invalid image buffer size");

        img.save(filename).expect("failed to save PNG image");
    }

    fn estimate_direct_sphere_lights(
        &self,
        x: &Vec3,
        n: &Vec3,
        geo_n: &Vec3,
        f_diffuse: &Vec3,
    ) -> Vec3 {
        // I honestly just asked AI to generate this function
        let n_lights = self.lights.len();
        if n_lights == 0 {
            return Vec3::ZERO;
        }

        let light_idx = fastrand::usize(0..n_lights);
        let light = &self.lights[light_idx];
        let p_sel = 1.0 / (n_lights as f64);

        let u = random_unit_vector();
        let y = light.center.add(&u.scalar_mul(light.radius));
        let n_y = u; // normal on sphere

        let d = y.sub(x);
        let dist2 = d.length_squared();
        if dist2 <= 1e-12 {
            return Vec3::ZERO;
        }
        let dist = dist2.sqrt();
        let wi = d.scalar_mul(1.0 / dist);

        let cos_surf = n.dot(&wi).max(0.0);
        if cos_surf <= 0.0 {
            return Vec3::ZERO;
        }

        let cos_light = n_y.dot(&wi.scalar_mul(-1.0)).max(0.0);
        if cos_light <= 0.0 {
            return Vec3::ZERO;
        }

        let eps = 1e-3;
        let origin = x.add(&geo_n.scalar_mul(eps));
        let shadow_ray = Ray::new(origin, wi);
        let t_max = (dist - eps).max(0.0);
        if t_max > 0.0 && self.objects.hit(&shadow_ray, t_max).is_some() {
            return Vec3::ZERO;
        }

        let pdf_area = 1.0 / (4.0 * PI * light.radius * light.radius);
        let pdf_omega = pdf_area * dist2 / cos_light;
        let pdf = p_sel * pdf_omega;
        if pdf <= 1e-20 {
            return Vec3::ZERO;
        }

        f_diffuse.mul(&light.Le).scalar_mul(cos_surf / pdf)
    }
}
