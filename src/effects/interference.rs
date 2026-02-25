use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct Interference {
    width: u32,
    height: u32,
    frequency: f64,
    speed: f64,
}

impl Interference {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            frequency: 3.0,
            speed: 1.0,
        }
    }

    fn palette(v: f64) -> (u8, u8, u8) {
        // v is in range [-2, 2] from sum of two sin waves, but with 3 sources [-3, 3]
        // Normalize to [0, 1]
        let n = (v / 3.0 + 1.0) * 0.5;
        let n = n.clamp(0.0, 1.0);

        // Dark blue -> purple -> cyan -> white
        if n < 0.25 {
            let t = n / 0.25;
            (
                (10.0 + t * 40.0) as u8,
                (5.0 + t * 10.0) as u8,
                (40.0 + t * 80.0) as u8,
            )
        } else if n < 0.5 {
            let t = (n - 0.25) / 0.25;
            (
                (50.0 + t * 80.0) as u8,
                (15.0 + t * 30.0) as u8,
                (120.0 + t * 60.0) as u8,
            )
        } else if n < 0.75 {
            let t = (n - 0.5) / 0.25;
            (
                (130.0 - t * 80.0) as u8,
                (45.0 + t * 160.0) as u8,
                (180.0 + t * 40.0) as u8,
            )
        } else {
            let t = (n - 0.75) / 0.25;
            (
                (50.0 + t * 205.0) as u8,
                (205.0 + t * 50.0) as u8,
                (220.0 + t * 35.0) as u8,
            )
        }
    }
}

impl Effect for Interference {
    fn name(&self) -> &str {
        "Interference"
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
        let cx = wf / 2.0;
        let cy = hf / 2.0;

        let ts = t * self.speed;

        // Three sources moving in circular paths
        let r1 = wf.min(hf) * 0.25;
        let r2 = wf.min(hf) * 0.3;
        let r3 = wf.min(hf) * 0.2;

        let s1x = cx + r1 * (ts * 0.4).cos();
        let s1y = cy + r1 * (ts * 0.4).sin();

        let s2x = cx + r2 * (ts * 0.3 + PI * 2.0 / 3.0).cos();
        let s2y = cy + r2 * (ts * 0.35 + PI * 2.0 / 3.0).sin();

        let s3x = cx + r3 * (ts * 0.5 + PI * 4.0 / 3.0).cos();
        let s3y = cy + r3 * (ts * 0.45 + PI * 4.0 / 3.0).sin();

        let freq = self.frequency * 0.15;

        for y in 0..h {
            let fy = y as f64;
            for x in 0..w {
                let fx = x as f64;

                let d1 = ((fx - s1x) * (fx - s1x) + (fy - s1y) * (fy - s1y)).sqrt();
                let d2 = ((fx - s2x) * (fx - s2x) + (fy - s2y) * (fy - s2y)).sqrt();
                let d3 = ((fx - s3x) * (fx - s3x) + (fy - s3y) * (fy - s3y)).sqrt();

                let v1 = (d1 * freq - ts * 3.0).sin();
                let v2 = (d2 * freq - ts * 3.0).sin();
                let v3 = (d3 * freq - ts * 3.0).sin();

                let combined = v1 + v2 + v3;

                let idx = (y * w + x) as usize;
                pixels[idx] = Self::palette(combined);
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "frequency".to_string(),
                min: 1.0,
                max: 5.0,
                value: self.frequency,
            },
            ParamDesc {
                name: "speed".to_string(),
                min: 0.5,
                max: 3.0,
                value: self.speed,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "frequency" => self.frequency = value,
            "speed" => self.speed = value,
            _ => {}
        }
    }
}
