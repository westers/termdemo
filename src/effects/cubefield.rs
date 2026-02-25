use crate::effect::{Effect, ParamDesc};

const MAX_CUBES: usize = 80;
const FAR_Z: f64 = 40.0;
const NEAR_Z: f64 = 0.5;
const CUBE_SIZE: f64 = 0.4;
const CAMERA_FOV: f64 = 1.5;

pub struct CubeField {
    width: u32,
    height: u32,
    speed: f64,
    density: f64,
}

impl CubeField {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            density: 1.0,
        }
    }
}

/// Deterministic pseudo-random from a seed
fn hash_f64(seed: u64) -> f64 {
    let mut x = seed;
    x = x.wrapping_mul(0x517CC1B727220A95);
    x ^= x >> 32;
    x = x.wrapping_mul(0x6C62272E07BB0142);
    x ^= x >> 32;
    (x & 0x00FF_FFFF) as f64 / 0x0100_0000 as f64
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

/// Project a 3D point to 2D screen coordinates
/// Returns (screen_x, screen_y, z) or None if behind camera
fn project(
    x: f64,
    y: f64,
    z: f64,
    cx: f64,
    cy: f64,
    scale: f64,
) -> Option<(f64, f64, f64)> {
    if z < 0.1 {
        return None;
    }
    let persp = CAMERA_FOV / z;
    Some((cx + x * scale * persp, cy + y * scale * persp, z))
}

/// Draw a line in the pixel buffer
fn draw_line(
    pixels: &mut [(u8, u8, u8)],
    w: u32,
    h: u32,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    color: (u8, u8, u8),
) {
    let mut ix0 = x0 as i32;
    let mut iy0 = y0 as i32;
    let ix1 = x1 as i32;
    let iy1 = y1 as i32;

    let dx = (ix1 - ix0).abs();
    let dy = -(iy1 - iy0).abs();
    let sx = if ix0 < ix1 { 1 } else { -1 };
    let sy = if iy0 < iy1 { 1 } else { -1 };
    let mut err = dx + dy;

    let max_steps = dx.abs().max(dy.abs()) + 1;
    let mut steps = 0;

    loop {
        if ix0 >= 0 && ix0 < w as i32 && iy0 >= 0 && iy0 < h as i32 {
            let idx = (iy0 as u32 * w + ix0 as u32) as usize;
            if idx < pixels.len() {
                pixels[idx] = color;
            }
        }
        if (ix0 == ix1 && iy0 == iy1) || steps > max_steps {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            ix0 += sx;
        }
        if e2 <= dx {
            err += dx;
            iy0 += sy;
        }
        steps += 1;
    }
}

/// Fill a convex quad defined by 4 vertices (in screen space, ordered)
fn fill_quad(
    pixels: &mut [(u8, u8, u8)],
    w: u32,
    h: u32,
    verts: &[(f64, f64); 4],
    color: (u8, u8, u8),
) {
    // Bounding box
    let min_y = verts.iter().map(|v| v.1).fold(f64::MAX, f64::min).max(0.0) as i32;
    let max_y = verts.iter().map(|v| v.1).fold(f64::MIN, f64::max).min(h as f64 - 1.0) as i32;

    for y in min_y..=max_y {
        let py = y as f64 + 0.5;
        // Find x intersections with all 4 edges
        let mut x_min = f64::MAX;
        let mut x_max = f64::MIN;

        for i in 0..4 {
            let j = (i + 1) % 4;
            let (x0, y0) = verts[i];
            let (x1, y1) = verts[j];

            if (y0 <= py && y1 > py) || (y1 <= py && y0 > py) {
                let t = (py - y0) / (y1 - y0);
                let ix = x0 + t * (x1 - x0);
                x_min = x_min.min(ix);
                x_max = x_max.max(ix);
            }
        }

        if x_min > x_max {
            continue;
        }

        let sx = (x_min.max(0.0)) as u32;
        let ex = (x_max.min(w as f64 - 1.0)) as u32;
        let row = y as u32 * w;
        for x in sx..=ex {
            let idx = (row + x) as usize;
            if idx < pixels.len() {
                pixels[idx] = color;
            }
        }
    }
}

struct CubeData {
    center_z: f64,
    faces: Vec<([(f64, f64); 4], (u8, u8, u8))>,
    edges: Vec<((f64, f64), (f64, f64), (u8, u8, u8))>,
}

impl Effect for CubeField {
    fn name(&self) -> &str {
        "Cube Field"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        // Clear to dark background
        for p in pixels.iter_mut() {
            *p = (3, 3, 8);
        }

        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        let scale = cx.min(cy);
        let t_speed = t * self.speed;

        // Camera weaves left/right
        let cam_x = (t_speed * 0.3).sin() * 2.0;
        let cam_y = (t_speed * 0.2).cos() * 0.5 - 0.3;
        let cam_z = t_speed * 5.0;

        let num_cubes = (MAX_CUBES as f64 * self.density) as usize;

        let mut cube_data: Vec<CubeData> = Vec::with_capacity(num_cubes);

        for i in 0..num_cubes {
            let seed = i as u64;
            // Deterministic position in world space
            let wx = (hash_f64(seed * 3 + 1) - 0.5) * 12.0;
            let wy = (hash_f64(seed * 3 + 2) - 0.5) * 6.0;
            let wz_base = hash_f64(seed * 3 + 3) * FAR_Z;

            // Repeat cubes along Z
            let wz_rel = ((wz_base - cam_z) % FAR_Z + FAR_Z) % FAR_Z + NEAR_Z;

            let rx = wx - cam_x;
            let ry = wy - cam_y;
            let rz = wz_rel;

            if rz < NEAR_Z || rz > FAR_Z {
                continue;
            }

            // Depth fog
            let fog = (1.0 - (rz / FAR_Z)).clamp(0.0, 1.0);
            let fog = fog * fog;

            if fog < 0.02 {
                continue;
            }

            // Cube color
            let hue = hash_f64(seed * 7 + 100);
            let (base_r, base_g, base_b) = hsv_to_rgb(hue, 0.7, 0.9);

            // 8 vertices of a cube centered at (rx, ry, rz)
            let s = CUBE_SIZE;
            let corners = [
                (rx - s, ry - s, rz - s),
                (rx + s, ry - s, rz - s),
                (rx + s, ry + s, rz - s),
                (rx - s, ry + s, rz - s),
                (rx - s, ry - s, rz + s),
                (rx + s, ry - s, rz + s),
                (rx + s, ry + s, rz + s),
                (rx - s, ry + s, rz + s),
            ];

            // Project all corners
            let mut proj: [(f64, f64); 8] = [(0.0, 0.0); 8];
            let mut all_visible = true;
            for (ci, c) in corners.iter().enumerate() {
                if let Some((sx, sy, _)) = project(c.0, c.1, c.2, cx, cy, scale) {
                    proj[ci] = (sx, sy);
                } else {
                    all_visible = false;
                    break;
                }
            }

            if !all_visible {
                continue;
            }

            // Define the 6 faces as indices into corners
            // We'll only draw the 3 faces most visible to the viewer
            let face_defs: [(usize, usize, usize, usize, f64, f64, f64); 6] = [
                (0, 1, 2, 3, 0.0, 0.0, -1.0), // front (-z)
                (5, 4, 7, 6, 0.0, 0.0, 1.0),  // back (+z)
                (4, 0, 3, 7, -1.0, 0.0, 0.0), // left (-x)
                (1, 5, 6, 2, 1.0, 0.0, 0.0),  // right (+x)
                (4, 5, 1, 0, 0.0, -1.0, 0.0), // top (-y)
                (3, 2, 6, 7, 0.0, 1.0, 0.0),  // bottom (+y)
            ];

            let mut faces = Vec::new();
            let mut edges = Vec::new();

            for (a, b, c, d, nx, ny, nz) in &face_defs {
                // Simple facing check: dot product of face normal with view direction
                // View direction is roughly (rx, ry, rz) normalized
                let view_dot = nx * rx + ny * ry + nz * rz;
                if view_dot >= 0.0 {
                    continue; // face pointing away
                }

                // Shade based on normal direction (simple directional light from upper-left)
                let light_dot = (nx * (-0.5) + ny * (-0.7) + nz * (-0.3)).abs();
                let shade = 0.4 + light_dot * 0.6;

                let fr = (base_r as f64 * shade * fog).clamp(0.0, 255.0) as u8;
                let fg = (base_g as f64 * shade * fog).clamp(0.0, 255.0) as u8;
                let fb = (base_b as f64 * shade * fog).clamp(0.0, 255.0) as u8;

                let quad = [proj[*a], proj[*b], proj[*c], proj[*d]];
                faces.push((quad, (fr, fg, fb)));

                // Edges for this face
                let edge_shade = (fog * 0.6).clamp(0.0, 1.0);
                let er = (base_r as f64 * edge_shade).clamp(0.0, 255.0) as u8;
                let eg = (base_g as f64 * edge_shade).clamp(0.0, 255.0) as u8;
                let eb = (base_b as f64 * edge_shade).clamp(0.0, 255.0) as u8;
                let edge_color = (er, eg, eb);

                let idxs = [*a, *b, *c, *d];
                for ei in 0..4 {
                    let ej = (ei + 1) % 4;
                    edges.push((proj[idxs[ei]], proj[idxs[ej]], edge_color));
                }
            }

            cube_data.push(CubeData {
                center_z: rz,
                faces,
                edges,
            });
        }

        // Sort cubes back-to-front
        cube_data.sort_by(|a, b| {
            b.center_z
                .partial_cmp(&a.center_z)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Draw all cubes
        for cube in &cube_data {
            for (quad, color) in &cube.faces {
                fill_quad(pixels, w, h, quad, *color);
            }
            for (p0, p1, color) in &cube.edges {
                draw_line(pixels, w, h, p0.0, p0.1, p1.0, p1.1, *color);
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
                min: 0.5,
                max: 3.0,
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
