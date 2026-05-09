use crate::effect::{Effect, ParamDesc};

#[derive(Copy, Clone, Debug)]
struct Circle {
    k: f64,    // curvature (1/r, negative for outer)
    cx: f64,   // center x
    cy: f64,   // center y
}

impl Circle {
    fn radius(&self) -> f64 {
        (1.0 / self.k).abs()
    }
}

pub struct Apollonian {
    width: u32,
    height: u32,
    rot_speed: f64,
    depth: u32,
    circles: Vec<(Circle, u32)>, // circle + depth
}

impl Apollonian {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            rot_speed: 1.0,
            depth: 7,
            circles: Vec::new(),
        }
    }

    fn build(&mut self) {
        self.circles.clear();

        // Outer + 3 equal inner circles
        let k_inner = (2.0_f64 * 3.0_f64.sqrt() + 3.0) / 3.0; // (2√3+3)/3
        let d = 1.0 - 1.0 / k_inner; // distance from origin to center of inner circle

        let c0 = Circle {
            k: -1.0,
            cx: 0.0,
            cy: 0.0,
        };
        let c1 = Circle {
            k: k_inner,
            cx: d,
            cy: 0.0,
        };
        let angle = std::f64::consts::TAU / 3.0;
        let c2 = Circle {
            k: k_inner,
            cx: d * angle.cos(),
            cy: d * angle.sin(),
        };
        let c3 = Circle {
            k: k_inner,
            cx: d * (2.0 * angle).cos(),
            cy: d * (2.0 * angle).sin(),
        };

        self.circles.push((c0, 0));
        self.circles.push((c1, 0));
        self.circles.push((c2, 0));
        self.circles.push((c3, 0));

        // Recurse: each member of quad can be "mirrored" through the other three
        let max_depth = self.depth;
        let min_radius = 0.005;
        let max_radius = 0.95;

        let mut stack: Vec<([Circle; 4], i32, u32)> = Vec::new();
        stack.push(([c0, c1, c2, c3], -1, 0));

        while let Some((quad, skip, depth)) = stack.pop() {
            if depth >= max_depth {
                continue;
            }
            for i in 0..4 {
                if i as i32 == skip {
                    continue;
                }
                let new = mirror(&quad, i);
                let r = new.radius();
                if !r.is_finite() || r < min_radius || r > max_radius {
                    continue;
                }
                // Reject if center is way outside
                let dist = (new.cx * new.cx + new.cy * new.cy).sqrt();
                if dist > 1.5 {
                    continue;
                }
                self.circles.push((new, depth + 1));
                let mut next = quad;
                next[i] = new;
                stack.push((next, i as i32, depth + 1));
            }
        }
    }
}

fn mirror(quad: &[Circle; 4], i: usize) -> Circle {
    let a = quad[i];
    let mut sum_k = 0.0;
    let mut sum_kx = 0.0;
    let mut sum_ky = 0.0;
    for (j, c) in quad.iter().enumerate() {
        if j == i {
            continue;
        }
        sum_k += c.k;
        sum_kx += c.k * c.cx;
        sum_ky += c.k * c.cy;
    }
    let k_new = 2.0 * sum_k - a.k;
    let kx_new = 2.0 * sum_kx - a.k * a.cx;
    let ky_new = 2.0 * sum_ky - a.k * a.cy;
    Circle {
        k: k_new,
        cx: kx_new / k_new,
        cy: ky_new / k_new,
    }
}

impl Effect for Apollonian {
    fn name(&self) -> &str {
        "Apollonian"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.build();
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        // Dark indigo bg
        for p in pixels.iter_mut() {
            *p = (4, 2, 14);
        }

        let wf = w as f64;
        let hf = h as f64;
        let cx = wf / 2.0;
        let cy = hf / 2.0;
        let scale = cx.min(cy) * 0.95;

        let t = t * self.rot_speed;
        let rot = t * 0.15;
        let breathe = 1.0 + 0.04 * (t * 0.7).sin();
        let (cosr, sinr) = (rot.cos(), rot.sin());

        for &(c, depth) in &self.circles {
            let (x, y) = (c.cx * cosr - c.cy * sinr, c.cx * sinr + c.cy * cosr);
            let sx = cx + x * scale * breathe;
            let sy = cy + y * scale * breathe;
            let r_pix = c.radius() * scale * breathe;

            // Hue cycles by depth + slow time
            let hue = (depth as f64 * 0.13 + t * 0.05) % 1.0;
            let sat = 0.6;
            let val = (1.0 - depth as f64 * 0.06).clamp(0.4, 1.0);
            let (cr, cg, cb) = hsv_to_rgb(hue, sat, val);

            // Anti-aliased ring
            draw_ring(pixels, w, h, sx, sy, r_pix, 1.2, cr, cg, cb);
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
                name: "depth".to_string(),
                min: 3.0,
                max: 9.0,
                value: self.depth as f64,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "rot_speed" => self.rot_speed = value,
            "depth" => {
                let nd = value as u32;
                if nd != self.depth {
                    self.depth = nd;
                    self.build();
                }
            }
            _ => {}
        }
    }
}

fn draw_ring(
    pixels: &mut [(u8, u8, u8)],
    w: u32,
    h: u32,
    cx: f64,
    cy: f64,
    radius: f64,
    thickness: f64,
    r: u8,
    g: u8,
    b: u8,
) {
    let half_t = thickness * 0.5;
    let r_outer = radius + half_t + 1.0;

    // Bounding box clipped to screen
    let x0 = ((cx - r_outer).max(0.0)) as i32;
    let x1 = ((cx + r_outer).min(w as f64 - 1.0)) as i32;
    let y0 = ((cy - r_outer).max(0.0)) as i32;
    let y1 = ((cy + r_outer).min(h as f64 - 1.0)) as i32;
    if x1 < x0 || y1 < y0 {
        return;
    }

    for y in y0..=y1 {
        let dy = y as f64 - cy;
        for x in x0..=x1 {
            let dx = x as f64 - cx;
            let d = (dx * dx + dy * dy).sqrt();
            let off = (d - radius).abs();
            if off > half_t + 0.5 {
                continue;
            }
            let alpha = (1.0 - (off / (half_t + 0.5)).clamp(0.0, 1.0)).powf(0.7);
            if alpha <= 0.01 {
                continue;
            }
            let idx = (y as u32 * w + x as u32) as usize;
            if idx < pixels.len() {
                let p = &mut pixels[idx];
                p.0 = ((p.0 as f64) * (1.0 - alpha) + r as f64 * alpha).min(255.0) as u8;
                p.1 = ((p.1 as f64) * (1.0 - alpha) + g as f64 * alpha).min(255.0) as u8;
                p.2 = ((p.2 as f64) * (1.0 - alpha) + b as f64 * alpha).min(255.0) as u8;
            }
        }
    }
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
