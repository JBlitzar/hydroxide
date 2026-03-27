use std::sync::Arc;

use crate::{geometry::{Hittable, HittableList}, material::HitRecord, vec3::{Ray, Vec3}};


// erm actually they're called axis-aligned bounding rectangular parallelepipeds
#[derive(Clone)]
pub struct AABB {
    pub(crate) min: Vec3,
    pub(crate) max: Vec3,
}
impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn of_boxes(box0: &AABB, box1: &AABB) -> AABB {
        let small = Vec3::new(
            box0.min.x.min(box1.min.x),
            box0.min.y.min(box1.min.y),
            box0.min.z.min(box1.min.z),
        );
        let big = Vec3::new(
            box0.max.x.max(box1.max.x),
            box0.max.y.max(box1.max.y),
            box0.max.z.max(box1.max.z),
        );
        AABB { min: small, max: big }
    }

    //TODO check for correctness?
    pub fn hit(&self, ray: &Ray) -> bool {
        for axis in 0..3 {
            let t0 = (self.min[axis] - ray.origin[axis]) / ray.direction[axis];
            let t1 = (self.max[axis] - ray.origin[axis]) / ray.direction[axis];
            let t_min = t0.min(t1);
            let t_max = t0.max(t1);
            
            if t_max < 0.0 || t_min > t_max {
                return false;
            }
        }
        true
    }
}


// TODO: BVHNode implementation (raytracing.github.io/books/RayTracingTheNextWeek.html#boundingvolumehierarchies)

pub struct BVHNode {
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
    bbox: AABB,
}
impl Hittable for BVHNode {
    fn hit(&'_ self, ray: &Ray) -> Option<HitRecord<'_>> {
        let left_hit = self.left.hit(ray);
        let right_hit = self.right.hit(ray);

        match (left_hit, right_hit) {
            (Some(l), Some(r)) => {
                if l.t <= r.t { Some(l) } else { Some(r) }
            }
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }

    fn bounding_box(&self) -> AABB {
        self.bbox.clone()
    }
}
impl BVHNode {
    pub fn new() -> Self {
        BVHNode {
            left: Arc::new(HittableList::new()),
            right: Arc::new(HittableList::new()),
            bbox: AABB::new(Vec3::ZERO, Vec3::ZERO),
        }
    }
    pub fn _new(left: Arc<dyn Hittable>, right: Arc<dyn Hittable>) -> Self {
        let bbox = AABB::of_boxes(&left.bounding_box(), &right.bounding_box());
        Self { left, right, bbox }
    }

    pub fn empty() -> Self {
        let empty: Arc<dyn Hittable> = Arc::new(HittableList::new());
        Self {
            left: Arc::clone(&empty),
            right: empty,
            bbox: AABB::new(Vec3::ZERO, Vec3::ZERO),
        }
    }

    pub fn from_children(left: Arc<dyn Hittable>, right: Arc<dyn Hittable>) -> Self {
        let bbox = AABB::of_boxes(&left.bounding_box(), &right.bounding_box());
        Self { left, right, bbox }
    }

    pub fn of_objects_and_endpoints(objects: &mut [Arc<dyn Hittable>]) -> Self {
        let axis = fastrand::usize(0..3);
        let comparator =
            |a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>| Self::box_compare(a, b, axis);

        let object_span = objects.len();

        let (left, right) = if object_span == 1 {
            let node = Arc::clone(&objects[0]);
            (Arc::clone(&node), node)
        } else if object_span == 2 {
            (Arc::clone(&objects[0]), Arc::clone(&objects[1]))
        } else {
            objects.sort_by(comparator);
            let mid = object_span / 2;
            let (left_slice, right_slice) = objects.split_at_mut(mid);

            let left_node: Arc<dyn Hittable> = Arc::new(Self::of_objects_and_endpoints(left_slice));
            let right_node: Arc<dyn Hittable> = Arc::new(Self::of_objects_and_endpoints(right_slice));
            (left_node, right_node)
        };

        let bbox = AABB::of_boxes(&left.bounding_box(), &right.bounding_box());
        Self { left, right, bbox }
    }

    pub fn hit(&self, ray: &Ray) -> Option<HitRecord> {
        if !self.bbox.hit(ray) {
            return None;
        }
        let left_hit = self.left.hit(ray);
        let right_hit = self.right.hit(ray);

        match (left_hit, right_hit) {
            (Some(lh), Some(rh)) => if lh.t < rh.t { Some(lh) } else { Some(rh) },
            (Some(lh), None) => Some(lh),
            (None, Some(rh)) => Some(rh),
            (None, None) => None,
        }
    }

    fn box_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>, axis: usize) -> std::cmp::Ordering {
        let box_a = a.bounding_box();
        let box_b = b.bounding_box();
        box_a.min[axis].partial_cmp(&box_b.min[axis]).unwrap()
    }

    pub fn add(&mut self, obj: Box<dyn Hittable>) {
       let obj_arc = Arc::new(obj);
        self.bbox = AABB::of_boxes(&self.bbox, &obj_arc.bounding_box());

        unimplemented!()
        

    }

}