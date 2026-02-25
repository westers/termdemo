use crate::effect::{Effect, ParamDesc};
use rand::Rng;
use std::f64::consts::TAU;

const NUM_STARS: usize = 4000;
const NUM_ARMS: usize = 4;

struct Star {
    r: f64,
    arm_angle: f64,
    brightness: f64,
    twinkle_phase: f64,
    size: u8,
}

pub struct Galaxy {
    width: u32,
    height: u32,
    speed: f64,
    twist: f64,
    stars: Vec<Star>,
}

impl Galaxy {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            twist: 1.0,
            stars: Vec::new(),
        }
    }
}

impl Effect for Galaxy {
    fn name(&self) -> &str {
        "Galaxy"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.stars.clear();

        let mut rng = rand::thread_rng();

        for i in 0..NUM_STARS {
            let r = rng.gen_range(0.01f64..1.0).powf(0.7);

            let arm_angle = if i < NUM_STARS * 85 / 100 {
                let arm = rng.gen_range(0..NUM_ARMS);
                let base = arm as f64 * TAU / NUM_ARMS as f64;
                let spread = rng.gen_range(-1.0f64..1.0) * (0.08 + r * 0.25);
                base + spread
            } else {
                rng.gen_range(0.0..TAU)
            };

            self.stars.push(Star {
                r,
                arm_angle,
                brightness: rng.gen_range(0.4..1.0),
                twinkle_phase: rng.gen_range(0.0..TAU),
                size: if rng.gen_range(0.0f64..1.0) < 0.12 { 2 } else { 1 },
            });
        }
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
        let scale = cx.min(cy) * 0.85;
        let t = t * self.speed;

        for p in pixels.iter_mut() {
            *p = (1, 1, 5);
        }

        // Gentle tilt oscillation for 3D depth
        let tilt = 0.5 + 0.2 * (t * 0.08).sin();
        let cos_tilt = tilt.cos();
        let sin_tilt = tilt.sin();

        // Central glow
        let glow_r = (scale * 0.18) as i32;
        for dy in -glow_r..=glow_r {
            for dx in -glow_r..=glow_r {
                let dist_sq = dx * dx + dy * dy;
                let max_sq = glow_r * glow_r;
                if dist_sq <= max_sq {
                    let falloff = 1.0 - (dist_sq as f64 / max_sq as f64).sqrt();
                    let bright = (falloff * falloff * 140.0) as u8;
                    let px = cx as i32 + dx;
                    let py = cy as i32 + dy;
                    if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                        let idx = (py as u32 * w + px as u32) as usize;
                        if idx < pixels.len() {
                            let p = &mut pixels[idx];
                            p.0 = p.0.saturating_add(bright);
                            p.1 = p.1.saturating_add((bright as f64 * 0.9) as u8);
                            p.2 = p.2.saturating_add((bright as f64 * 0.7) as u8);
                        }
                    }
                }
            }
        }

        for star in &self.stars {
            let angular_vel = 1.0 / star.r.max(0.05).sqrt();
            let spiral = star.r * TAU * 0.75 * self.twist;
            let angle = star.arm_angle + spiral + t * 0.15 * angular_vel;

            let gx = star.r * angle.cos();
            let gy = star.r * angle.sin();

            // Apply tilt
            let proj_x = gx;
            let proj_y = gy * cos_tilt;
            let proj_z = gy * sin_tilt;

            let sx = cx + proj_x * scale;
            let sy = cy + proj_y * scale;

            let twinkle = 0.7 + 0.3 * (t * 3.0 + star.twinkle_phase).sin();
            let depth_mod = 0.8 + 0.2 * (proj_z + 1.0) / 2.0;
            let bright = star.brightness * twinkle * depth_mod;

            let (cr, cg, cb) = star_color(star.r, bright);

            let size = star.size as i32;
            for dy in 0..size {
                for dx in 0..size {
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
                name: "speed".to_string(),
                min: 0.2,
                max: 3.0,
                value: self.speed,
            },
            ParamDesc {
                name: "twist".to_string(),
                min: 0.3,
                max: 3.0,
                value: self.twist,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "twist" => self.twist = value,
            _ => {}
        }
    }
}

fn star_color(r: f64, brightness: f64) -> (u8, u8, u8) {
    let b = brightness.clamp(0.0, 1.0);
    if r < 0.15 {
        ((220.0 * b) as u8, (225.0 * b) as u8, (255.0 * b) as u8)
    } else if r < 0.4 {
        ((255.0 * b) as u8, (235.0 * b) as u8, (180.0 * b) as u8)
    } else if r < 0.7 {
        ((240.0 * b) as u8, (180.0 * b) as u8, (100.0 * b) as u8)
    } else {
        let fade = (1.0 - (r - 0.7) / 0.3).clamp(0.3, 1.0);
        (
            (200.0 * b * fade) as u8,
            (100.0 * b * fade) as u8,
            (50.0 * b * fade) as u8,
        )
    }
}
