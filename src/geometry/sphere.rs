use crate::{
    bvh::AABB,
    geometry::Hittable,
    material::{HitRecord, Material},
    vec3::{Ray, Vec3},
};

pub(crate) struct Sphere {
    pub(crate) center: Vec3,
    pub(crate) radius: f64,
    pub(crate) material: Box<dyn Material>,
}

impl Hittable for Sphere {
    fn hit(&'_ self, ray: &Ray) -> Option<HitRecord<'_>> {
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
            Some(HitRecord {
                point,
                normal,
                material: self.material.as_ref(),
                t,
            })
        }
    }

    fn bounding_box(&self) -> AABB {
        let r = Vec3::new(self.radius, self.radius, self.radius);
        AABB::new(self.center.sub(&r), self.center.add(&r))
    }
}
