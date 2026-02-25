use crate::effect::{Effect, ParamDesc};

pub struct FilledVector {
    width: u32,
    height: u32,
    rot_speed: f64,
    scale: f64,
}

impl FilledVector {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            rot_speed: 1.0,
            scale: 1.0,
        }
    }
}

/// Icosahedron geometry: 12 vertices, 20 triangular faces
fn icosahedron_vertices() -> Vec<[f64; 3]> {
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let a = 1.0;
    let b = phi;
    // Normalize to unit sphere
    let len = (a * a + b * b).sqrt();
    let a = a / len;
    let b = b / len;
    vec![
        [-a,  b,  0.0], [ a,  b,  0.0], [-a, -b,  0.0], [ a, -b,  0.0],
        [ 0.0, -a,  b], [ 0.0,  a,  b], [ 0.0, -a, -b], [ 0.0,  a, -b],
        [ b,  0.0, -a], [ b,  0.0,  a], [-b,  0.0, -a], [-b,  0.0,  a],
    ]
}

fn icosahedron_faces() -> Vec<[usize; 3]> {
    vec![
        [0, 11, 5],  [0, 5, 1],   [0, 1, 7],   [0, 7, 10],  [0, 10, 11],
        [1, 5, 9],   [5, 11, 4],  [11, 10, 2], [10, 7, 6],  [7, 1, 8],
        [3, 9, 4],   [3, 4, 2],   [3, 2, 6],   [3, 6, 8],   [3, 8, 9],
        [4, 9, 5],   [2, 4, 11],  [6, 2, 10],  [8, 6, 7],   [9, 8, 1],
    ]
}

/// Compute face normal from 3D vertices (before projection)
fn face_normal(v0: &[f64; 3], v1: &[f64; 3], v2: &[f64; 3]) -> [f64; 3] {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let nx = e1[1] * e2[2] - e1[2] * e2[1];
    let ny = e1[2] * e2[0] - e1[0] * e2[2];
    let nz = e1[0] * e2[1] - e1[1] * e2[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 1.0];
    }
    [nx / len, ny / len, nz / len]
}

fn dot3(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

struct SortedFace {
    screen_verts: [(f64, f64); 3],
    depth: f64,
    color: (u8, u8, u8),
}

impl Effect for FilledVector {
    fn name(&self) -> &str {
        "FilledVector"
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

        let wf = w as f64;
        let hf = h as f64;
        let cx = wf / 2.0;
        let cy = hf / 2.0;

        // Dark background with subtle gradient
        for y in 0..h {
            let fy = y as f64 / hf;
            let bg_r = (5.0 + fy * 12.0) as u8;
            let bg_g = (3.0 + fy * 6.0) as u8;
            let bg_b = (15.0 + fy * 20.0) as u8;
            for x in 0..w {
                let idx = (y * w + x) as usize;
                pixels[idx] = (bg_r, bg_g, bg_b);
            }
        }

        let ts = t * self.rot_speed;
        let angle_y = ts * 0.7;
        let angle_x = ts * 0.5 + 0.3;

        let cos_y = angle_y.cos();
        let sin_y = angle_y.sin();
        let cos_x = angle_x.cos();
        let sin_x = angle_x.sin();

        let camera_z = 4.0;
        let proj_scale = self.scale * cx.min(cy) * 0.7;

        let verts = icosahedron_vertices();
        let faces = icosahedron_faces();

        // Light direction (normalized, from upper-left-front)
        let light_dir = {
            let lx: f64 = 0.4;
            let ly: f64 = -0.7;
            let lz: f64 = 0.6;
            let len = (lx * lx + ly * ly + lz * lz).sqrt();
            [lx / len, ly / len, lz / len]
        };

        // Transform all vertices
        let transformed: Vec<[f64; 3]> = verts
            .iter()
            .map(|v| {
                // Rotate Y
                let x1 = v[0] * cos_y + v[2] * sin_y;
                let z1 = -v[0] * sin_y + v[2] * cos_y;
                let y1 = v[1];
                // Rotate X
                let y2 = y1 * cos_x - z1 * sin_x;
                let z2 = y1 * sin_x + z1 * cos_x;
                [x1, y2, z2]
            })
            .collect();

        // Project vertices to screen
        let projected: Vec<(f64, f64)> = transformed
            .iter()
            .map(|v| {
                let persp = camera_z / (camera_z + v[2]);
                (cx + v[0] * proj_scale * persp, cy + v[1] * proj_scale * persp)
            })
            .collect();

        // 6 distinct hue values cycling with time
        let hues: [f64; 6] = [
            (0.0 / 6.0 + t * 0.05) % 1.0,
            (1.0 / 6.0 + t * 0.05) % 1.0,
            (2.0 / 6.0 + t * 0.05) % 1.0,
            (3.0 / 6.0 + t * 0.05) % 1.0,
            (4.0 / 6.0 + t * 0.05) % 1.0,
            (5.0 / 6.0 + t * 0.05) % 1.0,
        ];

        // Build sorted face list
        let mut sorted_faces: Vec<SortedFace> = Vec::with_capacity(faces.len());

        for (fi, face) in faces.iter().enumerate() {
            let v0 = &transformed[face[0]];
            let v1 = &transformed[face[1]];
            let v2 = &transformed[face[2]];

            let normal = face_normal(v0, v1, v2);

            // Back-face culling: skip faces pointing away from camera
            // Camera is at (0, 0, -camera_z), looking toward +Z
            // Face center
            let face_center = [
                (v0[0] + v1[0] + v2[0]) / 3.0,
                (v0[1] + v1[1] + v2[1]) / 3.0,
                (v0[2] + v1[2] + v2[2]) / 3.0,
            ];
            // View direction from face to camera
            let view_dir = [-face_center[0], -face_center[1], -camera_z - face_center[2]];
            let view_len = (view_dir[0] * view_dir[0]
                + view_dir[1] * view_dir[1]
                + view_dir[2] * view_dir[2])
            .sqrt();
            if view_len < 1e-10 {
                continue;
            }
            let view_dir_n = [
                view_dir[0] / view_len,
                view_dir[1] / view_len,
                view_dir[2] / view_len,
            ];

            if dot3(&normal, &view_dir_n) < 0.0 {
                continue;
            }

            let avg_z = face_center[2];

            // Lighting: diffuse shading
            let ndotl = dot3(&normal, &light_dir).max(0.0);
            let ambient = 0.2;
            let diffuse = ndotl * 0.8;
            let brightness = (ambient + diffuse).clamp(0.0, 1.0);

            // Face color based on hue cycling
            let hue = hues[fi % 6];
            let (cr, cg, cb) = hsv_to_rgb(hue, 0.75, brightness);

            sorted_faces.push(SortedFace {
                screen_verts: [
                    projected[face[0]],
                    projected[face[1]],
                    projected[face[2]],
                ],
                depth: avg_z,
                color: (cr, cg, cb),
            });
        }

        // Sort back-to-front (painter's algorithm): largest Z = furthest = draw first
        sorted_faces
            .sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));

        // Draw each face with flat shading using scanline fill
        for face in &sorted_faces {
            fill_triangle(pixels, w, h, &face.screen_verts, face.color);
        }

        // Draw edges over the filled faces for definition
        for face in &sorted_faces {
            let edge_color = (
                (face.color.0 as u16 * 3 / 4) as u8,
                (face.color.1 as u16 * 3 / 4) as u8,
                (face.color.2 as u16 * 3 / 4) as u8,
            );
            for i in 0..3 {
                let j = (i + 1) % 3;
                draw_line(
                    pixels,
                    w,
                    h,
                    face.screen_verts[i].0,
                    face.screen_verts[i].1,
                    face.screen_verts[j].0,
                    face.screen_verts[j].1,
                    edge_color,
                );
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "rot_speed".to_string(),
                min: 0.3,
                max: 3.0,
                value: self.rot_speed,
            },
            ParamDesc {
                name: "scale".to_string(),
                min: 0.5,
                max: 2.0,
                value: self.scale,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "rot_speed" => self.rot_speed = value,
            "scale" => self.scale = value,
            _ => {}
        }
    }
}

