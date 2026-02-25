use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct Oscilloscope {
    width: u32,
    height: u32,
    speed: f64,
    decay: f64,
    phosphor: Vec<f64>,
    phase: f64,
}

impl Oscilloscope {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            decay: 0.05,
            phosphor: Vec::new(),
            phase: 0.0,
        }
    }
}

impl Effect for Oscilloscope {
    fn name(&self) -> &str {
        "Oscilloscope"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.phosphor = vec![0.0; (width * height) as usize];
        self.phase = 0.0;
    }

    fn update(&mut self, t: f64, dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let wf = w as f64;
        let hf = h as f64;
        let cx = wf / 2.0;
        let cy = hf / 2.0;
        let scale = cx.min(cy) * 0.8;

        // Decay the phosphor buffer
        let decay_factor = (1.0 - self.decay).max(0.0);
        for p in self.phosphor.iter_mut() {
            *p *= decay_factor;
        }

        // Slowly drifting frequency ratios for organic Lissajous patterns
        let base_t = t * self.speed;
        let freq_x = 3.0 + (base_t * 0.037).sin() * 2.0;
        let freq_y = 2.0 + (base_t * 0.051).cos() * 2.0;
        let freq_x2 = 5.0 + (base_t * 0.023).sin() * 1.5;
        let freq_y2 = 7.0 + (base_t * 0.043).cos() * 1.5;
        let phase_offset = base_t * 0.13;

        // Advance phase and compute new points
        let points_per_frame = 2000;
        let phase_step = dt * self.speed * 8.0 / points_per_frame as f64;

        for _ in 0..points_per_frame {
            self.phase += phase_step;

            // Compound Lissajous: sum of two frequency components for complexity
            let x = 0.6 * (freq_x * self.phase).sin()
                + 0.4 * (freq_x2 * self.phase + phase_offset).sin();
            let y = 0.6 * (freq_y * self.phase + PI * 0.5).cos()
                + 0.4 * (freq_y2 * self.phase + phase_offset * 0.7).cos();

            let px = cx + x * scale;
            let py = cy + y * scale;

            let ix = px as i32;
            let iy = py as i32;

            // Plot with a small glow (radius ~2px)
            for dy in -2..=2_i32 {
                for dx in -2..=2_i32 {
                    let sx = ix + dx;
                    let sy = iy + dy;
                    if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                        let dist_sq = (dx * dx + dy * dy) as f64;
                        let intensity = (-dist_sq * 0.5).exp(); // gaussian falloff
                        let idx = (sy as u32 * w + sx as u32) as usize;
                        self.phosphor[idx] = (self.phosphor[idx] + intensity * 0.3).min(1.0);
                    }
                }
            }
        }

        // Render phosphor buffer to pixels with green CRT coloring and scanlines
        for y in 0..h {
            // Scanline effect: every other row is slightly dimmer
            let scanline = if y % 2 == 0 { 1.0 } else { 0.82 };
            let row_offset = (y * w) as usize;

            for x in 0..w {
                let idx = row_offset + x as usize;
                if idx >= self.phosphor.len() {
                    break;
                }

                let p = self.phosphor[idx];
                let v = p * scanline;

                // Green phosphor: bright is (0, 255, 50), dim is (0, 40, 10), off is black
                // Use a curve that makes the glow feel warm and CRT-like
                let v_sq = v * v; // quadratic for more glow at bright end
                let r = (v_sq * 30.0).min(255.0) as u8;
                let g = (v * 40.0 + v_sq * 215.0).min(255.0) as u8;
                let b = (v * 10.0 + v_sq * 40.0).min(255.0) as u8;

                pixels[idx] = (r, g, b);
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
                name: "decay".to_string(),
                min: 0.01,
                max: 0.15,
                value: self.decay,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "decay" => self.decay = value,
            _ => {}
        }
    }
}
