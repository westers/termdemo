use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

/// 16x16 map: 1 = wall, 0 = empty
const MAP_SIZE: usize = 16;
#[rustfmt::skip]
const MAP: [u8; MAP_SIZE * MAP_SIZE] = [
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,1,1,0,0,0,0,0,1,1,0,0,0,1,
    1,0,0,1,0,0,0,0,0,0,0,1,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,1,1,1,0,0,0,0,0,0,1,
    1,0,0,0,0,0,1,0,1,0,0,0,0,0,0,1,
    1,0,0,0,0,0,1,0,1,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,1,0,0,0,0,0,0,0,1,0,0,0,1,
    1,0,0,1,1,0,0,0,0,0,1,1,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,
    1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
];

pub struct Wolfenstein {
    width: u32,
    height: u32,
    move_speed: f64,
    fov: f64,
}

impl Wolfenstein {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            move_speed: 1.0,
            fov: 60.0,
        }
    }
}

fn map_at(mx: i32, my: i32) -> u8 {
    if mx < 0 || mx >= MAP_SIZE as i32 || my < 0 || my >= MAP_SIZE as i32 {
        return 1;
    }
    MAP[my as usize * MAP_SIZE + mx as usize]
}

impl Effect for Wolfenstein {
    fn name(&self) -> &str {
        "Wolfenstein"
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
        let t_move = t * self.move_speed;

        // Camera position: orbit around the center of the map
        let center_x = MAP_SIZE as f64 / 2.0;
        let center_y = MAP_SIZE as f64 / 2.0;
        let orbit_radius = 3.5;
        let orbit_speed = 0.3;

        // Camera orbits and looks inward with some variation
        let cam_angle = t_move * orbit_speed;
        let cam_x = center_x + cam_angle.cos() * orbit_radius;
        let cam_y = center_y + cam_angle.sin() * orbit_radius;

        // Look direction: mostly toward center with some rotation
        let look_angle = cam_angle + PI + (t_move * 0.7).sin() * 0.4;

        let fov_rad = self.fov * PI / 180.0;
        let half_fov = fov_rad / 2.0;

        // Draw ceiling and floor
        for y in 0..h {
            let fy = y as f64 / hf;
            for x in 0..w {
                let idx = (y * w + x) as usize;
                if fy < 0.5 {
                    // Ceiling: dark gray-blue
                    let shade = (30.0 + (0.5 - fy) * 40.0) as u8;
                    pixels[idx] = (shade / 3, shade / 3, shade);
                } else {
                    // Floor: dark green-brown
                    let shade = (20.0 + (fy - 0.5) * 50.0) as u8;
                    pixels[idx] = (shade / 2, shade, shade / 3);
                }
            }
        }

        // Raycast for each column
        for x in 0..w {
            let camera_x = 2.0 * x as f64 / wf - 1.0; // -1 to +1
            let ray_angle = look_angle + camera_x * half_fov;
            let ray_dir_x = ray_angle.cos();
            let ray_dir_y = ray_angle.sin();

            // DDA raycasting
            let mut map_x = cam_x.floor() as i32;
            let mut map_y = cam_y.floor() as i32;

            let delta_dist_x = if ray_dir_x.abs() < 1e-10 {
                1e10
            } else {
                (1.0 / ray_dir_x).abs()
            };
            let delta_dist_y = if ray_dir_y.abs() < 1e-10 {
                1e10
            } else {
                (1.0 / ray_dir_y).abs()
            };

            let step_x: i32;
            let step_y: i32;
            let mut side_dist_x: f64;
            let mut side_dist_y: f64;

            if ray_dir_x < 0.0 {
                step_x = -1;
                side_dist_x = (cam_x - map_x as f64) * delta_dist_x;
            } else {
                step_x = 1;
                side_dist_x = (map_x as f64 + 1.0 - cam_x) * delta_dist_x;
            }
            if ray_dir_y < 0.0 {
                step_y = -1;
                side_dist_y = (cam_y - map_y as f64) * delta_dist_y;
            } else {
                step_y = 1;
                side_dist_y = (map_y as f64 + 1.0 - cam_y) * delta_dist_y;
            }

            // Perform DDA
            let mut hit = false;
            let mut side = 0; // 0 = x-side, 1 = y-side
            let max_steps = 64;

            for _ in 0..max_steps {
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    map_x += step_x;
                    side = 0;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y += step_y;
                    side = 1;
                }
                if map_at(map_x, map_y) != 0 {
                    hit = true;
                    break;
                }
            }

            if !hit {
                continue;
            }

            // Calculate perpendicular wall distance (avoids fisheye)
            let perp_dist = if side == 0 {
                (map_x as f64 - cam_x + (1 - step_x) as f64 / 2.0) / ray_dir_x
            } else {
                (map_y as f64 - cam_y + (1 - step_y) as f64 / 2.0) / ray_dir_y
            };

            let perp_dist = perp_dist.abs().max(0.01);

            // Wall strip height
            let line_height = (hf / perp_dist).min(hf * 4.0);
            let draw_start = ((hf / 2.0 - line_height / 2.0).max(0.0)) as u32;
            let draw_end = ((hf / 2.0 + line_height / 2.0).min(hf - 1.0)) as u32;

            // Wall color: different for N/S vs E/W sides
            let base_color: (f64, f64, f64) = if side == 0 {
                // E/W walls: reddish brick
                (180.0, 70.0, 50.0)
            } else {
                // N/S walls: darker shade
                (120.0, 50.0, 35.0)
            };

            // Distance-based darkening
            let dist_factor = (1.0 / (1.0 + perp_dist * 0.15)).clamp(0.15, 1.0);

            // Calculate where on the wall the ray hit (for texture variation)
            let wall_x = if side == 0 {
                cam_y + perp_dist * ray_dir_y
            } else {
                cam_x + perp_dist * ray_dir_x
            };
            let wall_x = wall_x - wall_x.floor();

            // Simple vertical stripe pattern on walls
            let stripe = if (wall_x * 8.0) as i32 % 2 == 0 {
                1.0
            } else {
                0.85
            };

            let r = (base_color.0 * dist_factor * stripe).min(255.0) as u8;
            let g = (base_color.1 * dist_factor * stripe).min(255.0) as u8;
            let b = (base_color.2 * dist_factor * stripe).min(255.0) as u8;

            for y in draw_start..=draw_end {
                if y < h {
                    let idx = (y * w + x) as usize;
                    if idx < pixels.len() {
                        pixels[idx] = (r, g, b);
                    }
                }
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "move_speed".to_string(),
                min: 0.3,
                max: 2.0,
                value: self.move_speed,
            },
            ParamDesc {
                name: "fov".to_string(),
                min: 40.0,
                max: 120.0,
                value: self.fov,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "move_speed" => self.move_speed = value,
            "fov" => self.fov = value,
            _ => {}
        }
    }
}
