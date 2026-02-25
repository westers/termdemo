use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

const NUM_POINTS: usize = 500;
const NUM_SHAPES: usize = 3;
const TRANSITION_TIME: f64 = 3.0;
const HOLD_TIME: f64 = 2.0;
const CYCLE_TIME: f64 = TRANSITION_TIME + HOLD_TIME;

pub struct Morph {
    width: u32,
    height: u32,
    speed: f64,
    point_size: f64,
    shapes: Vec<Vec<[f64; 3]>>,
}

impl Morph {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            point_size: 1.0,
            shapes: Vec::new(),
        }
    }

    fn generate_shapes() -> Vec<Vec<[f64; 3]>> {
        let mut shapes = Vec::with_capacity(NUM_SHAPES);

        // Shape 0: Sphere
        shapes.push(generate_sphere(NUM_POINTS));

        // Shape 1: Cube
        shapes.push(generate_cube(NUM_POINTS));

        // Shape 2: Torus
        shapes.push(generate_torus(NUM_POINTS));

        shapes
    }
}

/// Fibonacci sphere distribution for even point placement
fn generate_sphere(count: usize) -> Vec<[f64; 3]> {
    let golden_ratio = (1.0 + 5.0_f64.sqrt()) / 2.0;
    let n = count as f64;
    (0..count)
        .map(|i| {
            let theta = 2.0 * PI * i as f64 / golden_ratio;
            let phi = (1.0 - 2.0 * (i as f64 + 0.5) / n).acos();
            [
                phi.sin() * theta.cos(),
                phi.sin() * theta.sin(),
                phi.cos(),
            ]
        })
        .collect()
}

/// Points distributed on cube faces
fn generate_cube(count: usize) -> Vec<[f64; 3]> {
    let per_face = count / 6;
    let mut points = Vec::with_capacity(count);
    let side = (per_face as f64).sqrt().ceil() as usize;

    for face in 0..6 {
        for i in 0..per_face {
            if points.len() >= count {
                break;
            }
            let row = i / side;
            let col = i % side;
            let u = (col as f64 / side as f64) * 2.0 - 1.0;
            let v = (row as f64 / side as f64) * 2.0 - 1.0;
            let p = match face {
                0 => [u, v, 1.0],
                1 => [u, v, -1.0],
                2 => [1.0, u, v],
                3 => [-1.0, u, v],
                4 => [u, 1.0, v],
                _ => [u, -1.0, v],
            };
            points.push(p);
        }
    }
    // Fill remaining with last face
    while points.len() < count {
        let i = points.len();
        let row = i / side;
        let col = i % side;
        let u = (col as f64 / side as f64) * 2.0 - 1.0;
        let v = (row as f64 / side as f64) * 2.0 - 1.0;
        points.push([u, v, 1.0]);
    }
    points
}

