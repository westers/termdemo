use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct BumpMapping {
    width: u32,
    height: u32,
    light_speed: f64,
    texture_scale: f64,
    heightmap: Vec<f64>,
}

impl BumpMapping {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            light_speed: 1.0,
            texture_scale: 1.0,
            heightmap: Vec::new(),
        }
    }

    fn generate_heightmap(&mut self) {
        let w = self.width as usize;
        let h = self.height as usize;
        self.heightmap = vec![0.0; w * h];
        let scale = self.texture_scale;

        for y in 0..h {
            for x in 0..w {
                let fx = x as f64 / w as f64 * scale;
                let fy = y as f64 / h as f64 * scale;

                // Multi-octave procedural heightmap: ripples + bumps
                let mut v = 0.0;
                // Large ripples
                v += (fx * 6.0 * PI).sin() * (fy * 6.0 * PI).cos() * 0.4;
                // Medium bumps
                v += (fx * 12.0 * PI + 1.0).sin() * (fy * 10.0 * PI + 2.0).cos() * 0.25;
                // Fine detail
                v += (fx * 24.0 * PI + 3.0).sin() * (fy * 20.0 * PI + 5.0).cos() * 0.15;
                // Radial rings from center
                let dx = fx * 2.0 - scale;
                let dy = fy * 2.0 - scale;
                let dist = (dx * dx + dy * dy).sqrt();
                v += (dist * 8.0 * PI).sin() * 0.2;

                self.heightmap[y * w + x] = v;
            }
        }
    }
}

impl Effect for BumpMapping {
    fn name(&self) -> &str {
        "BumpMapping"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.generate_heightmap();
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width as usize;
        let h = self.height as usize;
        if w < 2 || h < 2 || self.heightmap.is_empty() {
            return;
        }

        let t = t * self.light_speed;

        // Two moving light sources on Lissajous paths
        let lx0 = 0.5 + 0.4 * (t * 0.7).sin();
        let ly0 = 0.5 + 0.4 * (t * 0.9).cos();
        let lx1 = 0.5 + 0.35 * (t * 0.5 + 2.0).sin();
        let ly1 = 0.5 + 0.35 * (t * 0.8 + 1.0).cos();

        let light0_x = lx0 * w as f64;
        let light0_y = ly0 * h as f64;
        let light1_x = lx1 * w as f64;
        let light1_y = ly1 * h as f64;

        // Base color palette (warm copper/gold tones)
        let base_hue = (t * 0.03) % 1.0;

        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let idx = y * w + x;

                // Compute surface normal from heightmap gradient (central differences)
                let dhdx = self.heightmap[idx + 1] - self.heightmap[idx - 1];
                let dhdy = self.heightmap[idx + w] - self.heightmap[idx - w];

                // Normal = (-dhdx, -dhdy, 1), unnormalized (the 1 is the flat surface)
                let nx = -dhdx;
                let ny = -dhdy;
                // nz = 1.0 implicitly (skip normalization for speed, just scale)

                // Light 0: direction from surface to light
                let ldx0 = light0_x - x as f64;
                let ldy0 = light0_y - y as f64;
                let ldist0 = (ldx0 * ldx0 + ldy0 * ldy0).sqrt().max(1.0);
                // Dot product of normal with light direction (z component contributes)
                let dot0 = (nx * ldx0 + ny * ldy0 + ldist0 * 0.5) / (ldist0 * 1.5);
                // Distance attenuation
                let atten0 = 1.0 / (1.0 + ldist0 * 0.005);
                let light0 = (dot0 * atten0).max(0.0);

                // Light 1
                let ldx1 = light1_x - x as f64;
                let ldy1 = light1_y - y as f64;
                let ldist1 = (ldx1 * ldx1 + ldy1 * ldy1).sqrt().max(1.0);
                let dot1 = (nx * ldx1 + ny * ldy1 + ldist1 * 0.5) / (ldist1 * 1.5);
                let atten1 = 1.0 / (1.0 + ldist1 * 0.005);
                let light1 = (dot1 * atten1).max(0.0);

                // Combine lights with different tints
                let brightness = (light0 + light1).clamp(0.0, 1.5);

                // Color: warm base with height variation
                let height_hue = (base_hue + self.heightmap[idx] * 0.1) % 1.0;
                let (br, bg, bb) = hsv_to_rgb(height_hue.abs(), 0.7, 0.15);

                // Apply lighting
                let r = ((br as f64 + light0 * 200.0 + light1 * 120.0) * brightness.min(1.0))
                    .clamp(0.0, 255.0) as u8;
                let g = ((bg as f64 + light0 * 140.0 + light1 * 180.0) * brightness.min(1.0))
                    .clamp(0.0, 255.0) as u8;
                let b = ((bb as f64 + light0 * 80.0 + light1 * 200.0) * brightness.min(1.0))
                    .clamp(0.0, 255.0) as u8;

                pixels[idx] = (r, g, b);
            }
        }

        // Fill edges (skipped by the 1..h-1 loop)
        for x in 0..w {
            pixels[x] = pixels[w + x.min(w - 2).max(1)];
            pixels[(h - 1) * w + x] = pixels[(h - 2) * w + x.min(w - 2).max(1)];
        }
        for y in 0..h {
            pixels[y * w] = pixels[y * w + 1];
            pixels[y * w + w - 1] = pixels[y * w + w - 2];
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "light_speed".to_string(),
                min: 0.2,
                max: 3.0,
                value: self.light_speed,
            },
            ParamDesc {
                name: "texture_scale".to_string(),
                min: 0.5,
                max: 4.0,
                value: self.texture_scale,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "light_speed" => self.light_speed = value,
            "texture_scale" => {
                self.texture_scale = value;
                if self.width > 0 && self.height > 0 {
                    self.generate_heightmap();
                }
            }
            _ => {}
        }
    }
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}
