use crate::{
    bvh::AABB,
    geometry::Hittable,
    material::{HitRecord, Material},
    vec3::{Ray, Vec3},
};

pub struct Triangle {
    pub(crate) v0: Vec3,
    pub(crate) v1: Vec3,
    pub(crate) v2: Vec3,
    pub normal: Vec3,
    pub e01: Vec3,
    pub e02: Vec3,
}
impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        let e01 = v1.sub(&v0);
        let e02 = v2.sub(&v0);
        let normal = e01.cross(&e02).normalize();
        Triangle {
            v0,
            v1,
            v2,
            normal,
            e01,
            e02,
        }
    }

    fn hit<'a>(&self, ray: &Ray, material: &'a dyn Material) -> Option<HitRecord<'a>> {
        // https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection.html#:~:text=.-,Implementation,-Implementing%20the%20M%C3%B6ller
        let v0v1 = self.e01;
        let v0v2 = self.e02;
        let pvec = ray.direction.cross(&v0v2);
        let det = v0v1.dot(&pvec);
        if (det.abs() < 1e-8) {
            return None;
        }
        let inv_det = 1.0 / det;

        let tvec = ray.origin.sub(&self.v0);
        let u = tvec.dot(&pvec) * inv_det;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let qvec = tvec.cross(&v0v1);
        let v = ray.direction.dot(&qvec) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = v0v2.dot(&qvec) * inv_det;
        if t < 1e-8 {
            return None;
        }
        Some(HitRecord {
            point: ray.origin.add(&ray.direction.scalar_mul(t)),
            normal: self.normal,
            material,
            t,
        })
    }
}

pub struct Mesh {
    pub(crate) triangles: Vec<Triangle>,
    pub(crate) material: Box<dyn Material>,
}
impl Mesh {
    pub fn new(triangles: Vec<Triangle>, material: Box<dyn Material>) -> Self {
        Mesh {
            triangles,
            material,
        }
    }
    pub fn from_stl(path: &str, material: Box<dyn Material>) -> Self {
        let mut file = std::fs::File::open(path).expect("failed to open STL file");
        let stl = stl_io::read_stl(&mut file).expect("failed to read STL file");
        let triangles = stl
            .faces
            .into_iter()
            .map(|face| {
                let v0 = Vec3::new(
                    stl.vertices[face.vertices[0] as usize][0] as f64,
                    stl.vertices[face.vertices[0] as usize][1] as f64,
                    stl.vertices[face.vertices[0] as usize][2] as f64,
                );
                let v1 = Vec3::new(
                    stl.vertices[face.vertices[1] as usize][0] as f64,
                    stl.vertices[face.vertices[1] as usize][1] as f64,
                    stl.vertices[face.vertices[1] as usize][2] as f64,
                );
                let v2 = Vec3::new(
                    stl.vertices[face.vertices[2] as usize][0] as f64,
                    stl.vertices[face.vertices[2] as usize][1] as f64,
                    stl.vertices[face.vertices[2] as usize][2] as f64,
                );
                Triangle::new(v0, v1, v2)
            })
            .collect();
        Mesh {
            triangles,
            material,
        }
    }
    pub fn build_cube(center: Vec3, size: f64, material: Box<dyn Material>) -> Self {
        let half = size / 2.0;
        let v0 = center.add(&Vec3::new(-half, -half, -half));
        let v1 = center.add(&Vec3::new(half, -half, -half));
        let v2 = center.add(&Vec3::new(half, half, -half));
        let v3 = center.add(&Vec3::new(-half, half, -half));
        let v4 = center.add(&Vec3::new(-half, -half, half));
        let v5 = center.add(&Vec3::new(half, -half, half));
        let v6 = center.add(&Vec3::new(half, half, half));
        let v7 = center.add(&Vec3::new(-half, half, half));

        let triangles = vec![
            Triangle::new(v0, v1, v2),
            Triangle::new(v0, v2, v3),
            Triangle::new(v1, v5, v6),
            Triangle::new(v1, v6, v2),
            Triangle::new(v5, v4, v7),
            Triangle::new(v5, v7, v6),
            Triangle::new(v4, v0, v3),
            Triangle::new(v4, v3, v7),
            Triangle::new(v3, v2, v6),
            Triangle::new(v3, v6, v7),
            Triangle::new(v4, v5, v1),
            Triangle::new(v4, v1, v0),
        ];

        Mesh {
            triangles,
            material,
        }
    }
}
impl Hittable for Mesh {
    fn hit(&'_ self, ray: &Ray) -> Option<HitRecord<'_>> {
        let mut closest_hit: Option<HitRecord> = None;
        for tri in &self.triangles {
            if let Some(hit) = tri.hit(ray, self.material.as_ref()) {
                if closest_hit.is_none() || hit.t < closest_hit.as_ref().unwrap().t {
                    closest_hit = Some(hit);
                }
            }
        }
        closest_hit
    }

    fn bounding_box(&self) -> AABB {
        let mut min = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

        for t in &self.triangles {
            for v in [&t.v0, &t.v1, &t.v2] {
                min.x = min.x.min(v.x);
                min.y = min.y.min(v.y);
                min.z = min.z.min(v.z);
                max.x = max.x.max(v.x);
                max.y = max.y.max(v.y);
                max.z = max.z.max(v.z);
            }
        }

        AABB::new(min, max)
    }
}
