use crate::effect::{Effect, ParamDesc};

const CLOTH_W: usize = 40;
const CLOTH_H: usize = 30;
const REST_DIST: f64 = 1.0;
const CONSTRAINT_ITERS: usize = 4;
const SUB_STEPS: usize = 3;

#[derive(Clone, Copy)]
struct Particle {
    x: f64,
    y: f64,
    z: f64,
    prev_x: f64,
    prev_y: f64,
    prev_z: f64,
    pinned: bool,
}

pub struct ClothSim {
    width: u32,
    height: u32,
    wind: f64,
    gravity: f64,
    particles: Vec<Particle>,
}

impl ClothSim {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            wind: 1.0,
            gravity: 1.0,
            particles: Vec::new(),
        }
    }

    fn particle_idx(cx: usize, cy: usize) -> usize {
        cy * CLOTH_W + cx
    }

    fn reset_cloth(&mut self) {
        self.particles.clear();
        self.particles.reserve(CLOTH_W * CLOTH_H);

        for cy in 0..CLOTH_H {
            for cx in 0..CLOTH_W {
                let x = (cx as f64 - CLOTH_W as f64 / 2.0) * REST_DIST;
                let y = (cy as f64 - CLOTH_H as f64 / 4.0) * REST_DIST;
                let z = 0.0;

                // Pin top-left and top-right corners, plus a few points along the top
                let pinned = cy == 0 && (cx == 0 || cx == CLOTH_W - 1 || cx == CLOTH_W / 3 || cx == 2 * CLOTH_W / 3);

                self.particles.push(Particle {
                    x,
                    y,
                    z,
                    prev_x: x,
                    prev_y: y,
                    prev_z: z,
                    pinned,
                });
            }
        }
    }

    fn simulate(&mut self, t: f64, dt: f64) {
        let sub_dt = dt / SUB_STEPS as f64;

        for _ in 0..SUB_STEPS {
            // Apply forces (gravity + wind)
            let wind_x = (t * 1.5).sin() * 8.0 * self.wind;
            let wind_z = (t * 0.9 + 1.0).cos() * 5.0 * self.wind + 3.0 * self.wind;
            let grav_y = 15.0 * self.gravity;

            // Verlet integration
            for p in self.particles.iter_mut() {
                if p.pinned {
                    continue;
                }

                let vx = p.x - p.prev_x;
                let vy = p.y - p.prev_y;
                let vz = p.z - p.prev_z;

                // Damping
                let damping = 0.98;

                let new_x = p.x + vx * damping + wind_x * sub_dt * sub_dt;
                let new_y = p.y + vy * damping + grav_y * sub_dt * sub_dt;
                let new_z = p.z + vz * damping + wind_z * sub_dt * sub_dt;

                p.prev_x = p.x;
                p.prev_y = p.y;
                p.prev_z = p.z;
                p.x = new_x;
                p.y = new_y;
                p.z = new_z;
            }

            // Satisfy distance constraints
            for _ in 0..CONSTRAINT_ITERS {
                // Horizontal constraints
                for cy in 0..CLOTH_H {
                    for cx in 0..CLOTH_W - 1 {
                        self.satisfy_constraint(
                            Self::particle_idx(cx, cy),
                            Self::particle_idx(cx + 1, cy),
                            REST_DIST,
                        );
                    }
                }
                // Vertical constraints
                for cy in 0..CLOTH_H - 1 {
                    for cx in 0..CLOTH_W {
                        self.satisfy_constraint(
                            Self::particle_idx(cx, cy),
                            Self::particle_idx(cx, cy + 1),
                            REST_DIST,
                        );
                    }
                }
            }
        }
    }

    fn satisfy_constraint(&mut self, i: usize, j: usize, rest: f64) {
        let pi = self.particles[i];
        let pj = self.particles[j];

        let dx = pj.x - pi.x;
        let dy = pj.y - pi.y;
        let dz = pj.z - pi.z;
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist < 0.0001 {
            return;
        }

        let diff = (rest - dist) / dist;
        let offset_x = dx * diff * 0.5;
        let offset_y = dy * diff * 0.5;
        let offset_z = dz * diff * 0.5;

        if !self.particles[i].pinned {
            self.particles[i].x -= offset_x;
            self.particles[i].y -= offset_y;
            self.particles[i].z -= offset_z;
        }
        if !self.particles[j].pinned {
            self.particles[j].x += offset_x;
            self.particles[j].y += offset_y;
            self.particles[j].z += offset_z;
        }
    }
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let h = ((h % 1.0) + 1.0) % 1.0;
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
    (
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    )
}

impl Effect for ClothSim {
    fn name(&self) -> &str {
        "Cloth Simulation"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.reset_cloth();
    }

