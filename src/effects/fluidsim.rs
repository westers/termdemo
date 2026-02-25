use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct FluidSim {
    width: u32,
    height: u32,
    gw: usize,
    gh: usize,
    u_vel: Vec<f64>,
    v_vel: Vec<f64>,
    u_prev: Vec<f64>,
    v_prev: Vec<f64>,
    density: Vec<f64>,
    dens_prev: Vec<f64>,
    viscosity: f64,
    diffusion: f64,
}

fn set_bnd(gw: usize, gh: usize, b: i32, field: &mut [f64]) {
    if gw < 3 || gh < 3 {
        return;
    }

    for x in 1..gw - 1 {
        field[x] = if b == 2 {
            -field[gw + x]
        } else {
            field[gw + x]
        };
        field[(gh - 1) * gw + x] = if b == 2 {
            -field[(gh - 2) * gw + x]
        } else {
            field[(gh - 2) * gw + x]
        };
    }
    for y in 1..gh - 1 {
        field[y * gw] = if b == 1 {
            -field[y * gw + 1]
        } else {
            field[y * gw + 1]
        };
        field[y * gw + gw - 1] = if b == 1 {
            -field[y * gw + gw - 2]
        } else {
            field[y * gw + gw - 2]
        };
    }
    field[0] = 0.5 * (field[1] + field[gw]);
    field[gw - 1] = 0.5 * (field[gw - 2] + field[2 * gw - 1]);
    field[(gh - 1) * gw] = 0.5 * (field[(gh - 1) * gw + 1] + field[(gh - 2) * gw]);
    field[(gh - 1) * gw + gw - 1] =
        0.5 * (field[(gh - 1) * gw + gw - 2] + field[(gh - 2) * gw + gw - 1]);
}

fn diffuse(gw: usize, gh: usize, b: i32, x: &mut [f64], x0: &[f64], diff: f64, dt: f64) {
    if gw < 3 || gh < 3 {
        return;
    }
    let a = dt * diff * (gw - 2) as f64 * (gh - 2) as f64;
    let c = 1.0 + 4.0 * a;

    for _ in 0..4 {
        for y in 1..gh - 1 {
            for xi in 1..gw - 1 {
                let idx = y * gw + xi;
                x[idx] =
                    (x0[idx] + a * (x[idx - 1] + x[idx + 1] + x[idx - gw] + x[idx + gw])) / c;
            }
        }
        set_bnd(gw, gh, b, x);
    }
}

fn advect(
    gw: usize,
    gh: usize,
    b: i32,
    d: &mut [f64],
    d0: &[f64],
    u: &[f64],
    v: &[f64],
    dt: f64,
) {
    if gw < 3 || gh < 3 {
        return;
    }
    let dt_w = dt * (gw - 2) as f64;
    let dt_h = dt * (gh - 2) as f64;

    for y in 1..gh - 1 {
        for x in 1..gw - 1 {
            let idx = y * gw + x;
            let mut xx = x as f64 - dt_w * u[idx];
            let mut yy = y as f64 - dt_h * v[idx];

            xx = xx.clamp(0.5, gw as f64 - 1.5);
            yy = yy.clamp(0.5, gh as f64 - 1.5);

            let i0 = xx.floor() as usize;
            let j0 = yy.floor() as usize;
            let i1 = i0 + 1;
            let j1 = j0 + 1;
            let sx = xx - i0 as f64;
            let sy = yy - j0 as f64;

            d[idx] = (1.0 - sy) * ((1.0 - sx) * d0[j0 * gw + i0] + sx * d0[j0 * gw + i1])
                + sy * ((1.0 - sx) * d0[j1 * gw + i0] + sx * d0[j1 * gw + i1]);
        }
    }
    set_bnd(gw, gh, b, d);
}

