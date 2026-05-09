use crate::effect::{Effect, ParamDesc};

pub struct Wobbler {
    width: u32,
    height: u32,
    amplitude: f64,
    speed: f64,
    source: Vec<(u8, u8, u8)>,
    src_w: u32,
    src_h: u32,
}

impl Wobbler {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            amplitude: 1.0,
            speed: 1.0,
            source: Vec::new(),
            src_w: 0,
            src_h: 0,
        }
    }

    fn build_source(&mut self) {
        // Procedural source: bold diagonal rainbow stripes + black grid + vertical brightness bands
        let sw = self.width.max(80);
        let sh = self.height.max(80);
        self.src_w = sw;
        self.src_h = sh;
        self.source = vec![(0, 0, 0); (sw * sh) as usize];

        let cx = sw as f64 / 2.0;
        let cy = sh as f64 / 2.0;
        let scale = cx.min(cy);

        for y in 0..sh {
            for x in 0..sw {
                let nx = (x as f64 - cx) / scale;
                let ny = (y as f64 - cy) / scale;

                // Diagonal stripes for hue
                let stripe = nx * 0.7 + ny * 0.7;
                let hue = (stripe * 1.2 + 1.0).rem_euclid(1.0);

                // Radial vignette to keep edges interesting
                let r = (nx * nx + ny * ny).sqrt();
                let val = (1.0 - (r - 0.5).abs() * 0.6).clamp(0.4, 1.0);

                // Bold cross-grid lines
                let grid_x = (nx * 5.0).rem_euclid(1.0);
                let grid_y = (ny * 5.0).rem_euclid(1.0);
                let on_grid = grid_x < 0.05 || grid_x > 0.95 || grid_y < 0.05 || grid_y > 0.95;

                let (mut rr, mut gg, mut bb) = hsv_to_rgb(hue, 0.85, val);
                if on_grid {
                    // dim, darken
                    rr = (rr as f64 * 0.25) as u8;
                    gg = (gg as f64 * 0.25) as u8;
                    bb = (bb as f64 * 0.30) as u8;
                }

                // Highlight a central diamond shape
                let diamond = nx.abs() + ny.abs();
                if diamond < 0.45 && diamond > 0.4 {
                    rr = rr.saturating_add(60);
                    gg = gg.saturating_add(60);
                    bb = bb.saturating_add(60);
                }

                let idx = (y * sw + x) as usize;
                self.source[idx] = (rr, gg, bb);
            }
        }
    }
}

impl Effect for Wobbler {
    fn name(&self) -> &str {
        "Wobbler"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.build_source();
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }
        if self.src_w != w || self.src_h != h {
            self.build_source();
        }

        let sw = self.src_w as f64;
        let sh = self.src_h as f64;
        let t = t * self.speed;
        let amp_x = sw * 0.08 * self.amplitude;
        let amp_y = sh * 0.06 * self.amplitude;

        let freq_y1 = 18.0;
        let freq_x1 = 22.0;
        let freq_y2 = 7.0;
        let freq_x2 = 9.0;

        // Breathing zoom centered on midscreen
        let zoom = 1.0 + 0.12 * (t * 0.4).sin();

        for y in 0..h {
            let yf = y as f64;
            let row_phase = t * 1.6 + yf * 0.02;
            for x in 0..w {
                let xf = x as f64;

                // Two-octave sine displacement
                let dx = amp_x * ((yf / freq_y1 + t * 1.3).sin()
                    + 0.4 * (yf / freq_y2 - t * 0.8).sin());
                let dy = amp_y * ((xf / freq_x1 + t * 1.1).sin()
                    + 0.4 * (xf / freq_x2 + row_phase).cos());

                // Zoom about screen center
                let cx = w as f64 / 2.0;
                let cy = h as f64 / 2.0;
                let zx = (xf - cx) / zoom + cx;
                let zy = (yf - cy) / zoom + cy;

                let mut sx = (zx + dx).rem_euclid(sw);
                let mut sy = (zy + dy).rem_euclid(sh);
                if sx < 0.0 {
                    sx += sw;
                }
                if sy < 0.0 {
                    sy += sh;
                }

                // Bilinear sampling
                let x0 = sx as u32;
                let y0 = sy as u32;
                let x1 = (x0 + 1) % self.src_w;
                let y1 = (y0 + 1) % self.src_h;
                let fx = sx - sx.floor();
                let fy = sy - sy.floor();

                let i00 = (y0 * self.src_w + x0) as usize;
                let i01 = (y0 * self.src_w + x1) as usize;
                let i10 = (y1 * self.src_w + x0) as usize;
                let i11 = (y1 * self.src_w + x1) as usize;

                let p00 = self.source[i00];
                let p01 = self.source[i01];
                let p10 = self.source[i10];
                let p11 = self.source[i11];

                let r = bilerp(p00.0 as f64, p01.0 as f64, p10.0 as f64, p11.0 as f64, fx, fy);
                let g = bilerp(p00.1 as f64, p01.1 as f64, p10.1 as f64, p11.1 as f64, fx, fy);
                let b = bilerp(p00.2 as f64, p01.2 as f64, p10.2 as f64, p11.2 as f64, fx, fy);

                let idx = (y * w + x) as usize;
                pixels[idx] = (r as u8, g as u8, b as u8);
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "amplitude".to_string(),
                min: 0.1,
                max: 3.0,
                value: self.amplitude,
            },
            ParamDesc {
                name: "speed".to_string(),
                min: 0.2,
                max: 3.0,
                value: self.speed,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "amplitude" => self.amplitude = value,
            "speed" => self.speed = value,
            _ => {}
        }
    }
}

fn bilerp(a: f64, b: f64, c: f64, d: f64, fx: f64, fy: f64) -> f64 {
    let top = a * (1.0 - fx) + b * fx;
    let bot = c * (1.0 - fx) + d * fx;
    top * (1.0 - fy) + bot * fy
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}