/// Points distributed on a torus surface
fn generate_torus(count: usize) -> Vec<[f64; 3]> {
    let r_major = 0.7; // distance from center of torus to center of tube
    let r_minor = 0.35; // radius of the tube

    (0..count)
        .map(|i| {
            // Use golden angle distribution for even spacing on torus
            let golden_ratio = (1.0 + 5.0_f64.sqrt()) / 2.0;
            let theta = 2.0 * PI * i as f64 / golden_ratio; // around the tube
            let phi = 2.0 * PI * i as f64 / count as f64 * golden_ratio; // around the ring

            let x = (r_major + r_minor * theta.cos()) * phi.cos();
            let y = r_minor * theta.sin();
            let z = (r_major + r_minor * theta.cos()) * phi.sin();
            [x, y, z]
        })
        .collect()
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Smooth step function for pleasing transitions
fn smoothstep(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl Effect for Morph {
    fn name(&self) -> &str {
        "Morph"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.shapes = Self::generate_shapes();
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 || self.shapes.is_empty() {
            return;
        }

        let wf = w as f64;
        let hf = h as f64;
        let cx = wf / 2.0;
        let cy = hf / 2.0;

        // Dark background with subtle vertical gradient
        for y in 0..h {
            let fy = y as f64 / hf;
            let bg_r = (4.0 + fy * 10.0) as u8;
            let bg_g = (2.0 + fy * 6.0) as u8;
            let bg_b = (12.0 + fy * 18.0) as u8;
            for x in 0..w {
                let idx = (y * w + x) as usize;
                pixels[idx] = (bg_r, bg_g, bg_b);
            }
        }

        let ts = t * self.speed;
        let total_cycle = CYCLE_TIME * NUM_SHAPES as f64;
        let cycle_pos = ts % total_cycle;

        // Determine which shape we're on and the transition progress
        let shape_cycle = cycle_pos / CYCLE_TIME;
        let current_shape = shape_cycle.floor() as usize % NUM_SHAPES;
        let next_shape = (current_shape + 1) % NUM_SHAPES;
        let time_in_cycle = cycle_pos - current_shape as f64 * CYCLE_TIME;

        let morph_t = if time_in_cycle < HOLD_TIME {
            0.0 // holding current shape
        } else {
            smoothstep((time_in_cycle - HOLD_TIME) / TRANSITION_TIME)
        };

        // Rotation
        let rot_y = ts * 0.4;
        let rot_x = ts * 0.25;
        let cos_ry = rot_y.cos();
        let sin_ry = rot_y.sin();
        let cos_rx = rot_x.cos();
        let sin_rx = rot_x.sin();

        let camera_z = 3.5;
        let proj_scale = cx.min(cy) * 0.65;

        let shape_a = &self.shapes[current_shape];
        let shape_b = &self.shapes[next_shape];
        let point_radius = self.point_size;

        for i in 0..NUM_POINTS {
            let a = shape_a[i];
            let b = shape_b[i];

            // Interpolate position
            let px = lerp(a[0], b[0], morph_t);
            let py = lerp(a[1], b[1], morph_t);
            let pz = lerp(a[2], b[2], morph_t);

            // Rotate Y
            let x1 = px * cos_ry + pz * sin_ry;
            let z1 = -px * sin_ry + pz * cos_ry;
            let y1 = py;

            // Rotate X
            let y2 = y1 * cos_rx - z1 * sin_rx;
            let z2 = y1 * sin_rx + z1 * cos_rx;

            // Perspective projection
            let persp = camera_z / (camera_z + z2);
            let sx = cx + x1 * proj_scale * persp;
            let sy = cy + y2 * proj_scale * persp;

            // Color based on original 3D position (creates a nice spatial color mapping)
            let hue = ((px * 0.3 + py * 0.3 + pz * 0.3 + 0.5 + ts * 0.05) % 1.0 + 1.0) % 1.0;
            let depth_brightness = (0.4 + (z2 + 1.0) * 0.4).clamp(0.3, 1.0);
            let (cr, cg, cb) = hsv_to_rgb(hue, 0.8, depth_brightness);

            // Draw point with size based on depth and point_size param
            let dot_size = (point_radius * persp * 1.2).max(0.5);
            let half = dot_size.ceil() as i32;

            for dy in -half..=half {
                for dx in -half..=half {
                    let dist_sq = (dx * dx + dy * dy) as f64;
                    if dist_sq <= dot_size * dot_size {
                        let draw_x = sx as i32 + dx;
                        let draw_y = sy as i32 + dy;
                        if draw_x >= 0
                            && draw_x < w as i32
                            && draw_y >= 0
                            && draw_y < h as i32
                        {
                            let idx = (draw_y as u32 * w + draw_x as u32) as usize;
                            if idx < pixels.len() {
                                let p = &mut pixels[idx];
                                // Additive-like blending for glow
                                p.0 = p.0.max(cr);
                                p.1 = p.1.max(cg);
                                p.2 = p.2.max(cb);
                            }
                        }
                    }
                }
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "speed".to_string(),
                min: 0.3,
                max: 2.0,
                value: self.speed,
            },
            ParamDesc {
                name: "point_size".to_string(),
                min: 0.5,
                max: 2.0,
                value: self.point_size,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "point_size" => self.point_size = value,
            _ => {}
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
