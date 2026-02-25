use crate::effect::{Effect, ParamDesc};

pub struct CopperBars {
    width: u32,
    height: u32,
    bar_count: u32,
    scroll_speed: f64,
}

impl CopperBars {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            bar_count: 6,
            scroll_speed: 1.0,
        }
    }
}

// Metallic color palettes: (center_r, center_g, center_b)
const METALLIC_COLORS: [(f64, f64, f64); 4] = [
    (0.85, 0.55, 0.20), // copper
    (1.00, 0.84, 0.30), // gold
    (0.80, 0.82, 0.88), // silver
    (0.80, 0.50, 0.25), // bronze
];

impl Effect for CopperBars {
    fn name(&self) -> &str {
        "CopperBars"
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

        let t = t * self.scroll_speed;
        let hf = h as f64;
        let wf = w as f64;
        let bar_count = self.bar_count as usize;

        // Dark background gradient
        for y in 0..h {
            let fy = y as f64 / hf;
            let bg = (8.0 + fy * 16.0) as u8;
            let row = (y * w) as usize;
            for x in 0..w as usize {
                pixels[row + x] = (bg / 2, bg / 3, bg);
            }
        }

        // Draw bars with painter's algorithm (later bars overdraw)
        for i in 0..bar_count {
            let phase = i as f64 * 2.5;
            let freq = 0.8 + i as f64 * 0.15;
            let center_y = hf * 0.5 + (t * freq + phase).sin() * hf * 0.35;

            let metal = METALLIC_COLORS[i % METALLIC_COLORS.len()];
            let bar_half = 10.0;

            for y in 0..h {
                let dy = (y as f64 - center_y).abs();
                if dy > bar_half {
                    continue;
                }

                // Metallic shine: bright center, darker edges
                let norm = dy / bar_half;
                let shine = 1.0 - norm * norm;
                // Extra bright center stripe
                let center_stripe = if norm < 0.15 { 1.3 } else { 1.0 };
                let brightness = shine * center_stripe;

                let row = (y * w) as usize;
                for x in 0..w {
                    // Slight horizontal gradient for depth
                    let hx = 0.85 + 0.15 * (x as f64 / wf * std::f64::consts::PI).sin();
                    let b = brightness * hx;

                    let r = (metal.0 * b * 255.0).clamp(0.0, 255.0) as u8;
                    let g = (metal.1 * b * 255.0).clamp(0.0, 255.0) as u8;
                    let bl = (metal.2 * b * 255.0).clamp(0.0, 255.0) as u8;

                    pixels[row + x as usize] = (r, g, bl);
                }
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "bar_count".to_string(),
                min: 3.0,
                max: 12.0,
                value: self.bar_count as f64,
            },
            ParamDesc {
                name: "scroll_speed".to_string(),
                min: 0.3,
                max: 3.0,
                value: self.scroll_speed,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "bar_count" => self.bar_count = value as u32,
            "scroll_speed" => self.scroll_speed = value,
            _ => {}
        }
    }
}
