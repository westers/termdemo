use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct Sierpinski {
    width: u32,
    height: u32,
    speed: f64,
    rotation: f64,
    buffer: Vec<(u8, u8, u8)>,
    current_x: f64,
    current_y: f64,
    lcg_state: u64,
    total_iterations: u64,
}

impl Sierpinski {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            rotation: 0.5,
            buffer: Vec::new(),
            current_x: 0.0,
            current_y: 0.0,
            lcg_state: 12345,
            total_iterations: 0,
        }
    }

    fn lcg_next(&mut self) -> u64 {
        self.lcg_state = self.lcg_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.lcg_state
    }
}

impl Effect for Sierpinski {
    fn name(&self) -> &str {
        "Sierpinski"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.buffer = vec![(0, 0, 0); (width * height) as usize];
        self.current_x = width as f64 / 2.0;
        self.current_y = height as f64 / 2.0;
        self.lcg_state = (width as u64) * 31337 + (height as u64) * 7919 + 42;
        self.total_iterations = 0;
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }
        let wf = w as f64;
        let hf = h as f64;

        // Compute rotated vertices of the triangle
        let cx = wf / 2.0;
        let cy = hf / 2.0;
        let radius = (wf.min(hf) * 0.45).min(wf * 0.48);
        let angle_offset = t * self.rotation * 0.5;

        let vertices = [
            (
                cx + radius * (angle_offset - PI / 2.0).cos(),
                cy + radius * (angle_offset - PI / 2.0).sin(),
            ),
            (
                cx + radius * (angle_offset + PI / 6.0 * 5.0).cos(),
                cy + radius * (angle_offset + PI / 6.0 * 5.0).sin(),
            ),
            (
                cx + radius * (angle_offset + PI / 6.0).cos(),
                cy + radius * (angle_offset + PI / 6.0).sin(),
            ),
        ];

        // Vertex colors
        let colors: [(u8, u8, u8); 3] = [
            (255, 60, 60),   // red
            (60, 255, 80),   // green
            (60, 100, 255),  // blue
        ];

        // Slightly dim existing buffer to create a gentle fade for old rotated positions
        if self.total_iterations > 50000 {
            for pixel in self.buffer.iter_mut() {
                pixel.0 = pixel.0.saturating_sub(1);
                pixel.1 = pixel.1.saturating_sub(1);
                pixel.2 = pixel.2.saturating_sub(1);
            }
        }

        // Run chaos game iterations
        let iters = (5000.0 * self.speed) as usize;
        for _ in 0..iters {
            let r = self.lcg_next();
            let vertex_idx = (r % 3) as usize;

            let vx = vertices[vertex_idx].0;
            let vy = vertices[vertex_idx].1;

            // Move halfway toward chosen vertex
            self.current_x = (self.current_x + vx) * 0.5;
            self.current_y = (self.current_y + vy) * 0.5;

            let ix = self.current_x as i32;
            let iy = self.current_y as i32;

            if ix >= 0 && ix < w as i32 && iy >= 0 && iy < h as i32 {
                let idx = (iy as u32 * w + ix as u32) as usize;
                let c = colors[vertex_idx];
                let old = self.buffer[idx];
                // Brighten toward the vertex color
                let blend = |o: u8, c: u8| -> u8 {
                    if c > o {
                        o.saturating_add(((c - o) as u16).min(60) as u8)
                    } else {
                        o
                    }
                };
                self.buffer[idx] = (
                    blend(old.0, c.0),
                    blend(old.1, c.1),
                    blend(old.2, c.2),
                );
            }

            self.total_iterations += 1;
        }

        // Copy buffer to pixels
        let len = pixels.len().min(self.buffer.len());
        pixels[..len].copy_from_slice(&self.buffer[..len]);
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
                name: "rotation".to_string(),
                min: 0.0,
                max: 2.0,
                value: self.rotation,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "rotation" => self.rotation = value,
            _ => {}
        }
    }
}