/// Scanline triangle fill with solid color (overwrites pixels)
fn fill_triangle(
    pixels: &mut [(u8, u8, u8)],
    w: u32,
    h: u32,
    verts: &[(f64, f64); 3],
    color: (u8, u8, u8),
) {
    let min_y = verts[0].1.min(verts[1].1).min(verts[2].1).max(0.0) as i32;
    let max_y = verts[0]
        .1
        .max(verts[1].1)
        .max(verts[2].1)
        .min(h as f64 - 1.0) as i32;

    let v0 = verts[0];
    let v1 = verts[1];
    let v2 = verts[2];

    // Precompute for barycentric coordinates
    let denom = (v1.1 - v2.1) * (v0.0 - v2.0) + (v2.0 - v1.0) * (v0.1 - v2.1);
    if denom.abs() < 0.001 {
        return;
    }
    let inv_denom = 1.0 / denom;

    for y in min_y..=max_y {
        let py = y as f64 + 0.5;

        // Find x range for this scanline using edge intersections
        let mut x_min = w as f64;
        let mut x_max = 0.0f64;

        let edges = [(v0, v1), (v1, v2), (v2, v0)];
        for &(ea, eb) in &edges {
            if (ea.1 <= py && eb.1 > py) || (eb.1 <= py && ea.1 > py) {
                let t = (py - ea.1) / (eb.1 - ea.1);
                let ix = ea.0 + t * (eb.0 - ea.0);
                x_min = x_min.min(ix);
                x_max = x_max.max(ix);
            }
        }

        let start_x = (x_min.floor() as i32).max(0);
        let end_x = (x_max.ceil() as i32).min(w as i32 - 1);

        for x in start_x..=end_x {
            let px = x as f64 + 0.5;

            // Barycentric test
            let w0 = ((v1.1 - v2.1) * (px - v2.0) + (v2.0 - v1.0) * (py - v2.1)) * inv_denom;
            let w1 = ((v2.1 - v0.1) * (px - v2.0) + (v0.0 - v2.0) * (py - v2.1)) * inv_denom;
            let w2 = 1.0 - w0 - w1;

            if w0 >= -0.001 && w1 >= -0.001 && w2 >= -0.001 {
                let idx = (y as u32 * w + x as u32) as usize;
                if idx < pixels.len() {
                    pixels[idx] = color;
                }
            }
        }
    }
}

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

    loop {
        if ix0 >= 0 && ix0 < w as i32 && iy0 >= 0 && iy0 < h as i32 {
            let idx = (iy0 as u32 * w + ix0 as u32) as usize;
            if idx < pixels.len() {
                pixels[idx] = color;
            }
        }
        if ix0 == ix1 && iy0 == iy1 {
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
    }
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let h = ((h % 1.0) + 1.0) % 1.0;
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let tv = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i % 6 {
        0 => (v, tv, p),
        1 => (q, v, p),
        2 => (p, v, tv),
        3 => (p, q, v),
        4 => (tv, p, v),
        _ => (v, p, q),
    };
    (
        (r * 255.0).clamp(0.0, 255.0) as u8,
        (g * 255.0).clamp(0.0, 255.0) as u8,
        (b * 255.0).clamp(0.0, 255.0) as u8,
    )
}
