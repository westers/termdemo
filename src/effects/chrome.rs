use crate::effect::{Effect, ParamDesc};

pub struct Chrome {
    width: u32,
    height: u32,
    rot_speed: f64,
    distortion: f64,
}

impl Chrome {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            rot_speed: 1.0,
            distortion: 1.0,
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

fn rotate3(p: [f64; 3], rx: f64, ry: f64, rz: f64) -> [f64; 3] {
    let (cz, sz) = (rz.cos(), rz.sin());
    let (cy, sy) = (ry.cos(), ry.sin());
    let (cx, sx) = (rx.cos(), rx.sin());
    let (x, y, z) = (p[0], p[1], p[2]);
    let (x1, y1) = (x * cz - y * sz, x * sz + y * cz);
    let (x2, z1) = (x1 * cy + z * sy, -x1 * sy + z * cy);
    let (y2, z2) = (y1 * cx - z1 * sx, y1 * sx + z1 * cx);
    [x2, y2, z2]
}

// SDF for a torus with axis along z
fn sd_torus(p: [f64; 3], big_r: f64, small_r: f64) -> f64 {
    let q = length2(p[0], p[2]) - big_r;
    length2(q, p[1]) - small_r
}

// Sample a procedural environment based on view direction
fn sample_env(dir: [f64; 3], t: f64) -> (f64, f64, f64) {
    let d = normalize(dir);
    let v = d[1]; // -1 = down, +1 = up

    if v > 0.0 {
        // Sky: gradient with sun
        let sky_t = v.clamp(0.0, 1.0);
        let r = 0.45 + 0.35 * sky_t;
        let g = 0.55 + 0.30 * sky_t;
        let b = 0.85 + 0.10 * sky_t;

        // Sun direction rotates slowly
        let sun_dir = normalize([
            (t * 0.3).cos() * 0.6,
            0.5,
            (t * 0.3).sin() * 0.6,
        ]);
        let s_dot = dot3(d, sun_dir).max(0.0);
        let sun = s_dot.powf(64.0);
        let halo = s_dot.powf(8.0) * 0.4;

        (
            (r + sun + halo * 1.0).min(2.5),
            (g + sun + halo * 0.6).min(2.5),
            (b + sun + halo * 0.3).min(2.5),
        )
    } else {
        // Ground: warm checker
        let t_floor = -1.0 / v.min(-0.001);
        let fx = d[0] * t_floor;
        let fz = d[2] * t_floor;
        let check = ((fx * 1.5 + t * 0.2).floor() as i32 + (fz * 1.5).floor() as i32) & 1;
        let fade = (1.0 / (1.0 + fx * fx * 0.05 + fz * fz * 0.05)).clamp(0.05, 1.0);
        let base = 0.15 + check as f64 * 0.55;
        let r = base * 0.85 * fade + 0.10 * (1.0 - fade);
        let g = base * 0.70 * fade + 0.05 * (1.0 - fade);
        let b = base * 0.55 * fade + 0.15 * (1.0 - fade);
        (r, g, b)
    }
}

impl Chrome {
    fn scene(&self, p: [f64; 3], rx: f64, ry: f64, rz: f64, t: f64) -> f64 {
        let pr = rotate3(p, rx, ry, rz);
        // Twist along z for a bit of distortion
        let twist = (pr[2] * 0.6 + t * 0.4).sin() * 0.08 * self.distortion;
        let (cz, sz) = (twist.cos(), twist.sin());
        let pt = [pr[0] * cz - pr[1] * sz, pr[0] * sz + pr[1] * cz, pr[2]];
        sd_torus(pt, 1.1, 0.40)
    }

    fn calc_normal(&self, p: [f64; 3], rx: f64, ry: f64, rz: f64, t: f64) -> [f64; 3] {
        let e = 0.001;
        let dx = self.scene([p[0] + e, p[1], p[2]], rx, ry, rz, t)
            - self.scene([p[0] - e, p[1], p[2]], rx, ry, rz, t);
        let dy = self.scene([p[0], p[1] + e, p[2]], rx, ry, rz, t)
            - self.scene([p[0], p[1] - e, p[2]], rx, ry, rz, t);
        let dz = self.scene([p[0], p[1], p[2] + e], rx, ry, rz, t)
            - self.scene([p[0], p[1], p[2] - e], rx, ry, rz, t);
        normalize([dx, dy, dz])
    }
}

impl Effect for Chrome {
    fn name(&self) -> &str {
        "Chrome"
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

        let rx = t * 0.45;
        let ry = t * 0.65;
        let rz = t * 0.20;

        let cam_pos = [0.0_f64, 0.0, -3.5];

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
                    let d = self.scene(p, rx, ry, rz, t);
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
                    let (r, g, b) = sample_env(rd, t);
                    pixels[idx] = (
                        (r.clamp(0.0, 1.0) * 255.0) as u8,
                        (g.clamp(0.0, 1.0) * 255.0) as u8,
                        (b.clamp(0.0, 1.0) * 255.0) as u8,
                    );
                    continue;
                }

                // Reflection
                let n = self.calc_normal(hit_pos, rx, ry, rz, t);
                let v_dot_n = dot3(rd, n);
                let refl = [
                    rd[0] - 2.0 * v_dot_n * n[0],
                    rd[1] - 2.0 * v_dot_n * n[1],
                    rd[2] - 2.0 * v_dot_n * n[2],
                ];

                let (mut er, mut eg, mut eb) = sample_env(refl, t);

                // Fresnel-ish edge tint for that polished metal look
                let fres = (1.0 + v_dot_n).powf(3.0).clamp(0.0, 1.0);
                er = er * (0.85 + fres * 0.15);
                eg = eg * (0.85 + fres * 0.15);
                eb = eb * (0.85 + fres * 0.15) + fres * 0.10;

                pixels[idx] = (
                    (er.clamp(0.0, 1.0) * 255.0) as u8,
                    (eg.clamp(0.0, 1.0) * 255.0) as u8,
                    (eb.clamp(0.0, 1.0) * 255.0) as u8,
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
                name: "distortion".to_string(),
                min: 0.0,
                max: 3.0,
                value: self.distortion,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "rot_speed" => self.rot_speed = value,
            "distortion" => self.distortion = value,
            _ => {}
        }
    }
}
