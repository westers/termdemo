use crate::effect::{Effect, ParamDesc};

pub struct Parallax {
    width: u32,
    height: u32,
    scroll_speed: f64,
    layers: f64,
}

impl Parallax {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            scroll_speed: 1.0,
            layers: 5.0,
        }
    }
}

/// Deterministic pseudo-random hash for star placement
fn hash_u32(mut x: u32) -> u32 {
    x = x.wrapping_mul(0x9E3779B9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EBCA6B);
    x ^= x >> 13;
    x
}

/// Generate mountain height at a given x position for a layer
/// Uses several sine waves combined for a natural ridge shape
fn mountain_height(x: f64, layer: usize) -> f64 {
    let seed = layer as f64 * 17.3;
    let h1 = (x * 0.008 + seed).sin() * 0.35;
    let h2 = (x * 0.019 + seed * 1.7).sin() * 0.2;
    let h3 = (x * 0.041 + seed * 2.3).sin() * 0.1;
    let h4 = (x * 0.097 + seed * 3.1).sin() * 0.05;
    let h5 = (x * 0.003 + seed * 0.5).sin() * 0.25;
    h1 + h2 + h3 + h4 + h5
}

impl Effect for Parallax {
    fn name(&self) -> &str {
        "Parallax Landscape"
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
        let num_layers = self.layers as usize;
        let t = t * self.scroll_speed;

        // Horizon position (60% down from top)
        let horizon = hf * 0.6;

        // --- Draw sky gradient ---
        for y in 0..h {
            let yf = y as f64;
            let row = (y * w) as usize;

            if yf <= horizon {
                // Sky: deep blue at top -> orange/red at horizon
                let sky_t = yf / horizon;
                let sky_t2 = sky_t * sky_t; // accelerate toward horizon

                // Top: deep navy blue (10, 10, 40)
                // Mid: purple-ish (60, 20, 60)
                // Horizon: warm orange-red (200, 100, 40)
                let r;
                let g;
                let b;
                if sky_t < 0.7 {
                    let st = sky_t / 0.7;
                    r = 10.0 + st * 50.0;
                    g = 10.0 + st * 15.0;
                    b = 50.0 + st * 30.0;
                } else {
                    let st = (sky_t - 0.7) / 0.3;
                    r = 60.0 + st * 160.0;
                    g = 25.0 + st * 80.0;
                    b = 80.0 - st * 50.0;
                }

                let color = (r as u8, g as u8, b as u8);
                for x in 0..w as usize {
                    pixels[row + x] = color;
                }

                // Stars in upper portion of sky
                if sky_t2 < 0.3 {
                    for x in 0..w {
                        let star_hash = hash_u32(x.wrapping_mul(7919).wrapping_add(y.wrapping_mul(104729)));
                        if star_hash % 800 == 0 {
                            let brightness = (120 + (star_hash % 136) as u8).min(255);
                            // Twinkle
                            let twinkle = ((t * 2.0 + (star_hash % 100) as f64 * 0.37).sin() * 0.3 + 0.7).clamp(0.4, 1.0);
                            let b_val = (brightness as f64 * twinkle) as u8;
                            pixels[row + x as usize] = (b_val, b_val, (b_val as u16 * 9 / 10) as u8);
                        }
                    }
                }
            } else {
                // Below horizon: dark ground (will be covered by closest mountain layer)
                let ground_t = (yf - horizon) / (hf - horizon);
                let r = (15.0 - ground_t * 10.0) as u8;
                let g = (10.0 - ground_t * 5.0) as u8;
                let b = (8.0 - ground_t * 5.0) as u8;
                let color = (r, g, b);
                for x in 0..w as usize {
                    pixels[row + x] = color;
                }
            }
        }

        // --- Draw sun near horizon ---
        let sun_x = wf * 0.7;
        let sun_y = horizon - hf * 0.05;
        let sun_radius = hf * 0.08;
        let glow_radius = sun_radius * 2.5;

        for dy in -(glow_radius as i32)..=(glow_radius as i32) {
            for dx in -(glow_radius as i32)..=(glow_radius as i32) {
                let px = sun_x as i32 + dx;
                let py = sun_y as i32 + dy;
                if px < 0 || px >= w as i32 || py < 0 || py >= h as i32 {
                    continue;
                }
                let dist = ((dx * dx + dy * dy) as f64).sqrt();
                let idx = (py as u32 * w + px as u32) as usize;

                if dist < sun_radius {
                    // Sun body: bright yellow-white
                    let edge = (1.0 - dist / sun_radius).clamp(0.0, 1.0);
                    let r = 255;
                    let g = (220.0 + edge * 35.0) as u8;
                    let b = (140.0 + edge * 80.0) as u8;
                    pixels[idx] = (r, g, b);
                } else if dist < glow_radius {
                    // Glow around sun
                    let glow = ((1.0 - (dist - sun_radius) / (glow_radius - sun_radius)).powi(2) * 80.0) as u8;
                    let p = &mut pixels[idx];
                    p.0 = p.0.saturating_add(glow);
                    p.1 = p.1.saturating_add(glow / 2);
                    p.2 = p.2.saturating_add(glow / 6);
                }
            }
        }

        // --- Draw mountain layers (back to front) ---
        // Back layers are lighter/hazier, front layers are darker silhouettes
        for layer_i in 0..num_layers {
            let layer_depth = layer_i as f64 / (num_layers.max(1) - 1).max(1) as f64;
            // depth: 0.0 = farthest back, 1.0 = closest front

            // Scroll speed: front layers scroll faster
            let scroll = t * 15.0 * (0.1 + layer_depth * 0.9);

            // Mountain base: lower layers sit at horizon, front layers extend below
            let base_y = horizon + layer_depth * (hf - horizon) * 0.3;

            // Mountain height scales with layer
            let height_scale = hf * (0.12 + layer_depth * 0.18);

            // Layer color: far = light blue haze, close = dark black
            let inv_depth = 1.0 - layer_depth;
            let lr = (20.0 + inv_depth * 60.0) as u8;
            let lg = (15.0 + inv_depth * 40.0) as u8;
            let lb = (30.0 + inv_depth * 80.0) as u8;

            for x in 0..w {
                let world_x = x as f64 + scroll;
                let mh = mountain_height(world_x, layer_i);
                let peak_y = base_y - mh * height_scale;

                // Fill from peak down to bottom of screen
                let peak_y_int = peak_y.max(0.0) as u32;
                for y in peak_y_int..h {
                    let idx = (y * w + x) as usize;
                    pixels[idx] = (lr, lg, lb);
                }
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "scroll_speed".to_string(),
                min: 0.3,
                max: 3.0,
                value: self.scroll_speed,
            },
            ParamDesc {
                name: "layers".to_string(),
                min: 3.0,
                max: 7.0,
                value: self.layers,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "scroll_speed" => self.scroll_speed = value,
            "layers" => self.layers = value,
            _ => {}
        }
    }
}
