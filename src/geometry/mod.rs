use crate::{
    material::{HitRecord, Material},
    vec3::{Ray, Vec3},
};
pub mod mesh;
pub mod plane;
pub mod sphere;

pub(crate) trait Hittable {
    fn hit(&'_ self, ray: &Ray) -> Option<HitRecord<'_>>;
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

    pub fn hit(&'_ self, ray: &Ray) -> Option<HitRecord<'_>> {
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
