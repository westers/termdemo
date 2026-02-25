use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct LSystem {
    width: u32,
    height: u32,
    wind: f64,
    generations: f64,
}

struct TurtleState {
    x: f64,
    y: f64,
    angle: f64,
    depth: u32,
}

impl LSystem {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            wind: 0.5,
            generations: 4.0,
        }
    }

    /// Generate L-system string for given number of generations.
    fn generate_string(gens: u32) -> Vec<u8> {
        // Axiom: "F"
        // Rule: F -> FF+[+F-F-F]-[-F+F+F]
        let mut current: Vec<u8> = vec![b'F'];
        for _ in 0..gens {
            let mut next = Vec::with_capacity(current.len() * 6);
            for &ch in &current {
                if ch == b'F' {
                    next.extend_from_slice(b"FF+[+F-F-F]-[-F+F+F]");
                } else {
                    next.push(ch);
                }
            }
            current = next;
        }
        current
    }

    /// Draw a line into the pixel buffer with alpha blending.
    fn draw_line(
        pixels: &mut [(u8, u8, u8)],
        w: usize,
        h: usize,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        color: (u8, u8, u8),
        thickness: f64,
    ) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.5 {
            return;
        }

        let steps = (len * 2.0) as i32 + 1;
        for s in 0..=steps {
            let t = s as f64 / steps as f64;
            let cx = x0 + dx * t;
            let cy = y0 + dy * t;

            let half_t = (thickness * 0.5).ceil() as i32;
            for oy in -half_t..=half_t {
                for ox in -half_t..=half_t {
                    let px = (cx + ox as f64) as i32;
                    let py = (cy + oy as f64) as i32;
                    if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                        let dist = ((ox * ox + oy * oy) as f64).sqrt();
                        if dist <= thickness * 0.5 + 0.5 {
                            let alpha = (1.0 - (dist - thickness * 0.5).max(0.0)).clamp(0.0, 1.0);
                            let idx = py as usize * w + px as usize;
                            let (pr, pg, pb) = pixels[idx];
                            let r = pr as f64 * (1.0 - alpha) + color.0 as f64 * alpha;
                            let g = pg as f64 * (1.0 - alpha) + color.1 as f64 * alpha;
                            let b = pb as f64 * (1.0 - alpha) + color.2 as f64 * alpha;
                            pixels[idx] = (r as u8, g as u8, b as u8);
                        }
                    }
                }
            }
        }
    }

    /// Draw a small dot (leaf) at position.
    fn draw_leaf(
        pixels: &mut [(u8, u8, u8)],
        w: usize,
        h: usize,
        cx: f64,
        cy: f64,
        radius: f64,
        color: (u8, u8, u8),
    ) {
        let r_int = radius.ceil() as i32;
        for oy in -r_int..=r_int {
            for ox in -r_int..=r_int {
                let px = (cx + ox as f64) as i32;
                let py = (cy + oy as f64) as i32;
                if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                    let dist = ((ox * ox + oy * oy) as f64).sqrt();
                    if dist <= radius {
                        let alpha = (1.0 - dist / radius).clamp(0.0, 1.0) * 0.8;
                        let idx = py as usize * w + px as usize;
                        let (pr, pg, pb) = pixels[idx];
                        let r = pr as f64 * (1.0 - alpha) + color.0 as f64 * alpha;
                        let g = pg as f64 * (1.0 - alpha) + color.1 as f64 * alpha;
                        let b = pb as f64 * (1.0 - alpha) + color.2 as f64 * alpha;
                        pixels[idx] = (r as u8, g as u8, b as u8);
                    }
                }
            }
        }
    }
}

