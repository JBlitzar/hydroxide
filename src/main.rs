mod aabb;
mod bluenoise;
mod bvh;
mod camera;
mod geometry;
mod light;
mod material;
mod renderer;
mod scene;
mod sky;
mod vec3;
mod world;

use crate::camera::Camera;
use crate::geometry::mesh::MeshBVH;
use crate::scene::{HdrSkyId, MaterialDesc, ObjectDesc, SceneDescription, SkyDesc};
use crate::vec3::Vec3;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "output.png")]
    output: String,

    #[arg(long, default_value = "demo.scene")]
    scene: String,

    #[arg(short, long, default_value = "500")]
    samples: u32,

    #[arg(short, long, default_value = "0.1")]
    roulette: f64,

    #[arg(long, default_value = "1920")]
    width: u32,

    #[arg(long, default_value = "1080")]
    height: u32,
}

fn main() {
    fastrand::seed(42);
    let args = Args::parse();
    if std::env::args_os().len() <= 1 {
        println!("Hint: run with --help to see available options.");
    }

    let profile = env!("HYDROXIDE_PROFILE");
    let (width, height, samples, roulette) = match profile {
        "iteration" => (960, 540, 100, 0.1),
        "extra" => (3840, 2160, 1_000, 0.05),
        _ => (args.width, args.height, args.samples, args.roulette),
    };
    println!(
        "Rendering at {}x{} with {} samples per pixel and termination probability of {}",
        width, height, samples, roulette
    );

    // let scene = SceneDescription {
    //     camera: Camera::look_at(
    //         width,
    //         height,
    //         90.0_f64.to_radians(),
    //         Vec3::new(3.0, 2.5, 0.0),
    //         Vec3::new(0.5, 0.0, -5.0),
    //         5.385,
    //         0.04,
    //     ),
    //     objects: vec![
    //         {
    //             let (vertices, faces) = MeshBVH::load_stl_indexed(
    //                 "teapot_fixed.stl",
    //                 Some(2.0),
    //                 Some(Vec3::new(-2.0, 0.0, -5.0)),
    //                 None,
    //             );
    //             ObjectDesc::Mesh {
    //                 vertices,
    //                 faces,
    //                 material: MaterialDesc::Dielectric {
    //                     albedo: Vec3::new(1.0, 1.0, 1.0),
    //                     refractive_index: 1.7,
    //                 },
    //             }
    //         },
    //         {
    //             let (vertices, faces) = MeshBVH::load_stl_indexed(
    //                 "dragon_fixed.stl",
    //                 Some(2.0),
    //                 Some(Vec3::new(2.0, -0.5, -5.0)),
    //                 Some(Vec3::new(0.0, 0.0, 0.0)),
    //             );
    //             ObjectDesc::Mesh {
    //                 vertices,
    //                 faces,
    //                 material: MaterialDesc::Lambertian {
    //                     albedo: Vec3::new(0.7, 1.0, 1.0),
    //                 },
    //             }
    //         },
    //         ObjectDesc::Sphere {
    //             center: Vec3::new(0.0, 0.7, -5.0),
    //             radius: 0.7,
    //             material: MaterialDesc::Lambertian {
    //                 albedo: Vec3::new(0.2, 0.5, 0.5),
    //             },
    //         },
    //         ObjectDesc::Sphere {
    //             center: Vec3::new(0.0, -1000.0, 0.0),
    //             radius: 1000.0,
    //             material: MaterialDesc::Checkerboard {
    //                 color_a: Vec3::new(0.0, 0.0, 0.0),
    //                 color_b: Vec3::new(1.0, 1.0, 1.0),
    //                 scale: 1.0,
    //             },
    //         },
    //     ],
    //     sky: SkyDesc::Hdr {
    //         id: HdrSkyId::CitrusOrchard,
    //         exposure: 0.3,
    //     },
    //     samples,
    //     termination_prob: roulette,
    // };

    const EMBEDDED_DEMO: &[u8] = include_bytes!("../demo.scene");

    let mut scene = if std::path::Path::new(&args.scene).exists() {
        SceneDescription::load(&args.scene)
    } else if args.scene == "demo.scene" {
        SceneDescription::from_bytes(EMBEDDED_DEMO)
    } else {
        eprintln!("Scene file '{}' not found", args.scene);
        std::process::exit(1);
    };
    scene.samples = samples as usize;
    scene.termination_prob = roulette;
    scene.camera.half_tan_fov_y = scene.camera.half_tan_fov_x * (height as f64 / width as f64);
    scene.camera.width_px = width as usize;
    scene.camera.height_px = height as usize;
    let (world, mut renderer) = scene.build();
    let start = std::time::Instant::now();
    renderer.render(&world);
    println!("Render time: {:?}", start.elapsed());
    renderer.save_image(&args.output);
    println!("Image hash: {:x}", renderer.hash_buf());
    scene.save(&args.scene);
    println!("Scene saved to {}", args.scene);
    println!("Image saved to {}", args.output);
}

#[cfg(test)]
mod tests {
    use super::*;
}
