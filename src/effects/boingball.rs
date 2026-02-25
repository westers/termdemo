use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct BoingBall {
    width: u32,
    height: u32,
    speed: f64,
    bounce_height: f64,
}

// Colors
const RED: (u8, u8, u8) = (220, 30, 30);
const WHITE: (u8, u8, u8) = (240, 240, 240);
const BG_DARK: (u8, u8, u8) = (60, 50, 80);
const BG_LINE: (u8, u8, u8) = (90, 80, 110);
const FLOOR: (u8, u8, u8) = (45, 40, 65);
const FLOOR_LINE: (u8, u8, u8) = (80, 75, 100);
const SHADOW: (u8, u8, u8) = (30, 25, 45);

impl BoingBall {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            bounce_height: 1.0,
        }
    }
}

impl Effect for BoingBall {
    fn name(&self) -> &str {
        "BoingBall"
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
        let horizon_y = (hf * 0.65) as u32;
        let ball_radius = (wf.min(hf) * 0.15).max(4.0);

        // --- Animation from t ---
        let spd = self.speed;

        // Horizontal: triangle wave ±0.35 of screen width
        let h_period = 6.0 / spd;
        let h_phase = (t / h_period) % 1.0;
        let h_tri = if h_phase < 0.5 {
            h_phase * 2.0
        } else {
            2.0 - h_phase * 2.0
        };
        let ball_x = wf * (0.15 + 0.7 * h_tri);

        // Vertical: absolute sine for parabolic bounce
        let v_period = 1.6 / spd;
        let bounce_raw = (t * PI / v_period).sin().abs();
        let bounce = bounce_raw * self.bounce_height;
        let max_rise = hf * 0.45;
        let floor_y = horizon_y as f64 - ball_radius;
        let ball_y = floor_y - bounce * max_rise;

        // Rotation
        let rot_angle = t * spd * 2.5;

        // Squash on impact
        let squash_factor = if bounce_raw < 0.1 {
            let s = bounce_raw / 0.1;
            0.85 + 0.15 * s
        } else {
            1.0
        };
        let stretch_x = 1.0 / squash_factor.sqrt();
        let rx = ball_radius * stretch_x;
        let ry = ball_radius * squash_factor;

        // --- 1. Background grid ---
        for y in 0..h {
            let row = (y * w) as usize;
            if y < horizon_y {
                // Wall
                let grid_spacing = (hf * 0.08).max(6.0);
                let gy = (y as f64 % grid_spacing) / grid_spacing;
                for x in 0..w {
                    let gx = (x as f64 % grid_spacing) / grid_spacing;
                    let on_line = gy < 0.06 || gy > 0.94 || gx < 0.06 || gx > 0.94;
                    pixels[row + x as usize] = if on_line { BG_LINE } else { BG_DARK };
                }
            } else {
                // Floor with perspective grid
                let depth = (y - horizon_y) as f64 + 1.0;
                let z = hf * 0.5 / depth;
                let floor_spacing = (hf * 0.08).max(6.0);
                let gz = (z * floor_spacing) % floor_spacing / floor_spacing;
                let fog = (1.0 - (depth / (hf * 0.35)).min(1.0)) * 0.6 + 0.4;

                for x in 0..w {
                    let wx = (x as f64 - wf * 0.5) * z * 0.3;
                    let gx = ((wx % floor_spacing) + floor_spacing) % floor_spacing / floor_spacing;
                    let on_line = gz < 0.08 || gz > 0.92 || gx < 0.08 || gx > 0.92;
                    let base = if on_line { FLOOR_LINE } else { FLOOR };
                    pixels[row + x as usize] = (
                        (base.0 as f64 * fog) as u8,
                        (base.1 as f64 * fog) as u8,
                        (base.2 as f64 * fog) as u8,
                    );
                }
            }
        }

        // --- 2. Floor shadow ---
        let shadow_cy = horizon_y as f64 + 4.0;
        let shadow_rx = rx * 1.3;
        let height_above_floor = floor_y - ball_y;
        let height_norm = (height_above_floor / max_rise).clamp(0.0, 1.0);
        let shadow_ry = (ry * 0.3) * (1.0 + height_norm * 0.8);
        let shadow_opacity = (0.8 - height_norm * 0.6).max(0.1);

        let sy0 = (shadow_cy - shadow_ry - 2.0).max(0.0) as u32;
        let sy1 = (shadow_cy + shadow_ry + 2.0).min(hf - 1.0) as u32;
        let sx0 = (ball_x - shadow_rx - 2.0).max(0.0) as u32;
        let sx1 = (ball_x + shadow_rx + 2.0).min(wf - 1.0) as u32;

        for y in sy0..=sy1 {
            let dy = (y as f64 - shadow_cy) / shadow_ry;
            let row = (y * w) as usize;
            for x in sx0..=sx1 {
                let dx = (x as f64 - ball_x) / shadow_rx;
                let d2 = dx * dx + dy * dy;
                if d2 < 1.0 {
                    let alpha = (1.0 - d2) * shadow_opacity;
                    let p = &mut pixels[row + x as usize];
                    p.0 = lerp_u8(p.0, SHADOW.0, alpha);
                    p.1 = lerp_u8(p.1, SHADOW.1, alpha);
                    p.2 = lerp_u8(p.2, SHADOW.2, alpha);
                }
            }
        }

        // --- 3. Ball (bounding rect, ray-sphere) ---
        let by0 = (ball_y - ry - 2.0).max(0.0) as u32;
        let by1 = (ball_y + ry + 2.0).min(hf - 1.0) as u32;
        let bx0 = (ball_x - rx - 2.0).max(0.0) as u32;
        let bx1 = (ball_x + rx + 2.0).min(wf - 1.0) as u32;

        // Light direction (upper-right, toward viewer)
        let light = normalize(0.5, -0.5, 0.7);

        for y in by0..=by1 {
            let ny = (y as f64 - ball_y) / ry;
            let row = (y * w) as usize;
            for x in bx0..=bx1 {
                let nx = (x as f64 - ball_x) / rx;
                let r2 = nx * nx + ny * ny;
                if r2 > 1.0 {
                    continue;
                }

                let nz = (1.0 - r2).sqrt();

                // UV mapping: latitude/longitude
                let lat = (-ny).acos();
                let lon = nz.atan2(nx) + rot_angle;

                // Checker pattern: 8 lat bands, 16 lon strips
                let lat_band = ((lat / PI) * 8.0).floor() as i32;
                let lon_strip = ((lon / PI) * 8.0).floor() as i32;
                let checker = ((lat_band + lon_strip) % 2 + 2) % 2;
                let base_color = if checker == 0 { RED } else { WHITE };

                // Diffuse lighting
                let dot = nx * light.0 + ny * light.1 + nz * light.2;
                let diffuse = dot.max(0.0);
                let shade = 0.25 + 0.75 * diffuse;

                // Specular highlight
                // Reflect light about normal: R = 2(N·L)N - L
                // View direction is (0,0,1), so specular = R_z^16
                let reflect_z = 2.0 * dot * nz - light.2;
                let spec = reflect_z.max(0.0).powf(16.0) * 0.4;

                // Edge anti-aliasing: fade in outer 5% of radius
                let edge_alpha = if r2 > 0.9025 {
                    // 0.95^2 = 0.9025
                    let edge = (1.0 - r2) / (1.0 - 0.9025);
                    edge.clamp(0.0, 1.0)
                } else {
                    1.0
                };

                let r = ((base_color.0 as f64 * shade + spec * 255.0).min(255.0)) as u8;
                let g = ((base_color.1 as f64 * shade + spec * 255.0).min(255.0)) as u8;
                let b = ((base_color.2 as f64 * shade + spec * 255.0).min(255.0)) as u8;

                if edge_alpha >= 1.0 {
                    pixels[row + x as usize] = (r, g, b);
                } else {
                    let p = &mut pixels[row + x as usize];
                    p.0 = lerp_u8(p.0, r, edge_alpha);
                    p.1 = lerp_u8(p.1, g, edge_alpha);
                    p.2 = lerp_u8(p.2, b, edge_alpha);
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
                name: "bounce_height".to_string(),
                min: 0.3,
                max: 2.0,
                value: self.bounce_height,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "bounce_height" => self.bounce_height = value,
            _ => {}
        }
    }
}

#[inline]
fn lerp_u8(a: u8, b: u8, t: f64) -> u8 {
    (a as f64 + (b as f64 - a as f64) * t) as u8
}

#[inline]
fn normalize(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let len = (x * x + y * y + z * z).sqrt();
    (x / len, y / len, z / len)
}
