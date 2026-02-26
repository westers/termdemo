use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

pub enum Action {
    Quit,
    TogglePause,
    ToggleMode,
    NextScene,
    PrevScene,
    GotoScene(usize),
    ToggleHud,
    ToggleHold,
    ParamUp,
    ParamDown,
    ParamPrev,
    ParamNext,
    None,
}

pub fn poll_action() -> std::io::Result<Action> {
    if event::poll(Duration::ZERO)? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                return Ok(match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
                    KeyCode::Char(' ') => Action::TogglePause,
                    KeyCode::Tab => Action::ToggleMode,
                    KeyCode::Char('n') | KeyCode::Right => Action::NextScene,
                    KeyCode::Char('p') | KeyCode::Left => Action::PrevScene,
                    KeyCode::Char('h') => Action::ToggleHud,
                    KeyCode::Char('f') => Action::ToggleHold,
                    KeyCode::Up => Action::ParamUp,
                    KeyCode::Down => Action::ParamDown,
                    KeyCode::Char('[') => Action::ParamPrev,
                    KeyCode::Char(']') => Action::ParamNext,
                    KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                        Action::GotoScene((c as usize) - ('1' as usize))
                    }
                    _ => Action::None,
                });
            }
        }
    }
    Ok(Action::None)
}
