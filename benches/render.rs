use criterion::{Criterion, criterion_group, criterion_main};
use oxide::camera::Camera;
use oxide::geometry::HittableList;
use oxide::vec3::Vec3;
use oxide::world::World;

fn bench_render(c: &mut Criterion) {
    c.bench_function("render balls", |b| {
        b.iter(|| {
            let mut world = World::new_random_spheres(
                Camera::new(
                    100,
                    100,
                    90.0_f64.to_radians(),
                    Vec3::new(0.0, 2.0, 0.0),
                    Vec3::new(-0.2, 0.0, 0.0),
                ),
                100,
            );
            world.render();
        })
    });
}

fn bench_render_cube(c: &mut Criterion) {
    c.bench_function("render cube", |b| {
        b.iter(|| {
            let mut world = World::new(
                Camera::new(
                    100,
                    100,
                    90.0_f64.to_radians(),
                    Vec3::new(0.0, 2.0, 0.0),
                    Vec3::new(-0.2, 0.0, 0.0),
                ),
                HittableList {
                    objs: vec![Box::new(oxide::geometry::mesh::Mesh::build_cube(
                        Vec3::new(0.0, 0.0, -5.0),
                        1.0,
                        Box::new(oxide::material::Lambertian {
                            albedo: Vec3::new(0.8, 0.3, 0.3),
                        }),
                    ))],
                    bounding_box: None
                },
            );
            world.render();
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_render
}
criterion_group! {
    name = cube_bench;
    config = Criterion::default().sample_size(10);
    targets = bench_render_cube
}
criterion_main!(benches, cube_bench);
