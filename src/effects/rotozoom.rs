use crate::effect::{Effect, ParamDesc};

pub struct Rotozoom {
    width: u32,
    height: u32,
    rotation_speed: f64,
    zoom_speed: f64,
}

impl Rotozoom {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            rotation_speed: 1.0,
            zoom_speed: 1.0,
        }
    }
}

impl Effect for Rotozoom {
    fn name(&self) -> &str {
        "Rotozoom"
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

        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;

        let angle = t * self.rotation_speed * 0.8;
        let zoom = 1.5 + (t * self.zoom_speed * 0.7).sin() * 1.2;

        let cos_a = angle.cos() / zoom;
        let sin_a = angle.sin() / zoom;

        for y in 0..h {
            for x in 0..w {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;

                // Inverse-rotate to get texture coordinates
                let u = dx * cos_a + dy * sin_a;
                let v = -dx * sin_a + dy * cos_a;

                // XOR texture
                let pattern = ((u.abs() as u32) & 255) ^ ((v.abs() as u32) & 255);
                let normalized = pattern as f64 / 255.0;

                let (r, g, b) = hsv_to_rgb(
                    (normalized + t * 0.2) % 1.0,
                    0.8,
                    normalized * 0.7 + 0.3,
                );

                let idx = (y * w + x) as usize;
                pixels[idx] = (r, g, b);
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "rot_speed".to_string(),
                min: 0.1,
                max: 5.0,
                value: self.rotation_speed,
            },
            ParamDesc {
                name: "zoom_speed".to_string(),
                min: 0.1,
                max: 5.0,
                value: self.zoom_speed,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "rot_speed" => self.rotation_speed = value,
            "zoom_speed" => self.zoom_speed = value,
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
