use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct Neon {
    width: u32,
    height: u32,
    brightness: f64,
    flicker: f64,
    /// Precomputed glow buffer (distances to nearest neon shape).
    glow_r: Vec<f64>,
    glow_g: Vec<f64>,
    glow_b: Vec<f64>,
    brick_bg: Vec<(u8, u8, u8)>,
}

/// Simple 5x7 bitmap font for "DEMO".
const FONT_WIDTH: usize = 5;
const FONT_HEIGHT: usize = 7;

fn char_bitmap(c: char) -> [u8; 7] {
    match c {
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        _ => [0; 7],
    }
}

impl Neon {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            brightness: 1.0,
            flicker: 0.3,
            glow_r: Vec::new(),
            glow_g: Vec::new(),
            glow_b: Vec::new(),
            brick_bg: Vec::new(),
        }
    }

    /// Deterministic pseudo-random from seed.
    fn rng(seed: u32) -> f64 {
        let mut h = seed;
        h = h.wrapping_mul(747796405).wrapping_add(2891336453);
        h = ((h >> ((h >> 28).wrapping_add(4))) ^ h).wrapping_mul(277803737);
        h = h ^ (h >> 22);
        (h & 0x00FFFFFF) as f64 / 0x01000000 as f64
    }

    fn build_brick_bg(w: u32, h: u32) -> Vec<(u8, u8, u8)> {
        let mut bg = vec![(0u8, 0u8, 0u8); (w * h) as usize];
        let brick_w = 8;
        let brick_h = 4;
        let mortar: (u8, u8, u8) = (40, 35, 30);

        for y in 0..h {
            for x in 0..w {
                let row = y as usize / brick_h;
                let offset = if row % 2 == 0 { 0 } else { brick_w / 2 };
                let bx = ((x as usize + offset) % brick_w) as usize;
                let by = (y as usize % brick_h) as usize;

                if bx == 0 || by == 0 {
                    bg[(y * w + x) as usize] = mortar;
                } else {
                    let seed = (row * 997 + (x as usize + offset) / brick_w) as u32;
                    let variation = Self::rng(seed) * 30.0 - 15.0;
                    let r = (70.0 + variation).clamp(30.0, 110.0) as u8;
                    let g = (40.0 + variation * 0.6).clamp(20.0, 70.0) as u8;
                    let b = (35.0 + variation * 0.4).clamp(15.0, 60.0) as u8;
                    bg[(y * w + x) as usize] = (r, g, b);
                }
            }
        }
        bg
    }

    fn build_glow_layers(w: u32, h: u32) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let size = (w * h) as usize;
        let mut gr = vec![0.0f64; size];
        let mut gg = vec![0.0f64; size];
        let mut gb = vec![0.0f64; size];

        let wf = w as f64;
        let hf = h as f64;
        let cx = wf * 0.5;
        let _cy = hf * 0.5;

        // Neon circle: center-left area, pink/magenta
        let circle_cx = wf * 0.22;
        let circle_cy = hf * 0.4;
        let circle_r = hf.min(wf) * 0.15;

        // Neon triangle: center-right area, cyan
        let tri_cx = wf * 0.78;
        let tri_cy = hf * 0.4;
        let tri_size = hf.min(wf) * 0.15;

        // "DEMO" text: center, blue
        let text = "DEMO";
        let text_scale = (wf * 0.015).max(1.0);
        let text_total_w = text.len() as f64 * (FONT_WIDTH as f64 + 1.0) * text_scale;
        let text_start_x = cx - text_total_w * 0.5;
        let text_start_y = hf * 0.72;

        // Gather neon shape pixels
        // We precompute a sparse set of neon points, then for each pixel compute distance.
        // For efficiency, store neon points per shape.

        let mut circle_pts: Vec<(f64, f64)> = Vec::new();
        let nsteps = 200;
        for i in 0..nsteps {
            let angle = i as f64 / nsteps as f64 * PI * 2.0;
            circle_pts.push((
                circle_cx + angle.cos() * circle_r,
                circle_cy + angle.sin() * circle_r,
            ));
        }

        let mut tri_pts: Vec<(f64, f64)> = Vec::new();
        let tri_verts = [
            (tri_cx, tri_cy - tri_size),
            (tri_cx - tri_size * 0.866, tri_cy + tri_size * 0.5),
            (tri_cx + tri_size * 0.866, tri_cy + tri_size * 0.5),
        ];
        for edge in 0..3 {
            let (ax, ay) = tri_verts[edge];
            let (bx, by) = tri_verts[(edge + 1) % 3];
            let steps = 100;
            for i in 0..steps {
                let t = i as f64 / steps as f64;
                tri_pts.push((ax + (bx - ax) * t, ay + (by - ay) * t));
            }
        }

        let mut text_pts: Vec<(f64, f64)> = Vec::new();
        for (ci, ch) in text.chars().enumerate() {
            let bmp = char_bitmap(ch);
            let ox = text_start_x + ci as f64 * (FONT_WIDTH as f64 + 1.0) * text_scale;
            for row in 0..FONT_HEIGHT {
                for col in 0..FONT_WIDTH {
                    if (bmp[row] >> (FONT_WIDTH - 1 - col)) & 1 == 1 {
                        let px = ox + col as f64 * text_scale + text_scale * 0.5;
                        let py = text_start_y + row as f64 * text_scale + text_scale * 0.5;
                        text_pts.push((px, py));
                    }
                }
            }
        }

        // Compute glow for each pixel
        let glow_radius = 15.0_f64;
        let glow_radius_sq = glow_radius * glow_radius;

        for y in 0..h {
            for x in 0..w {
                let px = x as f64 + 0.5;
                let py = y as f64 + 0.5;
                let idx = (y * w + x) as usize;

                // Circle glow (pink: R=1.0, G=0.2, B=0.6)
                let mut min_d2 = f64::MAX;
                for &(nx, ny) in &circle_pts {
                    let dx = px - nx;
                    let dy = py - ny;
                    let d2 = dx * dx + dy * dy;
                    if d2 < min_d2 {
                        min_d2 = d2;
                    }
                }
                if min_d2 < glow_radius_sq {
                    let glow = 1.0 / (1.0 + min_d2 * 0.15);
                    gr[idx] += glow * 1.0;
                    gg[idx] += glow * 0.2;
                    gb[idx] += glow * 0.6;
                }

                // Triangle glow (cyan: R=0.1, G=0.9, B=1.0)
                min_d2 = f64::MAX;
                for &(nx, ny) in &tri_pts {
                    let dx = px - nx;
                    let dy = py - ny;
                    let d2 = dx * dx + dy * dy;
                    if d2 < min_d2 {
                        min_d2 = d2;
                    }
                }
                if min_d2 < glow_radius_sq {
                    let glow = 1.0 / (1.0 + min_d2 * 0.15);
                    gr[idx] += glow * 0.1;
                    gg[idx] += glow * 0.9;
                    gb[idx] += glow * 1.0;
                }

                // Text glow (blue-white: R=0.4, G=0.5, B=1.0)
                min_d2 = f64::MAX;
                for &(nx, ny) in &text_pts {
                    let dx = px - nx;
                    let dy = py - ny;
                    let d2 = dx * dx + dy * dy;
                    if d2 < min_d2 {
                        min_d2 = d2;
                    }
                }
                if min_d2 < glow_radius_sq {
                    let glow = 1.0 / (1.0 + min_d2 * 0.15);
                    gr[idx] += glow * 0.4;
                    gg[idx] += glow * 0.5;
                    gb[idx] += glow * 1.0;
                }
            }
        }

        (gr, gg, gb)
    }
}

