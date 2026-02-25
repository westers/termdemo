use crate::effect::{Effect, ParamDesc};

pub struct Lens {
    width: u32,
    height: u32,
    lens_size: f64,
    strength: f64,
}

impl Lens {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            lens_size: 1.0,
            strength: 0.5,
        }
    }
}

impl Effect for Lens {
    fn name(&self) -> &str {
        "Lens"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let wf = w as f64;
        let hf = h as f64;
        let dim = wf.min(hf);

        // Lens center on Lissajous path
        let lcx = 0.5 + 0.3 * (t * 0.5).sin();
        let lcy = 0.5 + 0.3 * (t * 0.7).cos();
        let radius = self.lens_size * dim * 0.25;

        for y in 0..h {
            let ny = y as f64 / hf;
            for x in 0..w {
                let nx = x as f64 / wf;

                // Distance from lens center (in pixel space)
                let dx = x as f64 - lcx * wf;
                let dy = y as f64 - lcy * hf;
                let dist = (dx * dx + dy * dy).sqrt();

                // Sample coordinates (may be displaced by lens)
                let (sx, sy) = if dist < radius {
                    let norm_dist = dist / radius;
                    // Displacement toward center
                    let displacement = 1.0 - self.strength * (1.0 - norm_dist * norm_dist);
                    let sample_x = lcx * wf + dx * displacement;
                    let sample_y = lcy * hf + dy * displacement;
                    (sample_x / wf, sample_y / hf)
                } else {
                    (nx, ny)
                };

                // Background: checkerboard + plasma tint
                let check_size = 0.05;
                let cx = (sx / check_size).floor() as i32;
                let cy = (sy / check_size).floor() as i32;
                let checker = ((cx + cy) & 1) as f64;

                // Plasma tint
                let plasma = (sx * 10.0 + t).sin() * 0.5 + 0.5;
                let hue = (plasma + t * 0.1) % 1.0;
                let base_v = 0.3 + checker * 0.4;

                let (mut r, mut g, mut b) = hsv_to_rgb(hue, 0.7, base_v);

                // Edge highlight ring at lens boundary
                if dist > radius * 0.9 && dist < radius * 1.1 {
                    let ring = 1.0 - ((dist - radius).abs() / (radius * 0.1)).clamp(0.0, 1.0);
                    let highlight = (ring * 80.0) as u8;
                    r = r.saturating_add(highlight);
                    g = g.saturating_add(highlight);
                    b = b.saturating_add(highlight);
                }

                let idx = (y * w + x) as usize;
                pixels[idx] = (r, g, b);
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "lens_size".to_string(),
                min: 0.3,
                max: 2.0,
                value: self.lens_size,
            },
            ParamDesc {
                name: "strength".to_string(),
                min: 0.1,
                max: 0.9,
                value: self.strength,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "lens_size" => self.lens_size = value,
            "strength" => self.strength = value,
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
