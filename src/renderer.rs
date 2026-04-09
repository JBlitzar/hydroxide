use std::f64::consts::PI;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::vec3::Vec3;
use crate::vec3::{Ray, random_hemisphere};
use crate::world::World;
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

pub struct Renderer {
    img_buffer: Vec<u8>,
    width: usize,
    height: usize,
    samples: usize,
    sample_chunk_size: usize,
    max_tolerance: f64,
    termination_prob: f64,
}

impl Renderer {
    pub fn new(
        width: usize,
        height: usize,
        samples: Option<usize>,
        termination_prob: Option<f64>,
    ) -> Self {
        Renderer {
            img_buffer: vec![0; width * height * 3],
            width,
            height,
            samples: samples.unwrap_or(20),
            sample_chunk_size: 32,
            max_tolerance: 0.05,
            termination_prob: termination_prob.unwrap_or(0.01),
        }
    }

    pub fn render(&mut self, world: &World) {
        let width = self.width;
        let height = self.height;
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
                        row[x] = self.cast_rays_and_average(world, x, y);
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
                        row[x] = self.cast_rays_and_average(world, x, y);
                    }
                });
            out
        };

        for (i, pixel) in pixels.iter().enumerate() {
            let x = i % width;
            let y = i / width;
            self.write_pixel(x, y, *pixel);
        }

        self.despeckle();
    }

    pub fn render_single_threaded(&mut self, world: &World) {
        for y in 0..self.height {
            for x in 0..self.width {
                let color = self.cast_rays_and_average(world, x, y);
                self.write_pixel(x, y, color);
            }
        }
    }

    pub fn cast_rays_and_average(&self, world: &World, x: usize, y: usize) -> [u8; 3] {
        let mut color_accumulator = Vec3::new(0.0, 0.0, 0.0);
        let mut s1 = 0.0;
        let mut s2 = 0.0;
        let mut n_valid: usize = 0;

        for i in 0..self.samples {
            let sample = self.cast_ray(world, x, y);
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
            if self.sample_chunk_size > 0 && used % self.sample_chunk_size == 0 && n_valid > 1 {
                let n = n_valid as f64;
                let mu = s1 / n;
                let sigma_squared = (s2 - (s1 * s1) / n) / (n - 1.0);
                if mu > 0.0 && sigma_squared.is_finite() && sigma_squared >= 0.0 {
                    const z: f64 = 2.576; // 1.96
                    const absolute_floor: f64 = 0.01;
                    let ci2 = (z * z) * (sigma_squared / n);
                    let tol2 = (self.max_tolerance * mu).max(absolute_floor)
                        * (self.max_tolerance * mu).max(absolute_floor);
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
        let mapped = aces_tonemap(&avg);
        [
            (mapped.x.powf(1.0 / 2.2) * 255.0).clamp(0.0, 255.0) as u8,
            (mapped.y.powf(1.0 / 2.2) * 255.0).clamp(0.0, 255.0) as u8,
            (mapped.z.powf(1.0 / 2.2) * 255.0).clamp(0.0, 255.0) as u8,
        ]
    }

    #[allow(non_snake_case)]
    pub fn cast_ray(&self, world: &World, x: usize, y: usize) -> Vec3 {
        let mut beta = Vec3::new(1.0, 1.0, 1.0);
        let mut L = Vec3::ZERO;
        let mut current_ray = world.camera().get_ray_direction(x, y);
        let max_depth: u32 = 100;

        let mut prev_bounce_was_specular = true;

        for depth in 0..max_depth {
            if let Some(hit) = world.objects().hit(&current_ray, f64::INFINITY) {
                let Le = hit.material.emitted(&current_ray, &hit);

                if prev_bounce_was_specular {
                    L = L.add(&beta.mul(&Le));
                }

                if let Some(f_diffuse) = hit.material.eval_diffuse_brdf(&current_ray, &hit) {
                    let direct = self.estimate_direct_sphere_lights(
                        world,
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
                let sky = world.sky().color(&current_ray);
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
        let index = (y * self.width + x) * 3;
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
        let mut rgba = Vec::with_capacity(self.width * self.height * 4);
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
            self.width as u32,
            self.height as u32,
            self.img_buffer.clone(),
        )
        .expect("invalid image buffer size");

        img.save(filename).expect("failed to save PNG image");
    }

    fn estimate_direct_sphere_lights(
        &self,
        world: &World,
        x: &Vec3,
        n: &Vec3,
        geo_n: &Vec3,
        f_diffuse: &Vec3,
    ) -> Vec3 {
        let n_lights = world.lights().len();
        if n_lights == 0 {
            return Vec3::ZERO;
        }

        let light_idx = fastrand::usize(0..n_lights);
        let light = &world.lights()[light_idx];
        let p_sel = 1.0 / (n_lights as f64);

        let toward_x = x.sub(&light.center).normalize();
        let u = random_hemisphere(&toward_x);
        let y = light.center.add(&u.scalar_mul(light.radius));
        let n_y = u;

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
        if t_max > 0.0 && world.objects().hit(&shadow_ray, t_max).is_some() {
            return Vec3::ZERO;
        }

        let pdf_area = 1.0 / (2.0 * PI * light.radius * light.radius);
        let pdf_omega = pdf_area * dist2 / cos_light;
        let pdf = p_sel * pdf_omega;
        if pdf <= 1e-20 {
            return Vec3::ZERO;
        }

        f_diffuse.mul(&light.Le).scalar_mul(cos_surf / pdf)
    }

    fn despeckle(&mut self) {
        for x in 1..self.height - 1 {
            for y in 1..self.width - 1 {
                let idx = (x * self.width + y) * 3;
                let current: f64 = self.img_buffer[idx] as f64 / 3.0
                    + self.img_buffer[idx + 1] as f64 / 3.0
                    + self.img_buffer[idx + 2] as f64 / 3.0;

                let mut avg_neighbor: Vec3 = Vec3::ZERO;
                let mut min_neighbor_brightness = f64::INFINITY;
                let mut max_neighbor_brightness = f64::NEG_INFINITY;
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let neighbor_idx = ((x as isize + dx) as usize * self.width
                            + (y as isize + dy) as usize)
                            * 3;
                        let neighbor: f64 = self.img_buffer[neighbor_idx] as f64 / 3.0
                            + self.img_buffer[neighbor_idx + 1] as f64 / 3.0
                            + self.img_buffer[neighbor_idx + 2] as f64 / 3.0;

                        avg_neighbor = avg_neighbor.add(&Vec3::new(
                            self.img_buffer[neighbor_idx] as f64,
                            self.img_buffer[neighbor_idx + 1] as f64,
                            self.img_buffer[neighbor_idx + 2] as f64,
                        ));
                        if neighbor < min_neighbor_brightness {
                            min_neighbor_brightness = neighbor;
                        }
                        if neighbor > max_neighbor_brightness {
                            max_neighbor_brightness = neighbor;
                        }
                    }
                }
                avg_neighbor = avg_neighbor.scalar_mul(1.0 / 8.0);

                if current < min_neighbor_brightness * 0.8
                    || current > max_neighbor_brightness * 1.2
                {
                    self.img_buffer[idx] = avg_neighbor.x as u8;
                    self.img_buffer[idx + 1] = avg_neighbor.y as u8;
                    self.img_buffer[idx + 2] = avg_neighbor.z as u8;
                }
            }
        }
    }
}
