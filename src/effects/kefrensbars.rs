use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct KefrensBars {
    width: u32,
    height: u32,
    speed: f64,
    bar_count: f64,
}

impl KefrensBars {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            bar_count: 8.0,
        }
    }
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let h = ((h % 1.0) + 1.0) % 1.0;
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

impl Effect for KefrensBars {
    fn name(&self) -> &str {
        "Kefrens Bars"
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
        let num_bars = self.bar_count as usize;

        // Clear to dark background
        for p in pixels.iter_mut() {
            *p = (2, 2, 4);
        }

        // Each bar is a vertical stripe that sweeps horizontally via sine wave.
        // The x-position of each bar varies per scanline, creating the classic
        // Kefrens wavy vertical bar look.
        for bar_i in 0..num_bars {
            let bi = bar_i as f64;
            let phase = bi * PI * 2.0 / num_bars as f64;
            let bar_width = 4.0 + (bi * 0.3).sin().abs() * 2.0;

            // Per-scanline: compute the x center of this bar
            for y in 0..h {
                let yf = y as f64 / hf;

                // Multiple sine waves for complex motion
                let x_center = wf * 0.5
                    + (t * 1.3 + phase + yf * 3.0).sin() * wf * 0.25
                    + (t * 0.7 + phase * 1.5 + yf * 5.0).sin() * wf * 0.1
                    + (t * 2.1 + phase * 0.7 + yf * 1.5).sin() * wf * 0.05;

                // Bar color: rainbow gradient along height, shifted per bar
                let hue = (yf * 1.0 + bi / num_bars as f64 + t * 0.1) % 1.0;
                let (cr, cg, cb) = hsv_to_rgb(hue, 0.8, 1.0);

                let half_w = bar_width / 2.0;
                let x_start = (x_center - half_w).max(0.0) as u32;
                let x_end = ((x_center + half_w).min(wf - 1.0)) as u32;

                let row = (y * w) as usize;
                for x in x_start..=x_end {
                    // Brightness profile: bright center, dimmer edges
                    let dx = (x as f64 - x_center).abs() / half_w;
                    let shine = 1.0 - dx * dx;
                    // Extra bright center stripe
                    let center_boost = if dx < 0.2 { 1.3 } else { 1.0 };
                    let brightness = (shine * center_boost).clamp(0.0, 1.0);

                    let idx = row + x as usize;
                    if idx < pixels.len() {
                        // Additive blending for overlapping bars
                        let p = &mut pixels[idx];
                        let nr = (cr * brightness * 255.0) as u8;
                        let ng = (cg * brightness * 255.0) as u8;
                        let nb = (cb * brightness * 255.0) as u8;
                        p.0 = p.0.saturating_add(nr);
                        p.1 = p.1.saturating_add(ng);
                        p.2 = p.2.saturating_add(nb);
                    }
                }
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
                name: "bar_count".to_string(),
                min: 4.0,
                max: 16.0,
                value: self.bar_count,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "bar_count" => self.bar_count = value,
            _ => {}
        }
    }
}
