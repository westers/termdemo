use crate::effect::{Effect, ParamDesc};


pub struct Terrain {
    width: u32,
    height: u32,
    speed: f64,
    roughness: f64,
}

impl Terrain {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            roughness: 1.0,
        }
    }

    /// Compute terrain height at world (x, z) using layered sine waves.
    fn terrain_height(&self, x: f64, z: f64) -> f64 {
        let r = self.roughness;
        let mut h = 0.0;
        h += (x * 0.031 * r + z * 0.047 * r).sin() * 1.0;
        h += (x * 0.067 * r - z * 0.073 * r + 1.3).sin() * 0.5;
        h += (x * 0.113 * r + z * 0.097 * r + 2.7).sin() * 0.25;
        h += (x * 0.191 * r - z * 0.157 * r + 4.1).sin() * 0.125;
        h += ((x * 0.051 * r).sin() * (z * 0.043 * r).cos()) * 0.6;
        h
    }

    /// Color by elevation: water -> grass -> hills -> snow.
    fn terrain_color(h: f64) -> (f64, f64, f64) {
        if h < -0.6 {
            // Deep water
            (0.1, 0.2, 0.6)
        } else if h < -0.2 {
            // Shallow water
            let t = (h + 0.6) / 0.4;
            (0.1 + t * 0.05, 0.2 + t * 0.15, 0.6 + t * 0.1)
        } else if h < 0.0 {
            // Beach / low grass
            let t = (h + 0.2) / 0.2;
            (0.15 + t * 0.05, 0.35 + t * 0.2, 0.1 + t * 0.05)
        } else if h < 0.5 {
            // Grass
            let t = h / 0.5;
            (0.2 + t * 0.15, 0.55 - t * 0.1, 0.15 - t * 0.05)
        } else if h < 1.0 {
            // Brown hills / rock
            let t = (h - 0.5) / 0.5;
            (0.35 + t * 0.2, 0.3 + t * 0.05, 0.1 + t * 0.15)
        } else {
            // Snow peaks
            let t = ((h - 1.0) / 0.5).min(1.0);
            (0.55 + t * 0.4, 0.35 + t * 0.6, 0.25 + t * 0.7)
        }
    }
}

impl Effect for Terrain {
    fn name(&self) -> &str {
        "Terrain"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width as usize;
        let h = self.height as usize;
        if w == 0 || h == 0 {
            return;
        }

        let sky_top: (f64, f64, f64) = (0.35, 0.55, 0.85);
        let sky_bottom: (f64, f64, f64) = (0.65, 0.78, 0.95);
        let fog_color: (f64, f64, f64) = (0.6, 0.72, 0.88);

        // Fill sky gradient
        for y in 0..h {
            let frac = y as f64 / h as f64;
            let r = sky_top.0 + (sky_bottom.0 - sky_top.0) * frac;
            let g = sky_top.1 + (sky_bottom.1 - sky_top.1) * frac;
            let b = sky_top.2 + (sky_bottom.2 - sky_top.2) * frac;
            let r8 = (r * 255.0) as u8;
            let g8 = (g * 255.0) as u8;
            let b8 = (b * 255.0) as u8;
            for x in 0..w {
                pixels[y * w + x] = (r8, g8, b8);
            }
        }

        // Camera scrolls forward along Z
        let cam_z = t * self.speed * 40.0;
        let cam_y = 3.0; // camera height
        let horizon = h as f64 * 0.35; // horizon line
        let fov = 1.2;
        let max_dist = 200.0;

        // For each screen column, cast rays from horizon downward
        for sx in 0..w {
            let screen_x = (sx as f64 / w as f64 - 0.5) * fov;

            let mut max_drawn_sy = h; // occlusion: track highest drawn pixel

            // March from near to far
            let mut dist = 1.0;
            while dist < max_dist {
                let world_x = screen_x * dist * 10.0;
                let world_z = cam_z + dist;

                let terrain_h = self.terrain_height(world_x, world_z);

                // Project: screen_y = horizon + (cam_y - terrain_h) / dist * scale
                let projected = (cam_y - terrain_h) / dist * (h as f64) * 0.8;
                let screen_y = (horizon + projected) as usize;

                if screen_y < max_drawn_sy && screen_y < h {
                    let (cr, cg, cb) = Self::terrain_color(terrain_h);

                    // Distance fog
                    let fog_t = (dist / max_dist).powi(2).clamp(0.0, 1.0);
                    let fr = cr * (1.0 - fog_t) + fog_color.0 * fog_t;
                    let fg = cg * (1.0 - fog_t) + fog_color.1 * fog_t;
                    let fb = cb * (1.0 - fog_t) + fog_color.2 * fog_t;

                    let r8 = (fr * 255.0).clamp(0.0, 255.0) as u8;
                    let g8 = (fg * 255.0).clamp(0.0, 255.0) as u8;
                    let b8 = (fb * 255.0).clamp(0.0, 255.0) as u8;

                    for sy in screen_y..max_drawn_sy {
                        if sy < h {
                            pixels[sy * w + sx] = (r8, g8, b8);
                        }
                    }
                    max_drawn_sy = screen_y;
                }

                dist += 0.3 + dist * 0.015;
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
                name: "roughness".to_string(),
                min: 0.3,
                max: 2.0,
                value: self.roughness,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "roughness" => self.roughness = value,
            _ => {}
        }
    }
}
