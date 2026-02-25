use crate::effect::{Effect, ParamDesc};

struct Bob {
    freq_x: f64,
    freq_y: f64,
    phase_x: f64,
    phase_y: f64,
    hue: f64,
}

pub struct Shadebobs {
    width: u32,
    height: u32,
    speed: f64,
    bob_size: f64,
    canvas: Vec<(f64, f64, f64)>, // accumulation buffer (float RGB)
    bobs: Vec<Bob>,
}

impl Shadebobs {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            bob_size: 1.0,
            canvas: Vec::new(),
            bobs: Vec::new(),
        }
    }
}

impl Effect for Shadebobs {
    fn name(&self) -> &str {
        "Shadebobs"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.canvas = vec![(0.0, 0.0, 0.0); (width * height) as usize];

        // Create bobs with different Lissajous frequencies
        self.bobs = vec![
            Bob { freq_x: 0.70, freq_y: 0.90, phase_x: 0.0, phase_y: 0.5, hue: 0.0 },
            Bob { freq_x: 1.10, freq_y: 0.60, phase_x: 1.2, phase_y: 2.1, hue: 0.2 },
            Bob { freq_x: 0.50, freq_y: 1.30, phase_x: 2.5, phase_y: 0.8, hue: 0.4 },
            Bob { freq_x: 0.90, freq_y: 0.40, phase_x: 3.8, phase_y: 3.3, hue: 0.6 },
            Bob { freq_x: 1.30, freq_y: 1.10, phase_x: 5.0, phase_y: 1.5, hue: 0.8 },
        ];
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let wf = w as f64;
        let hf = h as f64;
        let t = t * self.speed;

        // Fade the canvas toward black each frame (slow decay for long trails)
        for c in self.canvas.iter_mut() {
            c.0 *= 0.985;
            c.1 *= 0.985;
            c.2 *= 0.985;
        }

        // Stamp each bob onto the canvas with additive blending
        let base_radius = self.bob_size * wf.min(hf) * 0.08;

        for bob in &self.bobs {
            // Lissajous path
            let cx = 0.5 + 0.40 * (t * bob.freq_x + bob.phase_x).sin();
            let cy = 0.5 + 0.40 * (t * bob.freq_y + bob.phase_y).cos();

            let px = cx * wf;
            let py = cy * hf;
            let radius = base_radius;

            // Color for this bob (slowly rotating hue)
            let hue = (bob.hue + t * 0.05) % 1.0;
            let (cr, cg, cb) = hsv_to_rgb_f64(hue, 0.9, 1.0);

            // Stamp a radial gradient blob
            let r_sq = radius * radius;
            let x0 = (px - radius).max(0.0) as u32;
            let x1 = ((px + radius) as u32 + 1).min(w);
            let y0 = (py - radius).max(0.0) as u32;
            let y1 = ((py + radius) as u32 + 1).min(h);

            for y in y0..y1 {
                let dy = y as f64 - py;
                let dy2 = dy * dy;
                let row = (y * w) as usize;
                for x in x0..x1 {
                    let dx = x as f64 - px;
                    let dist_sq = dx * dx + dy2;
                    if dist_sq < r_sq {
                        // Smooth radial falloff
                        let norm = dist_sq / r_sq;
                        let intensity = (1.0 - norm) * (1.0 - norm);

                        let idx = row + x as usize;
                        self.canvas[idx].0 += cr * intensity * 0.4;
                        self.canvas[idx].1 += cg * intensity * 0.4;
                        self.canvas[idx].2 += cb * intensity * 0.4;
                    }
                }
            }
        }

        // Render canvas to pixels
        for i in 0..pixels.len().min(self.canvas.len()) {
            let c = &self.canvas[i];
            pixels[i] = (
                (c.0.min(1.0) * 255.0) as u8,
                (c.1.min(1.0) * 255.0) as u8,
                (c.2.min(1.0) * 255.0) as u8,
            );
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "speed".to_string(),
                min: 0.2,
                max: 3.0,
                value: self.speed,
            },
            ParamDesc {
                name: "bob_size".to_string(),
                min: 0.3,
                max: 3.0,
                value: self.bob_size,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "bob_size" => self.bob_size = value,
            _ => {}
        }
    }
}

fn hsv_to_rgb_f64(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
