use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct Moire {
    width: u32,
    height: u32,
    speed: f64,
    frequency: f64,
}

impl Moire {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            frequency: 1.0,
        }
    }
}

impl Effect for Moire {
    fn name(&self) -> &str {
        "Moire"
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

        let t = t * self.speed;
        let freq = self.frequency * 40.0;

        // Three ring centers on Lissajous paths (normalized 0–1)
        let cx0 = 0.5 + 0.3 * (t * 0.7).sin();
        let cy0 = 0.5 + 0.3 * (t * 0.9).cos();
        let cx1 = 0.5 + 0.3 * (t * 1.1 + 2.0).sin();
        let cy1 = 0.5 + 0.3 * (t * 0.8 + 1.0).cos();
        let cx2 = 0.5 + 0.3 * (t * 0.6 + 4.0).sin();
        let cy2 = 0.5 + 0.3 * (t * 1.3 + 3.0).cos();

        let wf = w as f64;
        let hf = h as f64;

        for y in 0..h {
            let ny = y as f64 / hf;
            for x in 0..w {
                let nx = x as f64 / wf;

                let d0 = ((nx - cx0).powi(2) + (ny - cy0).powi(2)).sqrt();
                let d1 = ((nx - cx1).powi(2) + (ny - cy1).powi(2)).sqrt();
                let d2 = ((nx - cx2).powi(2) + (ny - cy2).powi(2)).sqrt();

                let p0 = (d0 * freq).sin();
                let p1 = (d1 * freq).sin();
                let p2 = (d2 * freq).sin();

                let v = p0 * p1 * p2;
                let v = v * 0.5 + 0.5; // normalize to 0–1

                // Cosine palette with time hue cycling
                let hue = t * 0.15;
                let r = (0.5 + 0.5 * (PI * (v * 2.0 + hue)).cos()).clamp(0.0, 1.0);
                let g = (0.5 + 0.5 * (PI * (v * 2.0 + hue + 0.33)).cos()).clamp(0.0, 1.0);
                let b = (0.5 + 0.5 * (PI * (v * 2.0 + hue + 0.67)).cos()).clamp(0.0, 1.0);

                let idx = (y * w + x) as usize;
                pixels[idx] = (
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                );
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "speed".to_string(),
                min: 0.2,
                max: 3.0,
                value: self.speed,
            },
            ParamDesc {
                name: "frequency".to_string(),
                min: 0.5,
                max: 4.0,
                value: self.frequency,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "frequency" => self.frequency = value,
            _ => {}
        }
    }
}
