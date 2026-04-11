use crate::{
    light::SphereLight,
    vec3::{Ray, Vec3},
};
use image::ImageReader;
use std::io::Cursor;

pub trait Sky: Send + Sync {
    fn color(&self, ray: &Ray) -> Vec3;
    fn lights(&self) -> Vec<SphereLight> {
        vec![]
    }
    fn id(&self) -> usize {
        0
    }
    fn as_any(&self) -> &dyn std::any::Any;
}
pub struct GradientSky {
    pub top_color: Vec3,
    pub bottom_color: Vec3,
}
impl Sky for GradientSky {
    fn color(&self, ray: &Ray) -> Vec3 {
        let t = 0.5 * (ray.direction.normalize().y + 1.0);
        self.bottom_color
            .scalar_mul(1.0 - t)
            .add(&self.top_color.scalar_mul(t))
    }
    fn id(&self) -> usize {
        1
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct SolidColorSky {
    pub color: Vec3,
}
impl Sky for SolidColorSky {
    fn color(&self, _ray: &Ray) -> Vec3 {
        self.color
    }
    fn id(&self) -> usize {
        0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct HDRSky {
    pub data: Vec<Vec3>,
    pub width: usize,
    pub height: usize,
    pub exposure: f64,
}
impl Sky for HDRSky {
    fn color(&self, ray: &Ray) -> Vec3 {
        let dir = ray.direction.normalize();
        let u = 0.5 + dir.z.atan2(dir.x) / (2.0 * std::f64::consts::PI);
        let v = 0.5 - dir.y.asin() / std::f64::consts::PI;
        let x = (u * self.width as f64) as usize % self.width;
        let y = (v * self.height as f64) as usize % self.height;
        self.data[y * self.width + x].scalar_mul(self.exposure)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl HDRSky {
    pub fn from_hdr_bytes(bytes: &[u8]) -> Self {
        let img = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .expect("Failed to guess HDR format")
            .decode()
            .expect("Failed to decode HDR image")
            .to_rgb32f();
        let width = img.width() as usize;
        let height = img.height() as usize;
        let data = img
            .into_raw()
            .chunks(3)
            .map(|c| Vec3::new(c[0] as f64, c[1] as f64, c[2] as f64))
            .collect();
        HDRSky {
            data,
            width,
            height,
            exposure: 1.0,
        }
    }

    pub fn from_hdr_file(path: &str) -> Self {
        let data = ImageReader::open(path)
            .expect("Failed to open HDR file")
            .with_guessed_format()
            .expect("Failed to guess HDR format")
            .decode()
            .expect("Failed to decode HDR image")
            .to_rgb32f()
            .into_raw()
            .chunks(3)
            .map(|chunk| Vec3::new(chunk[0] as f64, chunk[1] as f64, chunk[2] as f64))
            .collect::<Vec<_>>()
            .into_iter()
            .fold(Vec::new(), |mut data, pixel| {
                data.push(pixel);
                data
            });

        let img = ImageReader::open(path)
            .expect("Failed to open HDR file")
            .with_guessed_format()
            .expect("Failed to guess HDR format")
            .decode()
            .expect("Failed to decode HDR image")
            .to_rgb32f();
        let width = img.width() as usize;
        let height = img.height() as usize;

        HDRSky {
            data,
            width,
            height,
            exposure: 1.0,
        }
    }

    fn id(&self) -> usize {
        3
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
