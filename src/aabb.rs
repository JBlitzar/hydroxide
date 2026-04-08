use crate::{
    camera::Camera,
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
                return (
                    0,
                    camera.width_px.saturating_sub(1),
                    0,
                    camera.height_px.saturating_sub(1),
                );
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
