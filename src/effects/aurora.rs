use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct Aurora {
    width: u32,
    height: u32,
    speed: f64,
    intensity: f64,
}

impl Aurora {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            intensity: 1.0,
        }
    }

    /// Deterministic hash for star placement based on position.
    fn star_hash(x: u32, y: u32) -> u32 {
        let mut h = x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263));
        h = (h ^ (h >> 13)).wrapping_mul(1274126177);
        h ^ (h >> 16)
    }
}

impl Effect for Aurora {
    fn name(&self) -> &str {
        "Aurora Borealis"
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
        let t = t * self.speed;
        let intensity = self.intensity;

        // Curtain layer definitions: (base_color_r, g, b, speed_mult, x_freq, x_offset, drop_center)
        let curtains: [(f64, f64, f64, f64, f64, f64, f64); 4] = [
            (0.1, 1.0, 0.4, 1.0, 3.0, 0.0, 0.35),   // green
            (0.0, 0.8, 0.7, 0.7, 2.5, 1.5, 0.30),    // teal
            (0.6, 0.2, 0.9, 1.3, 4.0, 3.0, 0.25),    // purple
            (0.9, 0.3, 0.6, 0.5, 2.0, 5.0, 0.40),    // pink
        ];

        for y in 0..h {
            let yf = y as f64 / hf;
            for x in 0..w {
                let xf = x as f64 / wf;
                let idx = (y * w + x) as usize;

                // Dark background with subtle navy gradient
                let bg_r = (5.0 + 3.0 * (1.0 - yf)) as f64;
                let bg_g = (8.0 + 5.0 * (1.0 - yf)) as f64;
                let bg_b = (20.0 + 15.0 * (1.0 - yf)) as f64;

                let mut r = bg_r;
                let mut g = bg_g;
                let mut b = bg_b;

                // Deterministic stars
                let sh = Self::star_hash(x, y);
                if sh % 397 == 0 {
                    let brightness = 100.0 + ((sh % 156) as f64);
                    r = brightness;
                    g = brightness;
                    b = brightness;
                }

                // Aurora curtain layers (additive)
                for &(cr, cg, cb, spd, freq, off, drop_center) in &curtains {
                    let tt = t * spd;

                    // Horizontal wave displacement for curtain position
                    let wave = (xf * freq * PI + tt * 0.8 + off).sin() * 0.15
                        + (xf * freq * 1.7 * PI + tt * 0.5 + off * 2.0).sin() * 0.08
                        + (xf * freq * 3.1 * PI + tt * 1.2 + off * 0.7).sin() * 0.04;

                    // Curtain hangs from top: vertical falloff
                    let curtain_top = 0.02 + wave * 0.5;
                    let curtain_bottom = drop_center + wave * 0.3
                        + (xf * 2.0 * PI + tt * 0.3).sin() * 0.1;

                    let vert_alpha = if yf < curtain_top {
                        0.0
                    } else if yf < curtain_bottom {
                        let norm = (yf - curtain_top) / (curtain_bottom - curtain_top);
                        // Bright near top, fading down with a bell-like curve
                        let fade = (norm * PI).sin();
                        fade * (1.0 - norm * 0.6)
                    } else {
                        // Tail fade
                        let over = (yf - curtain_bottom) / (1.0 - curtain_bottom).max(0.01);
                        (1.0 - over * 3.0).max(0.0) * 0.3
                    };

                    // Horizontal fold brightness variation
                    let fold = ((xf * freq * 2.5 * PI + tt * 0.6 + off).sin() * 0.5 + 0.5)
                        * 0.6
                        + 0.4;

                    // Shimmer
                    let shimmer = ((xf * 20.0 * PI + tt * 3.0).sin() * 0.1 + 0.9).min(1.0);

                    let alpha = vert_alpha * fold * shimmer * intensity;

                    r += cr * alpha * 180.0;
                    g += cg * alpha * 180.0;
                    b += cb * alpha * 180.0;
                }

                pixels[idx] = (
                    (r.clamp(0.0, 255.0)) as u8,
                    (g.clamp(0.0, 255.0)) as u8,
                    (b.clamp(0.0, 255.0)) as u8,
                );
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "speed".to_string(),
                min: 0.3,
                max: 2.0,
                value: self.speed,
            },
            ParamDesc {
                name: "intensity".to_string(),
                min: 0.5,
                max: 2.0,
                value: self.intensity,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "intensity" => self.intensity = value,
            _ => {}
        }
    }
}
