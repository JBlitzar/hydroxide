use std::sync::Arc;

use crate::{
    aabb::AABB,
    camera::Camera,
    geometry::{Hittable, HittableList},
    material::HitRecord,
    vec3::{Ray, Vec3},
};

// https://raytracing.github.io/books/RayTracingTheNextWeek.html#boundingvolumehierarchies
#[derive(Clone)]
pub struct BVHNode {
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
    bbox: AABB,
}

impl BVHNode {
    fn _hit(&'_ self, ray: &Ray, t_max: f64) -> Option<HitRecord<'_>> {
        if !self.bbox.hit(ray, t_max) {
            return None;
        }

        let left_hit = self.left.hit(ray, t_max);
        let right_hit = self.right.hit(
            ray,
            t_max.min(left_hit.as_ref().map_or(f64::INFINITY, |hit| hit.t)),
        );

        match (left_hit, right_hit) {
            (Some(l), Some(r)) => {
                if l.t <= r.t {
                    Some(l)
                } else {
                    Some(r)
                }
            }
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }

    fn pick_child(obj: &Arc<dyn Hittable>, ray: &Ray, t_max: f64) -> Option<PickHit> {
        if obj.is_leaf() {
            return obj.hit(ray, t_max).map(|h| PickHit {
                point: h.point,
                normal: h.normal,
                geo_normal: h.geo_normal,
                t: h.t,
                object: Arc::clone(obj),
            });
        }

        obj.as_any()
            .downcast_ref::<BVHNode>()
            .and_then(|node| node.pick(ray, t_max))
            .or_else(|| {
                obj.hit(ray, t_max).map(|h| PickHit {
                    point: h.point,
                    normal: h.normal,
                    geo_normal: h.geo_normal,
                    t: h.t,
                    object: Arc::clone(obj),
                })
            })
    }

    pub fn pick(&self, ray: &Ray, t_max: f64) -> Option<PickHit> {
        if !self.bbox.hit(ray, t_max) {
            return None;
        }

        let left_hit = Self::pick_child(&self.left, ray, t_max);
        let right_t_max = t_max.min(left_hit.as_ref().map_or(f64::INFINITY, |hit| hit.t));
        let right_hit = Self::pick_child(&self.right, ray, right_t_max);

        match (left_hit, right_hit) {
            (Some(l), Some(r)) => {
                if l.t <= r.t {
                    Some(l)
                } else {
                    Some(r)
                }
            }
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }
}

pub struct PickHit {
    pub point: Vec3,
    pub normal: Vec3,
    pub geo_normal: Vec3,
    pub t: f64,
    pub object: Arc<dyn Hittable>,
}
impl Hittable for BVHNode {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn hit(&'_ self, ray: &Ray, t_max: f64) -> Option<HitRecord<'_>> {
        self._hit(ray, t_max)
    }

    fn bounding_box(&self) -> AABB {
        self.bbox.clone()
    }

    fn is_leaf(&self) -> bool {
        false
    }
}
impl Default for BVHNode {
    fn default() -> Self {
        Self::new()
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
        // makes it 15% slower, so even though it's supposed to be optimized, it's not for me? empirical data will always win.

        let mut _box = AABB::new(Vec3::ZERO, Vec3::ZERO);
        for o in objects.iter() {
            let obox = o.bounding_box();
            if obox.min == Vec3::ZERO && obox.max == Vec3::ZERO {
                panic!("Object has no bounding box");
            }
            _box = AABB::of_boxes(&_box, &obox);
        }

        let axis = _box.widest_axis();
        // let axis = fastrand::usize(0..3);
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
            let right_node: Arc<dyn Hittable> =
                Arc::new(Self::of_objects_and_endpoints(right_slice));
            (left_node, right_node)
        };

        let bbox = AABB::of_boxes(&left.bounding_box(), &right.bounding_box());
        Self { left, right, bbox }
    }

    pub fn hit(&self, ray: &Ray, t_max: f64) -> Option<HitRecord<'_>> {
        if !self.bbox.hit(ray, t_max) {
            return None;
        }
        let left_hit = self.left.hit(ray, t_max);
        let right_hit = self.right.hit(
            ray,
            t_max.min(left_hit.as_ref().map_or(f64::INFINITY, |hit| hit.t)),
        );

        match (left_hit, right_hit) {
            (Some(lh), Some(rh)) => {
                if lh.t < rh.t {
                    Some(lh)
                } else {
                    Some(rh)
                }
            }
            (Some(lh), None) => Some(lh),
            (None, Some(rh)) => Some(rh),
            (None, None) => None,
        }
    }

    fn box_compare(
        a: &Arc<dyn Hittable>,
        b: &Arc<dyn Hittable>,
        axis: usize,
    ) -> std::cmp::Ordering {
        let box_a = a.bounding_box();
        let box_b = b.bounding_box();
        box_a.min[axis].partial_cmp(&box_b.min[axis]).unwrap()
    }
}
