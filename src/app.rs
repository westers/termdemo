use std::time::Instant;

use crate::framebuffer::PixelFramebuffer;
use crate::input::{self, Action};
use crate::sequencer::Sequencer;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    AutoPlay,
    Interactive,
}

pub struct App {
    pub fb: PixelFramebuffer,
    pub sequencer: Sequencer,
    pub mode: Mode,
    pub show_hud: bool,
    pub selected_param: usize,
    pub should_quit: bool,
    last_frame: Instant,
}

impl App {
    pub fn new(sequencer: Sequencer, mode: Mode) -> Self {
        Self {
            fb: PixelFramebuffer::new(0, 0),
            sequencer,
            mode,
            show_hud: mode == Mode::Interactive,
            selected_param: 0,
            should_quit: false,
            last_frame: Instant::now(),
        }
    }

    pub fn init(&mut self, width: u32, height: u32) {
        self.fb.resize(width, height);
        self.sequencer.init(width, height);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.fb.resize(width, height);
        self.sequencer.resize(width, height);
    }

    pub fn handle_input(&mut self) -> std::io::Result<()> {
        match input::poll_action()? {
            Action::Quit => self.should_quit = true,
            Action::TogglePause => self.sequencer.toggle_pause(),
            Action::ToggleMode => {
                self.mode = match self.mode {
                    Mode::AutoPlay => {
                        self.sequencer.looping = false;
                        self.show_hud = true;
                        Mode::Interactive
                    }
                    Mode::Interactive => {
                        self.sequencer.looping = true;
                        self.show_hud = false;
                        Mode::AutoPlay
                    }
                };
            }
            Action::NextScene => {
                self.sequencer.next_scene();
                self.selected_param = 0;
            }
            Action::PrevScene => {
                self.sequencer.prev_scene();
                self.selected_param = 0;
            }
            Action::GotoScene(idx) => {
                self.sequencer.goto_scene(idx);
                self.selected_param = 0;
            }
            Action::ToggleHud => self.show_hud = !self.show_hud,
            Action::ToggleHold => self.sequencer.toggle_hold(),
            Action::ParamUp => self.adjust_param(0.05),
            Action::ParamDown => self.adjust_param(-0.05),
            Action::ParamPrev => {
                self.selected_param = self.selected_param.saturating_sub(1);
            }
            Action::ParamNext => {
                if let Some(effect) = self.sequencer.current_effect_mut() {
                    let count = effect.params().len();
                    if count > 0 {
                        self.selected_param = (self.selected_param + 1).min(count - 1);
                    }
                }
            }
            Action::None => {}
        }
        Ok(())
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;
        self.sequencer.update(dt, &mut self.fb.pixels);
    }

    fn adjust_param(&mut self, delta: f64) {
        if self.mode != Mode::Interactive {
            return;
        }
        if let Some(effect) = self.sequencer.current_effect_mut() {
            let params = effect.params();
            if let Some(param) = params.get(self.selected_param) {
                let new_val = (param.value + delta).clamp(param.min, param.max);
                let name = param.name.clone();
                effect.set_param(&name, new_val);
            }
        }
    }
}
