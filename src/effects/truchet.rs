use crate::effect::{Effect, ParamDesc};

pub struct Truchet {
    width: u32,
    height: u32,
    tile_size: f64,
    morph_speed: f64,
}

impl Truchet {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            tile_size: 20.0,
            morph_speed: 0.5,
        }
    }
}

impl Effect for Truchet {
    fn name(&self) -> &str {
        "Truchet"
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
        let tile = self.tile_size;
        let line_thickness = tile * 0.15; // ~3px at default tile size

        for y in 0..h {
            for x in 0..w {
                let fx = x as f64;
                let fy = y as f64;

                // Which tile are we in?
                let tx = (fx / tile).floor();
                let ty = (fy / tile).floor();

                // Position within tile [0, tile)
                let lx = fx - tx * tile;
                let ly = fy - ty * tile;

                // Noise function to determine tile orientation
                // Uses a smooth noise that evolves with time
                let noise_val = smooth_noise(
                    tx * 0.7 + t * self.morph_speed * 0.3,
                    ty * 0.7 + t * self.morph_speed * 0.2,
                    t * self.morph_speed * 0.1,
                );

                // Two orientations:
                // Type A: arcs from top-left and bottom-right corners
                // Type B: arcs from top-right and bottom-left corners
                let is_type_a = noise_val > 0.0;

                // Compute distance to the nearest arc
                let half = tile / 2.0;
                let dist = if is_type_a {
                    // Arcs centered at (0,0) and (tile,tile)
                    let d1 = ((lx * lx + ly * ly).sqrt() - half).abs();
                    let d2 = (((lx - tile) * (lx - tile) + (ly - tile) * (ly - tile)).sqrt() - half).abs();
                    d1.min(d2)
                } else {
                    // Arcs centered at (tile,0) and (0,tile)
                    let d1 = (((lx - tile) * (lx - tile) + ly * ly).sqrt() - half).abs();
                    let d2 = ((lx * lx + (ly - tile) * (ly - tile)).sqrt() - half).abs();
                    d1.min(d2)
                };

                let idx = (y * w + x) as usize;

                if dist < line_thickness {
                    // On the arc — color with smooth rainbow gradient
                    let intensity = 1.0 - (dist / line_thickness);
                    let intensity = intensity * intensity; // sharper edges

                    // Rainbow based on screen position
                    let hue = ((fx + fy) / (wf + hf) * 2.0 + t * 0.1) % 1.0;
                    let (r, g, b) = hsv_to_rgb(hue, 0.8, 0.3 + 0.7 * intensity);
                    pixels[idx] = (r, g, b);
                } else {
                    // Dark background with subtle tile grid hint
                    let edge_dist = lx.min(ly).min(tile - lx).min(tile - ly);
                    let grid_hint = if edge_dist < 0.8 { 15 } else { 6 };
                    pixels[idx] = (grid_hint, grid_hint, grid_hint + 4);
                }
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "tile_size".to_string(),
                min: 10.0,
                max: 40.0,
                value: self.tile_size,
            },
            ParamDesc {
                name: "morph_speed".to_string(),
                min: 0.2,
                max: 2.0,
                value: self.morph_speed,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "tile_size" => self.tile_size = value,
            "morph_speed" => self.morph_speed = value,
            _ => {}
        }
    }
}

/// Simple smooth 3D noise using layered sine waves (value noise approximation)
fn smooth_noise(x: f64, y: f64, z: f64) -> f64 {
    let v = (x * 1.0 + y * 1.7 + z * 0.3).sin() * 0.5
        + (x * 2.3 - y * 0.9 + z * 1.1).sin() * 0.25
        + (x * 0.5 + y * 3.1 - z * 0.7).cos() * 0.25;
    v
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
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
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}
