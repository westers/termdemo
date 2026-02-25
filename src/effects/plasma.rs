use crate::effect::{Effect, ParamDesc};

pub struct Plasma {
    width: u32,
    height: u32,
    speed: f64,
    scale: f64,
}

impl Plasma {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            scale: 1.0,
        }
    }

    pub fn with_params(speed: f64, scale: f64) -> Self {
        Self {
            width: 0,
            height: 0,
            speed,
            scale,
        }
    }
}

impl Effect for Plasma {
    fn name(&self) -> &str {
        "Plasma"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width as f64;
        let h = self.height as f64;
        if w == 0.0 || h == 0.0 {
            return;
        }

        let t = t * self.speed;
        let scale = self.scale;

        for y in 0..self.height {
            for x in 0..self.width {
                let fx = x as f64 / w * scale;
                let fy = y as f64 / h * scale;

                let v1 = (fx * 10.0 + t).sin();
                let v2 = ((fy * 10.0 + t) * 0.7).sin();
                let v3 = ((fx * 6.0 + fy * 6.0 + t * 0.8).sin()
                    + (fx * fx + fy * fy).sqrt().sin())
                    * 0.5;
                let v4 = ((fx * fx + fy * fy).sqrt() * 4.0 - t * 1.2).sin();

                let v = (v1 + v2 + v3 + v4) * 0.25;

                let r = ((v * std::f64::consts::PI).cos() * 0.5 + 0.5) * 255.0;
                let g = ((v * std::f64::consts::PI + 2.094).cos() * 0.5 + 0.5) * 255.0;
                let b = ((v * std::f64::consts::PI + 4.189).cos() * 0.5 + 0.5) * 255.0;

                let idx = (y * self.width + x) as usize;
                pixels[idx] = (r as u8, g as u8, b as u8);
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "speed".to_string(),
                min: 0.1,
                max: 5.0,
                value: self.speed,
            },
            ParamDesc {
                name: "scale".to_string(),
                min: 0.2,
                max: 4.0,
                value: self.scale,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "scale" => self.scale = value,
            _ => {}
        }
    }
}
