use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::app::AppMode;
use crate::ui::theme;

pub struct ModeIndicatorWidget {
    pub current: AppMode,
}

impl Widget for ModeIndicatorWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 30 || area.height < 1 {
            return;
        }

        let modes = [AppMode::Tape, AppMode::Synth, AppMode::Drum, AppMode::Mixer];
        let mut x = area.x + 1;

        for mode in &modes {
            let is_current = *mode == self.current;
            let label = format!(" {} ", mode.label());

            let style = if is_current {
                Style::default().fg(theme::BG).bg(theme::ACCENT)
            } else {
                Style::default().fg(theme::DIM)
            };

            buf.set_string(x, area.y, &label, style);
            x += label.len() as u16 + 1;
        }
    }
}
