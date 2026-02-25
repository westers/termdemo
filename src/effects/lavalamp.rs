use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

const MAX_BLOBS: usize = 10;

pub struct LavaLamp {
    width: u32,
    height: u32,
    speed: f64,
    blob_count: f64,
}

struct Blob {
    base_x: f64,
    base_y: f64,
    radius: f64,
    freq_y: f64,
    freq_x: f64,
    phase_y: f64,
    phase_x: f64,
    amp_y: f64,
    amp_x: f64,
}

impl LavaLamp {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 0.7,
            blob_count: 6.0,
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

    fn make_blobs(count: usize) -> Vec<Blob> {
        let mut blobs = Vec::with_capacity(count);
        for i in 0..count {
            let seed = i as u32 * 7 + 42;
            blobs.push(Blob {
                base_x: 0.3 + Self::rng(seed) * 0.4,
                base_y: 0.2 + Self::rng(seed + 1) * 0.6,
                radius: 0.06 + Self::rng(seed + 2) * 0.06,
                freq_y: 0.3 + Self::rng(seed + 3) * 0.5,
                freq_x: 0.15 + Self::rng(seed + 4) * 0.3,
                phase_y: Self::rng(seed + 5) * PI * 2.0,
                phase_x: Self::rng(seed + 6) * PI * 2.0,
                amp_y: 0.15 + Self::rng(seed + 7) * 0.2,
                amp_x: 0.04 + Self::rng(seed + 8) * 0.06,
            });
        }
        blobs
    }

    /// Map metaball field value to warm lava color.
    fn lava_color(field: f64) -> (u8, u8, u8) {
        if field < 0.8 {
            // Below threshold: dark warm background inside lamp
            (25, 8, 5)
        } else if field < 1.0 {
            // Edge glow: bright orange/yellow
            let t = (field - 0.8) / 0.2;
            let r = (180.0 + t * 75.0).min(255.0);
            let g = (80.0 + t * 120.0).min(200.0);
            let b = (10.0 + t * 20.0).min(30.0);
            (r as u8, g as u8, b as u8)
        } else if field < 1.5 {
            // Interior: deep red/orange
            let t = (field - 1.0) / 0.5;
            let r = (255.0 - t * 55.0).max(180.0);
            let g = (200.0 - t * 120.0).max(40.0);
            let b = (30.0 - t * 15.0).max(5.0);
            (r as u8, g as u8, b as u8)
        } else {
            // Deep interior: dark red
            let t = ((field - 1.5) / 1.0).min(1.0);
            let r = (180.0 - t * 60.0).max(100.0);
            let g = (40.0 - t * 25.0).max(10.0);
            let b = 5;
            (r as u8, g as u8, b)
        }
    }
}

