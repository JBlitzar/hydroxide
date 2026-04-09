use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    camera::Camera,
    geometry::{self, Hittable},
    renderer::Renderer,
    vec3::Vec3,
    world::World,
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
            MaterialDesc::Lambertian { albedo } => {
                Box::new(crate::material::Lambertian { albedo: *albedo })
            }
            MaterialDesc::Metal { albedo, fuzz } => Box::new(crate::material::Metal {
                albedo: *albedo,
                fuzz: *fuzz,
            }),
            MaterialDesc::Dielectric {
                albedo,
                refractive_index,
            } => Box::new(crate::material::Dielectric {
                albedo: *albedo,
                refractive_index: *refractive_index,
            }),
            MaterialDesc::DiffuseLight { albedo } => {
                Box::new(crate::material::DiffuseLight { albedo: *albedo })
            }
            MaterialDesc::Checkerboard {
                color_a,
                color_b,
                scale,
            } => Box::new(crate::material::Checkerboard {
                color_a: *color_a,
                color_b: *color_b,
                scale: *scale,
            }),
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
        vertices: Vec<Vec3>,
        faces: Vec<[u32; 3]>,
        material: MaterialDesc,
    },
    Plane {
        point: Vec3,
        normal: Vec3,
        material: MaterialDesc,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum HdrSkyId {
    CitrusOrchard,
    QwantaniMoonrise,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SkyDesc {
    Gradient { top: Vec3, bottom: Vec3 },
    Solid { color: Vec3 },
    Hdr { id: HdrSkyId, exposure: f64 },
}

impl ObjectDesc {
    pub fn build(&self) -> Arc<dyn Hittable> {
        match self {
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
                vertices,
                faces,
                material,
            } => Arc::new(geometry::mesh::MeshBVH::from_indexed(
                vertices,
                faces,
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
        }
    }
}

impl SkyDesc {
    pub fn build_sky(&self) -> Box<dyn crate::sky::Sky> {
        match self {
            SkyDesc::Gradient { top, bottom } => Box::new(crate::sky::GradientSky {
                top_color: *top,
                bottom_color: *bottom,
            }),
            SkyDesc::Solid { color } => Box::new(crate::sky::SolidColorSky { color: *color }),
            SkyDesc::Hdr { id, exposure } => {
                let filename = match id {
                    HdrSkyId::CitrusOrchard => "citrus_orchard_road_puresky_4k.hdr",
                    HdrSkyId::QwantaniMoonrise => "qwantani_moonrise_puresky_4k.hdr",
                };
                let local_path = format!("web/res/{}", filename);
                let mut sky = if std::path::Path::new(&local_path).exists() {
                    crate::sky::HDRSky::from_hdr_file(&local_path)
                } else {
                    Self::download_hdr(filename)
                };
                sky.exposure = *exposure;
                Box::new(sky)
            }
        }
    }

    fn download_hdr(filename: &str) -> crate::sky::HDRSky {
        let url = format!("https://hydroxide.pages.dev/res/{}", filename);
        eprintln!("HDR not found locally, downloading from {}...", url);
        #[cfg(feature = "native")]
        {
            let resp = ureq::get(&url).call().expect("Failed to download HDR");
            let len = resp
                .header("Content-Length")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            let mut bytes = Vec::with_capacity(len);
            std::io::Read::read_to_end(&mut resp.into_reader(), &mut bytes)
                .expect("Failed to read HDR response");
            eprintln!("Downloaded {} bytes", bytes.len());
            crate::sky::HDRSky::from_hdr_bytes(&bytes)
        }
        #[cfg(not(feature = "native"))]
        {
            panic!("HDR download not supported in this build");
        }
    }
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
        let objects = self.objects.iter().map(|o| o.build()).collect();
        let world = World::new(self.camera.clone(), objects, Some(self.sky.build_sky()));
        let renderer = Renderer::new(
            self.camera.width_px,
            self.camera.height_px,
            Some(self.samples),
            Some(self.termination_prob),
        );
        (world, renderer)
    }

    pub fn save(&self, path: &str) {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::BufWriter;

        let file = std::fs::File::create(path).expect("Failed to create scene file");
        let buf = BufWriter::new(file);
        let encoder = GzEncoder::new(buf, Compression::best());
        bincode::serialize_into(encoder, self).expect("Failed to serialize scene");
    }

    pub fn load(path: &str) -> Self {
        use flate2::read::GzDecoder;
        use std::io::BufReader;

        let file = std::fs::File::open(path).expect("Failed to read scene file");
        let buf = BufReader::new(file);
        let decoder = GzDecoder::new(buf);
        bincode::deserialize_from(decoder).expect("Failed to deserialize scene")
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        use flate2::Compression;
        use flate2::write::GzEncoder;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        bincode::serialize_into(&mut encoder, self).expect("Failed to serialize scene");
        encoder.finish().expect("Failed to finish gzip")
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        use flate2::read::GzDecoder;

        let decoder = GzDecoder::new(bytes);
        bincode::deserialize_from(decoder).expect("Failed to deserialize scene")
    }
}
