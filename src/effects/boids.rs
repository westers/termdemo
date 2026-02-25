use crate::effect::{Effect, ParamDesc};
use rand::Rng;

const MAX_BOIDS: usize = 300;

struct Boid {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    hue: f64,
}

pub struct Boids {
    width: u32,
    height: u32,
    speed: f64,
    cohesion: f64,
    boids: Vec<Boid>,
}

impl Boids {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            cohesion: 1.0,
            boids: Vec::new(),
        }
    }
}

impl Effect for Boids {
    fn name(&self) -> &str {
        "Boids"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        let mut rng = rand::thread_rng();
        let wf = width as f64;
        let hf = height as f64;

        self.boids.clear();
        for i in 0..MAX_BOIDS {
            self.boids.push(Boid {
                x: rng.gen_range(0.0..wf),
                y: rng.gen_range(0.0..hf),
                vx: rng.gen_range(-50.0..50.0),
                vy: rng.gen_range(-50.0..50.0),
                hue: i as f64 / MAX_BOIDS as f64,
            });
        }
    }

    fn update(&mut self, t: f64, dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let wf = w as f64;
        let hf = h as f64;
        let dt = dt * self.speed;
        let n = self.boids.len();

        // Fade existing pixels for trails
        for p in pixels.iter_mut() {
            p.0 = p.0.saturating_sub(8);
            p.1 = p.1.saturating_sub(8);
            p.2 = p.2.saturating_sub(10);
        }

        // Compute flocking forces
        // For performance, use a simple O(n²) with early distance rejection
        let visual_range = 40.0;
        let protected_range = 12.0;
        let max_speed = 120.0;
        let visual_range_sq = visual_range * visual_range;
        let protected_range_sq = protected_range * protected_range;

        // Moving attractor that the flock loosely follows
        let attract_x = wf * 0.5 + wf * 0.35 * (t * 0.3).sin();
        let attract_y = hf * 0.5 + hf * 0.35 * (t * 0.4).cos();

        // Collect current positions (avoid borrow issues)
        let positions: Vec<(f64, f64, f64, f64)> = self
            .boids
            .iter()
            .map(|b| (b.x, b.y, b.vx, b.vy))
            .collect();

        for i in 0..n {
            let (bx, by, _, _) = positions[i];

            let mut sep_x = 0.0;
            let mut sep_y = 0.0;
            let mut align_vx = 0.0;
            let mut align_vy = 0.0;
            let mut coh_x = 0.0;
            let mut coh_y = 0.0;
            let mut neighbors = 0u32;

            for j in 0..n {
                if i == j {
                    continue;
                }
                let (ox, oy, ovx, ovy) = positions[j];
                let dx = ox - bx;
                let dy = oy - by;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < visual_range_sq {
                    // Alignment: match velocity of neighbors
                    align_vx += ovx;
                    align_vy += ovy;
                    // Cohesion: steer toward center of neighbors
                    coh_x += ox;
                    coh_y += oy;
                    neighbors += 1;

                    if dist_sq < protected_range_sq && dist_sq > 0.01 {
                        // Separation: steer away from very close neighbors
                        let inv_dist = 1.0 / dist_sq.sqrt();
                        sep_x -= dx * inv_dist;
                        sep_y -= dy * inv_dist;
                    }
                }
            }

            let boid = &mut self.boids[i];

            if neighbors > 0 {
                let nf = neighbors as f64;

                // Alignment
                align_vx /= nf;
                align_vy /= nf;
                boid.vx += (align_vx - boid.vx) * 0.05;
                boid.vy += (align_vy - boid.vy) * 0.05;

                // Cohesion
                coh_x = coh_x / nf - boid.x;
                coh_y = coh_y / nf - boid.y;
                boid.vx += coh_x * 0.01 * self.cohesion;
                boid.vy += coh_y * 0.01 * self.cohesion;
            }

            // Separation
            boid.vx += sep_x * 4.0;
            boid.vy += sep_y * 4.0;

            // Gentle attraction to moving point
            let dx = attract_x - boid.x;
            let dy = attract_y - boid.y;
            boid.vx += dx * 0.003;
            boid.vy += dy * 0.003;

            // Soft boundary steering (push away from edges)
            let margin = 30.0;
            if boid.x < margin {
                boid.vx += (margin - boid.x) * 0.3;
            }
            if boid.x > wf - margin {
                boid.vx -= (boid.x - (wf - margin)) * 0.3;
            }
            if boid.y < margin {
                boid.vy += (margin - boid.y) * 0.3;
            }
            if boid.y > hf - margin {
                boid.vy -= (boid.y - (hf - margin)) * 0.3;
            }

            // Clamp speed
            let speed = (boid.vx * boid.vx + boid.vy * boid.vy).sqrt();
            if speed > max_speed {
                boid.vx = boid.vx / speed * max_speed;
                boid.vy = boid.vy / speed * max_speed;
            }

            // Integrate position
            boid.x += boid.vx * dt;
            boid.y += boid.vy * dt;

            // Hard clamp to screen (safety)
            boid.x = boid.x.clamp(0.0, wf - 1.0);
            boid.y = boid.y.clamp(0.0, hf - 1.0);
        }

        // Draw boids
        for boid in &self.boids {
            let ix = boid.x as i32;
            let iy = boid.y as i32;

            // Color based on velocity direction + base hue
            let angle = boid.vy.atan2(boid.vx);
            let hue = (boid.hue + angle / std::f64::consts::TAU + t * 0.05) % 1.0;
            let speed = (boid.vx * boid.vx + boid.vy * boid.vy).sqrt();
            let brightness = (0.5 + speed / max_speed * 0.5).clamp(0.5, 1.0);
            let (cr, cg, cb) = hsv_to_rgb(hue.abs(), 0.85, brightness);

            // Draw 2x2 pixel dot
            for dy in 0..2 {
                for dx in 0..2 {
                    let px = ix + dx;
                    let py = iy + dy;
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
                name: "cohesion".to_string(),
                min: 0.1,
                max: 3.0,
                value: self.cohesion,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "cohesion" => self.cohesion = value,
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
