use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct PendulumWave {
    width: u32,
    height: u32,
    speed: f64,
    count: f64,
}

impl PendulumWave {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            speed: 1.0,
            count: 20.0,
        }
    }

    fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
        let h = ((h % 360.0) + 360.0) % 360.0;
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;
        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        (
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }

    fn plot_pixel(
        pixels: &mut [(u8, u8, u8)],
        w: u32,
        h: u32,
        x: i32,
        y: i32,
        color: (u8, u8, u8),
        alpha: f64,
    ) {
        if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
            let idx = (y as u32 * w + x as u32) as usize;
            let old = pixels[idx];
            pixels[idx] = (
                (old.0 as f64 * (1.0 - alpha) + color.0 as f64 * alpha) as u8,
                (old.1 as f64 * (1.0 - alpha) + color.1 as f64 * alpha) as u8,
                (old.2 as f64 * (1.0 - alpha) + color.2 as f64 * alpha) as u8,
            );
        }
    }

    fn draw_filled_circle(
        pixels: &mut [(u8, u8, u8)],
        w: u32,
        h: u32,
        cx: f64,
        cy: f64,
        radius: f64,
        color: (u8, u8, u8),
    ) {
        let r = radius.ceil() as i32;
        let icx = cx as i32;
        let icy = cy as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                let dist = ((dx * dx + dy * dy) as f64).sqrt();
                if dist <= radius {
                    // Anti-alias the edge
                    let alpha = if dist > radius - 1.0 {
                        radius - dist
                    } else {
                        1.0
                    };
                    Self::plot_pixel(pixels, w, h, icx + dx, icy + dy, color, alpha.max(0.0));
                }
            }
        }
    }

    fn draw_line(
        pixels: &mut [(u8, u8, u8)],
        w: u32,
        h: u32,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        color: (u8, u8, u8),
    ) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let steps = dx.abs().max(dy.abs()).ceil() as i32;
        if steps == 0 {
            return;
        }
        let step_x = dx / steps as f64;
        let step_y = dy / steps as f64;
        for i in 0..=steps {
            let px = (x0 + step_x * i as f64) as i32;
            let py = (y0 + step_y * i as f64) as i32;
            Self::plot_pixel(pixels, w, h, px, py, color, 1.0);
        }
    }
}

impl Effect for PendulumWave {
    fn name(&self) -> &str {
        "Pendulum Wave"
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

        // Dark gradient background
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize;
                let fy = y as f64 / h as f64;
                let base = (10.0 + fy * 15.0) as u8;
                pixels[idx] = (base / 3, base / 3, base);
            }
        }

        let n = self.count as usize;
        if n == 0 {
            return;
        }
        let pivot_y = h as f64 * 0.08;
        let max_length = h as f64 * 0.75;
        let spacing = w as f64 / (n + 1) as f64;
        let t = t * self.speed;

        // Draw pivot bar
        let bar_y = pivot_y as i32;
        let bar_x0 = (spacing * 0.5) as i32;
        let bar_x1 = (w as f64 - spacing * 0.5) as i32;
        for x in bar_x0..=bar_x1 {
            Self::plot_pixel(pixels, w, h, x, bar_y, (120, 120, 140), 1.0);
            Self::plot_pixel(pixels, w, h, x, bar_y - 1, (90, 90, 110), 1.0);
        }

        // The base period, so all pendulums reconverge every ~30 seconds
        let base_period = 30.0;

        // Draw each pendulum with a subtle trail, then the current position
        for i in 0..n {
            let px = spacing * (i + 1) as f64;
            let hue = i as f64 / n as f64 * 300.0;
            let color = Self::hsv_to_rgb(hue, 0.85, 1.0);
            let dim_color = Self::hsv_to_rgb(hue, 0.6, 0.4);

            // Each pendulum has n_i+51 oscillations in base_period
            let oscillations = (15 + i) as f64;
            let period = base_period / oscillations;
            let omega = 2.0 * PI / period;

            // Pendulum length determines max swing arc
            let length = max_length * (0.5 + 0.5 * (i as f64 / n as f64));
            let max_angle = PI * 0.3;

            // Draw motion trail (a few ghost positions)
            for ghost in 1..=4 {
                let gt = t - ghost as f64 * 0.05;
                let angle = max_angle * (omega * gt).sin();
                let bob_x = px + angle.sin() * length;
                let bob_y = pivot_y + angle.cos() * length;
                let alpha = 0.15 - ghost as f64 * 0.03;
                let radius: f64 = 3.0;
                let r = radius.ceil() as i32;
                let icx = bob_x as i32;
                let icy = bob_y as i32;
                for dy in -r..=r {
                    for dx in -r..=r {
                        let dist = ((dx * dx + dy * dy) as f64).sqrt();
                        if dist <= radius {
                            Self::plot_pixel(
                                pixels,
                                w,
                                h,
                                icx + dx,
                                icy + dy,
                                dim_color,
                                alpha.max(0.0),
                            );
                        }
                    }
                }
            }

            // Current position
            let angle = max_angle * (omega * t).sin();
            let bob_x = px + angle.sin() * length;
            let bob_y = pivot_y + angle.cos() * length;

            // Draw rod/string
            Self::draw_line(pixels, w, h, px, pivot_y, bob_x, bob_y, (80, 80, 100));

            // Draw bob
            let bob_radius = 4.0_f64.min(spacing * 0.3);
            Self::draw_filled_circle(pixels, w, h, bob_x, bob_y, bob_radius, color);

            // Highlight on the bob
            Self::draw_filled_circle(
                pixels,
                w,
                h,
                bob_x - bob_radius * 0.3,
                bob_y - bob_radius * 0.3,
                bob_radius * 0.4,
                (
                    (color.0 as u16 / 2 + 128).min(255) as u8,
                    (color.1 as u16 / 2 + 128).min(255) as u8,
                    (color.2 as u16 / 2 + 128).min(255) as u8,
                ),
            );
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "speed".to_string(),
                min: 0.3,
                max: 2.0,
                value: self.speed,
            },
            ParamDesc {
                name: "count".to_string(),
                min: 10.0,
                max: 30.0,
                value: self.count,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "speed" => self.speed = value,
            "count" => self.count = value,
            _ => {}
        }
    }
}
