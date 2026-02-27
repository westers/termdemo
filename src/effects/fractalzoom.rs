use crate::effect::{Effect, ParamDesc};

pub struct FractalZoom {
    width: u32,
    height: u32,
    zoom_speed: f64,
    max_iter: f64,
    center_re: f64,
    center_im: f64,
}

impl FractalZoom {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            zoom_speed: 0.8,
            max_iter: 100.0,
            center_re: TARGET_RE,
            center_im: TARGET_IM,
        }
    }
}

// Seahorse Valley target
const TARGET_RE: f64 = -0.743643887037158;
const TARGET_IM: f64 = 0.131825904205330;

fn mandelbrot_iter(c_re: f64, c_im: f64, max_iter: u32) -> u32 {
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
    iter
}

fn sample_variance(center_re: f64, center_im: f64, scale: f64, max_iter: u32) -> f64 {
    let grid = 16;
    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let n = (grid * grid) as f64;
    for gy in 0..grid {
        for gx in 0..grid {
            let nx = (gx as f64 / grid as f64 - 0.5) * 2.0;
            let ny = (gy as f64 / grid as f64 - 0.5) * 2.0;
            let c_re = center_re + nx * scale;
            let c_im = center_im + ny * scale;
            let it = mandelbrot_iter(c_re, c_im, max_iter) as f64;
            sum += it;
            sum_sq += it * it;
        }
    }
    let mean = sum / n;
    sum_sq / n - mean * mean
}

impl Effect for FractalZoom {
    fn name(&self) -> &str {
        "FractalZoom"
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
        // Exponential zoom: doubles every 1/zoom_speed seconds
        // Cycle to avoid f64 precision loss (~47 doublings is the limit)
        let cycle_period = 45.0 / self.zoom_speed;
        let cycle_t = t % cycle_period;

        // Scale max_iter with zoom depth so detail persists at deep zoom
        let dynamic_max_iter = (self.max_iter + cycle_t * self.zoom_speed * 8.0) as u32;

        // Reset center on cycle wrap (when cycle_t is near zero)
        if cycle_t < 0.05 {
            self.center_re = TARGET_RE;
            self.center_im = TARGET_IM;
        }

        let zoom = 2.0_f64.powf(cycle_t * self.zoom_speed);
        let scale = 1.5 / zoom;

        for y in 0..h {
            for x in 0..w {
                let nx = (x as f64 / wf - 0.5) * 2.0 * aspect;
                let ny = (y as f64 / hf - 0.5) * 2.0;

                let c_re = self.center_re + nx * scale;
                let c_im = self.center_im + ny * scale;

                let mut z_re = 0.0;
                let mut z_im = 0.0;
                let mut iter = 0u32;

                while iter < dynamic_max_iter {
                    let z_re2 = z_re * z_re;
                    let z_im2 = z_im * z_im;
                    if z_re2 + z_im2 > 256.0 {
                        break;
                    }
                    z_im = 2.0 * z_re * z_im + c_im;
                    z_re = z_re2 - z_im2 + c_re;
                    iter += 1;
                }

                let idx = (y * w + x) as usize;

                if iter == dynamic_max_iter {
                    pixels[idx] = (0, 0, 0);
                } else {
                    // Smooth iteration count for band-free coloring
                    let z_mag_sq = z_re * z_re + z_im * z_im;
                    let smooth = if z_mag_sq > 1.0 {
                        iter as f64 + 1.0
                            - (z_mag_sq.ln() / 2.0).ln() / std::f64::consts::LN_2
                    } else {
                        iter as f64
                    };

                    // Map to palette: blue -> cyan -> yellow -> red -> blue
                    let palette_t = (smooth * 0.03 + t * 0.02) % 1.0;
                    pixels[idx] = palette_color(palette_t);
                }
            }
        }

        // Steer toward interesting regions if current view is too uniform
        let current_var = sample_variance(self.center_re, self.center_im, scale, dynamic_max_iter);
        if current_var < 5.0 {
            let probe_dist = scale * 0.3;
            let directions: [(f64, f64); 4] = [
                (probe_dist, 0.0),
                (-probe_dist, 0.0),
                (0.0, probe_dist),
                (0.0, -probe_dist),
            ];
            let mut best_var = current_var;
            let mut best_re = self.center_re;
            let mut best_im = self.center_im;
            for (dre, dim) in &directions {
                let v = sample_variance(self.center_re + dre, self.center_im + dim, scale, dynamic_max_iter);
                if v > best_var {
                    best_var = v;
                    best_re = self.center_re + dre;
                    best_im = self.center_im + dim;
                }
            }
            // Nudge center 10% toward best direction
            self.center_re += (best_re - self.center_re) * 0.1;
            self.center_im += (best_im - self.center_im) * 0.1;
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "zoom_speed".to_string(),
                min: 0.3,
                max: 2.0,
                value: self.zoom_speed,
            },
            ParamDesc {
                name: "max_iter".to_string(),
                min: 50.0,
                max: 200.0,
                value: self.max_iter,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "zoom_speed" => self.zoom_speed = value,
            "max_iter" => self.max_iter = value,
            _ => {}
        }
    }
}

/// Palette cycling: blue -> cyan -> yellow -> red -> blue
/// t in [0, 1)
fn palette_color(t: f64) -> (u8, u8, u8) {
    // 4 control points evenly spaced
    let colors: [(f64, f64, f64); 5] = [
        (0.0, 0.1, 0.8),   // blue
        (0.0, 0.8, 0.9),   // cyan
        (1.0, 1.0, 0.2),   // yellow
        (0.9, 0.1, 0.1),   // red
        (0.0, 0.1, 0.8),   // blue (wraps)
    ];

    let segment = t * 4.0;
    let i = (segment as usize).min(3);
    let f = segment - i as f64;

    // Smooth interpolation (smoothstep)
    let f = f * f * (3.0 - 2.0 * f);

    let c0 = colors[i];
    let c1 = colors[i + 1];

    let r = c0.0 + (c1.0 - c0.0) * f;
    let g = c0.1 + (c1.1 - c0.1) * f;
    let b = c0.2 + (c1.2 - c0.2) * f;

    (
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    )
}
