use crate::effect::{Effect, ParamDesc};
use std::f64::consts::PI;

pub struct PixelSort {
    width: u32,
    height: u32,
    threshold: f64,
    chaos: f64,
}

impl PixelSort {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            threshold: 0.4,
            chaos: 1.0,
        }
    }

    fn plasma_color(x: f64, y: f64, t: f64, chaos: f64) -> (u8, u8, u8) {
        let tc = t * chaos;
        let v1 = (x * 0.08 + tc * 0.7).sin();
        let v2 = (y * 0.06 - tc * 0.5).sin();
        let v3 = ((x * 0.04 + y * 0.04 + tc * 0.3).sin()) * 0.8;
        let v4 = ((x * x * 0.0001 + y * y * 0.0001).sqrt() * 3.0 - tc * 0.6).sin();
        let v = (v1 + v2 + v3 + v4) * 0.25;

        let r = ((v * PI).cos() * 0.5 + 0.5) * 255.0;
        let g = ((v * PI + 2.094).cos() * 0.5 + 0.5) * 255.0;
        let b = ((v * PI + 4.189).cos() * 0.5 + 0.5) * 255.0;
        (r as u8, g as u8, b as u8)
    }

    fn brightness(c: &(u8, u8, u8)) -> f64 {
        (c.0 as f64 * 0.299 + c.1 as f64 * 0.587 + c.2 as f64 * 0.114) / 255.0
    }
}

impl Effect for PixelSort {
    fn name(&self) -> &str {
        "Pixel Sort"
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

        // Oscillating threshold
        let thresh = self.threshold + (t * 0.8).sin() * 0.15;

        // Generate base plasma image directly into pixels
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                pixels[idx] = Self::plasma_color(x as f64, y as f64, t, self.chaos);
            }
        }

        // Pixel sort each row
        for y in 0..h {
            let row_start = y * w;
            let reverse = y % 2 == 1;

            // Find runs of pixels above threshold, then sort them
            let mut x = 0;
            while x < w {
                // Skip pixels below threshold
                let b = Self::brightness(&pixels[row_start + x]);
                if b < thresh {
                    x += 1;
                    continue;
                }

                // Found start of a run
                let run_start = x;
                while x < w && Self::brightness(&pixels[row_start + x]) >= thresh {
                    x += 1;
                }
                let run_end = x;

                // Sort the run by brightness
                let slice = &mut pixels[row_start + run_start..row_start + run_end];
                if reverse {
                    slice.sort_by(|a, b| {
                        Self::brightness(b)
                            .partial_cmp(&Self::brightness(a))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                } else {
                    slice.sort_by(|a, b| {
                        Self::brightness(a)
                            .partial_cmp(&Self::brightness(b))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "threshold".to_string(),
                min: 0.2,
                max: 0.8,
                value: self.threshold,
            },
            ParamDesc {
                name: "chaos".to_string(),
                min: 0.3,
                max: 2.0,
                value: self.chaos,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "threshold" => self.threshold = value,
            "chaos" => self.chaos = value,
            _ => {}
        }
    }
}
