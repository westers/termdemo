use crate::effect::{Effect, ParamDesc};
use std::f64::consts::TAU;

pub struct Spirograph {
    width: u32,
    height: u32,
    speed: f64,
    complexity: f64,
    canvas: Vec<(f64, f64, f64)>,
    angle: f64,
}

impl Spirograph {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            complexity: 4.0,
            canvas: Vec::new(),
            angle: 0.0,
        }
    }
}

struct CurveParams {
    big_r: f64,
    small_r: f64,
    d: f64,
    hue: f64,
}

impl Effect for Spirograph {
    fn name(&self) -> &str {
        "Spirograph"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.canvas = vec![(0.0, 0.0, 0.0); (width * height) as usize];
        self.angle = 0.0;
    }

    fn update(&mut self, t: f64, dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let wf = w as f64;
        let hf = h as f64;
        let cx = wf / 2.0;
        let cy = hf / 2.0;
        let scale = cx.min(cy) * 0.85;

        // Fade existing canvas (darken by ~1% per frame)
        for c in self.canvas.iter_mut() {
            c.0 *= 0.965;
            c.1 *= 0.965;
            c.2 *= 0.965;
        }

        // Number of curves depends on complexity
        let num_curves = self.complexity as usize;

        // Define curves with slowly evolving parameters
        let curves: Vec<CurveParams> = (0..num_curves)
            .map(|i| {
                let fi = i as f64;
                let base_phase = fi * TAU / num_curves as f64;
                // Slowly evolve the ratios over time for variety
                let evolve = (t * 0.03 + base_phase).sin() * 0.3;
                let big_r = 1.0;
                let small_r = (0.2 + fi * 0.15 + evolve * 0.1).fract() * 0.6 + 0.15;
                let d = small_r * (0.7 + evolve * 0.3);
                let hue = (fi / num_curves as f64 + t * 0.02) % 1.0;
                CurveParams { big_r, small_r, d, hue }
            })
            .collect();

        // Advance angle and plot new points
        let angle_step = 0.005;
        let points_per_frame = (200.0 * self.speed) as usize;

        for _ in 0..points_per_frame {
            self.angle += angle_step;

            for curve in &curves {
                let r_diff = curve.big_r - curve.small_r;
                let ratio = r_diff / curve.small_r;

                // Hypotrochoid formula
                let x = r_diff * self.angle.cos()
                    + curve.d * (ratio * self.angle).cos();
                let y = r_diff * self.angle.sin()
                    - curve.d * (ratio * self.angle).sin();

                // Normalize to [-1, 1] range (max extent is big_r + d)
                let max_extent = curve.big_r + curve.d;
                let nx = x / max_extent;
                let ny = y / max_extent;

                let px = cx + nx * scale;
                let py = cy + ny * scale;

                let ix = px as i32;
                let iy = py as i32;

                let (cr, cg, cb) = hsv_to_rgb_f64(curve.hue, 0.85, 1.0);

                // Plot with a small soft dot (2px radius)
                for dy in -1..=1_i32 {
                    for dx in -1..=1_i32 {
                        let sx = ix + dx;
                        let sy = iy + dy;
                        if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                            let dist = ((dx * dx + dy * dy) as f64).sqrt();
                            let intensity = (1.0 - dist / 2.0).max(0.0) * 0.05;
                            let idx = (sy as u32 * w + sx as u32) as usize;
                            self.canvas[idx].0 = (self.canvas[idx].0 + cr * intensity).min(1.0);
                            self.canvas[idx].1 = (self.canvas[idx].1 + cg * intensity).min(1.0);
                            self.canvas[idx].2 = (self.canvas[idx].2 + cb * intensity).min(1.0);
                        }
                    }
                }
            }
        }

        // Render canvas to pixels
        for i in 0..pixels.len().min(self.canvas.len()) {
            let c = &self.canvas[i];
            pixels[i] = (
                (c.0.min(1.0) * 255.0) as u8,
                (c.1.min(1.0) * 255.0) as u8,
                (c.2.min(1.0) * 255.0) as u8,
            );
        }

        // Suppress unused warning for dt
        let _ = dt;
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "speed".to_string(),
                min: 0.3,
                max: 3.0,
                value: self.speed,
            },
            ParamDesc {
                name: "complexity".to_string(),
                min: 2.0,
                max: 8.0,
                value: self.complexity,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "complexity" => self.complexity = value,
            _ => {}
        }
    }
}

fn hsv_to_rgb_f64(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
