use std::sync::Arc;

use crate::{
    camera::Camera,
    geometry::{Hittable, HittableList},
    material::HitRecord,
    vec3::{Ray, Vec3},
};

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

    pub fn widest_axis(&self) -> usize {
        let x_extent = self.max.x - self.min.x;
        let y_extent = self.max.y - self.min.y;
        let z_extent = self.max.z - self.min.z;

        if x_extent > y_extent && x_extent > z_extent {
            0
        } else if y_extent > z_extent {
            1
        } else {
            2
        }
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
        AABB {
            min: small,
            max: big,
        }
    }

    pub fn hit(&self, ray: &Ray, t_max_bound: f64) -> bool {
        let mut t_enter: f64 = 0.001;
        let mut t_exit: f64 = t_max_bound;

        for axis in 0..3 {
            let t0 = (self.min[axis] - ray.origin[axis]) / ray.direction[axis];
            let t1 = (self.max[axis] - ray.origin[axis]) / ray.direction[axis];
            let (t0, t1) = if t0 < t1 { (t0, t1) } else { (t1, t0) };

            t_enter = t_enter.max(t0);
            t_exit = t_exit.min(t1);

            if t_exit < t_enter {
                return false;
            }
        }
        true
    }

    pub fn screen_space_aabb(&self, camera: &Camera) -> (usize, usize, usize, usize) {
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for corner in &corners {
            if let Some((u, v)) = camera.project_point(*corner) {
                min_x = min_x.min(u);
                max_x = max_x.max(u);
                min_y = min_y.min(v);
                max_y = max_y.max(v);
            } else {
                return (0, camera.width_px, 0, camera.height_px);
            }
        }

        (
            min_x.max(0.0).min(camera.width_px as f64 - 1.0) as usize,
            max_x.max(0.0).min(camera.width_px as f64 - 1.0) as usize,
            min_y.max(0.0).min(camera.height_px as f64 - 1.0) as usize,
            max_y.max(0.0).min(camera.height_px as f64 - 1.0) as usize,
        )
    }
}

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

        // let mut _box = AABB::new(Vec3::ZERO, Vec3::ZERO);
        // for o in objects.iter() {
        //     let obox = o.bounding_box();
        //     if obox.min == Vec3::ZERO && obox.max == Vec3::ZERO {
        //         panic!("Object has no bounding box");
        //     }
        //     _box = AABB::of_boxes(&_box, &obox);
        // }

        // let axis = _box.widest_axis();
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
