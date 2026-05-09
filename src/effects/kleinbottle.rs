use crate::effect::{Effect, ParamDesc};
use std::f64::consts::TAU;

const U_STEPS: usize = 90;
const V_STEPS: usize = 30;

pub struct KleinBottle {
    width: u32,
    height: u32,
    rot_speed: f64,
    glow: f64,
    points: Vec<[f64; 3]>, // pre-sampled surface
}

impl KleinBottle {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            rot_speed: 1.0,
            glow: 1.0,
            points: Vec::new(),
        }
    }

    fn build(&mut self) {
        // Figure-8 immersion of the Klein bottle
        let big_r = 1.4;
        let small_r = 0.55;
        self.points.clear();
        self.points.reserve(U_STEPS * V_STEPS);
        for ui in 0..U_STEPS {
            let u = ui as f64 / U_STEPS as f64 * TAU;
            for vi in 0..V_STEPS {
                let v = vi as f64 / V_STEPS as f64 * TAU;
                let cu2 = (u * 0.5).cos();
                let su2 = (u * 0.5).sin();
                let sv = v.sin();
                let s2v = (2.0 * v).sin();
                let radial = big_r + small_r * cu2 * sv - small_r * su2 * s2v;
                let x = radial * u.cos();
                let y = radial * u.sin();
                let z = small_r * su2 * sv + small_r * cu2 * s2v;
                self.points.push([x, y, z]);
            }
        }
    }
}

impl Effect for KleinBottle {
    fn name(&self) -> &str {
        "KleinBottle"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.build();
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        // Dark teal background
        for p in pixels.iter_mut() {
            *p = (2, 6, 10);
        }

        let wf = w as f64;
        let hf = h as f64;
        let cx = wf / 2.0;
        let cy = hf / 2.0;
        let scale = cx.min(cy) * 0.42;
        let camera_z = 5.0;

        let t = t * self.rot_speed;
        let rx = t * 0.30;
        let ry = t * 0.45;
        let rz = t * 0.18;
        let (cx_r, sx_r) = (rx.cos(), rx.sin());
        let (cy_r, sy_r) = (ry.cos(), ry.sin());
        let (cz_r, sz_r) = (rz.cos(), rz.sin());

        // Two-pass: outer glow, then bright dots
        for pass in 0..2u8 {
            for (i, &p) in self.points.iter().enumerate() {
                // Rotate Z, Y, X
                let (x1, y1) = (p[0] * cz_r - p[1] * sz_r, p[0] * sz_r + p[1] * cz_r);
                let (x2, z1) = (x1 * cy_r + p[2] * sy_r, -x1 * sy_r + p[2] * cy_r);
                let (y2, z2) = (y1 * cx_r - z1 * sx_r, y1 * sx_r + z1 * cx_r);

                let persp = camera_z / (camera_z + z2);
                let sx = cx + x2 * scale * persp;
                let sy = cy + y2 * scale * persp;

                if sx < 0.0 || sy < 0.0 || sx >= wf || sy >= hf {
                    continue;
                }

                let depth = ((z2 + 2.0) / 4.0).clamp(0.15, 1.0);
                let ui = i / V_STEPS;
                let vi = i % V_STEPS;
                let u_norm = ui as f64 / U_STEPS as f64;
                let v_norm = vi as f64 / V_STEPS as f64;

                // Hue runs around u (longitudinal); brightness modulates with v
                let hue = (u_norm + t * 0.05) % 1.0;
                let val = 0.5 + 0.5 * (v_norm * TAU).cos().abs();
                let brightness = depth * val;

                if pass == 0 {
                    let g = self.glow;
                    let size = ((persp * 2.6 * g) as i32).clamp(2, 5);
                    let half = size / 2;
                    let (cr, cg, cb) = hsv_to_rgb(hue, 0.55, brightness * 0.35);
                    for dy in 0..size {
                        for dx in 0..size {
                            let px = sx as i32 + dx - half;
                            let py = sy as i32 + dy - half;
                            if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                                let idx = (py as u32 * w + px as u32) as usize;
                                if idx < pixels.len() {
                                    let pp = &mut pixels[idx];
                                    pp.0 = pp.0.saturating_add(cr);
                                    pp.1 = pp.1.saturating_add(cg);
                                    pp.2 = pp.2.saturating_add(cb);
                                }
                            }
                        }
                    }
                } else {
                    let size = ((persp * 1.4) as i32).clamp(1, 2);
                    let half = size / 2;
                    let (cr, cg, cb) = hsv_to_rgb(hue, 0.65, brightness);
                    for dy in 0..size {
                        for dx in 0..size {
                            let px = sx as i32 + dx - half;
                            let py = sy as i32 + dy - half;
                            if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                                let idx = (py as u32 * w + px as u32) as usize;
                                if idx < pixels.len() {
                                    let pp = &mut pixels[idx];
                                    pp.0 = pp.0.max(cr);
                                    pp.1 = pp.1.max(cg);
                                    pp.2 = pp.2.max(cb);
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
    let (r, g, b) = match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}
