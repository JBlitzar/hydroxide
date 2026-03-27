use crate::vec3::Vec3;
use crate::{
    material::HitRecord,
    vec3::Ray,
};
use crate::bvh::AABB;
pub mod mesh;
pub mod plane;
pub mod sphere;

pub trait Hittable: Send + Sync {
     fn hit(&'_ self, ray: &Ray) -> Option<HitRecord<'_>>;

    fn bounding_box(&self) -> AABB;
}




pub struct HittableList {
    pub objs: Vec<Box<dyn Hittable>>,

    pub bounding_box: Option<AABB>,
}
impl Hittable for HittableList {
    fn hit(&'_ self, ray: &Ray) -> Option<HitRecord<'_>> {
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

    fn bounding_box(&self) -> AABB {
        self.bounding_box.clone().unwrap_or_else(|| AABB::new(Vec3::ZERO, Vec3::ZERO))
    }
}
impl HittableList {
    pub fn new() -> Self {
        HittableList { objs: Vec::new(), bounding_box: None }
    }

    pub fn add(&mut self, obj: Box<dyn Hittable>) {
        self.bounding_box = match &self.bounding_box {
            None => Some(obj.bounding_box()),
            Some(current_box) => Some(AABB::of_boxes(&current_box, &obj.bounding_box())),
        };
        self.objs.push(obj);

    }

    pub fn hit(&'_ self, ray: &Ray) -> Option<HitRecord<'_>> {
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
