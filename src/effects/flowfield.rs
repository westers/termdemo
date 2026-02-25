use crate::effect::{Effect, ParamDesc};

const NUM_PARTICLES: usize = 3000;

pub struct FlowField {
    width: u32,
    height: u32,
    speed: f64,
    trail_fade: f64,
    particles: Vec<(f64, f64)>,
    trail: Vec<(f64, f64, f64)>,
}

impl FlowField {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            trail_fade: 0.03,
            particles: Vec::new(),
            trail: Vec::new(),
        }
    }

    fn noise(x: f64, y: f64, t: f64) -> f64 {
        let v1 = (x * 0.03 + t * 0.2).sin() * (y * 0.04 - t * 0.15).cos();
        let v2 = (x * 0.02 - y * 0.03 + t * 0.1).sin();
        let v3 = ((x * 0.05 + y * 0.05) * 0.5 + t * 0.25).cos() * 0.5;
        let v4 = (x * 0.01 + t * 0.3).cos() * (y * 0.06 + t * 0.05).sin();
        v1 + v2 + v3 + v4
    }

    fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
        let h = ((h % 360.0) + 360.0) % 360.0;
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;
        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        (
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }
}

impl Effect for FlowField {
    fn name(&self) -> &str {
        "Flow Field"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let size = (width * height) as usize;
        self.trail = vec![(0.0, 0.0, 0.0); size];

        // Deterministic seed from dimensions
        let mut seed: u64 = (width as u64) * 7919 + (height as u64) * 6271;
        self.particles.clear();
        for _ in 0..NUM_PARTICLES {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let px = (seed >> 33) as f64 / (1u64 << 31) as f64 * width as f64;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let py = (seed >> 33) as f64 / (1u64 << 31) as f64 * height as f64;
            self.particles.push((px, py));
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

        // Fade the trail buffer
        let fade = 1.0 - self.trail_fade;
        for pixel in self.trail.iter_mut() {
            pixel.0 *= fade;
            pixel.1 *= fade;
            pixel.2 *= fade;
        }

        // Update particles
        let step = 1.5 * self.speed;
        for particle in self.particles.iter_mut() {
            let angle = Self::noise(particle.0, particle.1, t) * std::f64::consts::TAU;
            particle.0 += angle.cos() * step;
            particle.1 += angle.sin() * step;

            // Wrap around
            if particle.0 < 0.0 {
                particle.0 += wf;
            } else if particle.0 >= wf {
                particle.0 -= wf;
            }
            if particle.1 < 0.0 {
                particle.1 += hf;
            } else if particle.1 >= hf {
                particle.1 -= hf;
            }

            let ix = particle.0 as u32;
            let iy = particle.1 as u32;
            if ix < w && iy < h {
                let idx = (iy * w + ix) as usize;
                // Color based on angle and position
                let hue = (angle / std::f64::consts::TAU * 360.0
                    + particle.0 / wf * 60.0
                    + particle.1 / hf * 60.0)
                    % 360.0;
                let (r, g, b) = Self::hsv_to_rgb(hue, 0.9, 1.0);
                let trail = &mut self.trail[idx];
                // Additive blending, capped
                trail.0 = (trail.0 + r as f64 * 0.4).min(255.0);
                trail.1 = (trail.1 + g as f64 * 0.4).min(255.0);
                trail.2 = (trail.2 + b as f64 * 0.4).min(255.0);
            }
        }

        // Render trail to pixels
        for (i, pixel) in pixels.iter_mut().enumerate() {
            if i < self.trail.len() {
                let t = &self.trail[i];
                *pixel = (t.0 as u8, t.1 as u8, t.2 as u8);
            }
        }
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
                name: "trail_fade".to_string(),
                min: 0.01,
                max: 0.1,
                value: self.trail_fade,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "trail_fade" => self.trail_fade = value,
            _ => {}
        }
    }
}
