use crate::{
    aabb::AABB,
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
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn hit(&'_ self, ray: &Ray, t_max: f64) -> Option<HitRecord<'_>> {
        let denom = self.normal.dot(&ray.direction);
        if denom.abs() > 1e-6 {
            let t = (self.point.sub(&ray.origin)).dot(&self.normal) / denom;
            if t >= 0.001 && t < t_max {
                let hit_point = ray.origin.add(&ray.direction.scalar_mul(t));
                return Some(HitRecord {
                    point: hit_point,
                    geo_normal: self.normal,
                    normal: self.normal,
                    material: self.material.as_ref(),
                    t,
                });
            }
        }
        None
    }

    fn bounding_box(&self) -> AABB {
        // Infinite plane: return a very large box so BVH/AABB-based code can compile.
        let big = 1.0e30;
        AABB::new(Vec3::new(-big, -big, -big), Vec3::new(big, big, big))
    }
}