impl Effect for LavaLamp {
    fn name(&self) -> &str {
        "LavaLamp"
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

        let count = (self.blob_count as usize).clamp(3, MAX_BLOBS);
        let blobs = Self::make_blobs(count);
        let ts = t * self.speed;

        // Lamp geometry (tall rounded rectangle centered on screen)
        let lamp_left = wf * 0.3;
        let lamp_right = wf * 0.7;
        let lamp_top = hf * 0.08;
        let lamp_bottom = hf * 0.92;
        let lamp_cx = wf * 0.5;
        let lamp_w_half = (lamp_right - lamp_left) * 0.5;
        let lamp_h_half = (lamp_bottom - lamp_top) * 0.5;
        let lamp_cy = (lamp_top + lamp_bottom) * 0.5;
        let corner_r = lamp_w_half * 0.4;

        // Dark background
        let bg: (u8, u8, u8) = (10, 6, 8);
        for p in pixels.iter_mut() {
            *p = bg;
        }

        // Draw lamp contents and outline
        for y in 0..h {
            for x in 0..w {
                let px = x as f64 + 0.5;
                let py = y as f64 + 0.5;

                // Check if inside lamp (rounded rectangle)
                let dx = (px - lamp_cx).abs();
                let dy = (py - lamp_cy).abs();
                let inner_w = lamp_w_half - corner_r;
                let inner_h = lamp_h_half - corner_r;

                let inside = if dx <= inner_w || dy <= inner_h {
                    dx <= lamp_w_half && dy <= lamp_h_half
                } else {
                    let cdx = dx - inner_w;
                    let cdy = dy - inner_h;
                    cdx * cdx + cdy * cdy <= corner_r * corner_r
                };

                // Compute distance to lamp boundary for outline
                let dist_to_edge = if dx <= inner_w {
                    (lamp_h_half - dy).abs()
                } else if dy <= inner_h {
                    (lamp_w_half - dx).abs()
                } else {
                    let cdx = dx - inner_w;
                    let cdy = dy - inner_h;
                    (corner_r - (cdx * cdx + cdy * cdy).sqrt()).abs()
                };

                let idx = y * w + x;

                if inside {
                    // Compute metaball field in normalized coordinates
                    let nx = px / wf;
                    let ny = py / hf;

                    let mut field = 0.0;
                    for blob in &blobs {
                        let bx = blob.base_x + (ts * blob.freq_x + blob.phase_x).sin() * blob.amp_x;
                        let by = blob.base_y + (ts * blob.freq_y + blob.phase_y).sin() * blob.amp_y;
                        let r = blob.radius;

                        let ddx = nx - bx;
                        let ddy = (ny - by) * (wf / hf); // aspect correction
                        let dist_sq = ddx * ddx + ddy * ddy;
                        field += (r * r) / (dist_sq + 0.0001);
                    }

                    let color = Self::lava_color(field);

                    // Subtle glass tint at edges of lamp
                    let edge_fade = (dist_to_edge / 6.0).clamp(0.0, 1.0);
                    let r = color.0 as f64 * edge_fade + 30.0 * (1.0 - edge_fade);
                    let g = color.1 as f64 * edge_fade + 15.0 * (1.0 - edge_fade);
                    let b = color.2 as f64 * edge_fade + 10.0 * (1.0 - edge_fade);

                    pixels[idx] = (
                        r.clamp(0.0, 255.0) as u8,
                        g.clamp(0.0, 255.0) as u8,
                        b.clamp(0.0, 255.0) as u8,
                    );
                }

                // Lamp outline
                if dist_to_edge < 1.5 && (inside || dist_to_edge < 1.0) {
                    let outline_alpha = (1.0 - dist_to_edge / 1.5).clamp(0.0, 1.0) * 0.6;
                    let (pr, pg, pb) = pixels[idx];
                    let r = pr as f64 * (1.0 - outline_alpha) + 120.0 * outline_alpha;
                    let g = pg as f64 * (1.0 - outline_alpha) + 100.0 * outline_alpha;
                    let b = pb as f64 * (1.0 - outline_alpha) + 80.0 * outline_alpha;
                    pixels[idx] = (
                        r.clamp(0.0, 255.0) as u8,
                        g.clamp(0.0, 255.0) as u8,
                        b.clamp(0.0, 255.0) as u8,
                    );
                }

                // Subtle ambient glow outside lamp near the edges
                if !inside && dist_to_edge < 8.0 {
                    let glow = (1.0 - dist_to_edge / 8.0).powi(2) * 0.15;
                    let (pr, pg, pb) = pixels[idx];
                    let r = pr as f64 + glow * 120.0;
                    let g = pg as f64 + glow * 40.0;
                    let b = pb as f64 + glow * 10.0;
                    pixels[idx] = (
                        r.clamp(0.0, 255.0) as u8,
                        g.clamp(0.0, 255.0) as u8,
                        b.clamp(0.0, 255.0) as u8,
                    );
                }
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
                name: "blob_count".to_string(),
                min: 3.0,
                max: 10.0,
                value: self.blob_count,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "blob_count" => self.blob_count = value,
            _ => {}
        }
    }
}