fn project(gw: usize, gh: usize, u: &mut [f64], v: &mut [f64], p: &mut [f64], div: &mut [f64]) {
    if gw < 3 || gh < 3 {
        return;
    }
    let h_x = 1.0 / (gw - 2) as f64;

    for y in 1..gh - 1 {
        for x in 1..gw - 1 {
            let idx = y * gw + x;
            div[idx] = -0.5 * h_x * (u[idx + 1] - u[idx - 1] + v[idx + gw] - v[idx - gw]);
            p[idx] = 0.0;
        }
    }
    set_bnd(gw, gh, 0, div);
    set_bnd(gw, gh, 0, p);

    for _ in 0..4 {
        for y in 1..gh - 1 {
            for x in 1..gw - 1 {
                let idx = y * gw + x;
                p[idx] =
                    (div[idx] + p[idx - 1] + p[idx + 1] + p[idx - gw] + p[idx + gw]) / 4.0;
            }
        }
        set_bnd(gw, gh, 0, p);
    }

    let inv_h = 0.5 * (gw - 2) as f64;
    for y in 1..gh - 1 {
        for x in 1..gw - 1 {
            let idx = y * gw + x;
            u[idx] -= inv_h * (p[idx + 1] - p[idx - 1]);
            v[idx] -= inv_h * (p[idx + gw] - p[idx - gw]);
        }
    }
    set_bnd(gw, gh, 1, u);
    set_bnd(gw, gh, 2, v);
}

impl FluidSim {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            gw: 0,
            gh: 0,
            u_vel: Vec::new(),
            v_vel: Vec::new(),
            u_prev: Vec::new(),
            v_prev: Vec::new(),
            density: Vec::new(),
            dens_prev: Vec::new(),
            viscosity: 0.001,
            diffusion: 0.001,
        }
    }

    fn vel_step(&mut self, dt: f64) {
        let gw = self.gw;
        let gh = self.gh;
        let n = gw * gh;
        let visc = self.viscosity;

        // Add source (u_prev/v_prev hold forces)
        for i in 0..n {
            self.u_vel[i] += dt * self.u_prev[i];
            self.v_vel[i] += dt * self.v_prev[i];
        }

        // Diffuse
        let mut u_tmp = vec![0.0; n];
        let mut v_tmp = vec![0.0; n];
        std::mem::swap(&mut self.u_vel, &mut u_tmp);
        diffuse(gw, gh, 1, &mut self.u_vel, &u_tmp, visc, dt);

        std::mem::swap(&mut self.v_vel, &mut v_tmp);
        diffuse(gw, gh, 2, &mut self.v_vel, &v_tmp, visc, dt);

        // Project
        let mut p = vec![0.0; n];
        let mut div = vec![0.0; n];
        project(gw, gh, &mut self.u_vel, &mut self.v_vel, &mut p, &mut div);

        // Advect
        u_tmp.copy_from_slice(&self.u_vel);
        v_tmp.copy_from_slice(&self.v_vel);
        advect(gw, gh, 1, &mut self.u_vel, &u_tmp, &u_tmp, &v_tmp, dt);
        advect(gw, gh, 2, &mut self.v_vel, &v_tmp, &u_tmp, &v_tmp, dt);

        // Project again
        project(gw, gh, &mut self.u_vel, &mut self.v_vel, &mut p, &mut div);

        // Clear prev
        self.u_prev.iter_mut().for_each(|v| *v = 0.0);
        self.v_prev.iter_mut().for_each(|v| *v = 0.0);
    }

    fn dens_step(&mut self, dt: f64) {
        let gw = self.gw;
        let gh = self.gh;
        let n = gw * gh;
        let diff = self.diffusion;

        // Add source
        for i in 0..n {
            self.density[i] += dt * self.dens_prev[i];
        }

        // Diffuse
        let mut d_tmp = vec![0.0; n];
        std::mem::swap(&mut self.density, &mut d_tmp);
        diffuse(gw, gh, 0, &mut self.density, &d_tmp, diff, dt);

        // Advect
        d_tmp.copy_from_slice(&self.density);
        let u_snap: Vec<f64> = self.u_vel.clone();
        let v_snap: Vec<f64> = self.v_vel.clone();
        advect(gw, gh, 0, &mut self.density, &d_tmp, &u_snap, &v_snap, dt);

        // Clear prev
        self.dens_prev.iter_mut().for_each(|v| *v = 0.0);

        // Light decay
        for d in self.density.iter_mut() {
            *d *= 0.995;
        }
    }

    fn heat_color(val: f64) -> (u8, u8, u8) {
        let v = val.clamp(0.0, 1.0);
        // black -> deep blue -> magenta -> orange -> white
        let (r, g, b) = if v < 0.2 {
            let t = v / 0.2;
            (0.0, 0.0, t * 0.5)
        } else if v < 0.45 {
            let t = (v - 0.2) / 0.25;
            (t * 0.7, 0.0, 0.5 + t * 0.2)
        } else if v < 0.7 {
            let t = (v - 0.45) / 0.25;
            (0.7 + t * 0.3, t * 0.5, 0.7 - t * 0.5)
        } else {
            let t = (v - 0.7) / 0.3;
            (1.0, 0.5 + t * 0.5, 0.2 + t * 0.8)
        };
        (
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        )
    }
}

