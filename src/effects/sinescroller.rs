use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

const TEXT: &str = "TERMDEMO ** SINE SCROLLER ** GREETS TO ALL DEMOSCENERS!   ";

/// Simple 5x7 bitmap font for A-Z, space, !, *
/// Each character is 5 columns wide, 7 rows tall.
/// Stored as [u8; 7] per char where each u8 has bits 0..4 for columns.
const FONT_WIDTH: u32 = 5;
const FONT_HEIGHT: u32 = 7;
const GLYPH_SCALE: u32 = 2;
const SCALED_W: u32 = FONT_WIDTH * GLYPH_SCALE;
const SCALED_H: u32 = FONT_HEIGHT * GLYPH_SCALE;

fn get_glyph(ch: char) -> [u8; 7] {
    match ch {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
        '*' => [0b00000, 0b00100, 0b10101, 0b01110, 0b10101, 0b00100, 0b00000],
        ' ' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
        _ => [0b11111, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11111],
    }
}

const NUM_STARS: usize = 120;

pub struct SineScroller {
    width: u32,
    height: u32,
    speed: f64,
    amplitude: f64,
}

impl SineScroller {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            amplitude: 1.0,
        }
    }
}

impl Effect for SineScroller {
    fn name(&self) -> &str {
        "SineScroller"
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

        // Dark background
        for p in pixels.iter_mut() {
            *p = (2, 2, 8);
        }

        // Draw starfield background (deterministic from position, not time-stateful)
        draw_stars(pixels, w, h, t);

        let text_chars: Vec<char> = TEXT.chars().collect();
        let char_w = (SCALED_W + 1) as f64; // 1 pixel gap between chars
        let total_text_width = text_chars.len() as f64 * char_w;
        let scroll_offset = (t * self.speed * 80.0) % (total_text_width + w as f64);
        let center_y = h as f64 / 2.0 - SCALED_H as f64 / 2.0;
        let wave_amp = self.amplitude * h as f64 * 0.2;

        for (ci, &ch) in text_chars.iter().enumerate() {
            let char_x = ci as f64 * char_w - scroll_offset + w as f64;

            // Skip characters fully off-screen
            if char_x + SCALED_W as f64 <= 0.0 || char_x >= w as f64 {
                continue;
            }

            // Sine wave Y offset based on character's screen x position
            let freq = 2.0 * PI * 3.0 / w as f64;
            let wave_y = (char_x * freq + t * 2.5).sin() * wave_amp;
            let base_y = center_y + wave_y;

            // Rainbow color based on x position in the text
            let hue = (ci as f64 / text_chars.len() as f64 + t * 0.15) % 1.0;
            let (cr, cg, cb) = hsv_to_rgb(hue, 1.0, 1.0);

            let glyph = get_glyph(ch);
            for gy in 0..FONT_HEIGHT {
                let row_bits = glyph[gy as usize];
                for gx in 0..FONT_WIDTH {
                    if row_bits & (1 << (FONT_WIDTH - 1 - gx)) != 0 {
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
                min: 0.5,
                max: 3.0,
                value: self.speed,
            },
            ParamDesc {
                name: "amplitude".to_string(),
                min: 0.2,
                max: 1.5,
                value: self.amplitude,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "amplitude" => self.amplitude = value,
            _ => {}
        }
    }
}

/// Deterministic starfield using a simple hash for star placement
fn draw_stars(pixels: &mut [(u8, u8, u8)], w: u32, h: u32, t: f64) {
    for i in 0..NUM_STARS {
        // Deterministic pseudo-random positions using a simple hash
        let seed = i as u64;
        let sx = ((seed.wrapping_mul(2654435761) >> 8) % w as u64) as i32;
        let sy = ((seed.wrapping_mul(40503) >> 4) % h as u64) as i32;
        // Twinkle based on time
        let twinkle = ((t * 2.0 + i as f64 * 0.73).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        let brightness = (40.0 + twinkle * 180.0) as u8;
        if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
            let idx = (sy as u32 * w + sx as u32) as usize;
            if idx < pixels.len() {
                pixels[idx] = (brightness, brightness, brightness);
            }
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
