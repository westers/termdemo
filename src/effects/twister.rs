use crate::effect::{Effect, ParamDesc};
use std::f64::consts::{FRAC_PI_2, TAU};

pub struct Twister {
    width: u32,
    height: u32,
    twist_speed: f64,
    segments: f64,
}

impl Twister {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            twist_speed: 1.5,
            segments: 8.0,
        }
    }
}

// Four face colors
const FACE_COLORS: [(u8, u8, u8); 4] = [
    (220, 50, 50),
    (50, 220, 50),
    (50, 80, 220),
    (220, 200, 50),
];

impl Effect for Twister {
    fn name(&self) -> &str {
        "Twister"
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
        let cx = wf / 2.0;
        let radius = wf * 0.38;
        let t = t * self.twist_speed;

        // Dark background
        for p in pixels.iter_mut() {
            *p = (5, 5, 15);
        }

        for y in 0..h {
            let fy = y as f64 / hf;

            // Twist angle: varies smoothly along y, animated by time.
            // An extra sine modulation makes the twist amount breathe organically.
            let twist_amount = self.segments + 2.0 * (t * 0.3).sin();
            let angle = t + fy * TAU * (twist_amount / 8.0);

            // Classic demoscene twister: 4 edge positions on a sine curve
            let edges: [f64; 4] = [
                cx + radius * (angle).sin(),
                cx + radius * (angle + FRAC_PI_2).sin(),
                cx + radius * (angle + FRAC_PI_2 * 2.0).sin(),
                cx + radius * (angle + FRAC_PI_2 * 3.0).sin(),
            ];

            let row = (y * w) as usize;

            // Draw each face between consecutive edges.
            // A face is visible (front-facing) when edge[i+1] > edge[i].
            for i in 0..4 {
                let x_left = edges[i];
                let x_right = edges[(i + 1) % 4];
                let face_width = x_right - x_left;

                if face_width <= 0.0 {
                    continue; // back-facing, skip
                }

                // Brightness from projected width: wider = facing camera more = brighter
                let brightness = (face_width / (2.0 * radius)).clamp(0.0, 1.0);
                let shade = 0.15 + 0.85 * brightness;

                let color = FACE_COLORS[i];
                let x0 = x_left.max(0.0) as i32;
                let x1 = x_right.min(wf) as i32;

                for x in x0..x1.min(w as i32) {
                    // Subtle center-bright gradient within each face for extra roundedness
                    let face_pos = if face_width > 1.0 {
                        (x as f64 - x_left) / face_width
                    } else {
                        0.5
                    };
                    let gradient = 1.0 - (face_pos * 2.0 - 1.0).powi(2) * 0.2;
                    let s = shade * gradient;

                    pixels[row + x as usize] = (
                        (color.0 as f64 * s).min(255.0) as u8,
                        (color.1 as f64 * s).min(255.0) as u8,
                        (color.2 as f64 * s).min(255.0) as u8,
                    );
                }
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "twist_speed".to_string(),
                min: 0.3,
                max: 4.0,
                value: self.twist_speed,
            },
            ParamDesc {
                name: "segments".to_string(),
                min: 4.0,
                max: 20.0,
                value: self.segments,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "twist_speed" => self.twist_speed = value,
            "segments" => self.segments = value,
            _ => {}
        }
    }
}
