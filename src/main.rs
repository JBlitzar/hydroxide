mod vec3;
mod world;

use crate::vec3::Vec3;
use crate::world::World;
use crate::world::Camera;
use crate::world::Lambertian;
use crate::world::Metal;
use crate::world::Sphere;
use crate::world::HittableList;

fn main() {
    let red = Lambertian { albedo: Vec3::new(1.0, 0.0, 0.0) };
    let gray = Lambertian { albedo: Vec3::new(0.5, 0.5, 0.5) };
    let blue: Lambertian = Lambertian { albedo: Vec3::new(0.0, 0.0, 1.0) };
    let bluish = Metal { albedo: Vec3::new(0.0, 0.0, 1.0), fuzz: 0.3 };

    let shiny = Metal { albedo: Vec3::new(1.0, 1.0, 1.0), fuzz: 0.0};
    let ball = Sphere{center: Vec3::new(0.0, 0.0, -5.0), radius: 1.0, material: Box::new(shiny.clone())};
    let ball2 = Sphere{center: Vec3::new(1.0, 0.5, -3.5), radius: 0.5, material: Box::new(bluish)};
    let ball3 = Sphere{center: Vec3::new(-1.0, 0.5, -3.5), radius: 0.5, material: Box::new(red)};
    let ball4 = Sphere{center: Vec3::new(0.2, 0.5, -3.7), radius: 0.25, material: Box::new(shiny.clone())};
    let floor = Sphere {
        center: Vec3::new(0.0, 101.0, -5.0),
        radius: 100.0,
        material: Box::new(gray),
    };


    let mut world = World::new(
        Camera::new(1920, 1080, 90.0_f64.to_radians(), Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0)),
        HittableList { objs: vec![Box::new(ball), Box::new(floor), Box::new(ball2), Box::new(ball3), Box::new(ball4)] }
    );
    let start = std::time::Instant::now();
    world.render();
    world.save_image("output.png");
    let duration = start.elapsed();
    println!("Render time: {:?}", duration);


}

