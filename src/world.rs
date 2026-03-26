use crate::vec3::Vec3;
use crate::vec3::Ray;
use fastrand;
use stl_io;

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
        let x_cmp = ((x as f64 + fastrand::f64()) / self.width_px as f64 - 0.5) * self.half_tan_fov_x ;
        let y_cmp = (0.5 - (y as f64  + fastrand::f64())/ self.height_px as f64) * self.half_tan_fov_y;
        Ray::new(self.position, Vec3::new(x_cmp, y_cmp, -1.0).normalize().rotate(&self.euler_angles))
    }

}

pub(crate) trait Material {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Ray, Vec3)>;

}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
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
        let reflected_dir = reflect(&ray_in.direction, &hit_record.normal);
        let fuzz = random_in_unit_sphere().scalar_mul(self.fuzz);

        Some((Ray::new(hit_record.point, reflected_dir.add(&fuzz)), self.albedo))
    }
}

fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    v.sub(&n.scalar_mul(2.0 * v.dot(n)))
}
fn refract(uv: &Vec3, n: &Vec3, etai_over_etat: f64) -> Vec3 {
    let cos_theta = uv.scalar_mul(-1.0).dot(n).min(1.0);
    let r_out_perp = uv.add(&n.scalar_mul(cos_theta)).scalar_mul(etai_over_etat);
    let r_out_parallel = n.scalar_mul(-((1.0 - r_out_perp.length_squared()).abs().sqrt()));
    r_out_perp.add(&r_out_parallel)
}

pub struct Dielectric {
    pub albedo: Vec3,
    pub refractive_index: f64,
}

impl Dielectric {
    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        // schlick
        let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        let r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}
impl Material for Dielectric {
    // https://raytracing.github.io/books/RayTracingInOneWeekend.html#dielectrics
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Ray, Vec3)> {

        let attenuation = self.albedo;
        let mut ri = self.refractive_index;

        // if front face 
        if ray_in.direction.dot(&hit_record.normal) < 0.0{
            ri = 1.0 / ri;
        }

        let unit_direction = ray_in.direction.normalize();

        let cos_theta = unit_direction.scalar_mul(-1.0).dot(&hit_record.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;

        let direction: Vec3;
        if cannot_refract || Dielectric::reflectance(cos_theta, ri) > fastrand::f64() {
            direction = reflect(&unit_direction, &hit_record.normal);
        } else {
            direction = refract(&unit_direction, &hit_record.normal, ri);
        }

        let scattered = Ray::new(hit_record.point, direction);

        Some((scattered, attenuation))
        
    }
}

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

pub struct Triangle {
    pub(crate) v0: Vec3,
    pub(crate) v1: Vec3,
    pub(crate) v2: Vec3,
    pub normal: Vec3,
    pub e1: Vec3,
    pub e2: Vec3,
}
impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        let e1 = v1.sub(&v0);
        let e2 = v2.sub(&v0);
        let normal = e1.cross(&e2).normalize();
        Triangle { v0, v1, v2, normal, e1, e2 }
    }

    
    fn hit<'a>(&self, ray: &Ray, material: &'a dyn Material) -> Option<HitRecord<'a>> {
        // https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution.html
        let NdotRayDirection = self.normal.dot(&ray.direction);
        if NdotRayDirection.abs() < 1e-6 {
            return None;
        }
        let d = self.normal.dot(&self.v0);
        let t = (self.normal.dot(&ray.origin) + d) / NdotRayDirection;
        if t < 0.001 {
            return None;
        }
        let P = ray.origin.add(&ray.direction.scalar_mul(t));
        let mut Ne: Vec3;
        let v0p = P.sub(&self.v0);
        Ne = self.e1.cross(&v0p);
        if self.normal.dot(&Ne) < 0.0 {
            return None;
        }

        let v2v1 = self.v2.sub(&self.v1);
        let v1p = P.sub(&self.v1);
        Ne = self.e2.cross(&v1p);
        if self.normal.dot(&Ne) < 0.0 {
            return None;
        }

        let v2v0 = self.v0.sub(&self.v2);
        let v2p = P.sub(&self.v2);
        Ne = self.e1.cross(&v2p);
        if self.normal.dot(&Ne) < 0.0 {
            return None;
        }
        
        Some(HitRecord { point: P, normal: self.normal, material, t })
    }
}

// haha todo lol

// pub struct AABB {
//     min: Vec3,
//     max: Vec3,
// }
// impl AABB {
//     pub fn surrounding_box(box0: &AABB, box1: &AABB) -> AABB {
//         let small = Vec3::new(
//             box0.min.x.min(box1.min.x),
//             box0.min.y.min(box1.min.y),
//             box0.min.z.min(box1.min.z),
//         );
//         let big = Vec3::new(
//             box0.max.x.max(box1.max.x),
//             box0.max.y.max(box1.max.y),
//             box0.max.z.max(box1.max.z),
//         );
//         AABB { min: small, max: big }
//     }
// }

