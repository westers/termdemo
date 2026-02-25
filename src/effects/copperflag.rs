use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct CopperFlag {
    width: u32,
    height: u32,
    wave_speed: f64,
    wave_amount: f64,
}

impl CopperFlag {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            wave_speed: 1.0,
            wave_amount: 1.0,
        }
    }
}

impl Effect for CopperFlag {
    fn name(&self) -> &str {
        "CopperFlag"
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

        // Dark background
        for p in pixels.iter_mut() {
            *p = (4, 4, 10);
        }

        let t = t * self.wave_speed;

        // Flag dimensions (in normalized coordinates centered on screen)
        let flag_width = wf * 0.65;
        let flag_height = hf * 0.60;
        let flag_left = (wf - flag_width) / 2.0;
        let flag_top = (hf - flag_height) / 2.0;

        let wave_amp = self.wave_amount * flag_width * 0.06;
        let wave_freq = 2.0 * PI * 3.0 / flag_height;

        // Number of horizontal color stripes in the flag
        let num_stripes = 24.0;

        for y in 0..h {
            let fy = y as f64;

            // Normalized position within the flag vertically
            let flag_v = (fy - flag_top) / flag_height;
            if flag_v < 0.0 || flag_v > 1.0 {
                continue;
            }

            // Wave distortion: horizontal offset per row
            let wave_offset = (fy * wave_freq + t * 3.5).sin() * wave_amp;
            // Secondary wave for more complex motion
            let wave_offset2 = (fy * wave_freq * 0.5 + t * 2.1 + 1.0).sin() * wave_amp * 0.4;
            let total_offset = wave_offset + wave_offset2;

            // Perspective taper: the right edge is the "far" edge
            // Rows are slightly narrower as we go from left to right (simulated via per-row)
            // Actually, taper based on vertical position to simulate perspective
            let taper = 1.0 - flag_v * 0.12 * self.wave_amount.min(1.5);
            let row_width = flag_width * taper;
            let row_left = flag_left + (flag_width - row_width) / 2.0 + total_offset;

            for x in 0..w {
                let fx = x as f64;

                let flag_u = (fx - row_left) / row_width;
                if flag_u < 0.0 || flag_u > 1.0 {
                    continue;
                }

                // Copper bar coloring: horizontal stripes with rainbow gradient
                // The stripe index shifts with wave for extra motion
                let stripe_phase = flag_v * num_stripes + t * 0.5;
                let stripe_val = (stripe_phase * PI).sin() * 0.5 + 0.5;

                // Base hue cycles through rainbow based on vertical position
                let hue = (flag_v + t * 0.1) % 1.0;

                // Modulate brightness with copper bar pattern
                let brightness = 0.4 + stripe_val * 0.6;

                // Slight shading based on wave offset (simulates lighting on cloth)
                let shade_factor = 1.0
                    + (fy * wave_freq + t * 3.5).cos() * 0.15 * self.wave_amount;
                let final_brightness = (brightness * shade_factor).clamp(0.2, 1.0);

                let (r, g, b) = hsv_to_rgb(hue, 0.85, final_brightness);

                let idx = (y * w + x) as usize;
                if idx < pixels.len() {
                    pixels[idx] = (r, g, b);
                }
            }
        }

        // Draw a flagpole on the left side
        let pole_x = (flag_left - 3.0).max(0.0) as u32;
        let pole_top = (flag_top - 5.0).max(0.0) as u32;
        let pole_bottom = (flag_top + flag_height + 5.0).min(hf - 1.0) as u32;
        for y in pole_top..=pole_bottom {
            for dx in 0..2u32 {
                let x = pole_x + dx;
                if x < w {
                    let idx = (y * w + x) as usize;
                    if idx < pixels.len() {
                        let shade = 100 + ((y - pole_top) as f64
                            / (pole_bottom - pole_top) as f64
                            * 60.0) as u8;
                        pixels[idx] = (shade, shade, shade / 2);
                    }
                }
            }
        }

        // Pole knob at top
        let knob_y = pole_top.saturating_sub(1);
        for dy in 0..2u32 {
            for dx in 0..3u32 {
                let x = pole_x.saturating_sub(1) + dx;
                let y = knob_y + dy;
                if x < w && y < h {
                    let idx = (y * w + x) as usize;
                    if idx < pixels.len() {
                        pixels[idx] = (200, 180, 50);
                    }
                }
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "wave_speed".to_string(),
                min: 0.5,
                max: 3.0,
                value: self.wave_speed,
            },
            ParamDesc {
                name: "wave_amount".to_string(),
                min: 0.3,
                max: 2.0,
                value: self.wave_amount,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "wave_speed" => self.wave_speed = value,
            "wave_amount" => self.wave_amount = value,
            _ => {}
        }
    }
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let tv = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i % 6 {
        0 => (v, tv, p),
        1 => (q, v, p),
        2 => (p, v, tv),
        3 => (p, q, v),
        4 => (tv, p, v),
        _ => (v, p, q),
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}
