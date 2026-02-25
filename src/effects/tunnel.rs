use crate::effect::{Effect, ParamDesc};

pub struct Tunnel {
    width: u32,
    height: u32,
    angle_lut: Vec<f64>,
    distance_lut: Vec<f64>,
    speed: f64,
    texture_scale: f64,
}

impl Tunnel {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            angle_lut: Vec::new(),
            distance_lut: Vec::new(),
            speed: 1.0,
            texture_scale: 1.0,
        }
    }
}

impl Effect for Tunnel {
    fn name(&self) -> &str {
        "Tunnel"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let size = (width * height) as usize;
        self.angle_lut = vec![0.0; size];
        self.distance_lut = vec![0.0; size];

        let cx = width as f64 / 2.0;
        let cy = height as f64 / 2.0;

        for y in 0..height {
            for x in 0..width {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let idx = (y * width + x) as usize;

                self.angle_lut[idx] =
                    (dy.atan2(dx) / std::f64::consts::PI + 1.0) * 0.5;
                self.distance_lut[idx] =
                    32.0 / (dx * dx + dy * dy).sqrt().max(0.5);
            }
        }
    }

    fn update(&mut self, t: f64, _dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        let max_dist = (cx * cx + cy * cy).sqrt();
        let tex = self.texture_scale;

        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize;
                let angle = self.angle_lut[idx];
                let distance = self.distance_lut[idx];

                // Animate: rotation + forward motion
                let u = angle + t * self.speed * 0.1;
                let v = distance - t * self.speed * 2.0;

                // Texture: dual sine pattern
                let tex_val = (u * 8.0 * tex).sin() * (v * 8.0 * tex).sin();
                let stripe = ((u * 16.0 * tex).sin() * 0.3).abs();
                let pattern = (tex_val * 0.5 + 0.5) * 0.7 + stripe * 0.3;

                // Shade by distance from center (edge = brighter)
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                let edge_dist = (dx * dx + dy * dy).sqrt() / max_dist;
                let shade = (edge_dist * 1.5).clamp(0.1, 1.0);

                // Cosine palette with slow hue cycling
                let hue_offset = t * 0.15;
                let r = (0.5
                    + 0.5
                        * (std::f64::consts::PI * (pattern * 2.0 + hue_offset))
                            .cos())
                    * shade;
                let g = (0.5
                    + 0.5
                        * (std::f64::consts::PI
                            * (pattern * 2.0 + hue_offset + 0.33))
                            .cos())
                    * shade;
                let b = (0.5
                    + 0.5
                        * (std::f64::consts::PI
                            * (pattern * 2.0 + hue_offset + 0.67))
                            .cos())
                    * shade;

                pixels[idx] = (
                    (r.clamp(0.0, 1.0) * 255.0) as u8,
                    (g.clamp(0.0, 1.0) * 255.0) as u8,
                    (b.clamp(0.0, 1.0) * 255.0) as u8,
                );
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "speed".to_string(),
                min: 0.2,
                max: 5.0,
                value: self.speed,
            },
            ParamDesc {
                name: "tex_scale".to_string(),
                min: 0.3,
                max: 4.0,
                value: self.texture_scale,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "tex_scale" => self.texture_scale = value,
            _ => {}
        }
    }
}
