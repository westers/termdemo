use crate::effect::{Effect, ParamDesc};

pub struct Mandelbrot {
    width: u32,
    height: u32,
    zoom_speed: f64,
    max_iter: u32,
}

impl Mandelbrot {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            zoom_speed: 1.0,
            max_iter: 100,
        }
    }
}

// Target point near the Mandelbrot boundary
const TARGET_RE: f64 = -0.7435669;
const TARGET_IM: f64 = 0.1314023;

impl Effect for Mandelbrot {
    fn name(&self) -> &str {
        "Mandelbrot"
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

        // Cycle zoom every ~20s to avoid f64 precision loss
        let cycle_period = 20.0;
        let cycle_t = t % cycle_period;
        let zoom = 3.0 * (-cycle_t * self.zoom_speed * 0.3).exp();

        for y in 0..h {
            for x in 0..w {
                let nx = (x as f64 / wf - 0.5) * 2.0 * aspect;
                let ny = (y as f64 / hf - 0.5) * 2.0;

                let c_re = TARGET_RE + nx * zoom;
                let c_im = TARGET_IM + ny * zoom;

                let mut z_re = 0.0;
                let mut z_im = 0.0;
                let mut iter = 0u32;

                while iter < max_iter {
                    let z_re2 = z_re * z_re;
                    let z_im2 = z_im * z_im;
                    if z_re2 + z_im2 > 4.0 {
                        break;
                    }
                    z_im = 2.0 * z_re * z_im + c_im;
                    z_re = z_re2 - z_im2 + c_re;
                    iter += 1;
                }

                let idx = (y * w + x) as usize;

                if iter == max_iter {
                    pixels[idx] = (0, 0, 0);
                } else {
                    // Smooth coloring
                    let z_mag_sq = z_re * z_re + z_im * z_im;
                    let smooth = if z_mag_sq > 1.0 {
                        iter as f64 + 1.0 - (z_mag_sq.ln() / 2.0_f64.ln()).ln() / 2.0_f64.ln()
                    } else {
                        iter as f64
                    };

                    let hue = (smooth * 0.02 + t * 0.05) % 1.0;
                    let sat = 0.8;
                    let val = 1.0;
                    pixels[idx] = hsv_to_rgb(hue, sat, val);
                }
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "zoom_speed".to_string(),
                min: 0.2,
                max: 3.0,
                value: self.zoom_speed,
            },
            ParamDesc {
                name: "max_iter".to_string(),
                min: 50.0,
                max: 300.0,
                value: self.max_iter as f64,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "zoom_speed" => self.zoom_speed = value,
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
