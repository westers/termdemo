use crate::effect::Effect;
use crate::transition::TransitionKind;

pub struct Scene {
    pub effect: Box<dyn Effect>,
    pub duration: Option<f64>,
    pub transition_in: TransitionKind,
    pub transition_duration: f64,
}

impl Scene {
    pub fn new(effect: Box<dyn Effect>) -> Self {
        Self {
            effect,
            duration: None,
            transition_in: TransitionKind::Dissolve,
            transition_duration: 1.5,
        }
    }

    pub fn with_duration(mut self, secs: f64) -> Self {
        self.duration = Some(secs);
        self
    }

    pub fn with_transition(mut self, kind: TransitionKind, duration: f64) -> Self {
        self.transition_in = kind;
        self.transition_duration = duration;
        self
    }
}
