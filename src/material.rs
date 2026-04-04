use std::{any::Any, f64::consts::PI};

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

fn random_cosine_direction() -> Vec3 {
    // Cosine-weighted hemisphere sampling around +Z in local space.
    // See: https://raytracing.github.io/books/RayTracingTheRestOfYourLife.html#probabilitydensityfunctions/cosinesampling
    let r1 = fastrand::f64();
    let r2 = fastrand::f64();
    let phi = 2.0 * std::f64::consts::PI * r1;
    let x = phi.cos() * r2.sqrt();
    let y = phi.sin() * r2.sqrt();
    let z = (1.0 - r2).sqrt();
    Vec3::new(x, y, z)
}

fn cosine_weighted_hemisphere(normal: &Vec3) -> Vec3 {
    // Build an orthonormal basis (u,v,w) with w aligned to the surface normal.
    let w = normal.normalize();
    let a = if w.x.abs() > 0.9 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let v = w.cross(&a).normalize();
    let u = w.cross(&v);

    let d = random_cosine_direction();
    u.scalar_mul(d.x)
        .add(&v.scalar_mul(d.y))
        .add(&w.scalar_mul(d.z))
}

pub trait Material: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Ray, Vec3)>;
    fn emitted(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }
    fn eval_diffuse_brdf(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Option<Vec3> {
        None
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Lambertian {
    pub albedo: Vec3, // color lol
}
impl Material for Lambertian {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn scatter(&self, _ray_in: &Ray, hit_record: &HitRecord) -> Option<(Ray, Vec3)> {
        let dir = cosine_weighted_hemisphere(&hit_record.normal);

        Some((Ray::new(hit_record.point, dir), self.albedo))
    }
    fn eval_diffuse_brdf(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Option<Vec3> {
        Some(self.albedo.scalar_mul(1.0 / PI))
    }
}
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Metal {
    pub albedo: Vec3,
    pub fuzz: f64,
}
impl Material for Metal {
    fn as_any(&self) -> &dyn Any {
        self
    }
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
#[derive(Clone)]
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
    fn as_any(&self) -> &dyn Any {
        self
    }
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
#[derive(Clone)]
pub struct Checkerboard {
    pub scale: f64,
    pub color_a: Vec3,
    pub color_b: Vec3,
}

impl Material for Checkerboard {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn scatter(&self, _ray_in: &Ray, hit: &HitRecord) -> Option<(Ray, Vec3)> {
        let x = (hit.point.x * self.scale).floor() as i32;
        let z = (hit.point.z * self.scale).floor() as i32;
        let color = if (x + z) % 2 == 0 {
            self.color_a
        } else {
            self.color_b
        };

        let dir = cosine_weighted_hemisphere(&hit.normal);
        Some((Ray::new(hit.point, dir), color))
    }
    fn eval_diffuse_brdf(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Option<Vec3> {
        let x = (_hit_record.point.x * self.scale).floor() as i32;
        let z = (_hit_record.point.z * self.scale).floor() as i32;
        let color = if (x + z) % 2 == 0 {
            self.color_a
        } else {
            self.color_b
        };
        Some(color.scalar_mul(1.0 / PI))
    }
}

#[derive(Clone)]
pub struct DiffuseLight {
    pub albedo: Vec3,
}
impl Material for DiffuseLight {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn scatter(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Option<(Ray, Vec3)> {
        None
    }

    fn emitted(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Vec3 {
        self.albedo
    }
}