impl Effect for Neon {
    fn name(&self) -> &str {
        "Neon"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.brick_bg = Self::build_brick_bg(width, height);
        let (gr, gg, gb) = Self::build_glow_layers(width, height);
        self.glow_r = gr;
        self.glow_g = gg;
        self.glow_b = gb;
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 || self.brick_bg.is_empty() {
            return;
        }

        let size = (w * h) as usize;

        // Flicker functions for each neon shape
        // Circle: subtle flicker
        let flicker_circle = 1.0
            - self.flicker
                * 0.3
                * ((t * 17.3).sin() * 0.5 + (t * 31.7).sin() * 0.3 + (t * 53.1).sin() * 0.2).abs();

        // Triangle: subtle flicker
        let flicker_tri = 1.0
            - self.flicker
                * 0.3
                * ((t * 13.1 + 1.0).sin() * 0.4
                    + (t * 29.3 + 2.0).sin() * 0.35
                    + (t * 47.7).sin() * 0.25)
                    .abs();

        // Text: periodic on/off "broken sign" effect (off for ~0.3s every ~4s)
        let text_cycle = (t * 0.25).fract(); // period of 4 seconds
        let text_on = if text_cycle > 0.88 && text_cycle < 0.95 {
            // Brief off period
            0.05
        } else {
            1.0 - self.flicker
                * 0.2
                * ((t * 19.7 + 3.0).sin() * 0.5 + (t * 41.3 + 1.0).sin() * 0.5).abs()
        };

        let bright = self.brightness;

        for i in 0..size.min(pixels.len()) {
            let (br, bg, bb) = self.brick_bg[i];

            // Apply glow from each shape with its flicker multiplier
            // The glow_r/g/b contains combined contributions from all shapes,
            // but we stored them additively. We'll re-split by checking which shape
            // dominates, but for simplicity and perf, we apply a single weighted flicker.
            // The glow channels already encode per-shape color, so we apply a blended flicker.
            let gr = self.glow_r[i];
            let gg = self.glow_g[i];
            let gb = self.glow_b[i];

            // Approximate per-channel flicker weighting:
            // Pink (circle) is strongest in R, Cyan (tri) in G+B, Blue (text) in B
            // Use a heuristic blend of flicker factors.
            let total = gr + gg + gb;
            if total < 0.001 {
                pixels[i] = (br, bg, bb);
                continue;
            }

            // Weight flickers by relative channel contributions
            // Circle contributes heavily to R, triangle to G, text to B
            let f = flicker_circle * 0.33 + flicker_tri * 0.33 + text_on * 0.34;

            let glow_mult = bright * f;

            let out_r = br as f64 + gr * glow_mult * 255.0;
            let out_g = bg as f64 + gg * glow_mult * 255.0;
            let out_b = bb as f64 + gb * glow_mult * 255.0;

            pixels[i] = (
                out_r.clamp(0.0, 255.0) as u8,
                out_g.clamp(0.0, 255.0) as u8,
                out_b.clamp(0.0, 255.0) as u8,
            );
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "brightness".to_string(),
                min: 0.5,
                max: 2.0,
                value: self.brightness,
            },
            ParamDesc {
                name: "flicker".to_string(),
                min: 0.0,
                max: 1.0,
                value: self.flicker,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "brightness" => self.brightness = value,
            "flicker" => self.flicker = value,
            _ => {}
        }
    }
}
