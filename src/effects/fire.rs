use crate::effect::{Effect, ParamDesc};
use rand::Rng;

pub struct Fire {
    width: u32,
    height: u32,
    heat: Vec<f64>,
    palette: [(u8, u8, u8); 256],
    cooling: f64,
    intensity: f64,
}

impl Fire {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            heat: Vec::new(),
            palette: Self::build_palette(),
            cooling: 0.4,
            intensity: 1.0,
        }
    }

    fn build_palette() -> [(u8, u8, u8); 256] {
        let mut palette = [(0u8, 0u8, 0u8); 256];
        let control_points: &[(usize, (u8, u8, u8))] = &[
            (0, (0, 0, 0)),
            (60, (128, 0, 0)),
            (150, (255, 128, 0)),
            (220, (255, 255, 0)),
            (255, (255, 255, 255)),
        ];

        for window in control_points.windows(2) {
            let (i0, c0) = window[0];
            let (i1, c1) = window[1];
            for i in i0..=i1 {
                let t = if i1 == i0 {
                    0.0
                } else {
                    (i - i0) as f64 / (i1 - i0) as f64
                };
                palette[i] = (
                    (c0.0 as f64 + (c1.0 as f64 - c0.0 as f64) * t) as u8,
                    (c0.1 as f64 + (c1.1 as f64 - c0.1 as f64) * t) as u8,
                    (c0.2 as f64 + (c1.2 as f64 - c0.2 as f64) * t) as u8,
                );
            }
        }

        palette
    }
}

impl Effect for Fire {
    fn name(&self) -> &str {
        "Fire"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.heat = vec![0.0; (width * height) as usize];
    }

    fn update(&mut self, _t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width as usize;
        let h = self.height as usize;
        if w == 0 || h == 0 {
            return;
        }

        let mut rng = rand::thread_rng();

        // Seed bottom 2 rows with random heat
        for y in (h - 2)..h {
            for x in 0..w {
                self.heat[y * w + x] = rng.gen_range(0.0..1.0) * self.intensity;
            }
        }

        // Propagate heat upward: process from top so reads from below are unmodified
        for y in 0..(h - 2) {
            for x in 0..w {
                let below = self.heat[(y + 1) * w + x];
                let below_left = if x > 0 {
                    self.heat[(y + 1) * w + x - 1]
                } else {
                    self.heat[(y + 1) * w + x]
                };
                let below_right = if x < w - 1 {
                    self.heat[(y + 1) * w + x + 1]
                } else {
                    self.heat[(y + 1) * w + x]
                };
                let two_below = self.heat[(y + 2) * w + x];

                let avg = (below + below_left + below_right + two_below) / 4.0;
                self.heat[y * w + x] = (avg - self.cooling * 0.012).max(0.0);
            }
        }

        // Render heat to pixels via palette
        for i in 0..pixels.len().min(self.heat.len()) {
            let idx = (self.heat[i].clamp(0.0, 1.0) * 255.0) as usize;
            pixels[i] = self.palette[idx];
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "cooling".to_string(),
                min: 0.1,
                max: 1.5,
                value: self.cooling,
            },
            ParamDesc {
                name: "intensity".to_string(),
                min: 0.3,
                max: 2.0,
                value: self.intensity,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "cooling" => self.cooling = value,
            "intensity" => self.intensity = value,
            _ => {}
        }
    }
}
