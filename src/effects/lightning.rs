use crate::effect::{Effect, ParamDesc};
use rand::rngs::StdRng;
use rand::Rng;

pub struct Lightning {
    width: u32,
    height: u32,
    frequency: f64,
    branch_count: f64,
    seed_offset: u32,
}

/// A segment of a lightning bolt.
struct BoltSegment {
    x: f64,
    y: f64,
}

impl Lightning {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            frequency: 1.0,
            branch_count: 3.0,
            seed_offset: 0,
        }
    }

    /// Deterministic hash function.
    fn hash(mut seed: u32) -> u32 {
        seed = seed.wrapping_mul(747796405).wrapping_add(2891336453);
        seed = ((seed >> ((seed >> 28).wrapping_add(4))) ^ seed).wrapping_mul(277803737);
        seed ^ (seed >> 22)
    }

    /// Deterministic float from seed in -1.0..1.0.
    fn hash_f(seed: u32) -> f64 {
        let h = Self::hash(seed);
        (h & 0x00FFFFFF) as f64 / 0x00800000 as f64 - 1.0
    }

    /// Deterministic float from seed in 0.0..1.0.
    fn hash_u(seed: u32) -> f64 {
        let h = Self::hash(seed);
        (h & 0x00FFFFFF) as f64 / 0x01000000 as f64
    }

    /// Generate bolt path from (x0, y0) heading downward.
    fn generate_bolt(
        x0: f64,
        y0: f64,
        target_y: f64,
        width: f64,
        strike_seed: u32,
        sub_seed: u32,
    ) -> Vec<BoltSegment> {
        let mut segments = Vec::with_capacity(64);
        segments.push(BoltSegment { x: x0, y: y0 });

        let steps = ((target_y - y0).abs() / 3.0).max(5.0) as u32;
        let step_y = (target_y - y0) / steps as f64;

        let mut cx = x0;
        let mut cy = y0;
        for i in 1..=steps {
            let seed = strike_seed
                .wrapping_mul(1000)
                .wrapping_add(sub_seed.wrapping_mul(100))
                .wrapping_add(i);
            let jitter = Self::hash_f(seed) * width * 0.08;
            cx += jitter;
            cy += step_y;

            // Keep within bounds
            cx = cx.clamp(2.0, width - 2.0);

            segments.push(BoltSegment { x: cx, y: cy });
        }

        segments
    }

    /// Draw a bolt path onto the pixel buffer with given color and thickness.
    fn draw_bolt(
        segments: &[BoltSegment],
        pixels: &mut [(u8, u8, u8)],
        w: u32,
        h: u32,
        color: (f64, f64, f64),
        thickness: f64,
        alpha: f64,
    ) {
        if segments.len() < 2 {
            return;
        }
        for pair in segments.windows(2) {
            let (x0, y0) = (pair[0].x, pair[0].y);
            let (x1, y1) = (pair[1].x, pair[1].y);

            let dx = x1 - x0;
            let dy = y1 - y0;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            let steps = (len * 2.0) as u32 + 1;

            for s in 0..=steps {
                let t = s as f64 / steps as f64;
                let px = x0 + dx * t;
                let py = y0 + dy * t;

                let thick = (thickness + 0.5).ceil() as i32;
                for oy in -thick..=thick {
                    for ox in -thick..=thick {
                        let sx = px as i32 + ox;
                        let sy = py as i32 + oy;
                        if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                            let dist = ((ox * ox + oy * oy) as f64).sqrt();
                            if dist <= thickness {
                                let fade = (1.0 - dist / (thickness + 0.5)).clamp(0.0, 1.0);
                                let a = alpha * fade;
                                let idx = (sy as u32 * w + sx as u32) as usize;
                                let (pr, pg, pb) = pixels[idx];
                                // Additive blending
                                let r = (pr as f64 + color.0 * a * 255.0).min(255.0);
                                let g = (pg as f64 + color.1 * a * 255.0).min(255.0);
                                let b = (pb as f64 + color.2 * a * 255.0).min(255.0);
                                pixels[idx] = (r as u8, g as u8, b as u8);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Layered sine noise for cloud texture.
    fn cloud_noise(x: f64, y: f64, t: f64) -> f64 {
        let v1 = (x * 3.0 + t * 0.2).sin() * (y * 2.0 + t * 0.15).cos();
        let v2 = (x * 5.0 - t * 0.3).cos() * (y * 4.0 + t * 0.1).sin();
        let v3 = (x * 8.0 + y * 6.0 + t * 0.25).sin() * 0.5;
        (v1 + v2 + v3) / 3.0 * 0.5 + 0.5
    }
}

impl Effect for Lightning {
    fn name(&self) -> &str {
        "Lightning"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn randomize_init(&mut self, rng: &mut StdRng) {
        self.seed_offset = rng.gen();
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }
        let wf = w as f64;
        let hf = h as f64;

        // Determine lightning strike timing
        let strike_interval = 2.5 / self.frequency;
        let strike_index = (t / strike_interval).floor() as u32;
        let strike_phase = (t / strike_interval).fract();
        let time_since_strike = strike_phase * strike_interval;

        // Flash and afterglow
        let flash_duration = 0.15;
        let glow_duration = 0.5;
        let flash_alpha = if time_since_strike < flash_duration {
            1.0 - (time_since_strike / flash_duration) * 0.3
        } else if time_since_strike < flash_duration + glow_duration {
            let t_glow = (time_since_strike - flash_duration) / glow_duration;
            0.7 * (1.0 - t_glow).powi(2)
        } else {
            0.0
        };

        let bg_flash = if time_since_strike < flash_duration {
            0.4 * (1.0 - time_since_strike / flash_duration)
        } else {
            0.0
        };

        // Draw stormy background with clouds
        for y in 0..h {
            let yf = y as f64 / hf;
            for x in 0..w {
                let xf = x as f64 / wf;

                // Base dark sky
                let base_r = 10.0 + yf * 5.0;
                let base_g = 10.0 + yf * 5.0;
                let base_b = 18.0 + yf * 8.0;

                // Cloud texture (more prominent in upper half)
                let cloud = Self::cloud_noise(xf * 6.0, yf * 4.0, t);
                let cloud_factor = (1.0 - yf * 1.2).clamp(0.0, 1.0);
                let cloud_brightness = cloud * cloud_factor * 30.0;

                let r = base_r + cloud_brightness + bg_flash * 200.0;
                let g = base_g + cloud_brightness + bg_flash * 200.0;
                let b = base_b + cloud_brightness * 1.2 + bg_flash * 220.0;

                let idx = (y * w + x) as usize;
                pixels[idx] = (
                    r.clamp(0.0, 255.0) as u8,
                    g.clamp(0.0, 255.0) as u8,
                    b.clamp(0.0, 255.0) as u8,
                );
            }
        }

        // Generate and draw lightning bolt if visible
        if flash_alpha > 0.01 {
            let strike_seed = Self::hash(strike_index.wrapping_mul(7919).wrapping_add(self.seed_offset));

            // Main bolt: top-center to random bottom point
            let start_x = wf * 0.5 + Self::hash_f(strike_seed) * wf * 0.15;
            let end_x = wf * 0.2 + Self::hash_u(strike_seed.wrapping_add(1)) * wf * 0.6;
            let _ = end_x; // Target is implicit in the bolt generation

            let main_bolt = Self::generate_bolt(
                start_x,
                0.0,
                hf,
                wf,
                strike_seed,
                0,
            );

            // Afterglow: purple tint
            let is_afterglow = time_since_strike >= flash_duration;
            let bolt_color = if is_afterglow {
                (0.4, 0.2, 0.8) // purple afterglow
            } else {
                (0.9, 0.9, 1.0) // white-blue flash
            };

            // Draw main bolt with glow
            let glow_alpha = flash_alpha * 0.3;
            Self::draw_bolt(&main_bolt, pixels, w, h, bolt_color, 3.0, glow_alpha);
            Self::draw_bolt(&main_bolt, pixels, w, h, bolt_color, 1.5, flash_alpha * 0.7);
            Self::draw_bolt(
                &main_bolt,
                pixels,
                w,
                h,
                (1.0, 1.0, 1.0),
                0.5,
                flash_alpha,
            );

            // Branch bolts
            let num_branches = self.branch_count.round() as u32;
            for b in 0..num_branches {
                let branch_seed = strike_seed.wrapping_add(b + 100);
                // Pick a split point along the main bolt
                let split_idx_f = Self::hash_u(branch_seed) * 0.6 + 0.1;
                let split_idx = ((main_bolt.len() as f64 * split_idx_f) as usize)
                    .min(main_bolt.len().saturating_sub(1));

                let split_point = &main_bolt[split_idx];
                let branch_end_y =
                    split_point.y + (hf - split_point.y) * (Self::hash_u(branch_seed + 50) * 0.5 + 0.3);

                let branch = Self::generate_bolt(
                    split_point.x,
                    split_point.y,
                    branch_end_y.min(hf),
                    wf,
                    branch_seed,
                    b + 10,
                );

                let branch_alpha = flash_alpha * 0.5;
                Self::draw_bolt(&branch, pixels, w, h, bolt_color, 2.0, branch_alpha * 0.3);
                Self::draw_bolt(&branch, pixels, w, h, bolt_color, 0.8, branch_alpha * 0.6);
                Self::draw_bolt(
                    &branch,
                    pixels,
                    w,
                    h,
                    (1.0, 1.0, 1.0),
                    0.3,
                    branch_alpha,
                );
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "frequency".to_string(),
                min: 0.5,
                max: 3.0,
                value: self.frequency,
            },
            ParamDesc {
                name: "branch_count".to_string(),
                min: 1.0,
                max: 5.0,
                value: self.branch_count,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "frequency" => self.frequency = value,
            "branch_count" => self.branch_count = value,
            _ => {}
        }
    }
}
