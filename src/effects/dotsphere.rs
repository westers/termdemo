use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

struct Point3D {
    x: f64,
    y: f64,
    z: f64,
}

pub struct DotSphere {
    width: u32,
    height: u32,
    rot_speed: f64,
    dot_count: u32,
    points: Vec<Point3D>,
}

impl DotSphere {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            rot_speed: 1.0,
            dot_count: 300,
            points: Vec::new(),
        }
    }

    fn generate_points(count: u32) -> Vec<Point3D> {
        // Fibonacci spiral for even distribution on sphere
        let golden_ratio = (1.0 + 5.0_f64.sqrt()) / 2.0;
        let n = count as f64;
        (0..count)
            .map(|i| {
                let theta = 2.0 * PI * i as f64 / golden_ratio;
                let phi = (1.0 - 2.0 * (i as f64 + 0.5) / n).acos();
                Point3D {
                    x: phi.sin() * theta.cos(),
                    y: phi.sin() * theta.sin(),
                    z: phi.cos(),
                }
            })
            .collect()
    }
}

impl Effect for DotSphere {
    fn name(&self) -> &str {
        "DotSphere"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.points = Self::generate_points(self.dot_count);
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        // Dark background
        for p in pixels.iter_mut() {
            *p = (3, 3, 8);
        }

        let t_scaled = t * self.rot_speed;
        let angle_y = t_scaled * 0.6;
        let angle_x = t_scaled * 0.4;

        let cos_y = angle_y.cos();
        let sin_y = angle_y.sin();
        let cos_x = angle_x.cos();
        let sin_x = angle_x.sin();

        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        let radius = cx.min(cy) * 0.7;

        for point in &self.points {
            // Rotate Y then X
            let x1 = point.x * cos_y + point.z * sin_y;
            let z1 = -point.x * sin_y + point.z * cos_y;
            let y1 = point.y;

            let y2 = y1 * cos_x - z1 * sin_x;
            let z2 = y1 * sin_x + z1 * cos_x;

            // Back-face cull
            if z2 < -0.1 {
                continue;
            }

            // Perspective projection
            let camera_z = 3.0;
            let persp = camera_z / (camera_z + z2);
            let sx = cx + x1 * radius * persp;
            let sy = cy + y2 * radius * persp;

            // Brightness by z (closer = brighter)
            let brightness = (0.3 + z2 * 0.7).clamp(0.2, 1.0);

            // Hue from original y position + time offset
            let hue = (point.y * 0.5 + 0.5 + t * 0.1) % 1.0;
            let (cr, cg, cb) = hsv_to_rgb(hue, 0.8, brightness);

            // Dot size 1–2 px based on depth
            let dot_size = if z2 > 0.5 { 2 } else { 1 };

            for dy in 0..dot_size {
                for dx in 0..dot_size {
                    let px = sx as i32 + dx;
                    let py = sy as i32 + dy;
                    if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                        let idx = (py as u32 * w + px as u32) as usize;
                        if idx < pixels.len() {
                            let p = &mut pixels[idx];
                            p.0 = p.0.max(cr);
                            p.1 = p.1.max(cg);
                            p.2 = p.2.max(cb);
                        }
                    }
                }
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "rot_speed".to_string(),
                min: 0.2,
                max: 4.0,
                value: self.rot_speed,
            },
            ParamDesc {
                name: "dot_count".to_string(),
                min: 100.0,
                max: 600.0,
                value: self.dot_count as f64,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "rot_speed" => self.rot_speed = value,
            "dot_count" => {
                let new_count = value as u32;
                if new_count != self.dot_count {
                    self.dot_count = new_count;
                    self.points = Self::generate_points(self.dot_count);
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
