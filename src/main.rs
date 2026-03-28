mod bvh;
mod camera;
mod geometry;
mod material;
mod vec3;
mod world;

use std::sync::Arc;

use crate::material::Checkerboard;

use crate::material::Dielectric;

use crate::material::Metal;

use crate::bvh::BVHNode;
use crate::camera::Camera;
use crate::geometry::mesh::MeshBVH;
use crate::material::Lambertian;
use crate::vec3::Vec3;
use crate::vec3::toRadians;
use crate::world::World;

fn main() {
    fastrand::seed(42);
    let mut objects: Vec<Arc<dyn geometry::Hittable>> = vec![
        Arc::new(MeshBVH::from_stl(
            "teapot_fixed.stl",
            Box::new(Dielectric {
                albedo: Vec3::new(1.0, 1.0, 1.0),
                refractive_index: 1.7,
            }),
            Some(2.0),
            Some(Vec3::new(-1.0, 0.0, -5.0)),
            None,
        )),
        Arc::new(MeshBVH::from_stl(
            "dragon.stl",
            Box::new(Metal {
                albedo: Vec3::new(0.7, 1.0, 1.0),
                fuzz: 0.9,
            }),
            Some(2.0),
            Some(Vec3::new(1.0, -0.5, -5.0)),
            Some(Vec3::new(0.0, 0.0, 0.0)),
        )),
        Arc::new(geometry::sphere::Sphere {
            center: Vec3::new(-2.0, 0.7, -7.0),
            radius: 0.7,
            material: Box::new(Metal {
                albedo: Vec3::new(0.7, 0.6, 0.5),
                fuzz: 0.0,
            }),
        }),
        Arc::new(geometry::sphere::Sphere {
            center: Vec3::new(2.0, 0.7, -7.0),
            radius: 0.7,
            material:Box::new(Dielectric {
                albedo: Vec3::new(1.0, 1.0, 1.0),
                refractive_index: 1.5,
            }),
        }),
        Arc::new(geometry::sphere::Sphere {
            center: Vec3::new(0.0, 0.7, -7.0),
            radius: 0.7,
            material: Box::new(Lambertian {
                albedo: Vec3::new(0.1, 0.1, 0.9),
            }),
        }),
        Arc::new(geometry::sphere::Sphere {
            center: Vec3::new(0.0, -1000.0, 0.0),
            radius: 1000.0,
            material: Box::new(Checkerboard {
                color_a: Vec3::new(0.0, 0.0, 0.0),
                color_b: Vec3::new(1.0, 1.0, 1.0),
                scale: 1.0,
            }),
        }),
        
    ];
    let objects = BVHNode::of_objects_and_endpoints(&mut objects);

    let mut world = World::new(
        Camera::new(
            3840,
            2160,
            90.0_f64.to_radians(),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(-0.2, 0.0, 0.0),
        ),
        objects,
        Some(1_000),
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