    fn update(&mut self, t: f64, dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        // Clear background
        for y in 0..h {
            let yf = y as f64 / h as f64;
            let bg_r = (10.0 + yf * 15.0) as u8;
            let bg_g = (10.0 + yf * 10.0) as u8;
            let bg_b = (20.0 + yf * 20.0) as u8;
            let row = (y * w) as usize;
            for x in 0..w as usize {
                pixels[row + x] = (bg_r, bg_g, bg_b);
            }
        }

        // Clamp dt to avoid simulation explosion
        let sim_dt = dt.min(0.033);
        self.simulate(t, sim_dt);

        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        let scale = (cx.min(cy)) / (CLOTH_W as f64 * REST_DIST * 0.6);

        // Light direction (normalized)
        let light_x: f64 = -0.4;
        let light_y: f64 = -0.6;
        let light_z: f64 = -0.7;
        let light_len = (light_x * light_x + light_y * light_y + light_z * light_z).sqrt();
        let lx = light_x / light_len;
        let ly = light_y / light_len;
        let lz = light_z / light_len;

        // Z-buffer for proper depth handling
        let mut zbuf = vec![f64::MAX; (w * h) as usize];

        // Render each grid cell as a filled quad
        for cy_idx in 0..CLOTH_H - 1 {
            for cx_idx in 0..CLOTH_W - 1 {
                let p00 = self.particles[Self::particle_idx(cx_idx, cy_idx)];
                let p10 = self.particles[Self::particle_idx(cx_idx + 1, cy_idx)];
                let p01 = self.particles[Self::particle_idx(cx_idx, cy_idx + 1)];
                let p11 = self.particles[Self::particle_idx(cx_idx + 1, cy_idx + 1)];

                // Compute face normal from two triangle edges
                let e1x = p10.x - p00.x;
                let e1y = p10.y - p00.y;
                let e1z = p10.z - p00.z;
                let e2x = p01.x - p00.x;
                let e2y = p01.y - p00.y;
                let e2z = p01.z - p00.z;

                // Cross product
                let nx = e1y * e2z - e1z * e2y;
                let ny = e1z * e2x - e1x * e2z;
                let nz = e1x * e2y - e1y * e2x;
                let n_len = (nx * nx + ny * ny + nz * nz).sqrt();
                if n_len < 0.0001 {
                    continue;
                }
                let nx = nx / n_len;
                let ny = ny / n_len;
                let nz = nz / n_len;

                // Diffuse lighting (both sides of cloth)
                let ndotl = (nx * lx + ny * ly + nz * lz).abs();
                let diffuse = 0.2 + ndotl * 0.8;

                // Color based on cloth UV position + lighting
                let u = cx_idx as f64 / CLOTH_W as f64;
                let v = cy_idx as f64 / CLOTH_H as f64;
                // Checker pattern with smooth gradient
                let checker = ((cx_idx / 4 + cy_idx / 4) % 2) as f64;
                let hue = u * 0.3 + v * 0.15 + checker * 0.1 + 0.55;
                let sat = 0.5 + checker * 0.2;

                let (cr, cg, cb) = hsv_to_rgb(hue, sat, diffuse);

                // Project quad corners to screen
                let avg_z = (p00.z + p10.z + p01.z + p11.z) / 4.0;

                let screen = |p: &Particle| -> (f64, f64) {
                    (cx + p.x * scale, cy + p.y * scale)
                };

                let s00 = screen(&p00);
                let s10 = screen(&p10);
                let s01 = screen(&p01);
                let s11 = screen(&p11);

                // Rasterize the quad as two triangles
                fill_triangle_zbuf(
                    pixels,
                    &mut zbuf,
                    w,
                    h,
                    [s00, s10, s01],
                    avg_z,
                    (cr, cg, cb),
                );
                fill_triangle_zbuf(
                    pixels,
                    &mut zbuf,
                    w,
                    h,
                    [s10, s11, s01],
                    avg_z,
                    (cr, cg, cb),
                );
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "wind".to_string(),
                min: 0.0,
                max: 3.0,
                value: self.wind,
            },
            ParamDesc {
                name: "gravity".to_string(),
                min: 0.3,
                max: 2.0,
                value: self.gravity,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "wind" => self.wind = value,
            "gravity" => self.gravity = value,
            _ => {}
        }
    }
}

fn fill_triangle_zbuf(
    pixels: &mut [(u8, u8, u8)],
    zbuf: &mut [f64],
    w: u32,
    h: u32,
    verts: [(f64, f64); 3],
    z: f64,
    color: (u8, u8, u8),
) {
    let min_y = verts[0].1.min(verts[1].1).min(verts[2].1).max(0.0) as i32;
    let max_y = verts[0].1.max(verts[1].1).max(verts[2].1).min(h as f64 - 1.0) as i32;
    let min_x = verts[0].0.min(verts[1].0).min(verts[2].0).max(0.0) as i32;
    let max_x = verts[0].0.max(verts[1].0).max(verts[2].0).min(w as f64 - 1.0) as i32;

    let v0 = verts[0];
    let v1 = verts[1];
    let v2 = verts[2];

    let denom = (v1.1 - v2.1) * (v0.0 - v2.0) + (v2.0 - v1.0) * (v0.1 - v2.1);
    if denom.abs() < 0.001 {
        return;
    }
    let inv_denom = 1.0 / denom;

    for y in min_y..=max_y {
        let py = y as f64 + 0.5;
        for x in min_x..=max_x {
            let px = x as f64 + 0.5;

            let w0 = ((v1.1 - v2.1) * (px - v2.0) + (v2.0 - v1.0) * (py - v2.1)) * inv_denom;
            let w1 = ((v2.1 - v0.1) * (px - v2.0) + (v0.0 - v2.0) * (py - v2.1)) * inv_denom;
            let w2 = 1.0 - w0 - w1;

            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                let idx = (y as u32 * w + x as u32) as usize;
                if idx < pixels.len() && z < zbuf[idx] {
                    zbuf[idx] = z;
                    pixels[idx] = color;
                }
            }
        }
    }
}
