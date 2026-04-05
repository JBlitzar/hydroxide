use std::sync::Arc;

use crate::{
    geometry::{Hittable, sphere::Sphere},
    material::DiffuseLight,
    vec3::Vec3,
};

pub struct SphereLight {
    pub center: Vec3,
    pub radius: f64,
    pub Le: Vec3,
}

impl SphereLight {
    pub fn new(center: Vec3, radius: f64, Le: Vec3) -> Self {
        Self { center, radius, Le }
    }
    pub fn of_sphobject(sph: &Sphere) -> Self {
        if let Some(dl) = sph
            .material
            .as_ref()
            .as_any()
            .downcast_ref::<DiffuseLight>()
        {
            return Self {
                center: sph.center,
                radius: sph.radius,
                Le: dl.albedo,
            };
        }
        Self {
            center: sph.center,
            radius: sph.radius,
            Le: Vec3::new(0.0, 0.0, 0.0),
        }
    }
    pub fn of_mixed_objects(objects: Vec<Arc<dyn Hittable>>) -> Vec<Self> {
        objects
            .into_iter()
            .filter_map(|obj| {
                let sph = obj.as_ref().as_any().downcast_ref::<Sphere>()?;
                let dl = sph
                    .material
                    .as_ref()
                    .as_any()
                    .downcast_ref::<DiffuseLight>()?;

                Some(Self {
                    center: sph.center,
                    radius: sph.radius,
                    Le: dl.albedo,
                })
            })
            .collect()
    }
}
