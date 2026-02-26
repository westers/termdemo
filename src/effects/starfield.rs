use crate::effect::{Effect, ParamDesc};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const NUM_STARS: usize = 400;

struct Star {
    x: f64,
    y: f64,
    z: f64,
    prev_sx: f64,
    prev_sy: f64,
}

pub struct Starfield {
    width: u32,
    height: u32,
    stars: Vec<Star>,
    speed: f64,
    rng: StdRng,
}

impl Starfield {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            stars: Vec::new(),
            speed: 1.0,
            rng: StdRng::seed_from_u64(0),
        }
    }

    fn spawn_star(rng: &mut impl Rng) -> Star {
        Star {
            x: rng.gen_range(-1.0..1.0),
            y: rng.gen_range(-1.0..1.0),
            z: rng.gen_range(0.1..1.0),
            prev_sx: 0.0,
            prev_sy: 0.0,
        }
    }
}

impl Effect for Starfield {
    fn name(&self) -> &str {
        "Starfield"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.stars.clear();
    }

    fn randomize_init(&mut self, rng: &mut StdRng) {
        self.rng = StdRng::seed_from_u64(rng.gen());
        self.stars.clear();
        for _ in 0..NUM_STARS {
            self.stars.push(Self::spawn_star(&mut self.rng));
        }
    }

    fn update(&mut self, _t: f64, dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        // Fade existing pixels slightly for motion trails
        for pixel in pixels.iter_mut() {
            pixel.0 = pixel.0.saturating_sub(20);
            pixel.1 = pixel.1.saturating_sub(20);
            pixel.2 = pixel.2.saturating_sub(25);
        }

        let cx = w as f64 / 2.0;
        let cy = h as f64 / 2.0;

        for star in &mut self.stars {
            star.z -= dt * self.speed * 0.5;

            if star.z <= 0.01 {
                *star = Self::spawn_star(&mut self.rng);
                star.z = 1.0;
                let sx = star.x / star.z * cx + cx;
                let sy = star.y / star.z * cy + cy;
                star.prev_sx = sx;
                star.prev_sy = sy;
                continue;
            }

            let sx = star.x / star.z * cx + cx;
            let sy = star.y / star.z * cy + cy;

            // Brightness based on depth (closer = brighter)
            let brightness = ((1.0 - star.z) * 255.0).clamp(40.0, 255.0) as u8;

            // Draw a short trail from prev to current position
            let steps = 4;
            for i in 0..=steps {
                let t = i as f64 / steps as f64;
                let px = star.prev_sx + (sx - star.prev_sx) * t;
                let py = star.prev_sy + (sy - star.prev_sy) * t;
                let ix = px as i32;
                let iy = py as i32;

                if ix >= 0 && ix < w as i32 && iy >= 0 && iy < h as i32 {
                    let idx = (iy as u32 * w + ix as u32) as usize;
                    if idx < pixels.len() {
                        let trail_bright =
                            (brightness as f64 * (0.3 + 0.7 * t)) as u8;
                        let existing = pixels[idx];
                        pixels[idx] = (
                            existing.0.max(trail_bright),
                            existing.1.max(trail_bright),
                            existing.2.max(trail_bright),
                        );
                    }
                }
            }

            star.prev_sx = sx;
            star.prev_sy = sy;
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![ParamDesc {
            name: "speed".to_string(),
            min: 0.2,
            max: 5.0,
            value: self.speed,
        }]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        if name == "speed" {
            self.speed = value;
        }
    }
}
