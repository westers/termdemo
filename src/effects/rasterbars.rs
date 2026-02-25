use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct RasterBars {
    width: u32,
    height: u32,
    bar_count: u32,
    amplitude: f64,
}

impl RasterBars {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            bar_count: 7,
            amplitude: 1.0,
        }
    }
}

impl Effect for RasterBars {
    fn name(&self) -> &str {
        "RasterBars"
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

        // Clear to black
        for p in pixels.iter_mut() {
            *p = (0, 0, 0);
        }

        let hf = h as f64;
        let bar_count = self.bar_count as usize;

        for i in 0..bar_count {
            let phase = i as f64 * PI * 2.0 / bar_count as f64;
            let freq = 1.0 + i as f64 * 0.3;
            let center_y = hf * 0.5 + (t * freq + phase).sin() * self.amplitude * hf * 0.35;

            // Rainbow hue per bar
            let hue = (i as f64 / bar_count as f64 + t * 0.1) % 1.0;
            let (br, bg, bb) = hsv_to_rgb(hue, 1.0, 1.0);

            let bar_half = 8.0; // half-height of bar in pixels

            for y in 0..h {
                let dy = (y as f64 - center_y).abs();
                if dy > bar_half {
                    continue;
                }

                // Quadratic brightness falloff from center
                let falloff = 1.0 - (dy / bar_half).powi(2);
                let bright = (falloff * 255.0) as u8;

                let cr = ((br as u16 * bright as u16) / 255) as u8;
                let cg = ((bg as u16 * bright as u16) / 255) as u8;
                let cb = ((bb as u16 * bright as u16) / 255) as u8;

                let row_start = (y * w) as usize;
                for x in 0..w {
                    let idx = row_start + x as usize;
                    let p = &mut pixels[idx];
                    p.0 = p.0.saturating_add(cr);
                    p.1 = p.1.saturating_add(cg);
                    p.2 = p.2.saturating_add(cb);
                }
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "bar_count".to_string(),
                min: 3.0,
                max: 12.0,
                value: self.bar_count as f64,
            },
            ParamDesc {
                name: "amplitude".to_string(),
                min: 0.2,
                max: 2.0,
                value: self.amplitude,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "bar_count" => self.bar_count = value as u32,
            "amplitude" => self.amplitude = value,
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
