use crate::effect::{Effect, ParamDesc};
use font8x8::UnicodeFonts;

const GLYPH_SCALE: u32 = 2;
const GLYPH_W: u32 = 8 * GLYPH_SCALE;
const GLYPH_H: u32 = 8 * GLYPH_SCALE;

pub struct Scroller {
    text: String,
    width: u32,
    height: u32,
    speed: f64,
    wave_amp: f64,
}

impl Scroller {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            width: 0,
            height: 0,
            speed: 1.0,
            wave_amp: 1.0,
        }
    }

    fn get_glyph(ch: char) -> [u8; 8] {
        font8x8::BASIC_FONTS
            .get(ch)
            .unwrap_or(font8x8::BASIC_FONTS.get(' ').unwrap_or([0; 8]))
    }
}

impl Effect for Scroller {
    fn name(&self) -> &str {
        "Scroller"
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

        // Background gradient (dark blue to deep purple)
        for y in 0..h {
            let fy = y as f64 / h as f64;
            let r = (10.0 + fy * 20.0) as u8;
            let g = (5.0 + fy * 8.0) as u8;
            let b = (30.0 + fy * 40.0) as u8;
            for x in 0..w {
                let idx = (y * w + x) as usize;
                if idx < pixels.len() {
                    pixels[idx] = (r, g, b);
                }
            }
        }

        let text_bytes: Vec<char> = self.text.chars().collect();
        let total_text_width = text_bytes.len() as f64 * GLYPH_W as f64;
        let scroll_offset = (t * self.speed * 120.0) % (total_text_width + w as f64);
        let center_y = h as f64 / 2.0 - GLYPH_H as f64 / 2.0;

        for (ci, &ch) in text_bytes.iter().enumerate() {
            let char_x = ci as f64 * GLYPH_W as f64 - scroll_offset + w as f64;

            // Skip characters fully off-screen
            if char_x + GLYPH_W as f64 <= 0.0 || char_x >= w as f64 {
                continue;
            }

            // Sine wave vertical offset
            let wave_phase = char_x / w as f64 * std::f64::consts::PI * 4.0 + t * 3.0;
            let wave_y = (wave_phase.sin() * self.wave_amp * (h as f64 * 0.15)) as f64;
            let base_y = center_y + wave_y;

            // Rainbow color per character
            let hue = (ci as f64 * 0.12 + t * 0.8) % 1.0;
            let (cr, cg, cb) = hsv_to_rgb(hue, 1.0, 1.0);

            let glyph = Self::get_glyph(ch);
            for gy in 0..8u32 {
                let row_bits = glyph[gy as usize];
                for gx in 0..8u32 {
                    if row_bits & (1 << gx) != 0 {
                        // Draw scaled pixel
                        for sy in 0..GLYPH_SCALE {
                            for sx in 0..GLYPH_SCALE {
                                let px = char_x as i32 + (gx * GLYPH_SCALE + sx) as i32;
                                let py = base_y as i32 + (gy * GLYPH_SCALE + sy) as i32;
                                if px >= 0
                                    && px < w as i32
                                    && py >= 0
                                    && py < h as i32
                                {
                                    let idx = (py as u32 * w + px as u32) as usize;
                                    if idx < pixels.len() {
                                        pixels[idx] = (cr, cg, cb);
                                    }
                                }
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
                name: "speed".to_string(),
                min: 0.2,
                max: 3.0,
                value: self.speed,
            },
            ParamDesc {
                name: "wave".to_string(),
                min: 0.0,
                max: 3.0,
                value: self.wave_amp,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "wave" => self.wave_amp = value,
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
