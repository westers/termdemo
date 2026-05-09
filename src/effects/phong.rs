use crate::effect::{Effect, ParamDesc};

pub struct Phong {
    width: u32,
    height: u32,
    rot_speed: f64,
    shininess: f64,
}

impl Phong {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            rot_speed: 1.0,
            shininess: 1.0,
        }
    }
}

fn length3(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn length2(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let l = length3(v).max(1e-10);
    [v[0] / l, v[1] / l, v[2] / l]
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

// SDF for a torus in xy-plane (axis = z): major R, minor r
fn sd_torus(p: [f64; 3], big_r: f64, small_r: f64) -> f64 {
    let q = length2(p[0], p[2]) - big_r;
    length2(q, p[1]) - small_r
}

fn rotate3(p: [f64; 3], rx: f64, ry: f64) -> [f64; 3] {
    let (cx, sx) = (rx.cos(), rx.sin());
    let (cy, sy) = (ry.cos(), ry.sin());
    let (x, y, z) = (p[0], p[1], p[2]);
    let x1 = x * cy + z * sy;
    let z1 = -x * sy + z * cy;
    let y2 = y * cx - z1 * sx;
    let z2 = y * sx + z1 * cx;
    [x1, y2, z2]
}

impl Phong {
    fn scene(&self, p: [f64; 3], rx: f64, ry: f64) -> f64 {
        let pr = rotate3(p, rx, ry);
        sd_torus(pr, 1.2, 0.42)
    }

    fn calc_normal(&self, p: [f64; 3], rx: f64, ry: f64) -> [f64; 3] {
        let e = 0.001;
        let dx = self.scene([p[0] + e, p[1], p[2]], rx, ry)
            - self.scene([p[0] - e, p[1], p[2]], rx, ry);
        let dy = self.scene([p[0], p[1] + e, p[2]], rx, ry)
            - self.scene([p[0], p[1] - e, p[2]], rx, ry);
        let dz = self.scene([p[0], p[1], p[2] + e], rx, ry)
            - self.scene([p[0], p[1], p[2] - e, ], rx, ry);
        normalize([dx, dy, dz])
    }
}

impl Effect for Phong {
    fn name(&self) -> &str {
        "Phong"
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
        let t = t * self.rot_speed;

        let rx = t * 0.5;
        let ry = t * 0.7;

        let cam_pos = [0.0_f64, 0.0, -3.5];
        let light_pos = [
            2.5 * (t * 0.6).cos(),
            2.0,
            -3.0 + 0.8 * (t * 0.4).sin(),
        ];

        // Material color cycles slowly
        let hue_base = (t * 0.05) % 1.0;
        let (mr, mg, mb) = hsv_to_rgb_f(hue_base, 0.55, 1.0);
        let shine_pow = (24.0 + 64.0 * self.shininess).max(4.0);

        for y in 0..h {
            let ny = -(y as f64 / hf * 2.0 - 1.0);
            for x in 0..w {
                let nx = (x as f64 / wf * 2.0 - 1.0) * aspect;

                let rd = normalize([nx, ny, 1.4]);
                let mut total = 0.0_f64;
                let max_dist = 20.0;
                let mut hit = false;
                let mut hit_pos = cam_pos;

                for _ in 0..80 {
                    let p = [
                        cam_pos[0] + rd[0] * total,
                        cam_pos[1] + rd[1] * total,
                        cam_pos[2] + rd[2] * total,
                    ];
                    let d = self.scene(p, rx, ry);
                    if d < 0.0008 {
                        hit = true;
                        hit_pos = p;
                        break;
                    }
                    total += d;
                    if total > max_dist {
                        break;
                    }
                }

                let idx = (y * w + x) as usize;

                if !hit {
                    // Vignetted dark background with subtle vertical gradient
                    let v = (1.0 - ny.abs() * 0.5) * 0.06;
                    let r = (8.0 + v * 200.0) as u8;
                    let g = (4.0 + v * 100.0) as u8;
                    let b = (16.0 + v * 220.0) as u8;
                    pixels[idx] = (r, g, b);
                    continue;
                }

                let n = self.calc_normal(hit_pos, rx, ry);
                let l = normalize([
                    light_pos[0] - hit_pos[0],
                    light_pos[1] - hit_pos[1],
                    light_pos[2] - hit_pos[2],
                ]);
                let v = normalize([
                    cam_pos[0] - hit_pos[0],
                    cam_pos[1] - hit_pos[1],
                    cam_pos[2] - hit_pos[2],
                ]);
                let h_vec = normalize([l[0] + v[0], l[1] + v[1], l[2] + v[2]]);

                let n_dot_l = dot3(n, l).max(0.0);
                let n_dot_h = dot3(n, h_vec).max(0.0);
                let spec = n_dot_h.powf(shine_pow);

                // Rim light: edge-on glow
                let n_dot_v = dot3(n, v).max(0.0);
                let rim = (1.0 - n_dot_v).powf(3.0) * 0.4;

                let ambient = 0.08;
                let diffuse = n_dot_l * 0.9;

                let r = (mr * (ambient + diffuse) + spec + rim).clamp(0.0, 1.0);
                let g = (mg * (ambient + diffuse) + spec + rim * 0.7).clamp(0.0, 1.0);
                let b = (mb * (ambient + diffuse) + spec + rim * 1.0).clamp(0.0, 1.0);

                pixels[idx] = (
                    (r * 255.0) as u8,
                    (g * 255.0) as u8,
                    (b * 255.0) as u8,
                );
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "rot_speed".to_string(),
                min: 0.2,
                max: 3.0,
                value: self.rot_speed,
            },
            ParamDesc {
                name: "shininess".to_string(),
                min: 0.1,
                max: 2.5,
                value: self.shininess,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "rot_speed" => self.rot_speed = value,
            "shininess" => self.shininess = value,
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
