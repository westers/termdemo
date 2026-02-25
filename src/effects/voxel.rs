use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

const MAP_SIZE: usize = 1024;

pub struct VoxelLandscape {
    width: u32,
    height: u32,
    speed: f64,
    cam_height: f64,
    heightmap: Vec<f64>,
    colormap: Vec<(u8, u8, u8)>,
}

impl VoxelLandscape {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            cam_height: 1.5,
            heightmap: Vec::new(),
            colormap: Vec::new(),
        }
    }

    fn generate_terrain(&mut self) {
        let size = MAP_SIZE;
        self.heightmap = vec![0.0; size * size];
        self.colormap = vec![(0, 0, 0); size * size];

        // Multi-octave sine-based heightmap
        for y in 0..size {
            for x in 0..size {
                let fx = x as f64 / size as f64;
                let fy = y as f64 / size as f64;

                let mut h = 0.0;
                h += (fx * 3.0 * PI).sin() * (fy * 2.0 * PI).cos() * 0.4;
                h += (fx * 7.0 * PI + 1.0).sin() * (fy * 5.0 * PI + 2.0).cos() * 0.2;
                h += (fx * 13.0 * PI + 3.0).sin() * (fy * 11.0 * PI + 5.0).cos() * 0.1;
                h += (fx * 23.0 * PI).cos() * (fy * 19.0 * PI).sin() * 0.05;
                h = h * 0.5 + 0.5; // normalize to 0–1

                let idx = y * size + x;
                self.heightmap[idx] = h;

                // Altitude coloring
                self.colormap[idx] = if h < 0.3 {
                    // Water
                    let d = h / 0.3;
                    ((20.0 * d) as u8, (40.0 + 40.0 * d) as u8, (120.0 + 80.0 * d) as u8)
                } else if h < 0.5 {
                    // Grass
                    let d = (h - 0.3) / 0.2;
                    ((40.0 + 30.0 * d) as u8, (120.0 + 40.0 * d) as u8, (30.0 + 20.0 * d) as u8)
                } else if h < 0.75 {
                    // Rock
                    let d = (h - 0.5) / 0.25;
                    let v = 100.0 + 50.0 * d;
                    (v as u8, (v * 0.9) as u8, (v * 0.8) as u8)
                } else {
                    // Snow
                    let d = (h - 0.75) / 0.25;
                    let v = (200.0 + 55.0 * d).min(255.0);
                    (v as u8, v as u8, v as u8)
                };
            }
        }
    }
}

impl Effect for VoxelLandscape {
    fn name(&self) -> &str {
        "VoxelLandscape"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.generate_terrain();
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width as usize;
        let h = self.height as usize;
        if w == 0 || h == 0 || self.heightmap.is_empty() {
            return;
        }

        // Sky color
        let sky: (u8, u8, u8) = (100, 140, 200);
        for p in pixels.iter_mut() {
            *p = sky;
        }

        let t_scaled = t * self.speed;

        // Camera position: moves forward
        let cam_x = (t_scaled * 60.0) % (MAP_SIZE as f64);
        let cam_y = (t_scaled * 30.0 + MAP_SIZE as f64 * 0.3) % (MAP_SIZE as f64);
        let cam_z = self.cam_height * 120.0 + (t_scaled * 0.5).sin() * 15.0;
        let cam_angle = t_scaled * 0.1;

        let cos_a = cam_angle.cos();
        let sin_a = cam_angle.sin();

        // Per screen column: Comanche-style raycasting
        for sx in 0..w {
            // Ray direction in screen space
            let rx = (sx as f64 / w as f64 - 0.5) * 2.0;
            // Rotate ray direction
            let dir_x = rx * cos_a - sin_a;
            let dir_y = rx * sin_a + cos_a;

            let mut max_screen_y = h; // front-to-back occlusion

            // March forward with increasing step size
            let mut dist = 1.0;
            let max_dist = 400.0;

            while dist < max_dist {
                let world_x = cam_x + dir_x * dist;
                let world_y = cam_y + dir_y * dist;

                // Sample heightmap with wrapping
                let mx = ((world_x as isize).rem_euclid(MAP_SIZE as isize)) as usize;
                let my = ((world_y as isize).rem_euclid(MAP_SIZE as isize)) as usize;
                let map_idx = my * MAP_SIZE + mx;

                let terrain_h = self.heightmap[map_idx] * 120.0;

                // Project to screen: higher terrain or closer = higher on screen
                let height_on_screen = (cam_z - terrain_h) / dist * (h as f64) * 0.5;
                let screen_y = (h as f64 * 0.5 + height_on_screen) as usize;

                if screen_y < max_screen_y {
                    // Get terrain color
                    let base_color = self.colormap[map_idx];

                    // Distance fog
                    let fog = (dist / max_dist).clamp(0.0, 1.0);
                    let r = (base_color.0 as f64 * (1.0 - fog) + sky.0 as f64 * fog) as u8;
                    let g = (base_color.1 as f64 * (1.0 - fog) + sky.1 as f64 * fog) as u8;
                    let b = (base_color.2 as f64 * (1.0 - fog) + sky.2 as f64 * fog) as u8;

                    // Draw vertical column from screen_y to max_screen_y
                    for sy in screen_y..max_screen_y {
                        if sy < h {
                            let idx = sy * w + sx;
                            if idx < pixels.len() {
                                pixels[idx] = (r, g, b);
                            }
                        }
                    }

                    max_screen_y = screen_y;
                }

                // Increasing step size for performance
                dist += 0.5 + dist * 0.01;
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
                name: "cam_height".to_string(),
                min: 0.5,
                max: 3.0,
                value: self.cam_height,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "cam_height" => self.cam_height = value,
            _ => {}
        }
    }
}