// pub BVHNode {
//     left: Box<dyn Hittable>,
//     right: Box<dyn Hittable>,
//     bbox: AABB,
// }
// impl BVHNode {
//     pub fn build(objects: &mut [Box<dyn Hittable>], start: usize, end: usize) -> Self {
//         node.bbox = AABB::surrounding_box(&left.box, &right.box);

//         let n = end - start;
//         if n <= 4 {
//         }


        
//     }
// }

pub struct Mesh {
    pub(crate) triangles: Vec<Triangle>,
    pub(crate) material: Box<dyn Material>,
}
impl Mesh {
    pub fn new(triangles: Vec<Triangle>, material: Box<dyn Material>) -> Self {
        Mesh { triangles, material }
    }
    pub fn from_stl(path: &str, material: Box<dyn Material>) -> Self {
        let mut file = std::fs::File::open(path).expect("failed to open STL file");
        let stl = stl_io::read_stl(&mut file).expect("failed to read STL file");
        let triangles = stl.faces.into_iter().map(|face| {
            let v0 = Vec3::new(stl.vertices[face.vertices[0] as usize][0] as f64, stl.vertices[face.vertices[0] as usize][1] as f64, stl.vertices[face.vertices[0] as usize][2] as f64);
            let v1 = Vec3::new(stl.vertices[face.vertices[1] as usize][0] as f64, stl.vertices[face.vertices[1] as usize][1] as f64, stl.vertices[face.vertices[1] as usize][2] as f64);
            let v2 = Vec3::new(stl.vertices[face.vertices[2] as usize][0] as f64, stl.vertices[face.vertices[2] as usize][1] as f64, stl.vertices[face.vertices[2] as usize][2] as f64);
            Triangle::new(v0, v1, v2)
        }).collect();
        Mesh { triangles, material }
    }
    pub fn build_cube(center: Vec3, size: f64, material: Box<dyn Material>) -> Self {
        let half = size / 2.0;
        let v0 = center.add(&Vec3::new(-half, -half, -half));
        let v1 = center.add(&Vec3::new(half, -half, -half));
        let v2 = center.add(&Vec3::new(half, half, -half));
        let v3 = center.add(&Vec3::new(-half, half, -half));
        let v4 = center.add(&Vec3::new(-half, -half, half));
        let v5 = center.add(&Vec3::new(half, -half, half));
        let v6 = center.add(&Vec3::new(half, half, half));
        let v7 = center.add(&Vec3::new(-half, half, half));

        let triangles = vec![
            Triangle::new(v0, v1, v2),
            Triangle::new(v0, v2, v3),
            Triangle::new(v1, v5, v6),
            Triangle::new(v1, v6, v2),
            Triangle::new(v5, v4, v7),
            Triangle::new(v5, v7, v6),
            Triangle::new(v4, v0, v3),
            Triangle::new(v4, v3, v7),
            Triangle::new(v3, v2, v6),
            Triangle::new(v3, v6, v7),
            Triangle::new(v4, v5, v1),
            Triangle::new(v4, v1, v0),
        ];

        Mesh { triangles, material }
    }
}
impl Hittable for Mesh {
    fn hit(&self, ray: &Ray) -> Option<HitRecord> {
        let mut closest_hit: Option<HitRecord> = None;
        for tri in &self.triangles {
            if let Some(hit) = tri.hit(ray, self.material.as_ref()) {
                if closest_hit.is_none() || hit.t < closest_hit.as_ref().unwrap().t {
                    closest_hit = Some(hit);
                }
            }
        }
        closest_hit

    }
}
pub struct Plane {
    pub(crate) point: Vec3,
    pub(crate) normal: Vec3,
    pub(crate) material: Box<dyn Material>,
}
impl Hittable for Plane {
    fn hit(&self, ray: &Ray) -> Option<HitRecord> {
        let denom = self.normal.dot(&ray.direction);
        if denom.abs() > 1e-6 {
            let t = (self.point.sub(&ray.origin)).dot(&self.normal) / denom;
            if t >= 0.001 {
                let hit_point = ray.origin.add(&ray.direction.scalar_mul(t));
                return Some(HitRecord { point: hit_point, normal: self.normal, material: self.material.as_ref(), t });
            }
        }
        None
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
        World { camera, img_buffer, objects, depth: 5, samples: 20 }
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
                    if current_color.max_component() < 0.01 {
                        return Vec3::ZERO;
                    }
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