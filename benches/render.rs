use std::hint::black_box;
use std::sync::Arc;

use criterion::{Criterion, criterion_group, criterion_main};
use oxide::camera::Camera;
use oxide::geometry::Hittable;
use oxide::geometry::mesh::MeshBVH;
use oxide::material::Lambertian;
use oxide::renderer::Renderer;
use oxide::vec3::Vec3;
use oxide::world::World;

fn bench_render(c: &mut Criterion) {
    c.bench_function("render balls", |b| {
        b.iter(|| {
            let world = World::new_random_spheres(
                Camera::look_at(
                    320,
                    240,
                    90.0_f64.to_radians(),
                    Vec3::new(0.0, 2.0, 0.0),
                    Vec3::new(0.0, 0.0, -5.0),
                    5.385,
                    0.04,
                ),
                100,
            );
            let mut renderer = Renderer::new(320, 240, None, None);
            renderer.render(&world);
            black_box(renderer.hash_buf());
        })
    });
}

// fn bench_render_cube(c: &mut Criterion) {
//     c.bench_function("render cube", |b| {
//         b.iter(|| {
//             let objects_vec: Vec<Arc<dyn Hittable>> = vec![Arc::new(MeshBVH::build_cube(
//                 Vec3::new(0.0, 0.0, -5.0),
//                 1.0,
//                 Box::new(Lambertian {
//                     albedo: Vec3::new(0.5, 0.5, 0.5),
//                 }),
//             ))];
//             let world = World::new(
//                 Camera::look_at(
//                     320,
//                     240,
//                     90.0_f64.to_radians(),
//                     Vec3::new(0.0, 2.0, 0.0),
//                     Vec3::new(0.0, 0.0, -5.0),
//                     5.385,
//                     0.04,
//                 ),
//                 objects_vec,
//                 None,
//             );
//             let mut renderer = Renderer::new(320, 240, Some(20), Some(0.1));
//             renderer.render(&world);
//             black_box(renderer.hash_buf());
//         })
//     });
// }

fn bench_render_teapot(c: &mut Criterion) {
    let teapot: Arc<dyn Hittable> = Arc::new(MeshBVH::from_stl(
        "teapot_fixed.stl",
        Box::new(Lambertian {
            albedo: Vec3::new(0.7, 0.7, 0.7),
        }),
        Some(2.0),
        Some(Vec3::new(0.0, 0.0, -5.0)),
        None,
    ));

    c.bench_function("render teapot", |b| {
        b.iter(|| {
            let world = World::new(
                Camera::look_at(
                    320,
                    240,
                    90.0_f64.to_radians(),
                    Vec3::new(0.0, 2.0, 0.0),
                    Vec3::new(0.0, 0.0, -5.0),
                    5.385,
                    0.04,
                ),
                vec![teapot.clone()],
                None,
            );
            let mut renderer = Renderer::new(320, 240, Some(20), Some(0.1));
            renderer.render(&world);
            black_box(renderer.hash_buf());
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(50);
    targets = bench_render
}
// criterion_group! {
//     name = cube_bench;
//     config = Criterion::default().sample_size(50);
//     targets = bench_render_cube
// }
criterion_group! {
    name = teapot_bench;
    config = Criterion::default().sample_size(50);
    targets = bench_render_teapot
}
criterion_main!(benches, teapot_bench);
