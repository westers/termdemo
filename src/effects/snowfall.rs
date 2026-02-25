use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct Snowfall {
    width: u32,
    height: u32,
    wind: f64,
    density: f64,
}

impl Snowfall {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            wind: 0.3,
            density: 1.0,
        }
    }

    /// Deterministic pseudo-random from seed, returns 0.0..1.0.
    fn rng(seed: u32) -> f64 {
        let mut h = seed;
        h = h.wrapping_mul(747796405).wrapping_add(2891336453);
        h = ((h >> ((h >> 28).wrapping_add(4))) ^ h).wrapping_mul(277803737);
        h = h ^ (h >> 22);
        (h & 0x00FFFFFF) as f64 / 0x01000000 as f64
    }
}

impl Effect for Snowfall {
    fn name(&self) -> &str {
        "Snowfall"
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

        // Background: dark blue gradient (lighter at bottom)
        for y in 0..h {
            let yf = y as f64 / hf;
            let r = (8.0 + yf * 12.0) as u8;
            let g = (12.0 + yf * 18.0) as u8;
            let b = (30.0 + yf * 35.0) as u8;
            for x in 0..w {
                pixels[(y * w + x) as usize] = (r, g, b);
            }
        }

        // Ground: white strip at bottom ~10% with undulation
        let ground_base = (hf * 0.90) as u32;
        for y in ground_base..h {
            for x in 0..w {
                let xf = x as f64 / wf;
                // Gentle undulation
                let undulation = (xf * PI * 4.0).sin() * 2.0 + (xf * PI * 7.0).sin() * 1.0;
                let ground_line = ground_base as f64 + undulation;
                if y as f64 >= ground_line {
                    let depth = (y as f64 - ground_line) / (hf - ground_line);
                    let brightness = (200.0 + depth * 40.0).min(240.0);
                    let r = brightness as u8;
                    let g = brightness as u8;
                    let b = (brightness + 10.0).min(255.0) as u8;
                    pixels[(y * w + x) as usize] = (r, g, b);
                }
            }
        }

        // Snowflake layers: (count, speed, drift_amount, size, brightness, drift_freq)
        let layers: [(u32, f64, f64, f64, f64, f64); 3] = [
            (200, 25.0, 4.0, 2.0, 255.0, 1.5), // front: large, fast, bright
            (150, 15.0, 3.0, 1.5, 180.0, 1.0),  // middle
            (100, 8.0, 2.0, 1.0, 120.0, 0.7),   // back: small, slow, dim
        ];

        for (layer_idx, &(base_count, speed, drift_amount, size, brightness, drift_freq)) in
            layers.iter().enumerate()
        {
            let count = ((base_count as f64) * self.density) as u32;
            for i in 0..count {
                let seed_base = (layer_idx as u32) * 10000 + i;

                let start_x = Self::rng(seed_base * 3 + 1) * wf;
                let start_y = Self::rng(seed_base * 3 + 2) * hf;
                let offset = Self::rng(seed_base * 3 + 3) * PI * 2.0;

                // Y position wraps around screen
                let fall_y = (start_y + t * speed) % hf;

                // X position drifts with wind and sine
                let drift = (t * drift_freq + offset).sin() * drift_amount
                    + self.wind * t * speed * 0.15;
                let fall_x = ((start_x + drift) % wf + wf) % wf;

                // Don't draw below ground
                if fall_y >= ground_base as f64 - 1.0 {
                    continue;
                }

                // Draw snowflake as small bright dot(s)
                let half = (size * 0.5).ceil() as i32;
                let br = brightness as u8;
                for dy in -half..=half {
                    for dx in -half..=half {
                        let px = fall_x as i32 + dx;
                        let py = fall_y as i32 + dy;
                        if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                            let dist_sq = (dx * dx + dy * dy) as f64;
                            let max_dist = size * size * 0.3;
                            if dist_sq <= max_dist {
                                let fade =
                                    (1.0 - dist_sq / (max_dist + 0.01)).clamp(0.0, 1.0);
                                let idx = (py as u32 * w + px as u32) as usize;
                                let (pr, pg, pb) = pixels[idx];
                                let alpha = fade * 0.9;
                                let r =
                                    (pr as f64 * (1.0 - alpha) + br as f64 * alpha) as u8;
                                let g =
                                    (pg as f64 * (1.0 - alpha) + br as f64 * alpha) as u8;
                                let b = (pb as f64 * (1.0 - alpha)
                                    + (br as f64 + 10.0).min(255.0) * alpha)
                                    as u8;
                                pixels[idx] = (r, g, b);
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
                name: "wind".to_string(),
                min: -2.0,
                max: 2.0,
                value: self.wind,
            },
            ParamDesc {
                name: "density".to_string(),
                min: 0.5,
                max: 3.0,
                value: self.density,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "wind" => self.wind = value,
            "density" => self.density = value,
            _ => {}
        }
    }
}
