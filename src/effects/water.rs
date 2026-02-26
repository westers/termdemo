use crate::effect::{Effect, ParamDesc};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct Water {
    width: u32,
    height: u32,
    damping: f64,
    drop_freq: f64,
    buf_current: Vec<f64>,
    buf_previous: Vec<f64>,
    drop_accum: f64,
    rng: StdRng,
}

impl Water {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            damping: 0.97,
            drop_freq: 3.0,
            buf_current: Vec::new(),
            buf_previous: Vec::new(),
            drop_accum: 0.0,
            rng: StdRng::seed_from_u64(0),
        }
    }
}

impl Effect for Water {
    fn name(&self) -> &str {
        "Water"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let size = (width * height) as usize;
        self.buf_current = vec![0.0; size];
        self.buf_previous = vec![0.0; size];
        self.drop_accum = 0.0;
    }

    fn randomize_init(&mut self, rng: &mut StdRng) {
        self.rng = StdRng::seed_from_u64(rng.gen());
    }

    fn update(&mut self, _t: f64, dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width as usize;
        let h = self.height as usize;
        if w < 3 || h < 3 {
            return;
        }

        // Random raindrops
        self.drop_accum += dt * self.drop_freq;
        while self.drop_accum >= 1.0 {
            self.drop_accum -= 1.0;
            let dx = self.rng.gen_range(2..w - 2);
            let dy = self.rng.gen_range(2..h - 2);
            let strength = self.rng.gen_range(200.0..500.0);
            self.buf_current[dy * w + dx] = strength;
        }

        // Wave equation: new = avg_4_neighbors(current)*2 - previous, then * damping
        // Skip edge pixels
        let mut new_buf = vec![0.0; w * h];
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let idx = y * w + x;
                let avg = (self.buf_current[idx - 1]
                    + self.buf_current[idx + 1]
                    + self.buf_current[idx - w]
                    + self.buf_current[idx + w])
                    / 2.0
                    - self.buf_previous[idx];
                new_buf[idx] = avg * self.damping;
            }
        }

        // Swap buffers
        std::mem::swap(&mut self.buf_previous, &mut self.buf_current);
        self.buf_current = new_buf;

        // Render
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let wave = self.buf_current[idx];

                // Base water color
                let depth = 0.3 + (wave * 0.002).clamp(-0.2, 0.3);
                let r = (20.0 + wave.abs() * 0.1).clamp(0.0, 80.0) as u8;
                let g = (40.0 + wave * 0.2 + depth * 100.0).clamp(0.0, 180.0) as u8;
                let b = (120.0 + depth * 200.0 + wave * 0.3).clamp(0.0, 255.0) as u8;

                // Caustic highlights on wave peaks
                let caustic = if wave > 10.0 {
                    ((wave - 10.0) * 0.5).clamp(0.0, 80.0) as u8
                } else {
                    0
                };

                pixels[idx] = (
                    r.saturating_add(caustic),
                    g.saturating_add(caustic),
                    b.saturating_add(caustic / 2),
                );
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "damping".to_string(),
                min: 0.9,
                max: 0.999,
                value: self.damping,
            },
            ParamDesc {
                name: "drop_freq".to_string(),
                min: 0.5,
                max: 10.0,
                value: self.drop_freq,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "damping" => self.damping = value,
            "drop_freq" => self.drop_freq = value,
            _ => {}
        }
    }
}
