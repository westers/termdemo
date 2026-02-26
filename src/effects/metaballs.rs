use crate::effect::{Effect, ParamDesc};
use rand::rngs::StdRng;
use rand::Rng;

struct Blob {
    freq_x: f64,
    freq_y: f64,
    phase_x: f64,
    phase_y: f64,
    radius: f64,
}

pub struct Metaballs {
    width: u32,
    height: u32,
    blobs: Vec<Blob>,
    speed: f64,
    threshold: f64,
}

impl Metaballs {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            blobs: Vec::new(),
            speed: 1.0,
            threshold: 1.0,
        }
    }
}

impl Effect for Metaballs {
    fn name(&self) -> &str {
        "Metaballs"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.blobs.clear();
    }

    fn randomize_init(&mut self, rng: &mut StdRng) {
        self.blobs.clear();
        for _ in 0..5 {
            self.blobs.push(Blob {
                freq_x: rng.gen_range(0.3..1.5),
                freq_y: rng.gen_range(0.3..1.5),
                phase_x: rng.gen_range(0.0..std::f64::consts::TAU),
                phase_y: rng.gen_range(0.0..std::f64::consts::TAU),
                radius: rng.gen_range(0.06..0.14),
            });
        }
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let t_scaled = t * self.speed;

        // Work in normalized 0..1 coordinates so effect scales with screen size
        let centers: Vec<(f64, f64)> = self
            .blobs
            .iter()
            .map(|b| {
                let bx = 0.5 + (t_scaled * b.freq_x + b.phase_x).sin() * 0.35;
                let by = 0.5 + (t_scaled * b.freq_y + b.phase_y).cos() * 0.35;
                (bx, by)
            })
            .collect();

        for y in 0..h {
            let ny = y as f64 / h as f64;
            for x in 0..w {
                let nx = x as f64 / w as f64;

                // Sum metaball field in normalized space
                let mut field = 0.0;
                for (i, blob) in self.blobs.iter().enumerate() {
                    let dx = nx - centers[i].0;
                    let dy = ny - centers[i].1;
                    let dist_sq = dx * dx + dy * dy + 0.0001;
                    field += blob.radius * blob.radius / dist_sq;
                }

                let idx = (y * w + x) as usize;

                // Three-zone coloring
                if field < self.threshold * 0.3 {
                    // Dark background with subtle tint
                    let tint = (field / (self.threshold * 0.3)).clamp(0.0, 1.0);
                    pixels[idx] = (
                        (tint * 15.0) as u8,
                        (tint * 5.0) as u8,
                        (tint * 30.0) as u8,
                    );
                } else if field < self.threshold {
                    // Edge glow: cyan/blue boundary
                    let edge_t =
                        ((field - self.threshold * 0.3) / (self.threshold * 0.7))
                            .clamp(0.0, 1.0);
                    let glow = edge_t * edge_t;
                    pixels[idx] = (
                        (glow * 50.0) as u8,
                        (glow * 180.0) as u8,
                        (glow * 255.0) as u8,
                    );
                } else {
                    // Bright interior with color cycling
                    let interior = ((field - self.threshold) / self.threshold)
                        .clamp(0.0, 1.0);
                    let hue = (t * 0.1 + interior * 0.3) % 1.0;
                    let brightness = 0.7 + interior * 0.3;
                    pixels[idx] = hsv_to_rgb(hue, 0.6, brightness);
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
                name: "threshold".to_string(),
                min: 0.3,
                max: 3.0,
                value: self.threshold,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "threshold" => self.threshold = value,
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
