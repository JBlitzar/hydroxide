use crate::vec3::{Ray, Vec3};

pub struct Camera {
    pub(crate) width_px: usize,
    pub(crate) height_px: usize,
    position: Vec3,
    euler_angles: Vec3,
    half_tan_fov_x: f64,
    half_tan_fov_y: f64,
}
impl Camera {
    pub fn new(
        width_px: usize,
        height_px: usize,
        x_fov: f64,
        position: Vec3,
        euler_angles: Vec3,
    ) -> Self {
        let half_tan_fov_x = (x_fov / 2.0).tan();
        let half_tan_fov_y = half_tan_fov_x * (height_px as f64 / width_px as f64);
        let _y_fov = 2.0 * half_tan_fov_y.atan();
        Camera {
            width_px,
            height_px,
            position,
            euler_angles,
            half_tan_fov_x: half_tan_fov_x,
            half_tan_fov_y: half_tan_fov_y,
        }
    }

    pub fn get_ray_direction(&self, x: usize, y: usize) -> Ray {
        let x_cmp =
            ((x as f64 + fastrand::f64()) / self.width_px as f64 - 0.5) * self.half_tan_fov_x;
        let y_cmp =
            (0.5 - (y as f64 + fastrand::f64()) / self.height_px as f64) * self.half_tan_fov_y;
        Ray::new(
            self.position,
            Vec3::new(x_cmp, y_cmp, -1.0)
                .normalize()
                .rotate(&self.euler_angles),
        )
    }
}
