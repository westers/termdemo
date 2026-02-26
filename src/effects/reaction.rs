use crate::effect::{Effect, ParamDesc};
use rand::rngs::StdRng;
use rand::Rng;

pub struct ReactionDiffusion {
    width: u32,
    height: u32,
    grid_w: usize,
    grid_h: usize,
    u_grid: Vec<f64>,
    v_grid: Vec<f64>,
    feed_rate: f64,
    kill_rate: f64,
}

impl ReactionDiffusion {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            grid_w: 0,
            grid_h: 0,
            u_grid: Vec::new(),
            v_grid: Vec::new(),
            feed_rate: 0.035,
            kill_rate: 0.065,
        }
    }

    fn init_grids(&mut self, rng: &mut StdRng) {
        let n = self.grid_w * self.grid_h;
        self.u_grid = vec![1.0; n];
        self.v_grid = vec![0.0; n];

        // Place seed spots of V
        let gw = self.grid_w;
        let gh = self.grid_h;
        let num_seeds = 8 + (gw * gh / 2000).min(20);
        for _ in 0..num_seeds {
            let cx = rng.gen_range(0..gw);
            let cy = rng.gen_range(0..gh);
            let radius = 3_usize;
            for dy in 0..=(radius * 2) {
                for dx in 0..=(radius * 2) {
                    let sx = (cx + dx).wrapping_sub(radius);
                    let sy = (cy + dy).wrapping_sub(radius);
                    if sx < self.grid_w && sy < self.grid_h {
                        let dist_sq = (dx as f64 - radius as f64).powi(2)
                            + (dy as f64 - radius as f64).powi(2);
                        if dist_sq <= (radius as f64).powi(2) {
                            let idx = sy * self.grid_w + sx;
                            self.v_grid[idx] = 1.0;
                            self.u_grid[idx] = 0.5;
                        }
                    }
                }
            }
        }
    }

    fn step(&mut self) {
        let gw = self.grid_w;
        let gh = self.grid_h;
        if gw < 3 || gh < 3 {
            return;
        }

        let du = 0.21;
        let dv = 0.105;
        let dt = 1.0;
        let f = self.feed_rate;
        let k = self.kill_rate;

        let n = gw * gh;
        let mut new_u = vec![0.0_f64; n];
        let mut new_v = vec![0.0_f64; n];

        for y in 0..gh {
            let ym = if y == 0 { gh - 1 } else { y - 1 };
            let yp = if y == gh - 1 { 0 } else { y + 1 };
            for x in 0..gw {
                let xm = if x == 0 { gw - 1 } else { x - 1 };
                let xp = if x == gw - 1 { 0 } else { x + 1 };

                let idx = y * gw + x;
                let u_c = self.u_grid[idx];
                let v_c = self.v_grid[idx];

                // 3x3 Laplacian with center weight -1, adjacent 0.2, diagonal 0.05
                let lap_u = self.u_grid[ym * gw + x] * 0.2
                    + self.u_grid[yp * gw + x] * 0.2
                    + self.u_grid[y * gw + xm] * 0.2
                    + self.u_grid[y * gw + xp] * 0.2
                    + self.u_grid[ym * gw + xm] * 0.05
                    + self.u_grid[ym * gw + xp] * 0.05
                    + self.u_grid[yp * gw + xm] * 0.05
                    + self.u_grid[yp * gw + xp] * 0.05
                    - u_c;

                let lap_v = self.v_grid[ym * gw + x] * 0.2
                    + self.v_grid[yp * gw + x] * 0.2
                    + self.v_grid[y * gw + xm] * 0.2
                    + self.v_grid[y * gw + xp] * 0.2
                    + self.v_grid[ym * gw + xm] * 0.05
                    + self.v_grid[ym * gw + xp] * 0.05
                    + self.v_grid[yp * gw + xm] * 0.05
                    + self.v_grid[yp * gw + xp] * 0.05
                    - v_c;

                let uvv = u_c * v_c * v_c;
                new_u[idx] = u_c + (du * lap_u - uvv + f * (1.0 - u_c)) * dt;
                new_v[idx] = v_c + (dv * lap_v + uvv - (f + k) * v_c) * dt;

                new_u[idx] = new_u[idx].clamp(0.0, 1.0);
                new_v[idx] = new_v[idx].clamp(0.0, 1.0);
            }
        }

        self.u_grid = new_u;
        self.v_grid = new_v;
    }
}

impl Effect for ReactionDiffusion {
    fn name(&self) -> &str {
        "Reaction-Diffusion"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.grid_w = (width / 2).max(2) as usize;
        self.grid_h = (height / 2).max(2) as usize;
        let n = self.grid_w * self.grid_h;
        self.u_grid = vec![1.0; n];
        self.v_grid = vec![0.0; n];
    }

    fn randomize_init(&mut self, rng: &mut StdRng) {
        self.init_grids(rng);
    }

    fn update(&mut self, _t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 || self.grid_w == 0 || self.grid_h == 0 {
            return;
        }

        // Run multiple simulation steps per frame for faster evolution
        for _ in 0..8 {
            self.step();
        }

        let gw = self.grid_w;
        let gh = self.grid_h;

        // Render: map V concentration to color
        for y in 0..h {
            let gy = (y as usize * gh) / h as usize;
            let gy = gy.min(gh - 1);
            for x in 0..w {
                let gx = (x as usize * gw) / w as usize;
                let gx = gx.min(gw - 1);
                let v = self.v_grid[gy * gw + gx];
                let u_val = self.u_grid[gy * gw + gx];

                // Color mapping: dark blue -> teal -> white based on V
                let (r, g, b) = if v < 0.15 {
                    // Low V: dark blue, modulated slightly by U
                    let base = v / 0.15;
                    let dim = u_val * 0.3;
                    (
                        (10.0 + dim * 20.0) as u8,
                        (10.0 + base * 40.0 + dim * 15.0) as u8,
                        (30.0 + base * 80.0 + dim * 30.0) as u8,
                    )
                } else if v < 0.4 {
                    // Mid V: teal
                    let t = (v - 0.15) / 0.25;
                    (
                        (10.0 + t * 60.0) as u8,
                        (50.0 + t * 120.0) as u8,
                        (110.0 + t * 60.0) as u8,
                    )
                } else {
                    // High V: bright white
                    let t = ((v - 0.4) / 0.6).min(1.0);
                    (
                        (70.0 + t * 185.0) as u8,
                        (170.0 + t * 85.0) as u8,
                        (170.0 + t * 85.0) as u8,
                    )
                };

                let idx = (y * w + x) as usize;
                pixels[idx] = (r, g, b);
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "feed_rate".to_string(),
                min: 0.02,
                max: 0.06,
                value: self.feed_rate,
            },
            ParamDesc {
                name: "kill_rate".to_string(),
                min: 0.05,
                max: 0.075,
                value: self.kill_rate,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "feed_rate" => self.feed_rate = value,
            "kill_rate" => self.kill_rate = value,
            _ => {}
        }
    }
}
