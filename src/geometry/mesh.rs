use crate::{
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
    pub e12: Vec3,
    pub e20: Vec3,
}
impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        let e01 = v1.sub(&v0);
        let e02 = v2.sub(&v0);
        let e12 = v2.sub(&v1);
        let e20 = v0.sub(&v2);
        let normal = e01.cross(&e02).normalize();
        Triangle {
            v0,
            v1,
            v2,
            normal,
            e01,
            e12,
            e20,
        }
    }

    fn hit<'a>(&self, ray: &Ray, material: &'a dyn Material) -> Option<HitRecord<'a>> {
        // https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution.html
        let NdotRayDirection = self.normal.dot(&ray.direction);
        if NdotRayDirection.abs() < 1e-6 {
            return None;
        }
        let d = self.normal.dot(&self.v0);
        let t = (d - self.normal.dot(&ray.origin)) / NdotRayDirection;
        if t < 0.001 {
            return None;
        }
        let P = ray.origin.add(&ray.direction.scalar_mul(t));
        let mut Ne: Vec3;
        let v0p = P.sub(&self.v0);
        Ne = self.e01.cross(&v0p);
        if self.normal.dot(&Ne) < 0.0 {
            return None;
        }

        let v1p = P.sub(&self.v1);
        Ne = self.e12.cross(&v1p);
        if self.normal.dot(&Ne) < 0.0 {
            return None;
        }

        let v2p = P.sub(&self.v2);
        Ne = self.e20.cross(&v2p);
        if self.normal.dot(&Ne) < 0.0 {
            return None;
        }

        Some(HitRecord {
            point: P,
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
}
