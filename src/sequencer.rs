use crate::effect::Effect;
use crate::scene::Scene;
use crate::transition::apply_transition;

pub struct Sequencer {
    pub scenes: Vec<Scene>,
    pub current: usize,
    pub scene_time: f64,
    pub global_time: f64,
    pub paused: bool,
    pub looping: bool,
    transitioning: bool,
    transition_elapsed: f64,
    prev_frame: Vec<(u8, u8, u8)>,
    next_frame: Vec<(u8, u8, u8)>,
    width: u32,
    height: u32,
}

impl Sequencer {
    pub fn new(scenes: Vec<Scene>, looping: bool) -> Self {
        Self {
            scenes,
            current: 0,
            scene_time: 0.0,
            global_time: 0.0,
            paused: false,
            looping,
            transitioning: false,
            transition_elapsed: 0.0,
            prev_frame: Vec::new(),
            next_frame: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    pub fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let len = (width * height) as usize;
        self.prev_frame.resize(len, (0, 0, 0));
        self.next_frame.resize(len, (0, 0, 0));
        if let Some(scene) = self.scenes.get_mut(self.current) {
            scene.effect.init(width, height);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let len = (width * height) as usize;
        self.prev_frame.resize(len, (0, 0, 0));
        self.next_frame.resize(len, (0, 0, 0));
        if let Some(scene) = self.scenes.get_mut(self.current) {
            scene.effect.init(width, height);
        }
    }

    pub fn current_scene_name(&self) -> &str {
        self.scenes
            .get(self.current)
            .map(|s| s.effect.name())
            .unwrap_or("---")
    }

    pub fn scene_count(&self) -> usize {
        self.scenes.len()
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn goto_scene(&mut self, index: usize) {
        if index >= self.scenes.len() || index == self.current {
            return;
        }
        self.start_transition(index);
    }

    pub fn next_scene(&mut self) {
        if self.scenes.is_empty() {
            return;
        }
        let next = if self.current + 1 >= self.scenes.len() {
            if self.looping {
                0
            } else {
                return;
            }
        } else {
            self.current + 1
        };
        self.start_transition(next);
    }

    pub fn prev_scene(&mut self) {
        if self.scenes.is_empty() {
            return;
        }
        let prev = if self.current == 0 {
            if self.looping {
                self.scenes.len() - 1
            } else {
                return;
            }
        } else {
            self.current - 1
        };
        self.start_transition(prev);
    }

    fn start_transition(&mut self, next_index: usize) {
        // Snapshot current frame into prev_frame
        self.transitioning = true;
        self.transition_elapsed = 0.0;

        // prev_frame already holds the last rendered output
        // init next scene
        let next_scene = &mut self.scenes[next_index];
        next_scene.effect.init(self.width, self.height);
        self.current = next_index;
        self.scene_time = 0.0;
    }

    pub fn update(&mut self, dt: f64, pixels: &mut [(u8, u8, u8)]) {
        if self.paused || self.scenes.is_empty() {
            return;
        }

        self.global_time += dt;
        self.scene_time += dt;

        let current = self.current;

        if self.transitioning {
            self.transition_elapsed += dt;
            let scene = &self.scenes[current];
            let duration = scene.transition_duration;
            let progress = (self.transition_elapsed / duration).min(1.0);

            // Render the new scene into next_frame
            self.next_frame.resize(pixels.len(), (0, 0, 0));
            self.scenes[current]
                .effect
                .update(self.scene_time, dt, &mut self.next_frame);

            // Blend prev_frame -> next_frame into output
            let kind = self.scenes[current].transition_in;
            apply_transition(
                kind,
                &self.prev_frame,
                &self.next_frame,
                pixels,
                self.width,
                self.height,
                progress,
            );

            if progress >= 1.0 {
                self.transitioning = false;
            }
        } else {
            // Normal rendering
            self.scenes[current]
                .effect
                .update(self.scene_time, dt, pixels);

            // Snapshot for potential upcoming transition
            self.prev_frame.resize(pixels.len(), (0, 0, 0));
            self.prev_frame.copy_from_slice(pixels);

            // Check if scene duration expired
            if let Some(dur) = self.scenes[current].duration {
                if self.scene_time >= dur {
                    self.next_scene();
                }
            }
        }
    }

    pub fn current_effect_mut(&mut self) -> Option<&mut Box<dyn Effect>> {
        self.scenes.get_mut(self.current).map(|s| &mut s.effect)
    }

    #[allow(dead_code)]
    pub fn is_transitioning(&self) -> bool {
        self.transitioning
    }
}
