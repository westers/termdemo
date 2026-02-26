use crate::effect::{Effect, ParamDesc};
use font8x8::UnicodeFonts;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

struct Column {
    head_y: f64,
    speed: f64,
    trail: Vec<u8>, // ASCII chars in trail
    active: bool,
}

pub struct Matrix {
    width: u32,
    height: u32,
    speed: f64,
    density: f64,
    columns: Vec<Column>,
    rng: StdRng,
}

impl Matrix {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            density: 0.6,
            columns: Vec::new(),
            rng: StdRng::seed_from_u64(0),
        }
    }

    fn init_columns(&mut self) {
        let num_cols = (self.width / 8).max(1);
        self.columns.clear();

        for _ in 0..num_cols {
            let trail_len = self.rng.gen_range(8..25);
            self.columns.push(Column {
                head_y: self.rng.gen_range(-(self.height as f64)..0.0),
                speed: self.rng.gen_range(40.0..120.0),
                trail: (0..trail_len)
                    .map(|_| self.rng.gen_range(33..127))
                    .collect(),
                active: self.rng.gen::<f64>() < self.density,
            });
        }
    }
}

impl Effect for Matrix {
    fn name(&self) -> &str {
        "Matrix"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.columns.clear();
    }

    fn randomize_init(&mut self, rng: &mut StdRng) {
        self.rng = StdRng::seed_from_u64(rng.gen());
        self.init_columns();
    }

    fn update(&mut self, _t: f64, dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        // Clear to black
        for p in pixels.iter_mut() {
            *p = (0, 0, 0);
        }

        let num_cols = self.columns.len();

        for col_idx in 0..num_cols {
            let col = &mut self.columns[col_idx];

            if !col.active {
                // Random activation based on density
                if self.rng.gen::<f64>() < self.density * dt * 2.0 {
                    col.active = true;
                    col.head_y = 0.0;
                    col.speed = self.rng.gen_range(40.0..120.0);
                }
                continue;
            }

            col.head_y += col.speed * self.speed * dt;

            // Occasional char mutation (2% per frame)
            if self.rng.gen::<f64>() < 0.02 {
                let idx = self.rng.gen_range(0..col.trail.len());
                col.trail[idx] = self.rng.gen_range(33..127);
            }

            let pixel_x = col_idx as u32 * 8;
            if pixel_x >= w {
                continue;
            }

            let trail_len = col.trail.len();

            for (ti, &ch) in col.trail.iter().enumerate() {
                let char_y = col.head_y as i32 - (ti as i32 * 8);

                if char_y < -8 || char_y >= h as i32 {
                    continue;
                }

                // Brightness: head char is brightest, fades down trail
                let fade = if ti == 0 {
                    1.0
                } else {
                    1.0 - (ti as f64 / trail_len as f64)
                };

                let (cr, cg, cb) = if ti == 0 {
                    // Head: bright white-green
                    (200, 255, 200)
                } else {
                    // Trail: fades to dark green
                    let g = (200.0 * fade) as u8;
                    let r = (30.0 * fade) as u8;
                    (r, g, 0)
                };

                // Render 8×8 glyph
                let glyph = font8x8::BASIC_FONTS
                    .get(ch as char)
                    .unwrap_or([0; 8]);

                for gy in 0..8u32 {
                    let py = char_y + gy as i32;
                    if py < 0 || py >= h as i32 {
                        continue;
                    }

                    let row_bits = glyph[gy as usize];
                    let row_start = (py as u32 * w) as usize;

                    for gx in 0..8u32 {
                        if row_bits & (1 << gx) != 0 {
                            let px = pixel_x + gx;
                            if px < w {
                                let idx = row_start + px as usize;
                                if idx < pixels.len() {
                                    let p = &mut pixels[idx];
                                    p.0 = p.0.max(cr);
                                    p.1 = p.1.max(cg);
                                    p.2 = p.2.max(cb);
                                }
                            }
                        }
                    }
                }
            }

            // Deactivate if fully off screen
            let tail_y = col.head_y as i32 - (trail_len as i32 * 8);
            if tail_y > h as i32 {
                col.active = false;
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "speed".to_string(),
                min: 0.3,
                max: 3.0,
                value: self.speed,
            },
            ParamDesc {
                name: "density".to_string(),
                min: 0.2,
                max: 1.0,
                value: self.density,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "density" => self.density = value,
            _ => {}
        }
    }
}