impl Effect for FluidSim {
    fn name(&self) -> &str {
        "Fluid Simulation"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.gw = (width / 4).max(8) as usize;
        self.gh = (height / 4).max(8) as usize;
        let n = self.gw * self.gh;
        self.u_vel = vec![0.0; n];
        self.v_vel = vec![0.0; n];
        self.u_prev = vec![0.0; n];
        self.v_prev = vec![0.0; n];
        self.density = vec![0.0; n];
        self.dens_prev = vec![0.0; n];
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 || self.gw < 3 || self.gh < 3 {
            return;
        }

        let gw = self.gw;
        let gh = self.gh;
        let sim_dt = 0.05;

        // Add rotating injection points
        let num_emitters = 3;
        for i in 0..num_emitters {
            let angle =
                t * (0.5 + i as f64 * 0.3) + i as f64 * PI * 2.0 / num_emitters as f64;
            let cx = gw as f64 * 0.5 + angle.cos() * gw as f64 * 0.2;
            let cy = gh as f64 * 0.5 + (angle * 0.7).sin() * gh as f64 * 0.2;
            let ix = (cx as usize).clamp(1, gw - 2);
            let iy = (cy as usize).clamp(1, gh - 2);

            // Force direction tangent to circle
            let fx = -(angle * 0.7).cos() * 20.0;
            let fy = angle.sin() * 20.0;

            for dy in 0..3_usize {
                for dx in 0..3_usize {
                    let xx = (ix + dx).saturating_sub(1);
                    let yy = (iy + dy).saturating_sub(1);
                    if xx > 0 && xx < gw - 1 && yy > 0 && yy < gh - 1 {
                        let idx = yy * gw + xx;
                        self.u_prev[idx] += fx;
                        self.v_prev[idx] += fy;
                        self.dens_prev[idx] += 50.0;
                    }
                }
            }
        }

        self.vel_step(sim_dt);
        self.dens_step(sim_dt);

        // Render with bilinear interpolation from coarse grid to pixels
        for y in 0..h {
            let gy_f = y as f64 / h as f64 * (gh - 1) as f64;
            let gy0 = (gy_f.floor() as usize).min(gh - 2);
            let gy1 = gy0 + 1;
            let fy = gy_f - gy0 as f64;

            for x in 0..w {
                let gx_f = x as f64 / w as f64 * (gw - 1) as f64;
                let gx0 = (gx_f.floor() as usize).min(gw - 2);
                let gx1 = gx0 + 1;
                let fx = gx_f - gx0 as f64;

                let d = (1.0 - fy)
                    * ((1.0 - fx) * self.density[gy0 * gw + gx0]
                        + fx * self.density[gy0 * gw + gx1])
                    + fy * ((1.0 - fx) * self.density[gy1 * gw + gx0]
                        + fx * self.density[gy1 * gw + gx1]);

                let idx = (y * w + x) as usize;
                pixels[idx] = Self::heat_color(d * 0.15);
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "viscosity".to_string(),
                min: 0.0001,
                max: 0.01,
                value: self.viscosity,
            },
            ParamDesc {
                name: "diffusion".to_string(),
                min: 0.0001,
                max: 0.01,
                value: self.diffusion,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "viscosity" => self.viscosity = value,
            "diffusion" => self.diffusion = value,
            _ => {}
        }
    }
}
