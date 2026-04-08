use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    camera::Camera, geometry::{self, Hittable, triangle::Triangle}, renderer::Renderer, vec3::Vec3, world::World,
};
#[derive(Serialize, Deserialize, Clone)]
pub enum MaterialDesc {
    Lambertian {
        albedo: Vec3,
    },
    Metal {
        albedo: Vec3,
        fuzz: f64,
    },
    Dielectric {
        albedo: Vec3,
        refractive_index: f64,
    },
    DiffuseLight {
        albedo: Vec3,
    },
    Checkerboard {
        color_a: Vec3,
        color_b: Vec3,
        scale: f64,
    },
}
impl MaterialDesc {
    pub fn build(&self) -> Box<dyn crate::material::Material> {
        match self {
            MaterialDesc::Lambertian { albedo } => Box::new(crate::material::Lambertian { albedo: *albedo }),
            MaterialDesc::Metal { albedo, fuzz } => Box::new(crate::material::Metal { albedo: *albedo, fuzz: *fuzz }),
            MaterialDesc::Dielectric { albedo, refractive_index } => Box::new(crate::material::Dielectric { albedo: *albedo, refractive_index: *refractive_index }),
            MaterialDesc::DiffuseLight { albedo } => Box::new(crate::material::DiffuseLight { albedo: *albedo }),
            MaterialDesc::Checkerboard { color_a, color_b, scale } => Box::new(crate::material::Checkerboard { color_a: *color_a, color_b: *color_b, scale: *scale }),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ObjectDesc {
    Sphere {
        center: Vec3,
        radius: f64,
        material: MaterialDesc,
    },
    Mesh {
        triangles: Vec<Triangle>,
        material: MaterialDesc,
    },
    Plane {
        point: Vec3,
        normal: Vec3,
        material: MaterialDesc,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SkyDesc {
    Gradient {
        top: Vec3,
        bottom: Vec3,
    },
    Solid {
        color: Vec3,
    },
    HdrData {
        pixels: Vec<Vec3>,
        width: usize,
        height: usize,
        exposure: f64,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SceneDescription {
    pub camera: Camera,
    pub objects: Vec<ObjectDesc>,
    pub sky: SkyDesc,
    pub samples: usize,
    pub termination_prob: f64,
}

impl SceneDescription {
    pub fn build(&self) -> (World, Renderer) {
        let objects = self
            .objects
            .iter()
            .map(|obj_desc| match obj_desc {
                ObjectDesc::Sphere {
                    center,
                    radius,
                    material,
                } => Arc::new(geometry::sphere::Sphere {
                    center: *center,
                    radius: *radius,
                    material: material.build(),
                }) as Arc<dyn Hittable>,
                ObjectDesc::Mesh {
                    triangles,
                    material,
                } => Arc::new(geometry::mesh::MeshBVH::new(
                    triangles.clone(),
                    material.build(),
                )) as Arc<dyn Hittable>,
                ObjectDesc::Plane {
                    point,
                    normal,
                    material,
                } => Arc::new(geometry::plane::Plane {
                    point: *point,
                    normal: *normal,
                    material: material.build(),
                }) as Arc<dyn Hittable>,
            })
            .collect();

        let sky = match &self.sky {
            SkyDesc::Gradient { top, bottom } => Box::new(crate::sky::GradientSky {
                top_color: *top,
                bottom_color: *bottom,
            }) as Box<dyn crate::sky::Sky>,
            SkyDesc::Solid { color } => {
                Box::new(crate::sky::SolidColorSky { color: *color }) as Box<dyn crate::sky::Sky>
            }
            SkyDesc::HdrData {
                pixels,
                width,
                height,
                exposure,
            } => Box::new(crate::sky::HDRSky {
                data: pixels.clone(),
                width: *width,
                height: *height,
                exposure: *exposure,
            }) as Box<dyn crate::sky::Sky>,
        };

        let world = World::new(self.camera.clone(), objects, Some(sky));
        let renderer = Renderer::new(
            self.camera.width_px,
            self.camera.height_px,
            Some(self.samples),
            Some(self.termination_prob),
        );
        (world, renderer)
    }

    fn save(&self, path: &str) {
        let bytes = bincode::serialize(self).expect("Failed to serialize scene");
        std::fs::write(path, bytes).expect("Failed to write scene file");
    }
    fn load(path: &str) -> Self {
        let bytes = std::fs::read(path).expect("Failed to read scene file");
        bincode::deserialize(&bytes).expect("Failed to deserialize scene")
    }

    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize scene")
    }
    fn from_bytes(bytes: &[u8]) -> Self {
        bincode::deserialize(bytes).expect("Failed to deserialize scene")
    }
}
