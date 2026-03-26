use crate::{
    geometry::Hittable,
    material::{HitRecord, Material},
    vec3::{Ray, Vec3},
};

pub struct Plane {
    pub(crate) point: Vec3,
    pub(crate) normal: Vec3,
    pub(crate) material: Box<dyn Material>,
}
impl Hittable for Plane {
    fn hit(&'_ self, ray: &Ray) -> Option<HitRecord<'_>> {
        let denom = self.normal.dot(&ray.direction);
        if denom.abs() > 1e-6 {
            let t = (self.point.sub(&ray.origin)).dot(&self.normal) / denom;
            if t >= 0.001 {
                let hit_point = ray.origin.add(&ray.direction.scalar_mul(t));
                return Some(HitRecord {
                    point: hit_point,
                    normal: self.normal,
                    material: self.material.as_ref(),
                    t,
                });
            }
        }
        None
    }
}
