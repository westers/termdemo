use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct Kaleidoscope {
    width: u32,
    height: u32,
    speed: f64,
    segments: f64,
}

impl Kaleidoscope {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            segments: 6.0,
        }
    }
}

impl Effect for Kaleidoscope {
    fn name(&self) -> &str {
        "Kaleidoscope"
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
        let t = t * self.speed;
        let num_segments = self.segments.round().max(2.0) as u32;
        let segment_angle = PI * 2.0 / num_segments as f64;

        for y in 0..h {
            for x in 0..w {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;

                // Polar coordinates
                let r = (dx * dx + dy * dy).sqrt();
                let mut angle = dy.atan2(dx);
                if angle < 0.0 {
                    angle += PI * 2.0;
                }

                // Mirror into first segment
                let seg = angle / segment_angle;
                let mut local_angle = (seg.fract()) * segment_angle;
                // Mirror odd segments for seamless reflection
                if seg as u32 % 2 == 1 {
                    local_angle = segment_angle - local_angle;
                }

                // Convert back to cartesian for pattern sampling
                // Add slow rotation
                let rot = t * 0.2;
                let sx = r * (local_angle + rot).cos();
                let sy = r * (local_angle + rot).sin();

                // Normalize to a reasonable scale
                let scale = 0.02;
                let fx = sx * scale;
                let fy = sy * scale;

                // Multi-layer procedural pattern (plasma-like)
                let v1 = (fx * 5.0 + t * 0.8).sin();
                let v2 = (fy * 7.0 - t * 0.6).cos();
                let v3 = ((fx + fy) * 4.0 + t * 0.5).sin();
                let v4 = ((fx * fx + fy * fy).sqrt() * 6.0 - t * 1.2).sin();
                let v5 = ((fx * 3.0 - fy * 2.0 + t * 0.3).sin()
                    * (fx * 2.0 + fy * 3.0 - t * 0.4).cos()) * 0.8;

                let v = (v1 + v2 + v3 + v4 + v5) * 0.2;

                // Color: rich saturated palette
                let hue = (v * 0.5 + 0.5 + t * 0.03) % 1.0;

                // Radial brightness: bright center, gentle fade at edges
                let max_r = (cx * cx + cy * cy).sqrt();
                let brightness = (1.0 - (r / max_r).powf(1.5) * 0.5).clamp(0.3, 1.0);

                // Boost saturation at pattern peaks
                let sat = 0.7 + 0.3 * v.abs();

                let (cr, cg, cb) = hsv_to_rgb(hue.abs(), sat.clamp(0.0, 1.0), brightness);

                let idx = (y * w + x) as usize;
                pixels[idx] = (cr, cg, cb);
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
                name: "segments".to_string(),
                min: 2.0,
                max: 12.0,
                value: self.segments,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "segments" => self.segments = value,
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