impl Effect for LSystem {
    fn name(&self) -> &str {
        "LSystem"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width as usize;
        let h = self.height as usize;
        if w == 0 || h == 0 {
            return;
        }
        let wf = w as f64;
        let hf = h as f64;

        // Background: sunset sky gradient
        let ground_line = (hf * 0.85) as usize;
        for y in 0..h {
            let yf = y as f64 / hf;
            let idx_start = y * w;
            if y < ground_line {
                // Sky: light blue top -> warm orange near horizon
                let sky_t = yf / 0.85;
                let r = (100.0 + sky_t * 155.0).min(255.0);
                let g = (160.0 + sky_t * 40.0 - sky_t * sky_t * 80.0).clamp(80.0, 200.0);
                let b = (220.0 - sky_t * 150.0).clamp(60.0, 220.0);
                let r8 = r as u8;
                let g8 = g as u8;
                let b8 = b as u8;
                for x in 0..w {
                    pixels[idx_start + x] = (r8, g8, b8);
                }
            } else {
                // Ground: green strip
                let gt = (y - ground_line) as f64 / (h - ground_line) as f64;
                let r = (45.0 + gt * 20.0) as u8;
                let g = (100.0 - gt * 30.0) as u8;
                let b = (30.0 + gt * 10.0) as u8;
                for x in 0..w {
                    pixels[idx_start + x] = (r, g, b);
                }
            }
        }

        // Generate L-system string
        let gens = (self.generations as u32).clamp(3, 6);
        let lstring = Self::generate_string(gens);

        // Interpret as turtle graphics
        let base_angle = 22.5 * PI / 180.0;
        let base_length = hf * 0.12 / (1.8_f64).powi(gens as i32);
        let start_x = wf * 0.5;
        let start_y = ground_line as f64;

        let mut state = TurtleState {
            x: start_x,
            y: start_y,
            angle: -PI / 2.0, // pointing up
            depth: 0,
        };
        let mut stack: Vec<TurtleState> = Vec::new();
        let mut max_depth: u32 = 0;

        // First pass: find max depth to scale colors
        {
            let mut d: u32 = 0;
            for &ch in &lstring {
                match ch {
                    b'[' => d += 1,
                    b']' => d = d.saturating_sub(1),
                    _ => {}
                }
                if d > max_depth {
                    max_depth = d;
                }
            }
        }
        if max_depth == 0 {
            max_depth = 1;
        }

        // Second pass: draw
        for &ch in &lstring {
            match ch {
                b'F' => {
                    let depth_frac = state.depth as f64 / max_depth as f64;

                    // Wind sway: angle offset depends on depth and time
                    let wind_offset =
                        self.wind * 0.02 * (t * 1.5 + state.depth as f64 * 0.5).sin() * depth_frac;

                    let angle = state.angle + wind_offset;
                    let length = base_length * (1.0 - depth_frac * 0.3);

                    let nx = state.x + angle.cos() * length;
                    let ny = state.y + angle.sin() * length;

                    // Color: brown trunk -> green tips
                    let r = (100.0 + (1.0 - depth_frac) * 50.0 - depth_frac * 60.0)
                        .clamp(30.0, 150.0) as u8;
                    let g = (60.0 + depth_frac * 120.0).clamp(60.0, 180.0) as u8;
                    let b = (30.0 + depth_frac * 10.0).clamp(20.0, 50.0) as u8;

                    // Thickness: thicker at base, thinner at tips
                    let thickness = (2.5 - depth_frac * 2.0).max(0.5);

                    Self::draw_line(pixels, w, h, state.x, state.y, nx, ny, (r, g, b), thickness);

                    state.x = nx;
                    state.y = ny;
                }
                b'+' => {
                    state.angle += base_angle;
                }
                b'-' => {
                    state.angle -= base_angle;
                }
                b'[' => {
                    stack.push(TurtleState {
                        x: state.x,
                        y: state.y,
                        angle: state.angle,
                        depth: state.depth,
                    });
                    state.depth += 1;
                }
                b']' => {
                    // Draw leaf at tip before popping
                    let depth_frac = state.depth as f64 / max_depth as f64;
                    if depth_frac > 0.6 {
                        let leaf_size = 1.0 + depth_frac * 1.5;
                        let green_var = ((state.x * 7.3 + state.y * 3.1).sin() * 30.0) as i32;
                        let lr = (40 + green_var).clamp(20, 80) as u8;
                        let lg = (150 + green_var).clamp(100, 200) as u8;
                        let lb = (40 + green_var / 2).clamp(20, 60) as u8;
                        Self::draw_leaf(pixels, w, h, state.x, state.y, leaf_size, (lr, lg, lb));
                    }

                    if let Some(saved) = stack.pop() {
                        state = saved;
                    }
                }
                _ => {}
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "wind".to_string(),
                min: 0.0,
                max: 2.0,
                value: self.wind,
            },
            ParamDesc {
                name: "generations".to_string(),
                min: 3.0,
                max: 6.0,
                value: self.generations,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "wind" => self.wind = value,
            "generations" => self.generations = value,
            _ => {}
        }
    }
}
