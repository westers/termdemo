use crate::effect::{Effect, ParamDesc};

pub struct Glenz {
    width: u32,
    height: u32,
    rot_speed: f64,
    zoom: f64,
}

impl Glenz {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            rot_speed: 1.0,
            zoom: 1.0,
        }
    }
}

// Icosahedron geometry: 12 vertices, 20 triangular faces
fn icosahedron_vertices() -> Vec<[f64; 3]> {
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0; // golden ratio
    let a = 1.0;
    let b = phi;
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

struct ProjectedTri {
    verts: [(f64, f64); 3],
    depth: f64,
    face_idx: usize,
}

impl Effect for Glenz {
    fn name(&self) -> &str {
        "Glenz"
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

        // Dark background
        for p in pixels.iter_mut() {
            *p = (4, 4, 12);
        }

        let t = t * self.rot_speed;
        let ay = t * 0.6;
        let ax = t * 0.4;
        let az = t * 0.25;

        let cos_y = ay.cos();
        let sin_y = ay.sin();
        let cos_x = ax.cos();
        let sin_x = ax.sin();
        let cos_z = az.cos();
        let sin_z = az.sin();

        let camera_z = 6.0;
        let scale = self.zoom * cx.min(cy) * 0.45;

        let verts = icosahedron_vertices();
        let faces = icosahedron_faces();

        // Transform vertices
        let projected: Vec<(f64, f64, f64)> = verts
            .iter()
            .map(|v| {
                // Rotate Y
                let x1 = v[0] * cos_y + v[2] * sin_y;
                let z1 = -v[0] * sin_y + v[2] * cos_y;
                let y1 = v[1];
                // Rotate X
                let y2 = y1 * cos_x - z1 * sin_x;
                let z2 = y1 * sin_x + z1 * cos_x;
                // Rotate Z
                let x3 = x1 * cos_z - y2 * sin_z;
                let y3 = x1 * sin_z + y2 * cos_z;
                // Perspective
                let persp = camera_z / (camera_z + z2);
                (cx + x3 * scale * persp, cy + y3 * scale * persp, z2)
            })
            .collect();

        // Build projected triangles and sort back-to-front (painter's algorithm)
        let mut tris: Vec<ProjectedTri> = faces
            .iter()
            .enumerate()
            .map(|(fi, f)| {
                let v0 = projected[f[0]];
                let v1 = projected[f[1]];
                let v2 = projected[f[2]];
                let depth = (v0.2 + v1.2 + v2.2) / 3.0;
                ProjectedTri {
                    verts: [(v0.0, v0.1), (v1.0, v1.1), (v2.0, v2.1)],
                    depth,
                    face_idx: fi,
                }
            })
            .collect();

        // Sort back-to-front (largest depth = furthest = draw first)
        tris.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));

        // Draw each triangle with additive transparency
        for tri in &tris {
            // Face color based on index, cycling with time
            let hue = (tri.face_idx as f64 / 20.0 + t * 0.05) % 1.0;
            let (cr, cg, cb) = hsv_to_rgb(hue, 0.7, 0.8);

            // Transparency: scale color down, then add
            let alpha = 0.35;
            let ar = (cr as f64 * alpha) as u8;
            let ag = (cg as f64 * alpha) as u8;
            let ab = (cb as f64 * alpha) as u8;

            fill_triangle_additive(pixels, w, h, &tri.verts, (ar, ag, ab));
        }

        // Draw edges for wireframe outline
        for tri in &tris {
            let hue = (tri.face_idx as f64 / 20.0 + t * 0.05) % 1.0;
            let (cr, cg, cb) = hsv_to_rgb(hue, 0.5, 1.0);
            let edge_color = (cr / 2, cg / 2, cb / 2);
            for i in 0..3 {
                let j = (i + 1) % 3;
                draw_line_additive(
                    pixels,
                    w,
                    h,
                    tri.verts[i].0,
                    tri.verts[i].1,
                    tri.verts[j].0,
                    tri.verts[j].1,
                    edge_color,
                );
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "rot_speed".to_string(),
                min: 0.2,
                max: 4.0,
                value: self.rot_speed,
            },
            ParamDesc {
                name: "zoom".to_string(),
                min: 0.5,
                max: 3.0,
                value: self.zoom,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "rot_speed" => self.rot_speed = value,
            "zoom" => self.zoom = value,
            _ => {}
        }
    }
}

/// Rasterize a filled triangle with additive blending using scanline algorithm
fn fill_triangle_additive(
    pixels: &mut [(u8, u8, u8)],
    w: u32,
    h: u32,
    verts: &[(f64, f64); 3],
    color: (u8, u8, u8),
) {
    // Bounding box
    let min_y = verts[0].1.min(verts[1].1).min(verts[2].1).max(0.0) as i32;
    let max_y = verts[0].1.max(verts[1].1).max(verts[2].1).min(h as f64 - 1.0) as i32;
    let min_x = verts[0].0.min(verts[1].0).min(verts[2].0).max(0.0) as i32;
    let max_x = verts[0].0.max(verts[1].0).max(verts[2].0).min(w as f64 - 1.0) as i32;

    let v0 = verts[0];
    let v1 = verts[1];
    let v2 = verts[2];

    // Precompute edge function denominators
    let denom = (v1.1 - v2.1) * (v0.0 - v2.0) + (v2.0 - v1.0) * (v0.1 - v2.1);
    if denom.abs() < 0.001 {
        return; // degenerate triangle
    }
    let inv_denom = 1.0 / denom;

    for y in min_y..=max_y {
        let py = y as f64 + 0.5;
        for x in min_x..=max_x {
            let px = x as f64 + 0.5;

            // Barycentric coordinates
            let w0 = ((v1.1 - v2.1) * (px - v2.0) + (v2.0 - v1.0) * (py - v2.1)) * inv_denom;
            let w1 = ((v2.1 - v0.1) * (px - v2.0) + (v0.0 - v2.0) * (py - v2.1)) * inv_denom;
            let w2 = 1.0 - w0 - w1;

            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                let idx = (y as u32 * w + x as u32) as usize;
                if idx < pixels.len() {
                    let p = &mut pixels[idx];
                    p.0 = p.0.saturating_add(color.0);
                    p.1 = p.1.saturating_add(color.1);
                    p.2 = p.2.saturating_add(color.2);
                }
            }
        }
    }
}

fn draw_line_additive(
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
                let p = &mut pixels[idx];
                p.0 = p.0.saturating_add(color.0);
                p.1 = p.1.saturating_add(color.1);
                p.2 = p.2.saturating_add(color.2);
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
