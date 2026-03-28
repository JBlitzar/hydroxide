use crate::vec3::{Ray, Vec3};

pub enum MaterialType {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
}

pub struct HitRecord<'a> {
    pub(crate) point: Vec3,
    pub(crate) normal: Vec3,
    pub(crate) material: &'a dyn Material,
    pub(crate) t: f64,
}

fn random_in_unit_sphere() -> Vec3 {
    // better way, because it's rejection sampling
    let mut p;
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

pub trait Material: Send + Sync {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Ray, Vec3)>;
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Lambertian {
    pub albedo: Vec3, // color lol
}
impl Material for Lambertian {
    fn scatter(&self, _ray_in: &Ray, hit_record: &HitRecord) -> Option<(Ray, Vec3)> {
        let mut random_in_unit_hemisphere = random_in_unit_sphere();
        if random_in_unit_hemisphere.dot(&hit_record.normal) < 0.0 {
            random_in_unit_hemisphere = random_in_unit_hemisphere.scalar_mul(-1.0);
        }

        let target = hit_record
            .point
            .add(&hit_record.normal)
            .add(&random_in_unit_hemisphere);

        Some((
            Ray::new(hit_record.point, target.sub(&hit_record.point)),
            self.albedo,
        ))
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

        Some((
            Ray::new(hit_record.point, reflected_dir.add(&fuzz)),
            self.albedo,
        ))
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
        if ray_in.direction.dot(&hit_record.normal) < 0.0 {
            ri = 1.0 / ri;
        }

        let unit_direction = ray_in.direction.normalize();

        let cos_theta = unit_direction
            .scalar_mul(-1.0)
            .dot(&hit_record.normal)
            .min(1.0);
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

pub struct Checkerboard {
    pub scale: f64,
    pub color_a: Vec3,
    pub color_b: Vec3,
}

impl Material for Checkerboard {
    fn scatter(&self, _ray_in: &Ray, hit: &HitRecord) -> Option<(Ray, Vec3)> {
        let x = (hit.point.x * self.scale).floor() as i32;
        let z = (hit.point.z * self.scale).floor() as i32;
        let color = if (x + z) % 2 == 0 {
            self.color_a
        } else {
            self.color_b
        };

        let mut dir = random_in_unit_sphere();
        if dir.dot(&hit.normal) < 0.0 {
            dir = dir.scalar_mul(-1.0);
        }
        Some((Ray::new(hit.point, dir), color))
    }
}
