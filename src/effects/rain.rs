use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct Rain {
    width: u32,
    height: u32,
    intensity: f64,
    wind: f64,
}

impl Rain {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            intensity: 1.0,
            wind: 0.2,
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

impl Effect for Rain {
    fn name(&self) -> &str {
        "Rain"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width as usize;
        let h = self.height as usize;
        if w == 0 || h == 0 {
            return;
        }
        let wf = w as f64;
        let hf = h as f64;

        // Lightning flash: every ~4 seconds, a brief bright flash
        let lightning_cycle = t % 4.2;
        let lightning_brightness = if lightning_cycle > 3.9 && lightning_cycle < 4.05 {
            let phase = (lightning_cycle - 3.9) / 0.15;
            let flash = (phase * PI).sin();
            flash * 0.6
        } else if lightning_cycle > 4.05 && lightning_cycle < 4.12 {
            // Secondary smaller flash
            let phase = (lightning_cycle - 4.05) / 0.07;
            let flash = (phase * PI).sin();
            flash * 0.25
        } else {
            0.0
        };

        // Background: stormy sky gradient
        let sky_line = (hf * 0.7) as usize;
        for y in 0..h {
            let yf = y as f64 / hf;
            let (r, g, b) = if y < sky_line {
                // Sky: dark blue-grey gradient
                let grad = yf / 0.7;
                (
                    18.0 + grad * 15.0 + lightning_brightness * 120.0,
                    22.0 + grad * 18.0 + lightning_brightness * 130.0,
                    40.0 + grad * 20.0 + lightning_brightness * 140.0,
                )
            } else {
                // City silhouette region: very dark
                (
                    8.0 + lightning_brightness * 30.0,
                    10.0 + lightning_brightness * 30.0,
                    15.0 + lightning_brightness * 35.0,
                )
            };
            for x in 0..w {
                pixels[y * w + x] = (
                    r.clamp(0.0, 255.0) as u8,
                    g.clamp(0.0, 255.0) as u8,
                    b.clamp(0.0, 255.0) as u8,
                );
            }
        }

        // City silhouette: simple rectangular buildings
        let ground_y = (hf * 0.92) as usize;
        let num_buildings = (w / 6).max(5);
        for i in 0..num_buildings {
            let seed = i as u32 * 7 + 100;
            let bx = (Self::rng(seed) * wf) as usize;
            let bw = (Self::rng(seed + 1) * 8.0 + 3.0) as usize;
            let bh = (Self::rng(seed + 2) * hf * 0.25 + hf * 0.05) as usize;
            let building_top = ground_y.saturating_sub(bh);

            let shade = 12.0 + Self::rng(seed + 3) * 10.0 + lightning_brightness * 25.0;
            let br = shade.clamp(0.0, 255.0) as u8;
            let bg = (shade * 1.05).clamp(0.0, 255.0) as u8;
            let bb = (shade * 1.2).clamp(0.0, 255.0) as u8;

            for y in building_top..ground_y {
                for dx in 0..bw {
                    let x = bx + dx;
                    if x < w {
                        pixels[y * w + x] = (br, bg, bb);
                    }
                }
            }

            // Occasional lit windows
            let win_spacing_x = 3;
            let win_spacing_y = 4;
            for wy in (building_top + 2..ground_y).step_by(win_spacing_y) {
                for wx_offset in (1..bw.saturating_sub(1)).step_by(win_spacing_x) {
                    let wx = bx + wx_offset;
                    if wx < w {
                        let win_seed = (i as u32) * 1000 + (wy as u32) * 100 + wx_offset as u32;
                        if Self::rng(win_seed) > 0.5 {
                            let warm = 140.0 + Self::rng(win_seed + 1) * 60.0;
                            pixels[wy * w + wx] = (
                                warm.clamp(0.0, 255.0) as u8,
                                (warm * 0.85).clamp(0.0, 255.0) as u8,
                                (warm * 0.4).clamp(0.0, 255.0) as u8,
                            );
                        }
                    }
                }
            }
        }

        // Ground
        for y in ground_y..h {
            for x in 0..w {
                let puddle = ((x as f64 * 0.3).sin() * 0.5 + 0.5) * 0.3;
                let base = 15.0 + lightning_brightness * 40.0;
                pixels[y * w + x] = (
                    (base + puddle * 10.0).clamp(0.0, 255.0) as u8,
                    (base + puddle * 12.0).clamp(0.0, 255.0) as u8,
                    (base + puddle * 20.0).clamp(0.0, 255.0) as u8,
                );
            }
        }

        // Rain layers: (count, speed, streak_len, brightness, thickness)
        let layers: [(u32, f64, f64, f64, f64); 3] = [
            (80, 120.0, 6.0, 180.0, 1.0),   // back: dim, short
            (120, 160.0, 9.0, 210.0, 1.0),  // middle
            (100, 200.0, 12.0, 240.0, 1.5), // front: bright, long
        ];

        let wind_angle = self.wind * 0.15; // radians offset from vertical

        for (layer_idx, &(base_count, speed, streak_len, brightness, _thickness)) in
            layers.iter().enumerate()
        {
            let count = ((base_count as f64) * self.intensity) as u32;
            let fall_speed = speed * self.intensity.sqrt();

            for i in 0..count {
                let seed_base = (layer_idx as u32) * 50000 + i;

                let start_x = Self::rng(seed_base * 3 + 1) * (wf + 40.0) - 20.0;
                let phase_offset = Self::rng(seed_base * 3 + 2);

                // Continuous falling: y wraps around
                let total_travel = hf + streak_len + 20.0;
                let raw_y = (phase_offset * total_travel + t * fall_speed) % total_travel;
                let head_y = raw_y - 10.0; // start above screen
                let head_x = start_x + wind_angle * head_y + self.wind * t * 15.0;

                // Draw streak as a line from (head_x, head_y) upward/back
                let dx_per_step = -wind_angle;
                let dy_per_step = -1.0;
                let steps = streak_len as i32;
                let br = brightness as u8;

                for s in 0..steps {
                    let sf = s as f64;
                    let px = (head_x + dx_per_step * sf) as i32;
                    let py = (head_y + dy_per_step * sf) as i32;
                    if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                        let fade = 1.0 - sf / streak_len;
                        let idx = py as usize * w + px as usize;
                        let (pr, pg, pb) = pixels[idx];
                        let alpha = fade * 0.7;
                        let r = pr as f64 * (1.0 - alpha) + (br as f64 * 0.7) * alpha;
                        let g = pg as f64 * (1.0 - alpha) + (br as f64 * 0.8) * alpha;
                        let b = pb as f64 * (1.0 - alpha) + br as f64 * alpha;
                        pixels[idx] = (
                            r.clamp(0.0, 255.0) as u8,
                            g.clamp(0.0, 255.0) as u8,
                            b.clamp(0.0, 255.0) as u8,
                        );
                    }
                }

                // Splash effect at ground level
                let splash_y = ground_y as f64;
                if head_y >= splash_y && head_y < splash_y + fall_speed * 0.1 {
                    let splash_x = head_x;
                    let splash_age = (head_y - splash_y) / (fall_speed * 0.1);
                    let splash_radius = (1.0 + splash_age * 4.0).min(5.0);
                    let splash_alpha = (1.0 - splash_age).max(0.0) * 0.5;

                    // Draw expanding ring
                    let ring_steps = 16;
                    for j in 0..ring_steps {
                        let angle = j as f64 / ring_steps as f64 * PI * 2.0;
                        let rx = (splash_x + angle.cos() * splash_radius) as i32;
                        let ry = (splash_y + angle.sin() * splash_radius * 0.3) as i32;
                        if rx >= 0 && rx < w as i32 && ry >= 0 && ry < h as i32 {
                            let idx = ry as usize * w + rx as usize;
                            let (pr, pg, pb) = pixels[idx];
                            let r = pr as f64 + splash_alpha * 80.0;
                            let g = pg as f64 + splash_alpha * 90.0;
                            let b = pb as f64 + splash_alpha * 120.0;
                            pixels[idx] = (
                                r.clamp(0.0, 255.0) as u8,
                                g.clamp(0.0, 255.0) as u8,
                                b.clamp(0.0, 255.0) as u8,
                            );
                        }
                    }
                }
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "intensity".to_string(),
                min: 0.5,
                max: 3.0,
                value: self.intensity,
            },
            ParamDesc {
                name: "wind".to_string(),
                min: -1.0,
                max: 1.0,
                value: self.wind,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "intensity" => self.intensity = value,
            "wind" => self.wind = value,
            _ => {}
        }
    }
}
