use crate::vec3::{Ray, Vec3};

pub struct Camera {
    pub(crate) width_px: usize,
    pub(crate) height_px: usize,
    position: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    half_tan_fov_x: f64,
    half_tan_fov_y: f64,
}
impl Camera {
    pub fn look_at(
        width_px: usize,
        height_px: usize,
        x_fov: f64,
        position: Vec3,
        target: Vec3,
    ) -> Self {
        let world_up = Vec3::new(0.0, 1.0, 0.0);
        let forward = target.sub(&position).normalize();
        let right = forward.cross(&world_up).normalize();
        let up = right.cross(&forward).normalize();

        let half_tan_fov_x = (x_fov / 2.0).tan();
        let half_tan_fov_y = half_tan_fov_x * (height_px as f64 / width_px as f64);
        Camera {
            width_px,
            height_px,
            position,
            forward,
            right,
            up,
            half_tan_fov_x,
            half_tan_fov_y,
        }
    }

    pub fn get_ray_direction(&self, x: usize, y: usize) -> Ray {
        let x_cmp =
            ((x as f64 + fastrand::f64()) / self.width_px as f64 - 0.5) * self.half_tan_fov_x;
        let y_cmp =
            (0.5 - (y as f64 + fastrand::f64()) / self.height_px as f64) * self.half_tan_fov_y;
        let dir = self
            .forward
            .add(&self.right.scalar_mul(x_cmp))
            .add(&self.up.scalar_mul(y_cmp))
            .normalize();
        Ray::new(self.position, dir)
    }
}
