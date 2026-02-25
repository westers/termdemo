use crate::effect::{Effect, ParamDesc};

pub struct Wireframe {
    width: u32,
    height: u32,
    rot_speed: f64,
    zoom: f64,
}

impl Wireframe {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            rot_speed: 1.0,
            zoom: 1.0,
        }
    }
}

// Unit cube vertices
const VERTICES: [[f64; 3]; 8] = [
    [-1.0, -1.0, -1.0],
    [ 1.0, -1.0, -1.0],
    [ 1.0,  1.0, -1.0],
    [-1.0,  1.0, -1.0],
    [-1.0, -1.0,  1.0],
    [ 1.0, -1.0,  1.0],
    [ 1.0,  1.0,  1.0],
    [-1.0,  1.0,  1.0],
];

// 12 edges as vertex index pairs
const EDGES: [(usize, usize); 12] = [
    (0, 1), (1, 2), (2, 3), (3, 0), // front face
    (4, 5), (5, 6), (6, 7), (7, 4), // back face
    (0, 4), (1, 5), (2, 6), (3, 7), // connecting
];

impl Effect for Wireframe {
    fn name(&self) -> &str {
        "Wireframe"
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

        // Dark background
        for p in pixels.iter_mut() {
            *p = (5, 5, 12);
        }

        let t_scaled = t * self.rot_speed;
        let angle_y = t_scaled * 0.7;
        let angle_x = t_scaled * 0.5;

        let cos_y = angle_y.cos();
        let sin_y = angle_y.sin();
        let cos_x = angle_x.cos();
        let sin_x = angle_x.sin();

        let camera_z = 5.0;
        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        let scale = self.zoom * cx.min(cy) * 0.6;

        // Transform and project vertices
        let mut projected = [(0.0f64, 0.0f64); 8];
        let mut depths = [0.0f64; 8];

        for (i, v) in VERTICES.iter().enumerate() {
            // Rotate Y then X
            let x1 = v[0] * cos_y + v[2] * sin_y;
            let z1 = -v[0] * sin_y + v[2] * cos_y;
            let y1 = v[1];

            let y2 = y1 * cos_x - z1 * sin_x;
            let z2 = y1 * sin_x + z1 * cos_x;

            // Perspective projection
            let persp = camera_z / (camera_z + z2);
            projected[i] = (cx + x1 * scale * persp, cy + y2 * scale * persp);
            depths[i] = z2;
        }

        // Draw edges
        for (ei, &(a, b)) in EDGES.iter().enumerate() {
            let (x0, y0) = projected[a];
            let (x1, y1) = projected[b];
            let avg_depth = (depths[a] + depths[b]) / 2.0;

            // HSV color per edge based on depth
            let hue = (ei as f64 / EDGES.len() as f64 + t * 0.1) % 1.0;
            let brightness = (0.5 + (1.0 - avg_depth / 3.0) * 0.5).clamp(0.3, 1.0);
            let color = hsv_to_rgb(hue, 0.8, brightness);

            // Main line
            draw_line(pixels, w, h, x0, y0, x1, y1, color);

            // Glow: offset lines at half brightness
            let glow = (
                color.0 / 2,
                color.1 / 2,
                color.2 / 2,
            );
            draw_line(pixels, w, h, x0 + 1.0, y0, x1 + 1.0, y1, glow);
            draw_line(pixels, w, h, x0, y0 + 1.0, x1, y1 + 1.0, glow);
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

/// Bresenham's line drawing algorithm
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
                let p = &mut pixels[idx];
                p.0 = p.0.max(color.0);
                p.1 = p.1.max(color.1);
                p.2 = p.2.max(color.2);
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
