use raylib::prelude::*;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub colors: Vec<Vector3>,
    pub depths: Vec<f32>,
    background: Vec<Vector3>,
}

fn hash(x: u32, y: u32) -> f32 {
    let mut h = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 0xffff) as f32 / 65535.0
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        // Fondo: cielo oscuro con estrellitas generadas con un hash (no es textura).
        let mut background = vec![Vector3::new(0.006, 0.008, 0.02); width * height];
        for y in 0..height {
            for x in 0..width {
                let h = hash(x as u32, y as u32);
                if h > 0.9975 {
                    let b = (h - 0.9975) / 0.0025;
                    background[y * width + x] = Vector3::new(0.5, 0.55, 0.7) * (0.3 + b * 0.7);
                }
            }
        }
        Self {
            width,
            height,
            colors: background.clone(),
            depths: vec![f32::INFINITY; width * height],
            background,
        }
    }

    pub fn clear(&mut self) {
        self.colors.copy_from_slice(&self.background);
        self.depths.fill(f32::INFINITY);
    }

    pub fn to_rgba(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.width * self.height * 4);
        for c in &self.colors {
            for v in [c.x, c.y, c.z] {
                out.push((v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
            }
            out.push(255);
        }
        out
    }

    pub fn save_png(&self, path: &str) {
        let img = image::RgbaImage::from_raw(self.width as u32, self.height as u32, self.to_rgba())
            .expect("tamaño de imagen");
        if let Some(dir) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        img.save(path).expect("no se pudo guardar el PNG");
    }
}
