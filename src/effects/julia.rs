use crate::effect::{Effect, ParamDesc};

pub struct Julia {
    width: u32,
    height: u32,
    morph_speed: f64,
    max_iter: u32,
}

impl Julia {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            morph_speed: 1.0,
            max_iter: 80,
        }
    }
}

impl Effect for Julia {
    fn name(&self) -> &str {
        "Julia"
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
        let aspect = wf / hf;
        let max_iter = self.max_iter;
        let s = self.morph_speed;

        // c parameter traces an interesting path
        let c_re = 0.35 * (t * s * 0.2).cos() - 0.1 * (t * s * 0.15).sin();
        let c_im = 0.35 * (t * s * 0.2).sin() + 0.1 * (t * s * 0.3).cos();

        let view = 1.5;

        for y in 0..h {
            for x in 0..w {
                // z starts at pixel position (unlike Mandelbrot)
                let mut z_re = (x as f64 / wf - 0.5) * 2.0 * view * aspect;
                let mut z_im = (y as f64 / hf - 0.5) * 2.0 * view;

                let mut iter = 0u32;

                while iter < max_iter {
                    let z_re2 = z_re * z_re;
                    let z_im2 = z_im * z_im;
                    if z_re2 + z_im2 > 4.0 {
                        break;
                    }
                    let new_re = z_re2 - z_im2 + c_re;
                    z_im = 2.0 * z_re * z_im + c_im;
                    z_re = new_re;
                    iter += 1;
                }

                let idx = (y * w + x) as usize;

                if iter == max_iter {
                    // Deep purple interior
                    pixels[idx] = (20, 5, 30);
                } else {
                    // Smooth coloring
                    let z_mag_sq = z_re * z_re + z_im * z_im;
                    let smooth = if z_mag_sq > 1.0 {
                        iter as f64 + 1.0 - (z_mag_sq.ln() / 2.0_f64.ln()).ln() / 2.0_f64.ln()
                    } else {
                        iter as f64
                    };

                    let hue = (smooth * 0.03 + t * 0.05) % 1.0;
                    pixels[idx] = hsv_to_rgb(hue, 0.85, 1.0);
                }
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "morph_speed".to_string(),
                min: 0.2,
                max: 3.0,
                value: self.morph_speed,
            },
            ParamDesc {
                name: "max_iter".to_string(),
                min: 30.0,
                max: 300.0,
                value: self.max_iter as f64,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "morph_speed" => self.morph_speed = value,
            "max_iter" => self.max_iter = value as u32,
            _ => {}
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
