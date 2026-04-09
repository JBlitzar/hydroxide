use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use hydroxide::camera::Camera;
use hydroxide::geometry::mesh::MeshBVH;
use hydroxide::scene::{MaterialDesc, ObjectDesc, SceneDescription, SkyDesc};
use hydroxide::vec3::Vec3;

fn bench_render(c: &mut Criterion) {
    c.bench_function("render balls", |b| {
        b.iter(|| {
            let mut objects = Vec::new();
            for _ in 0..100 {
                let radius = fastrand::f64() * 0.5 + 0.1;
                let center = Vec3::new(
                    fastrand::f64() * 20.0 - 10.0,
                    -1.0 + radius,
                    fastrand::f64() * -20.0 - 5.0,
                );
                let rand_type = fastrand::u8(0..3_u8);
                let material = match rand_type {
                    0 => MaterialDesc::Lambertian {
                        albedo: Vec3::new(fastrand::f64(), fastrand::f64(), fastrand::f64()),
                    },
                    1 => MaterialDesc::Metal {
                        albedo: Vec3::new(fastrand::f64(), fastrand::f64(), fastrand::f64()),
                        fuzz: fastrand::f64() * 0.5,
                    },
                    _ => MaterialDesc::Dielectric {
                        albedo: Vec3::new(1.0, 1.0, 1.0),
                        refractive_index: fastrand::f64() * 2.0 + 1.0,
                    },
                };
                objects.push(ObjectDesc::Sphere {
                    center,
                    radius,
                    material,
                });
            }
            objects.push(ObjectDesc::Sphere {
                center: Vec3::new(0.0, -1001.0, -5.0),
                radius: 1000.0,
                material: MaterialDesc::Lambertian {
                    albedo: Vec3::new(0.5, 0.5, 0.5),
                },
            });

            let scene = SceneDescription {
                camera: Camera::look_at(
                    320,
                    240,
                    90.0_f64.to_radians(),
                    Vec3::new(0.0, 2.0, 0.0),
                    Vec3::new(0.0, 0.0, -5.0),
                    5.385,
                    0.04,
                ),
                objects,
                sky: SkyDesc::Gradient {
                    top: Vec3::new(0.87, 0.92, 1.0),
                    bottom: Vec3::new(1.0, 1.0, 1.0),
                },
                samples: 20,
                termination_prob: 0.01,
            };
            let (world, mut renderer) = scene.build();
            renderer.render(&world);
            black_box(renderer.hash_buf());
        })
    });
}

fn bench_render_teapot(c: &mut Criterion) {
    let (teapot_verts, teapot_faces) = MeshBVH::load_stl_indexed(
        "teapot_fixed.stl",
        Some(2.0),
        Some(Vec3::new(0.0, 0.0, -5.0)),
        None,
    );

    c.bench_function("render teapot", |b| {
        b.iter(|| {
            let scene = SceneDescription {
                camera: Camera::look_at(
                    320,
                    240,
                    90.0_f64.to_radians(),
                    Vec3::new(0.0, 2.0, 0.0),
                    Vec3::new(0.0, 0.0, -5.0),
                    5.385,
                    0.04,
                ),
                objects: vec![ObjectDesc::Mesh {
                    vertices: teapot_verts.clone(),
                    faces: teapot_faces.clone(),
                    material: MaterialDesc::Lambertian {
                        albedo: Vec3::new(0.7, 0.7, 0.7),
                    },
                }],
                sky: SkyDesc::Gradient {
                    top: Vec3::new(0.87, 0.92, 1.0),
                    bottom: Vec3::new(1.0, 1.0, 1.0),
                },
                samples: 20,
                termination_prob: 0.1,
            };
            let (world, mut renderer) = scene.build();
            renderer.render(&world);
            black_box(renderer.hash_buf());
        })
    });
}

fn bench_render_teapot_only(c: &mut Criterion) {
    let (teapot_verts, teapot_faces) = MeshBVH::load_stl_indexed(
        "teapot_fixed.stl",
        Some(2.0),
        Some(Vec3::new(0.0, 0.0, -5.0)),
        None,
    );

    let scene = SceneDescription {
        camera: Camera::look_at(
            320,
            240,
            90.0_f64.to_radians(),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, 0.0, -5.0),
            5.385,
            0.04,
        ),
        objects: vec![ObjectDesc::Mesh {
            vertices: teapot_verts,
            faces: teapot_faces,
            material: MaterialDesc::Lambertian {
                albedo: Vec3::new(0.7, 0.7, 0.7),
            },
        }],
        sky: SkyDesc::Gradient {
            top: Vec3::new(0.87, 0.92, 1.0),
            bottom: Vec3::new(1.0, 1.0, 1.0),
        },
        samples: 20,
        termination_prob: 0.1,
    };
    let (world, mut renderer) = scene.build();

    c.bench_function("render teapot (render only)", |b| {
        b.iter(|| {
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
criterion_group! {
    name = teapot_bench;
    config = Criterion::default().sample_size(50);
    targets = bench_render_teapot
}
criterion_group! {
    name = teapot_render_only;
    config = Criterion::default().sample_size(50);
    targets = bench_render_teapot_only
}
criterion_main!(benches, teapot_bench, teapot_render_only);
