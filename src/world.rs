use crate::vec3::Vec3;
use crate::vec3::Ray;
use fastrand;

fn random_in_unit_sphere() -> Vec3 {
    // better way, because it's rejection sampling
    let mut p = Vec3::new(0.0, 0.0, 0.0);
    loop {
        p = Vec3::new(
            fastrand::f64() * 2.0 - 1.0,
            fastrand::f64() * 2.0 - 1.0,
            fastrand::f64() * 2.0 - 1.0,
        );
        if p.length_squared() < 1.0 {
            break;
        }
    }
    p
}

pub(crate) struct Camera {
    width_px: usize,
    height_px: usize,
    x_fov: f64,
    y_fov: f64,
    position: Vec3,
    euler_angles: Vec3,
    half_tan_fov_x: f64,
    half_tan_fov_y: f64,
}
impl Camera {
    pub fn new(width_px: usize, height_px: usize, x_fov: f64, position: Vec3, euler_angles: Vec3) -> Self {
        
        let half_tan_fov_x = (x_fov / 2.0).tan();
        let half_tan_fov_y = half_tan_fov_x * (height_px as f64 / width_px as f64);
        let y_fov = 2.0 * half_tan_fov_y.atan();
        Camera {
            width_px,
            height_px,
            x_fov,
            y_fov,
            position,
            euler_angles,
            half_tan_fov_x: half_tan_fov_x,
            half_tan_fov_y: half_tan_fov_y
        }
    }

    pub fn get_ray_direction(&self, x: usize, y: usize) -> Ray {
        let x_cmp = (x as f64 / self.width_px as f64 - 0.5) * self.half_tan_fov_x;
        let y_cmp = (y as f64 / self.height_px as f64 - 0.5) * self.half_tan_fov_y;
        Ray::new(self.position, Vec3::new(x_cmp, y_cmp, -1.0).normalize().rotate(&self.euler_angles))
    }

}

pub(crate) trait Material {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Ray, Vec3)>;

}

pub(crate) struct Lambertian {
    pub albedo: Vec3, // color lol
}
impl Material for Lambertian {

    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Ray, Vec3)> {

        let mut random_in_unit_hemisphere = random_in_unit_sphere();
        if random_in_unit_hemisphere.dot(&hit_record.normal) < 0.0 {
            random_in_unit_hemisphere = random_in_unit_hemisphere.scalar_mul(-1.0);
        }
        
        let target = hit_record.point.add(&hit_record.normal).add(&random_in_unit_hemisphere);
        
        Some((Ray::new(hit_record.point, target.sub(&hit_record.point)), self.albedo))
    }

}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Metal {
    pub albedo: Vec3,
    pub fuzz: f64,

}
impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Ray, Vec3)> {
        let reflected_dir = ray_in.direction.sub(&hit_record.normal.scalar_mul(2.0 * ray_in.direction.dot(&hit_record.normal)));
        let fuzz = random_in_unit_sphere().scalar_mul(self.fuzz);

        Some((Ray::new(hit_record.point, reflected_dir.add(&fuzz)), self.albedo))
    }
}

// pub struct Glass {
//     pub albedo: Vec3,
//     pub refractive_index: f64,
// }
// impl Material for Glass {
//     fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Ray, Vec3)> {
        

//     }
// }

struct HitRecord<'a> {
    point: Vec3,
    normal: Vec3,
    material: &'a dyn Material,
    t: f64,
}


pub(crate) trait Hittable {
    fn hit(&self, ray: &Ray) -> Option<HitRecord>;
}

pub(crate) struct Sphere {
    pub(crate) center: Vec3,
    pub(crate) radius: f64,
    pub(crate) material: Box<dyn Material>,
}

impl Hittable for Sphere {

    fn hit(&self, ray: &Ray) -> Option<HitRecord> {
        // https://raytracing.github.io/books/RayTracingInOneWeekend.html#surfacenormalsandmultipleobjects/simplifyingtheray-sphereintersectioncode
        let oc = ray.origin.sub(&self.center);
        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * oc.dot(&ray.direction);
        let c = oc.dot(&oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            None
        } else {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t < 0.001 {
                return None;
            }
            let point = ray.origin.add(&ray.direction.scalar_mul(t));
            let normal = (point.sub(&self.center)).normalize();
            Some(HitRecord { point, normal, material: self.material.as_ref(), t })
        }
    }

}

pub(crate) struct HittableList {
    pub(crate) objs: Vec<Box<dyn Hittable>>,
}
impl HittableList {
    pub fn new() -> Self {
        HittableList { objs: Vec::new() }
    }

    pub fn add(&mut self, obj: Box<dyn Hittable>) {
        self.objs.push(obj);
    }

    pub fn hit(&self, ray: &Ray) -> Option<HitRecord> {
        // TODO some fancy binary search tree by bounding boxes or something (BVH)

        let mut closest_hit: Option<HitRecord> = None;
        for obj in &self.objs {
            if let Some(hit) = obj.hit(ray) {
                if closest_hit.is_none() || hit.t < closest_hit.as_ref().unwrap().t {
                    closest_hit = Some(hit);
                }
            }
        }
        closest_hit
    }
}

pub struct World {
    // one world has one camera
    camera: Camera,
    img_buffer: Vec<u8>,
    objects: HittableList,
    depth: usize,
    samples: usize,
}

impl World {
    pub fn new(camera: Camera, objects: HittableList) -> Self {
        let img_buffer = vec![0; camera.width_px * camera.height_px * 3];
        World { camera, img_buffer, objects, depth: 5, samples: 100}
    }

    pub fn render(&mut self) {
        for y in 0..self.camera.height_px {
            for x in 0..self.camera.width_px {
                let pixel = self.cast_rays_and_average(x, y, self.samples);
                self.write_pixel(x, y, pixel);
            }
        }
    }

    pub fn cast_rays_and_average(&self, x: usize, y: usize, samples: usize) -> [u8; 3] {
        let mut color_accumulator = Vec3::new(0.0, 0.0, 0.0);
        for _ in 0..samples {
            color_accumulator = color_accumulator.add(&self.cast_ray(x, y));
        }
        [
            (color_accumulator.x / samples as f64 * 255.0).clamp(0.0, 255.0) as u8,
            (color_accumulator.y / samples as f64 * 255.0).clamp(0.0, 255.0) as u8,
            (color_accumulator.z / samples as f64 * 255.0).clamp(0.0, 255.0) as u8,
        ]
    }

    pub fn cast_ray(&self, x: usize, y: usize) -> Vec3 {
        let mut current_ray = self.camera.get_ray_direction(x, y);
        let mut current_color = Vec3::new(1.0, 1.0, 1.0);
        for _ in 0..self.depth {
            if let Some(hit) = self.objects.hit(&current_ray) {
                if let Some((scattered, attenuation)) = hit.material.scatter(&current_ray, &hit) {
                    current_ray = scattered;
                    current_color = current_color.mul(&attenuation);
                } else {
                    return Vec3::ZERO;
                }
            } else {
                let unit_dir = current_ray.direction.normalize();
                let t = 0.5 * (unit_dir.y + 1.0);
                let sky = Vec3::new(1.0, 1.0, 1.0).scalar_mul(1.0 - t)
                    .add(&Vec3::new(0.5, 0.7, 1.0).scalar_mul(t));
                return current_color.mul(&sky);
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
}