use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::ui::theme;

pub struct StepGridWidget {
    /// patterns[instrument][step]
    pub patterns: [[bool; 16]; 6],
    pub current_step: usize,
    pub selected_instrument: usize,
    pub instrument_names: [&'static str; 6],
}

impl Widget for StepGridWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 30 || area.height < 7 {
            return;
        }

        let start_x = area.x + 6;
        let step_width = ((area.width - 7) / 16).max(1);

        for (inst, name) in self.instrument_names.iter().enumerate() {
            let y = area.y + inst as u16;
            if y >= area.y + area.height {
                break;
            }

            // Instrument label
            let label_color = if inst == self.selected_instrument {
                theme::ACCENT
            } else {
                theme::DIM
            };
            buf.set_string(area.x, y, &format!("{:<5}", name), Style::default().fg(label_color));

            // Steps
            for step in 0..16 {
                let x = start_x + (step as u16 * step_width);
                if x >= area.x + area.width {
                    break;
                }

                let is_active = self.patterns[inst][step];
                let is_current = step == self.current_step;

                let (ch, color) = if is_active && is_current {
                    ("█", theme::ACCENT)
                } else if is_active {
                    ("■", theme::TRACK_COLORS[inst % 4])
                } else if is_current {
                    ("▪", theme::PLAYING_GREEN)
                } else if step % 4 == 0 {
                    ("·", theme::FG)
                } else {
                    ("·", theme::DIM)
                };

                buf.set_string(x, y, ch, Style::default().fg(color));
            }
        }
    }
}
