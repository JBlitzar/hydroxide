mod camera;
mod geometry;
mod material;
mod vec3;
mod world;
mod bvh;

use crate::camera::Camera;
use crate::geometry::mesh::Mesh;
use crate::geometry::sphere::Sphere;
use crate::material::Dielectric;
use crate::material::Lambertian;
use crate::material::Metal;
use crate::vec3::Vec3;
use crate::world::World;
use geometry::HittableList;

fn main() {
    fastrand::seed(42);
    let mut world = World::new_random_spheres(
        Camera::new(
            480,
            320,
            90.0_f64.to_radians(),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(-0.2, 0.0, 0.0),
        ),
        100,
    );
    let start = std::time::Instant::now();
    world.render();
    world.save_image("output.png");
    let duration = start.elapsed();
    println!("Render time: {:?}", duration);
    println!("Image hash: {:x}", world.hash_buf());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render() {
        fastrand::seed(42);
        let mut world = World::new_random_spheres(
            Camera::new(
                480,
                320,
                90.0_f64.to_radians(),
                Vec3::new(0.0, 2.0, 0.0),
                Vec3::new(-0.2, 0.0, 0.0),
            ),
            100,
        );
        let start = std::time::Instant::now();
        world.render_single_threaded();
        let duration = start.elapsed();
        println!("Render time: {:?}", duration);

        assert_eq!(world.hash_buf(), 0x6ed57fccf41dabca);
    }
}
