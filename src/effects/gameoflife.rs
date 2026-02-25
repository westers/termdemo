use crate::effect::{Effect, ParamDesc};
use rand::Rng;

pub struct GameOfLife {
    width: u32,
    height: u32,
    tick_rate: f64,
    seed_density: f64,
    cells: Vec<bool>,
    next_cells: Vec<bool>,
    age: Vec<u16>,
    tick_accum: f64,
}

impl GameOfLife {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            tick_rate: 10.0,
            seed_density: 0.3,
            cells: Vec::new(),
            next_cells: Vec::new(),
            age: Vec::new(),
            tick_accum: 0.0,
        }
    }

    fn seed(&mut self) {
        let mut rng = rand::thread_rng();
        let size = (self.width * self.height) as usize;
        self.cells = (0..size)
            .map(|_| rng.gen::<f64>() < self.seed_density)
            .collect();
        self.next_cells = vec![false; size];
        self.age = vec![0; size];
    }
}

impl Effect for GameOfLife {
    fn name(&self) -> &str {
        "GameOfLife"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.tick_accum = 0.0;
        self.seed();
    }

    fn update(&mut self, _t: f64, dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width as i32;
        let h = self.height as i32;
        if w == 0 || h == 0 {
            return;
        }

        let t_global = _t;

        // Tick at reduced rate, cap 5 ticks/frame
        self.tick_accum += dt * self.tick_rate;
        let ticks = (self.tick_accum as u32).min(5);
        self.tick_accum -= ticks as f64;

        for _ in 0..ticks {
            let mut alive_count = 0u32;

            for y in 0..h {
                for x in 0..w {
                    // Count neighbors with toroidal wrap
                    let mut neighbors = 0u8;
                    for dy in -1..=1i32 {
                        for dx in -1..=1i32 {
                            if dy == 0 && dx == 0 {
                                continue;
                            }
                            let nx = (x + dx).rem_euclid(w);
                            let ny = (y + dy).rem_euclid(h);
                            if self.cells[(ny * w + nx) as usize] {
                                neighbors += 1;
                            }
                        }
                    }

                    let idx = (y * w + x) as usize;
                    let alive = self.cells[idx];
                    let new_alive = if alive {
                        neighbors == 2 || neighbors == 3
                    } else {
                        neighbors == 3
                    };

                    self.next_cells[idx] = new_alive;

                    if new_alive {
                        self.age[idx] = self.age[idx].saturating_add(1);
                        alive_count += 1;
                    } else if self.age[idx] > 0 {
                        // Ghost: age decays for dead cells
                        self.age[idx] = self.age[idx].saturating_sub(3);
                    }
                }
            }

            std::mem::swap(&mut self.cells, &mut self.next_cells);

            // Auto-reseed at <5% population
            let total = (w * h) as u32;
            if total > 0 && alive_count * 100 / total < 5 {
                self.seed();
            }
        }

        // Render
        for i in 0..pixels.len().min(self.cells.len()) {
            if self.cells[i] {
                // Living cells: rainbow-cycle by age
                let hue = (self.age[i] as f64 * 0.01 + t_global * 0.1) % 1.0;
                let brightness = 0.7 + (self.age[i] as f64 * 0.003).min(0.3);
                pixels[i] = hsv_to_rgb(hue, 0.9, brightness);
            } else if self.age[i] > 0 {
                // Ghost: fade out
                let ghost = (self.age[i] as f64 / 30.0).clamp(0.0, 1.0);
                let v = (ghost * 40.0) as u8;
                pixels[i] = (v / 2, v / 3, v);
            } else {
                pixels[i] = (0, 0, 0);
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "tick_rate".to_string(),
                min: 2.0,
                max: 30.0,
                value: self.tick_rate,
            },
            ParamDesc {
                name: "seed_density".to_string(),
                min: 0.1,
                max: 0.6,
                value: self.seed_density,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "tick_rate" => self.tick_rate = value,
            "seed_density" => self.seed_density = value,
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
