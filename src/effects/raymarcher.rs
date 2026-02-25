use crate::effect::{Effect, ParamDesc};

pub struct Raymarcher {
    width: u32,
    height: u32,
    speed: f64,
    complexity: f64,
}

impl Raymarcher {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            complexity: 1.0,
        }
    }
}

// SDF primitives
fn sd_sphere(p: [f64; 3], r: f64) -> f64 {
    length(p) - r
}

fn sd_box(p: [f64; 3], b: [f64; 3]) -> f64 {
    let q = [p[0].abs() - b[0], p[1].abs() - b[1], p[2].abs() - b[2]];
    length([q[0].max(0.0), q[1].max(0.0), q[2].max(0.0)])
        + q[0].max(q[1].max(q[2])).min(0.0)
}

fn sd_plane(p: [f64; 3], h: f64) -> f64 {
    p[1] - h
}

fn op_smooth_union(d1: f64, d2: f64, k: f64) -> f64 {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    d2 * (1.0 - h) + d1 * h - k * h * (1.0 - h)
}

fn op_rep(p: f64, s: f64) -> f64 {
    p - s * (p / s + 0.5).floor()
}

fn length(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let l = length(v).max(1e-10);
    [v[0] / l, v[1] / l, v[2] / l]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

impl Raymarcher {
    fn scene_sdf(&self, p: [f64; 3], t: f64) -> (f64, u8) {
        // Ground plane
        let d_plane = sd_plane(p, -1.0);

        // Central sphere, bobbing up and down
        let sphere_pos = [p[0], p[1] - 0.3 * (t * 1.5).sin(), p[2]];
        let d_sphere = sd_sphere(sphere_pos, 0.8);

        // Orbiting smaller spheres
        let orbit_r = 2.0;
        let s1 = [
            p[0] - orbit_r * (t * 0.7).cos(),
            p[1] - 0.2 * (t * 2.0).sin(),
            p[2] - orbit_r * (t * 0.7).sin(),
        ];
        let d_s1 = sd_sphere(s1, 0.4);

        let s2 = [
            p[0] - orbit_r * (t * 0.7 + 2.094).cos(),
            p[1] - 0.2 * (t * 2.0 + 1.0).sin(),
            p[2] - orbit_r * (t * 0.7 + 2.094).sin(),
        ];
        let d_s2 = sd_sphere(s2, 0.4);

        let s3 = [
            p[0] - orbit_r * (t * 0.7 + 4.189).cos(),
            p[1] - 0.2 * (t * 2.0 + 2.0).sin(),
            p[2] - orbit_r * (t * 0.7 + 4.189).sin(),
        ];
        let d_s3 = sd_sphere(s3, 0.4);

        // Smooth union of all spheres
        let k = 0.5 * self.complexity;
        let d_spheres = op_smooth_union(
            d_sphere,
            op_smooth_union(d_s1, op_smooth_union(d_s2, d_s3, k), k),
            k,
        );

        // Repeated box pillars (if complexity is high enough)
        let d_boxes = if self.complexity > 0.5 {
            let bp = [op_rep(p[0], 4.0), p[1], op_rep(p[2], 4.0)];
            sd_box(bp, [0.3, 1.5, 0.3])
        } else {
            f64::MAX
        };

        // Combine: find closest object and material ID
        let mut d = d_plane;
        let mut mat = 0u8; // 0=floor, 1=spheres, 2=boxes

        if d_spheres < d {
            d = d_spheres;
            mat = 1;
        }
        if d_boxes < d {
            d = d_boxes;
            mat = 2;
        }

        (d, mat)
    }

    fn calc_normal(&self, p: [f64; 3], t: f64) -> [f64; 3] {
        let e = 0.001;
        let (dx, _) = self.scene_sdf([p[0] + e, p[1], p[2]], t);
        let (dxn, _) = self.scene_sdf([p[0] - e, p[1], p[2]], t);
        let (dy, _) = self.scene_sdf([p[0], p[1] + e, p[2]], t);
        let (dyn_, _) = self.scene_sdf([p[0], p[1] - e, p[2]], t);
        let (dz, _) = self.scene_sdf([p[0], p[1], p[2] + e], t);
        let (dzn, _) = self.scene_sdf([p[0], p[1], p[2] - e], t);
        normalize([dx - dxn, dy - dyn_, dz - dzn])
    }
}

impl Effect for Raymarcher {
    fn name(&self) -> &str {
        "Raymarcher"
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
        let t = t * self.speed;

        // Camera orbits the scene
        let cam_angle = t * 0.3;
        let cam_dist = 5.0;
        let cam_pos = [
            cam_angle.cos() * cam_dist,
            1.5 + 0.5 * (t * 0.4).sin(),
            cam_angle.sin() * cam_dist,
        ];

        // Look at origin
        let target = [0.0, 0.0, 0.0];
        let forward = normalize([
            target[0] - cam_pos[0],
            target[1] - cam_pos[1],
            target[2] - cam_pos[2],
        ]);
        let up = [0.0, 1.0, 0.0];
        let right = normalize([
            forward[1] * up[2] - forward[2] * up[1],
            forward[2] * up[0] - forward[0] * up[2],
            forward[0] * up[1] - forward[1] * up[0],
        ]);
        let cam_up = [
            right[1] * forward[2] - right[2] * forward[1],
            right[2] * forward[0] - right[0] * forward[2],
            right[0] * forward[1] - right[1] * forward[0],
        ];

        // Light position
        let light_pos = [
            3.0 * (t * 0.5).sin(),
            4.0,
            3.0 * (t * 0.5).cos(),
        ];

        for y in 0..h {
            let ny = -(y as f64 / hf * 2.0 - 1.0);
            for x in 0..w {
                let nx = (x as f64 / wf * 2.0 - 1.0) * aspect;

                // Ray direction
                let rd = normalize([
                    forward[0] + nx * right[0] + ny * cam_up[0],
                    forward[1] + nx * right[1] + ny * cam_up[1],
                    forward[2] + nx * right[2] + ny * cam_up[2],
                ]);

                // Raymarch
                let mut total_dist = 0.0;
                let max_dist = 30.0;
                let max_steps = 64;
                let mut hit_mat = 255u8;
                let mut hit_pos = cam_pos;

                for _ in 0..max_steps {
                    let p = [
                        cam_pos[0] + rd[0] * total_dist,
                        cam_pos[1] + rd[1] * total_dist,
                        cam_pos[2] + rd[2] * total_dist,
                    ];

                    let (d, mat) = self.scene_sdf(p, t);

                    if d < 0.001 {
                        hit_mat = mat;
                        hit_pos = p;
                        break;
                    }

                    total_dist += d;
                    if total_dist > max_dist {
                        break;
                    }
                }

                let idx = (y * w + x) as usize;

                if hit_mat == 255 {
                    // Sky gradient
                    let sky_t = (ny * 0.5 + 0.5).clamp(0.0, 1.0);
                    let r = (30.0 + sky_t * 50.0) as u8;
                    let g = (20.0 + sky_t * 40.0) as u8;
                    let b = (50.0 + sky_t * 100.0) as u8;
                    pixels[idx] = (r, g, b);
                    continue;
                }

                // Compute normal and lighting
                let normal = self.calc_normal(hit_pos, t);
                let light_dir = normalize([
                    light_pos[0] - hit_pos[0],
                    light_pos[1] - hit_pos[1],
                    light_pos[2] - hit_pos[2],
                ]);

                let diffuse = dot(normal, light_dir).max(0.0);
                let ambient = 0.15;

                // Specular (Blinn-Phong)
                let half_dir = normalize([
                    light_dir[0] - rd[0],
                    light_dir[1] - rd[1],
                    light_dir[2] - rd[2],
                ]);
                let spec = dot(normal, half_dir).max(0.0).powf(32.0) * 0.5;

                // Distance fog
                let fog = (total_dist / max_dist).clamp(0.0, 1.0);
                let fog = fog * fog;

                // Material colors
                let (mr, mg, mb) = match hit_mat {
                    0 => {
                        // Floor: checkerboard
                        let check = ((hit_pos[0].floor() + hit_pos[2].floor()) as i32 & 1) as f64;
                        let v = 0.3 + check * 0.3;
                        (v * 0.9, v * 0.9, v)
                    }
                    1 => {
                        // Spheres: colorful
                        let hue = (t * 0.1 + hit_pos[1] * 0.2) % 1.0;
                        let (r, g, b) = hsv_to_rgb_f(hue, 0.6, 0.9);
                        (r, g, b)
                    }
                    _ => {
                        // Boxes: metallic gray
                        (0.6, 0.55, 0.5)
                    }
                };

                let light = ambient + diffuse * 0.8;
                let r = ((mr * light + spec) * (1.0 - fog) + 0.12 * fog).clamp(0.0, 1.0);
                let g = ((mg * light + spec) * (1.0 - fog) + 0.08 * fog).clamp(0.0, 1.0);
                let b = ((mb * light + spec * 0.5) * (1.0 - fog) + 0.2 * fog).clamp(0.0, 1.0);

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
                name: "speed".to_string(),
                min: 0.2,
                max: 3.0,
                value: self.speed,
            },
            ParamDesc {
                name: "complexity".to_string(),
                min: 0.1,
                max: 2.0,
                value: self.complexity,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "complexity" => self.complexity = value,
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
    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
