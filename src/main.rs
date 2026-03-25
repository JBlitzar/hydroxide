mod vec3;
mod world;

use crate::vec3::Vec3;
use crate::world::World;
use crate::world::Camera;
use crate::world::Lambertian;
use crate::world::Sphere;
use crate::world::HittableList;

fn main() {
    let red = Lambertian { albedo: Vec3::new(1.0, 0.0, 0.0) };
    let ball = Sphere{center: Vec3::new(0.0, 0.0, -5.0), radius: 1.0, material: Box::new(red)};
    let floor = Sphere {
        center: Vec3::new(0.0, 101.0, -5.0),
        radius: 100.0,
        material: Box::new(Lambertian { albedo: Vec3::new(0.5, 0.5, 0.5) }),
    };


    let mut world = World::new(
        Camera::new(400, 300, 90.0_f64.to_radians(), 70.0_f64.to_radians(), Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0)),
        HittableList { objs: vec![Box::new(ball), Box::new(floor)] }
    );
    let start = std::time::Instant::now();
    world.render();
    world.save_image("output.png");
    let duration = start.elapsed();
    println!("Render time: {:?}", duration);


}

