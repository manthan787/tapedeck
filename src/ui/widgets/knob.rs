use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::ui::theme;

pub struct KnobWidget {
    pub label: String,
    pub value: f32, // 0.0 - 1.0
    pub selected: bool,
}

impl Widget for KnobWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 7 || area.height < 4 {
            return;
        }

        let cx = area.x + area.width / 2;
        let cy = area.y + 1;

        let color = if self.selected { theme::ACCENT } else { theme::FG };

        // Draw knob as ASCII art circle with indicator
        // Top
        buf.set_string(cx - 2, cy, "╭───╮", Style::default().fg(theme::DIM));
        // Middle with indicator position
        let indicator_pos = (self.value * 4.0) as usize;
        let mid = match indicator_pos {
            0 => "│◄  │",
            1 => "│ ◄ │",
            2 => "│ ● │",
            3 => "│ ► │",
            _ => "│  ►│",
        };
        buf.set_string(cx - 2, cy + 1, mid, Style::default().fg(color));
        // Bottom
        buf.set_string(cx - 2, cy + 2, "╰───╯", Style::default().fg(theme::DIM));

        // Value display
        let val_str = format!("{:.0}%", self.value * 100.0);
        let val_x = cx - (val_str.len() as u16 / 2);
        buf.set_string(val_x, cy + 3, &val_str, Style::default().fg(color));

        // Label
        let label_x = cx.saturating_sub(self.label.len() as u16 / 2);
        if cy + 4 < area.y + area.height {
            buf.set_string(label_x, cy + 4, &self.label, Style::default().fg(theme::DIM));
        }
    }
}
