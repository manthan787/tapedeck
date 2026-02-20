use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::ui::theme;

pub struct KeyboardHintWidget {
    pub hints: Vec<(&'static str, &'static str)>,
}

impl Widget for KeyboardHintWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        let mut x = area.x + 1;
        let y = area.y;

        for (key, desc) in &self.hints {
            if x + (key.len() + desc.len() + 3) as u16 > area.x + area.width {
                break;
            }
            buf.set_string(x, y, key, Style::default().fg(theme::ACCENT));
            x += key.len() as u16;
            buf.set_string(x, y, ":", Style::default().fg(theme::DIM));
            x += 1;
            buf.set_string(x, y, desc, Style::default().fg(theme::FG));
            x += desc.len() as u16 + 2;
        }
    }
}
