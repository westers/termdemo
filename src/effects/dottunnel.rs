use crate::effect::{Effect, ParamDesc};
use std::f64::consts::TAU;

const NUM_RINGS: usize = 32;
const DOTS_PER_RING: usize = 20;
const RING_SPACING: f64 = 1.5;
const TUNNEL_RADIUS: f64 = 1.2;
const CAMERA_Z: f64 = 2.0;

pub struct DotTunnel {
    width: u32,
    height: u32,
    speed: f64,
    twist: f64,
}

impl DotTunnel {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            twist: 1.0,
        }
    }
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let h = ((h % 1.0) + 1.0) % 1.0;
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
    (
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    )
}

impl Effect for DotTunnel {
    fn name(&self) -> &str {
        "Dot Tunnel"
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

        // Clear to dark background
        for p in pixels.iter_mut() {
            *p = (2, 2, 6);
        }

        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        let scale = cx.min(cy) * 0.8;
        let t_speed = t * self.speed;

        // The tunnel moves toward the viewer: rings advance along Z
        // cycle_offset makes rings continuously spawn at the far end
        let cycle_len = NUM_RINGS as f64 * RING_SPACING;
        let cycle_offset = (t_speed * 3.0) % cycle_len;

        // Collect all dots with depth for sorting
        struct Dot {
            sx: f64,
            sy: f64,
            z: f64,
            ring_idx: usize,
        }

        let mut dots: Vec<Dot> = Vec::with_capacity(NUM_RINGS * DOTS_PER_RING);

        for ring in 0..NUM_RINGS {
            // Z position of this ring: starts far away, moves toward viewer
            let base_z = ring as f64 * RING_SPACING - cycle_offset;
            // Wrap around so rings recycle
            let z = ((base_z % cycle_len) + cycle_len) % cycle_len - 0.5;

            // Skip dots that are behind the camera
            if z < -CAMERA_Z * 0.8 {
                continue;
            }

            // Twist angle increases as ring approaches
            let twist_angle = (z * 0.3 * self.twist) + t_speed * 0.2;

            for dot_i in 0..DOTS_PER_RING {
                let angle = dot_i as f64 / DOTS_PER_RING as f64 * TAU + twist_angle;
                let x = angle.cos() * TUNNEL_RADIUS;
                let y = angle.sin() * TUNNEL_RADIUS;

                // Perspective projection
                let persp = CAMERA_Z / (CAMERA_Z + z);
                let sx = cx + x * scale * persp;
                let sy = cy + y * scale * persp;

                dots.push(Dot {
                    sx,
                    sy,
                    z,
                    ring_idx: ring,
                });
            }
        }

        // Sort back-to-front (far dots drawn first)
        dots.sort_by(|a, b| b.z.partial_cmp(&a.z).unwrap_or(std::cmp::Ordering::Equal));

        let max_z = NUM_RINGS as f64 * RING_SPACING;

        for dot in &dots {
            // Brightness fades with distance
            let depth_factor = (1.0 - (dot.z / max_z).clamp(-0.2, 1.0)).powf(0.7);
            let brightness = depth_factor * depth_factor;

            // Dot radius: closer = larger
            let persp = CAMERA_Z / (CAMERA_Z + dot.z);
            let radius = (persp * 3.5).max(0.5);

            // Color: each ring has a different hue, cycling with time
            let hue = (dot.ring_idx as f64 / NUM_RINGS as f64 + t_speed * 0.05) % 1.0;
            let (cr, cg, cb) = hsv_to_rgb(hue, 0.85, brightness);

            // Draw filled circle
            let ri = radius.ceil() as i32;
            let r_sq = radius * radius;

            for dy in -ri..=ri {
                for dx in -ri..=ri {
                    let dist_sq = (dx * dx + dy * dy) as f64;
                    if dist_sq > r_sq {
                        continue;
                    }

                    let px = dot.sx as i32 + dx;
                    let py = dot.sy as i32 + dy;

                    if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                        let idx = (py as u32 * w + px as u32) as usize;
                        // Soft edge: fade at the border of the dot
                        let edge = 1.0 - (dist_sq / r_sq).sqrt();
                        let edge = edge.clamp(0.0, 1.0);

                        let existing = pixels[idx];
                        // Additive-ish blend: take max to make dots glow
                        pixels[idx] = (
                            existing.0.max((cr as f64 * edge) as u8),
                            existing.1.max((cg as f64 * edge) as u8),
                            existing.2.max((cb as f64 * edge) as u8),
                        );
                    }
                }
            }
        }

        // Add subtle center glow
        let glow_radius = 15.0;
        for dy in -(glow_radius as i32)..=(glow_radius as i32) {
            for dx in -(glow_radius as i32)..=(glow_radius as i32) {
                let dist = ((dx * dx + dy * dy) as f64).sqrt();
                if dist > glow_radius {
                    continue;
                }
                let px = cx as i32 + dx;
                let py = cy as i32 + dy;
                if px >= 0 && px < w as i32 && py >= 0 && py < h as i32 {
                    let idx = (py as u32 * w + px as u32) as usize;
                    let intensity = ((1.0 - dist / glow_radius) * 15.0) as u8;
                    let p = &mut pixels[idx];
                    p.0 = p.0.saturating_add(intensity / 2);
                    p.1 = p.1.saturating_add(intensity / 2);
                    p.2 = p.2.saturating_add(intensity);
                }
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
                name: "twist".to_string(),
                min: 0.0,
                max: 3.0,
                value: self.twist,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "twist" => self.twist = value,
            _ => {}
        }
    }
}
