use crate::effect::{Effect, ParamDesc};
use rand::rngs::StdRng;
use rand::Rng;

const POINTS_PER_FRAME: usize = 4000;

#[derive(Copy, Clone)]
enum Kind {
    Aizawa,
    Lorenz,
    Thomas,
}

pub struct Attractor {
    width: u32,
    height: u32,
    speed: f64,
    fade: f64,
    canvas: Vec<(f64, f64, f64)>,
    px: f64,
    py: f64,
    pz: f64,
    kind: Kind,
}

impl Attractor {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            fade: 1.0,
            canvas: Vec::new(),
            px: 0.1,
            py: 0.0,
            pz: 0.0,
            kind: Kind::Aizawa,
        }
    }

    fn step(&self, x: f64, y: f64, z: f64, dt: f64) -> (f64, f64, f64) {
        match self.kind {
            Kind::Aizawa => {
                // Classic Aizawa parameters
                let a = 0.95;
                let b = 0.7;
                let c = 0.6;
                let d = 3.5;
                let e = 0.25;
                let f = 0.1;
                let dx = (z - b) * x - d * y;
                let dy = d * x + (z - b) * y;
                let dz = c + a * z - z * z * z / 3.0
                    - (x * x + y * y) * (1.0 + e * z)
                    + f * z * x * x * x;
                (x + dx * dt, y + dy * dt, z + dz * dt)
            }
            Kind::Lorenz => {
                let sigma = 10.0;
                let rho = 28.0;
                let beta = 8.0 / 3.0;
                let dx = sigma * (y - x);
                let dy = x * (rho - z) - y;
                let dz = x * y - beta * z;
                (x + dx * dt, y + dy * dt, z + dz * dt)
            }
            Kind::Thomas => {
                let b = 0.208186;
                let dx = y.sin() - b * x;
                let dy = z.sin() - b * y;
                let dz = x.sin() - b * z;
                (x + dx * dt, y + dy * dt, z + dz * dt)
            }
        }
    }

    fn projection(&self) -> (f64, f64, f64, [f64; 3]) {
        // (scale, center_offset_y, camera_z, color_seed)
        match self.kind {
            Kind::Aizawa => (1.0, 0.0, 4.0, [0.55, 0.85, 1.0]),
            Kind::Lorenz => (0.04, -0.5, 5.0, [1.0, 0.55, 0.25]),
            Kind::Thomas => (0.18, 0.0, 5.0, [0.6, 1.0, 0.7]),
        }
    }
}

impl Effect for Attractor {
    fn name(&self) -> &str {
        "Attractor"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.canvas = vec![(0.0, 0.0, 0.0); (width * height) as usize];
        self.px = 0.1;
        self.py = 0.0;
        self.pz = 0.0;
    }

    fn randomize_init(&mut self, rng: &mut StdRng) {
        let kinds = [Kind::Aizawa, Kind::Lorenz, Kind::Thomas];
        self.kind = kinds[rng.gen_range(0..kinds.len())];
        match self.kind {
            Kind::Lorenz => {
                self.px = rng.gen_range(-1.0..1.0);
                self.py = rng.gen_range(-1.0..1.0);
                self.pz = rng.gen_range(0.0..2.0);
            }
            _ => {
                self.px = rng.gen_range(-0.2..0.2);
                self.py = rng.gen_range(-0.2..0.2);
                self.pz = rng.gen_range(-0.2..0.2);
            }
        }
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }
        if self.canvas.len() != (w * h) as usize {
            self.canvas = vec![(0.0, 0.0, 0.0); (w * h) as usize];
        }

        // Fade canvas
        let fade = (0.92_f64).powf(self.fade);
        for c in self.canvas.iter_mut() {
            c.0 *= fade;
            c.1 *= fade;
            c.2 *= fade;
        }

        let wf = w as f64;
        let hf = h as f64;
        let cx = wf / 2.0;
        let cy = hf / 2.0;
        let scale_base = cx.min(cy);

        let (scale_factor, y_offset, camera_z, base_col) = self.projection();
        let scale = scale_base * scale_factor;

        let rx = t * 0.15 * self.speed;
        let ry = t * 0.25 * self.speed;
        let (cosx, sinx) = (rx.cos(), rx.sin());
        let (cosy, siny) = (ry.cos(), ry.sin());

        let dt_step = match self.kind {
            Kind::Aizawa => 0.012 * self.speed,
            Kind::Lorenz => 0.005 * self.speed,
            Kind::Thomas => 0.04 * self.speed,
        };

        for i in 0..POINTS_PER_FRAME {
            let (nx, ny, nz) = self.step(self.px, self.py, self.pz, dt_step);
            self.px = nx;
            self.py = ny;
            self.pz = nz;

            // Center offset for Lorenz
            let py_centered = self.py + y_offset * 60.0;
            let pz_centered = match self.kind {
                Kind::Lorenz => self.pz - 25.0,
                _ => self.pz,
            };

            // Rotate around Y then X
            let x1 = self.px * cosy + pz_centered * siny;
            let z1 = -self.px * siny + pz_centered * cosy;
            let y2 = py_centered * cosx - z1 * sinx;
            let z2 = py_centered * sinx + z1 * cosx;

            let persp = camera_z / (camera_z + z2 * scale_factor * 0.7);
            let sx = cx + x1 * scale * persp;
            let sy = cy + y2 * scale * persp;

            if sx < 0.0 || sy < 0.0 || sx >= wf || sy >= hf {
                continue;
            }

            let idx = (sy as u32 * w + sx as u32) as usize;
            if idx >= self.canvas.len() {
                continue;
            }

            // Hue cycles along the orbit's parameter
            let progress = (i as f64 / POINTS_PER_FRAME as f64 + t * 0.02) % 1.0;
            let (hr, hg, hb) = hsv_to_rgb_f(progress, 0.45, 1.0);
            let bright = (0.55 + 0.45 * persp).clamp(0.3, 1.4);

            let c = &mut self.canvas[idx];
            c.0 += (base_col[0] * 0.5 + hr * 0.5) * bright * 0.7;
            c.1 += (base_col[1] * 0.5 + hg * 0.5) * bright * 0.7;
            c.2 += (base_col[2] * 0.5 + hb * 0.5) * bright * 0.7;
        }

        // Tone-map canvas to pixels
        for (px, c) in pixels.iter_mut().zip(self.canvas.iter()) {
            let r = (c.0 / (1.0 + c.0)).clamp(0.0, 1.0);
            let g = (c.1 / (1.0 + c.1)).clamp(0.0, 1.0);
            let b = (c.2 / (1.0 + c.2)).clamp(0.0, 1.0);
            *px = (
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
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
                name: "fade".to_string(),
                min: 0.3,
                max: 3.0,
                value: self.fade,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "fade" => self.fade = value,
            _ => {}
        }
    }
}

fn hsv_to_rgb_f(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
