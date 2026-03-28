mod bvh;
mod camera;
mod geometry;
mod material;
mod vec3;
mod world;

use std::sync::Arc;

use crate::bvh::BVHNode;
use crate::camera::Camera;
use crate::geometry::mesh::MeshBVH;
use crate::material::Lambertian;
use crate::vec3::Vec3;
use crate::world::World;

fn main() {
    fastrand::seed(42);
    let mut objects: Vec<Arc<dyn geometry::Hittable>> = vec![Arc::new(MeshBVH::build_cube(
        Vec3::new(0.0, 0.0, -5.0),
        2.0,
        Box::new(Lambertian {
            albedo: Vec3::new(0.5, 0.5, 0.5),
        }),
    ))];
    let objects = BVHNode::of_objects_and_endpoints(&mut objects);

    let mut world = World::new(
        Camera::new(
            480,
            320,
            90.0_f64.to_radians(),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(-0.2, 0.0, 0.0),
        ),
        objects,
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

    // #[test]
    // fn test_render() {
    //     fastrand::seed(42);
    //     let mut world = World::new_random_spheres(
    //         Camera::new(
    //             100,
    //             100,
    //             90.0_f64.to_radians(),
    //             Vec3::new(0.0, 2.0, 0.0),
    //             Vec3::new(-0.2, 0.0, 0.0),
    //         ),
    //         100,
    //     );
    //     let start = std::time::Instant::now();
    //     world.render_single_threaded();
    //     let duration = start.elapsed();
    //     println!("Render time: {:?}", duration);

    //     assert_eq!(world.hash_buf(), 0x38b8338d2d58b14c);
    // }
}
