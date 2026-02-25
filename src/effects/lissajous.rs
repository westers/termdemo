use crate::effect::{Effect, ParamDesc};

const TRAIL_LENGTH: usize = 800;

pub struct Lissajous3D {
    width: u32,
    height: u32,
    speed: f64,
    complexity: f64,
    trail: Vec<(f64, f64, f64)>, // 3D positions in trail
    trail_head: usize,
    trail_filled: bool,
}

impl Lissajous3D {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            complexity: 1.0,
            trail: Vec::new(),
            trail_head: 0,
            trail_filled: false,
        }
    }
}

impl Effect for Lissajous3D {
    fn name(&self) -> &str {
        "Lissajous3D"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.trail = vec![(0.0, 0.0, 0.0); TRAIL_LENGTH];
        self.trail_head = 0;
        self.trail_filled = false;
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

        // Dark background
        for p in pixels.iter_mut() {
            *p = (2, 2, 6);
        }

        // Slowly morphing harmonic ratios for organic evolution
        let c = self.complexity;
        let a_x = 3.0 + (t * 0.07).sin() * c;
        let a_y = 2.0 + (t * 0.11).cos() * c;
        let a_z = 5.0 + (t * 0.05).sin() * c * 0.5;
        let phase_y = t * 0.13;
        let phase_z = t * 0.09;

        // Add several new points per frame for smooth trails
        let points_per_frame = 4;
        let step = 0.015;
        for i in 0..points_per_frame {
            let tt = t * 2.0 + i as f64 * step;

            let x = (a_x * tt).sin();
            let y = (a_y * tt + phase_y).sin();
            let z = (a_z * tt + phase_z).cos();

            self.trail[self.trail_head] = (x, y, z);
            self.trail_head = (self.trail_head + 1) % TRAIL_LENGTH;
            if self.trail_head == 0 {
                self.trail_filled = true;
            }
        }

        let total = if self.trail_filled {
            TRAIL_LENGTH
        } else {
            self.trail_head
        };

        if total == 0 {
            return;
        }

        // Slow rotation of the entire figure
        let rot_y = t * 0.3;
        let rot_x = t * 0.2;
        let cos_ry = rot_y.cos();
        let sin_ry = rot_y.sin();
        let cos_rx = rot_x.cos();
        let sin_rx = rot_x.sin();

        let scale = cx.min(cy) * 0.7;
        let camera_z = 4.0;

        // Draw trail from oldest to newest
        for i in 0..total {
            // Read from oldest first
            let idx = if self.trail_filled {
                (self.trail_head + i) % TRAIL_LENGTH
            } else {
                i
            };

            let (px, py, pz) = self.trail[idx];

            // Rotate Y then X
            let x1 = px * cos_ry + pz * sin_ry;
            let z1 = -px * sin_ry + pz * cos_ry;
            let y1 = py;

            let y2 = y1 * cos_rx - z1 * sin_rx;
            let z2 = y1 * sin_rx + z1 * cos_rx;

            // Perspective
            let persp = camera_z / (camera_z + z2);
            let sx = cx + x1 * scale * persp;
            let sy = cy + y2 * scale * persp;

            // Age: 0.0 = oldest, 1.0 = newest
            let age = i as f64 / total as f64;

            // Brightness and size fade with age
            let brightness = age * age; // quadratic: mostly dim, bright tip
            let dot_size = if age > 0.9 { 3 } else if age > 0.5 { 2 } else { 1 };

            // Hue shifts along the trail for a rainbow ribbon
            let hue = (age * 2.0 + t * 0.1) % 1.0;
            let (cr, cg, cb) = hsv_to_rgb(hue, 0.8, brightness);

            // Draw dot
            for dy in 0..dot_size {
                for dx in 0..dot_size {
                    let px = sx as i32 + dx - dot_size / 2;
                    let py = sy as i32 + dy - dot_size / 2;
                    if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                        let pidx = (py as u32 * w + px as u32) as usize;
                        if pidx < pixels.len() {
                            let p = &mut pixels[pidx];
                            p.0 = p.0.max(cr);
                            p.1 = p.1.max(cg);
                            p.2 = p.2.max(cb);
                        }
                    }
                }
            }
        }

        // Extra glow: draw the head point brighter and larger
        if total > 0 {
            let head_idx = if self.trail_head == 0 {
                TRAIL_LENGTH - 1
            } else {
                self.trail_head - 1
            };
            let (px, py, pz) = self.trail[head_idx];

            let x1 = px * cos_ry + pz * sin_ry;
            let z1 = -px * sin_ry + pz * cos_ry;
            let y1 = py;
            let y2 = y1 * cos_rx - z1 * sin_rx;
            let z2 = y1 * sin_rx + z1 * cos_rx;
            let persp = camera_z / (camera_z + z2);
            let sx = cx + x1 * scale * persp;
            let sy = cy + y2 * scale * persp;

            // Bright glow around head
            let glow_r = 5;
            for dy in -glow_r..=glow_r {
                for dx in -glow_r..=glow_r {
                    let dist_sq = dx * dx + dy * dy;
                    if dist_sq <= glow_r * glow_r {
                        let falloff =
                            1.0 - (dist_sq as f64 / (glow_r * glow_r) as f64);
                        let bright = (falloff * 255.0) as u8;
                        let ppx = sx as i32 + dx;
                        let ppy = sy as i32 + dy;
                        if ppx >= 0 && ppx < w as i32 && ppy >= 0 && ppy < h as i32 {
                            let pidx = (ppy as u32 * w + ppx as u32) as usize;
                            if pidx < pixels.len() {
                                let p = &mut pixels[pidx];
                                p.0 = p.0.saturating_add(bright);
                                p.1 = p.1.saturating_add(bright);
                                p.2 = p.2.saturating_add(bright);
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
                name: "speed".to_string(),
                min: 0.2,
                max: 3.0,
                value: self.speed,
            },
            ParamDesc {
                name: "complexity".to_string(),
                min: 0.1,
                max: 3.0,
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
