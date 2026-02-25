use crate::effect::{Effect, ParamDesc};
use rand::Rng;
use std::f64::consts::TAU;

struct Spark {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64,
    hue: f64,
}

struct Rocket {
    x: f64,
    y: f64,
    vy: f64,
    target_y: f64,
    hue: f64,
}

pub struct Fireworks {
    width: u32,
    height: u32,
    intensity: f64,
    gravity: f64,
    sparks: Vec<Spark>,
    rockets: Vec<Rocket>,
    launch_accum: f64,
}

impl Fireworks {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            intensity: 1.0,
            gravity: 1.0,
            sparks: Vec::new(),
            rockets: Vec::new(),
            launch_accum: 0.0,
        }
    }
}

const MAX_SPARKS: usize = 2000;

impl Effect for Fireworks {
    fn name(&self) -> &str {
        "Fireworks"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.sparks.clear();
        self.rockets.clear();
        self.launch_accum = 0.0;
    }

    fn update(&mut self, _t: f64, dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let wf = w as f64;
        let hf = h as f64;
        let mut rng = rand::thread_rng();
        let grav = self.gravity * 120.0;

        // Fade existing pixels (night sky with trails)
        for p in pixels.iter_mut() {
            p.0 = p.0.saturating_sub(10);
            p.1 = p.1.saturating_sub(10);
            p.2 = p.2.saturating_sub(12);
        }

        // Launch rockets
        self.launch_accum += dt * self.intensity * 2.5;
        while self.launch_accum >= 1.0 && self.rockets.len() < 8 {
            self.launch_accum -= 1.0;
            self.rockets.push(Rocket {
                x: rng.gen_range(wf * 0.15..wf * 0.85),
                y: hf - 1.0,
                vy: rng.gen_range(-280.0..-180.0),
                target_y: rng.gen_range(hf * 0.15..hf * 0.45),
                hue: rng.gen_range(0.0..1.0),
            });
        }

        // Update rockets
        let mut new_explosions: Vec<(f64, f64, f64)> = Vec::new();
        self.rockets.retain_mut(|r| {
            r.y += r.vy * dt;
            r.vy += grav * 0.3 * dt; // slight drag

            // Draw rocket trail (bright white pixel)
            let ix = r.x as i32;
            let iy = r.y as i32;
            if ix >= 0 && ix < w as i32 && iy >= 0 && iy < h as i32 {
                let idx = (iy as u32 * w + ix as u32) as usize;
                if idx < pixels.len() {
                    pixels[idx] = (255, 240, 200);
                }
                // Small tail
                for dy in 1..4 {
                    let ty = iy + dy;
                    if ty >= 0 && ty < h as i32 {
                        let tidx = (ty as u32 * w + ix as u32) as usize;
                        if tidx < pixels.len() {
                            let b = (200 - dy * 50).max(0) as u8;
                            let p = &mut pixels[tidx];
                            p.0 = p.0.max(b);
                            p.1 = p.1.max(b / 2);
                            p.2 = p.2.max(b / 4);
                        }
                    }
                }
            }

            // Explode when reaching target height or slowing down
            if r.y <= r.target_y || r.vy >= 0.0 {
                new_explosions.push((r.x, r.y, r.hue));
                return false;
            }
            true
        });

        // Create explosion sparks
        for (ex, ey, hue) in new_explosions {
            let num_sparks = rng.gen_range(60..120);
            let remaining = MAX_SPARKS.saturating_sub(self.sparks.len());
            let to_create = num_sparks.min(remaining);

            for _ in 0..to_create {
                let angle = rng.gen_range(0.0..TAU);
                let speed = rng.gen_range(30.0..180.0);
                // Slight hue variation per spark
                let spark_hue = (hue + rng.gen_range(-0.08..0.08) + 1.0) % 1.0;

                self.sparks.push(Spark {
                    x: ex,
                    y: ey,
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                    life: rng.gen_range(0.6..1.0),
                    hue: spark_hue,
                });
            }
        }

        // Update and draw sparks
        self.sparks.retain_mut(|s| {
            s.vy += grav * dt;
            s.vx *= 0.99; // air drag
            s.vy *= 0.99;
            s.x += s.vx * dt;
            s.y += s.vy * dt;
            s.life -= dt * 1.2;

            if s.life <= 0.0 {
                return false;
            }

            let ix = s.x as i32;
            let iy = s.y as i32;
            if ix >= 0 && ix < w as i32 && iy >= 0 && iy < h as i32 {
                // Color: saturated at birth, fades to orange/red then dark
                let brightness = s.life.clamp(0.0, 1.0);
                let sat = (0.5 + s.life * 0.5).clamp(0.0, 1.0);
                let (cr, cg, cb) = hsv_to_rgb(s.hue, sat, brightness);

                let idx = (iy as u32 * w + ix as u32) as usize;
                if idx < pixels.len() {
                    let p = &mut pixels[idx];
                    p.0 = p.0.max(cr);
                    p.1 = p.1.max(cg);
                    p.2 = p.2.max(cb);
                }
            }

            true
        });
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "intensity".to_string(),
                min: 0.3,
                max: 4.0,
                value: self.intensity,
            },
            ParamDesc {
                name: "gravity".to_string(),
                min: 0.3,
                max: 3.0,
                value: self.gravity,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "intensity" => self.intensity = value,
            "gravity" => self.gravity = value,
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
