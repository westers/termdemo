use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::app::{App, Mode};

pub struct HudWidget<'a> {
    pub app: &'a App,
}

impl<'a> Widget for HudWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 3 || area.width < 20 {
            return;
        }

        let seq = &self.app.sequencer;
        let mode_str = match self.app.mode {
            Mode::AutoPlay => "AUTO",
            Mode::Interactive => "INTERACTIVE",
        };

        let paused = if seq.paused { " [PAUSED]" } else { "" };
        let held = if seq.held { " [HELD]" } else { "" };

        // Status bar at bottom
        let bar_y = area.y + area.height - 1;
        let status = format!(
            " Scene {}/{}: {} | Mode: {}{}{} | t={:.1}s ",
            seq.current + 1,
            seq.scene_count(),
            seq.current_scene_name(),
            mode_str,
            paused,
            held,
            seq.scene_time,
        );

        let bar_style = Style::default()
            .fg(Color::White)
            .bg(Color::Rgb(30, 30, 60));

        // Fill status bar background
        for x in area.x..area.x + area.width {
            let cell = buf.get_mut(x, bar_y);
            cell.set_symbol(" ");
            cell.set_style(bar_style);
        }

        // Write status text
        for (i, ch) in status.chars().enumerate() {
            let x = area.x + i as u16;
            if x >= area.x + area.width {
                break;
            }
            let cell = buf.get_mut(x, bar_y);
            cell.set_symbol(&ch.to_string());
            cell.set_style(bar_style);
        }

        // Controls hint on the right side
        let hint = "q:quit Space:pause f:hold Tab:mode h:hud [/]:param n/p:scene";
        let hint_start = (area.x + area.width).saturating_sub(hint.len() as u16 + 1);
        let hint_style = Style::default()
            .fg(Color::Rgb(140, 140, 180))
            .bg(Color::Rgb(30, 30, 60));
        for (i, ch) in hint.chars().enumerate() {
            let x = hint_start + i as u16;
            if x >= area.x + area.width || x < area.x + status.len() as u16 {
                continue;
            }
            let cell = buf.get_mut(x, bar_y);
            cell.set_symbol(&ch.to_string());
            cell.set_style(hint_style);
        }

        // Parameter panel (interactive mode only, if effect has params)
        if self.app.mode == Mode::Interactive {
            if let Some(effect) = self.app.sequencer.scenes.get(seq.current) {
                let params = effect.effect.params();
                if !params.is_empty() {
                    let panel_y = bar_y.saturating_sub(params.len() as u16 + 1);
                    let panel_x = area.x + 1;

                    // Panel header
                    if panel_y > area.y {
                        let header = " Parameters ([/] select, Up/Down adjust) ";
                        let header_style = Style::default()
                            .fg(Color::Yellow)
                            .bg(Color::Rgb(20, 20, 40))
                            .add_modifier(Modifier::BOLD);
                        for (i, ch) in header.chars().enumerate() {
                            let x = panel_x + i as u16;
                            if x < area.x + area.width {
                                let cell = buf.get_mut(x, panel_y);
                                cell.set_symbol(&ch.to_string());
                                cell.set_style(header_style);
                            }
                        }
                    }

                    for (pi, param) in params.iter().enumerate() {
                        let y = panel_y + 1 + pi as u16;
                        if y >= bar_y || y <= area.y {
                            continue;
                        }

                        let selected = pi == self.app.selected_param;
                        let marker = if selected { ">" } else { " " };
                        let line = format!(
                            "{} {}: {:.2} [{:.1}..{:.1}]",
                            marker, param.name, param.value, param.min, param.max
                        );

                        let style = if selected {
                            Style::default()
                                .fg(Color::Cyan)
                                .bg(Color::Rgb(20, 20, 40))
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                                .fg(Color::White)
                                .bg(Color::Rgb(20, 20, 40))
                        };

                        for (i, ch) in line.chars().enumerate() {
                            let x = panel_x + i as u16;
                            if x < area.x + area.width {
                                let cell = buf.get_mut(x, y);
                                cell.set_symbol(&ch.to_string());
                                cell.set_style(style);
                            }
                        }
                    }
                }
            }
        }
    }
}
