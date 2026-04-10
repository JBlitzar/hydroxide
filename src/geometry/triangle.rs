use crate::{
    material::{HitRecord, Material},
    vec3::{Ray, Vec3},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone)]
pub struct Triangle {
    pub(crate) v0: Vec3,
    pub(crate) v1: Vec3,
    pub(crate) v2: Vec3,
    pub normal: Vec3,
    pub n0: Vec3,
    pub n1: Vec3,
    pub n2: Vec3,
    pub e01: Vec3,
    pub e02: Vec3,
}

#[derive(Serialize, Deserialize)]
struct TriangleSerde {
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    n0: Vec3,
    n1: Vec3,
    n2: Vec3,
}

impl Serialize for Triangle {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TriangleSerde {
            v0: self.v0,
            v1: self.v1,
            v2: self.v2,
            n0: self.n0,
            n1: self.n1,
            n2: self.n2,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Triangle {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = TriangleSerde::deserialize(deserializer)?;
        Ok(Triangle::new_with_normals(
            s.v0,
            s.v1,
            s.v2,
            Some(s.n0),
            Some(s.n1),
            Some(s.n2),
        ))
    }
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        Self::new_with_normals(v0, v1, v2, None, None, None)
    }

    pub fn new_with_normals(
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        n0: Option<Vec3>,
        n1: Option<Vec3>,
        n2: Option<Vec3>,
    ) -> Self {
        let e01 = v1.sub(&v0);
        let e02 = v2.sub(&v0);
        let normal = e01.cross(&e02).normalize();
        let n0 = n0.unwrap_or(normal);
        let n1 = n1.unwrap_or(normal);
        let n2 = n2.unwrap_or(normal);
        Triangle {
            v0,
            v1,
            v2,
            normal,
            n0,
            n1,
            n2,
            e01,
            e02,
        }
    }

    pub fn hit<'a>(&self, ray: &Ray, material: &'a dyn Material) -> Option<HitRecord<'a>> {
        // https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection.html#:~:text=.-,Implementation,-Implementing%20the%20M%C3%B6ller
        let v0v1 = self.e01;
        let v0v2 = self.e02;
        let pvec = ray.direction.cross(&v0v2);
        let det = v0v1.dot(&pvec);
        if det.abs() < 1e-8 {
            return None;
        }
        let inv_det = 1.0 / det;

        let tvec = ray.origin.sub(&self.v0);
        let u = tvec.dot(&pvec) * inv_det;
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let qvec = tvec.cross(&v0v1);
        let v = ray.direction.dot(&qvec) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = v0v2.dot(&qvec) * inv_det;
        if t < 0.001 {
            return None;
        }
        let w = 1.0 - u - v;
        let cross = self.e01.cross(&self.e02);
        if cross.length_squared() < 1e-12 {
            return None;
        }
        let geo_normal = cross.normalize();
        let mut normal = self
            .n0
            .scalar_mul(w)
            .add(&self.n1.scalar_mul(u))
            .add(&self.n2.scalar_mul(v))
            .normalize();
        // Ensure interpolated normal is on the same side as the geometric normal
        if normal.dot(&geo_normal) < 0.0 {
            normal = normal.scalar_mul(-1.0);
        }
        // Flip both if ray hits from the back
        let mut gn = geo_normal;
        if ray.direction.dot(&geo_normal) > 0.0 {
            normal = normal.scalar_mul(-1.0);
            gn = gn.scalar_mul(-1.0);
        }
        Some(HitRecord {
            point: ray.origin.add(&ray.direction.scalar_mul(t)),
            normal,
            geo_normal: gn,
            material,
            t,
        })
    }
}
