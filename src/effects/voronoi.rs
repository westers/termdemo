use crate::effect::{Effect, ParamDesc};

const NUM_SEEDS: usize = 24;

struct Seed {
    freq_x: f64,
    freq_y: f64,
    phase_x: f64,
    phase_y: f64,
}

pub struct Voronoi {
    width: u32,
    height: u32,
    speed: f64,
    edge_glow: f64,
    seeds: Vec<Seed>,
}

impl Voronoi {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            edge_glow: 1.0,
            seeds: Vec::new(),
        }
    }
}

impl Effect for Voronoi {
    fn name(&self) -> &str {
        "Voronoi"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;

        // Deterministic seeds with varied Lissajous frequencies
        self.seeds.clear();
        for i in 0..NUM_SEEDS {
            let fi = i as f64;
            self.seeds.push(Seed {
                freq_x: 0.3 + (fi * 0.17) % 0.8,
                freq_y: 0.4 + (fi * 0.23) % 0.7,
                phase_x: fi * 1.3,
                phase_y: fi * 1.7,
            });
        }
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

        // Compute seed positions in pixel space
        let positions: Vec<(f64, f64)> = self
            .seeds
            .iter()
            .map(|s| {
                let x = (0.5 + 0.45 * (t * s.freq_x + s.phase_x).sin()) * wf;
                let y = (0.5 + 0.45 * (t * s.freq_y + s.phase_y).cos()) * hf;
                (x, y)
            })
            .collect();

        for y in 0..h {
            let py = y as f64;
            let row = (y * w) as usize;

            for x in 0..w {
                let px = x as f64;

                // Find closest and second-closest seed
                let mut d1 = f64::MAX;
                let mut d2 = f64::MAX;
                let mut closest = 0usize;

                for (i, &(sx, sy)) in positions.iter().enumerate() {
                    let dx = px - sx;
                    let dy = py - sy;
                    let d = dx * dx + dy * dy;

                    if d < d1 {
                        d2 = d1;
                        d1 = d;
                        closest = i;
                    } else if d < d2 {
                        d2 = d;
                    }
                }

                let d1 = d1.sqrt();
                let d2 = d2.sqrt();

                // Edge detection: how close to the boundary between cells
                let edge = (d2 - d1) / (d2 + d1 + 0.001);

                // Cell color from seed index + time
                let hue = (closest as f64 / NUM_SEEDS as f64 + t * 0.03) % 1.0;

                // Interior brightness: slight gradient from center
                let interior = (1.0 - d1 * 0.003).clamp(0.5, 1.0);

                // Edge glow: bright white/cyan at cell boundaries
                let edge_factor = (1.0 - edge * 4.0 * self.edge_glow).clamp(0.0, 1.0);
                let edge_bright = (1.0 - edge_factor) * self.edge_glow;

                let (cr, cg, cb) = hsv_to_rgb(hue, 0.75, interior * 0.7);

                let idx = row + x as usize;
                pixels[idx] = (
                    (cr as f64 + edge_bright * 180.0).clamp(0.0, 255.0) as u8,
                    (cg as f64 + edge_bright * 220.0).clamp(0.0, 255.0) as u8,
                    (cb as f64 + edge_bright * 255.0).clamp(0.0, 255.0) as u8,
                );
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
                name: "edge_glow".to_string(),
                min: 0.0,
                max: 3.0,
                value: self.edge_glow,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "edge_glow" => self.edge_glow = value,
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
