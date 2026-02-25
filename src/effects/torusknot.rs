use crate::effect::{Effect, ParamDesc};
use std::f64::consts::TAU;

const NUM_SAMPLES: usize = 1500;

pub struct TorusKnot {
    width: u32,
    height: u32,
    rot_speed: f64,
    glow: f64,
}

impl TorusKnot {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            rot_speed: 1.0,
            glow: 1.0,
        }
    }
}

impl Effect for TorusKnot {
    fn name(&self) -> &str {
        "TorusKnot"
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
        let scale = cx.min(cy) * 0.5;
        let t = t * self.rot_speed;

        for p in pixels.iter_mut() {
            *p = (2, 1, 8);
        }

        // Trefoil torus knot (p=2, q=3)
        let p_k = 2.0_f64;
        let q_k = 3.0_f64;
        let big_r = 1.0;
        let small_r = 0.45;

        let rot_y = t * 0.35;
        let rot_x = t * 0.22;
        let rot_z = t * 0.13;
        let (cos_ry, sin_ry) = (rot_y.cos(), rot_y.sin());
        let (cos_rx, sin_rx) = (rot_x.cos(), rot_x.sin());
        let (cos_rz, sin_rz) = (rot_z.cos(), rot_z.sin());

        let camera_z = 4.5;

        // Pass 0: glow (larger, dimmer, additive)
        // Pass 1: core (smaller, brighter, max blend)
        for pass in 0..2u8 {
            for i in 0..NUM_SAMPLES {
                let u = i as f64 / NUM_SAMPLES as f64 * TAU;

                let cos_qu = (q_k * u).cos();
                let sin_qu = (q_k * u).sin();
                let cos_pu = (p_k * u).cos();
                let sin_pu = (p_k * u).sin();

                let x = (big_r + small_r * cos_qu) * cos_pu;
                let y = (big_r + small_r * cos_qu) * sin_pu;
                let z = small_r * sin_qu;

                // Rotate Z → Y → X
                let (x1, y1) = (x * cos_rz - y * sin_rz, x * sin_rz + y * cos_rz);
                let (x2, z1) = (x1 * cos_ry + z * sin_ry, -x1 * sin_ry + z * cos_ry);
                let (y2, z2) = (y1 * cos_rx - z1 * sin_rx, y1 * sin_rx + z1 * cos_rx);

                let persp = camera_z / (camera_z + z2);
                let sx = cx + x2 * scale * persp;
                let sy = cy + y2 * scale * persp;

                let depth = ((z2 + 2.0) / 4.0).clamp(0.15, 1.0);
                let hue = (i as f64 / NUM_SAMPLES as f64 + t * 0.04) % 1.0;

                if pass == 0 {
                    let glow_size = ((persp * 2.5 * self.glow) as i32).max(2).min(5);
                    let half = glow_size / 2;
                    let (cr, cg, cb) = hsv_to_rgb(hue, 0.7, depth * 0.3);

                    for dy in 0..glow_size {
                        for dx in 0..glow_size {
                            let px = sx as i32 + dx - half;
                            let py = sy as i32 + dy - half;
                            if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                                let idx = (py as u32 * w + px as u32) as usize;
                                if idx < pixels.len() {
                                    let p = &mut pixels[idx];
                                    p.0 = p.0.saturating_add(cr);
                                    p.1 = p.1.saturating_add(cg);
                                    p.2 = p.2.saturating_add(cb);
                                }
                            }
                        }
                    }
                } else {
                    let core_size = ((persp * 1.5 * self.glow) as i32).max(1).min(3);
                    let half = core_size / 2;
                    let (cr, cg, cb) = hsv_to_rgb(hue, 0.6, depth);

                    for dy in 0..core_size {
                        for dx in 0..core_size {
                            let px = sx as i32 + dx - half;
                            let py = sy as i32 + dy - half;
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
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "rot_speed".to_string(),
                min: 0.2,
                max: 3.0,
                value: self.rot_speed,
            },
            ParamDesc {
                name: "glow".to_string(),
                min: 0.5,
                max: 3.0,
                value: self.glow,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "rot_speed" => self.rot_speed = value,
            "glow" => self.glow = value,
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
