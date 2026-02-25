use crate::effect::{Effect, ParamDesc};

/// Cell states for Brian's Brain automaton
#[derive(Clone, Copy, PartialEq)]
enum CellState {
    Off,
    On,
    Dying,
}

pub struct CellularAutomata {
    width: u32,
    height: u32,
    speed: f64,
    density: f64,
    grid: Vec<CellState>,
    next_grid: Vec<CellState>,
    tick_accum: f64,
}

impl CellularAutomata {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            density: 0.3,
            grid: Vec::new(),
            next_grid: Vec::new(),
            tick_accum: 0.0,
        }
    }

    fn seed(&mut self) {
        // Deterministic seed derived from dimensions
        let size = (self.width * self.height) as usize;
        let mut rng_state: u64 = self.width as u64 * 7919 + self.height as u64 * 6271;
        self.grid = (0..size)
            .map(|_| {
                rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let val = ((rng_state >> 33) as f64) / (u32::MAX as f64);
                if val < self.density {
                    CellState::On
                } else {
                    CellState::Off
                }
            })
            .collect();
        self.next_grid = vec![CellState::Off; size];
    }

    fn step(&mut self) {
        let w = self.width as i32;
        let h = self.height as i32;

        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize;
                let cell = self.grid[idx];

                self.next_grid[idx] = match cell {
                    CellState::On => CellState::Dying,
                    CellState::Dying => CellState::Off,
                    CellState::Off => {
                        // Count ON neighbors (Moore neighborhood)
                        let mut on_count = 0u8;
                        for dy in -1..=1_i32 {
                            for dx in -1..=1_i32 {
                                if dy == 0 && dx == 0 {
                                    continue;
                                }
                                let nx = (x + dx).rem_euclid(w);
                                let ny = (y + dy).rem_euclid(h);
                                if self.grid[(ny * w + nx) as usize] == CellState::On {
                                    on_count += 1;
                                }
                            }
                        }
                        if on_count == 2 {
                            CellState::On
                        } else {
                            CellState::Off
                        }
                    }
                };
            }
        }

        std::mem::swap(&mut self.grid, &mut self.next_grid);
    }
}

impl Effect for CellularAutomata {
    fn name(&self) -> &str {
        "CellularAutomata"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.tick_accum = 0.0;
        self.seed();
    }

    fn update(&mut self, _t: f64, dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        // Accumulate time and step at controlled rate
        self.tick_accum += dt * self.speed * 15.0;
        let steps = (self.tick_accum as u32).min(5);
        self.tick_accum -= steps as f64;

        for _ in 0..steps {
            self.step();
        }

        // Render
        for i in 0..pixels.len().min(self.grid.len()) {
            pixels[i] = match self.grid[i] {
                CellState::Off => (5, 5, 20),        // dark blue/black
                CellState::On => (200, 255, 255),     // bright white/cyan
                CellState::Dying => (230, 100, 30),   // orange/red
            };
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
                name: "density".to_string(),
                min: 0.1,
                max: 0.5,
                value: self.density,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "density" => {
                self.density = value;
                // Re-seed if density changes and we have dimensions
                if self.width > 0 && self.height > 0 {
                    self.seed();
                }
            }
            _ => {}
        }
    }
}
