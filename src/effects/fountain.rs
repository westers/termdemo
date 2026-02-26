use crate::effect::{Effect, ParamDesc};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: f64, // 0–1, decreasing
}

pub struct Fountain {
    width: u32,
    height: u32,
    gravity: f64,
    emission: f64,
    particles: Vec<Particle>,
    emit_accum: f64,
    rng: StdRng,
}

impl Fountain {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            gravity: 1.0,
            emission: 80.0,
            particles: Vec::new(),
            emit_accum: 0.0,
            rng: StdRng::seed_from_u64(0),
        }
    }
}

const MAX_PARTICLES: usize = 500;

impl Effect for Fountain {
    fn name(&self) -> &str {
        "Fountain"
    }

    fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.particles.clear();
        self.emit_accum = 0.0;
    }

    fn randomize_init(&mut self, rng: &mut StdRng) {
        self.rng = StdRng::seed_from_u64(rng.gen());
    }

    fn update(&mut self, _t: f64, dt: f64, pixels: &mut [(u8, u8, u8)]) {
        let w = self.width;
        let h = self.height;
        if w == 0 || h == 0 {
            return;
        }

        let wf = w as f64;
        let hf = h as f64;

        // Fade existing pixels for trails
        for p in pixels.iter_mut() {
            p.0 = p.0.saturating_sub(12);
            p.1 = p.1.saturating_sub(12);
            p.2 = p.2.saturating_sub(15);
        }

        // Emit new particles from bottom-center
        self.emit_accum += dt * self.emission;
        while self.emit_accum >= 1.0 && self.particles.len() < MAX_PARTICLES {
            self.emit_accum -= 1.0;
            let angle = self.rng.gen_range(-0.4..0.4);
            let speed = self.rng.gen_range(150.0..300.0);
            self.particles.push(Particle {
                x: wf * 0.5 + self.rng.gen_range(-3.0..3.0),
                y: hf - 1.0,
                vx: angle * speed,
                vy: -speed,
                life: 1.0,
            });
        }

        // Update particles
        let gravity = self.gravity * 200.0;
        self.particles.retain_mut(|p| {
            p.vy += gravity * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt * 0.5;
            p.life > 0.0
        });

        // Draw particles
        for p in &self.particles {
            let ix = p.x as i32;
            let iy = p.y as i32;
            if ix < 0 || ix >= w as i32 || iy < 0 || iy >= h as i32 {
                continue;
            }

            // Color by life: white → yellow → orange → red → dark
            let (cr, cg, cb) = life_color(p.life);

            let idx = (iy as u32 * w + ix as u32) as usize;
            if idx < pixels.len() {
                let px = &mut pixels[idx];
                px.0 = px.0.max(cr);
                px.1 = px.1.max(cg);
                px.2 = px.2.max(cb);
            }
        }
    }

    fn params(&self) -> Vec<ParamDesc> {
        vec![
            ParamDesc {
                name: "gravity".to_string(),
                min: 0.3,
                max: 3.0,
                value: self.gravity,
            },
            ParamDesc {
                name: "emission".to_string(),
                min: 20.0,
                max: 200.0,
                value: self.emission,
            },
        ]
    }

    fn set_param(&mut self, name: &str, value: f64) {
        match name {
            "gravity" => self.gravity = value,
            "emission" => self.emission = value,
            _ => {}
        }
    }
}

fn life_color(life: f64) -> (u8, u8, u8) {
    if life > 0.75 {
        // White to yellow
        let t = (life - 0.75) / 0.25;
        (255, 255, (255.0 * t) as u8)
    } else if life > 0.5 {
        // Yellow to orange
        let t = (life - 0.5) / 0.25;
        (255, (200.0 * t + 55.0 * (1.0 - t)) as u8, 0)
    } else if life > 0.25 {
        // Orange to red
        let t = (life - 0.25) / 0.25;
        ((255.0 * t + 100.0 * (1.0 - t)) as u8, (55.0 * t) as u8, 0)
    } else {
        // Red to dark
        let t = life / 0.25;
        ((100.0 * t) as u8, 0, 0)
    }
}
