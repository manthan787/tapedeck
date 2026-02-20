use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

use crate::ui::theme;

pub struct VuMeterWidget {
    pub label: String,
    pub level: f32,
    pub peak: f32,
    pub color: Color,
}

impl Widget for VuMeterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height < 1 {
            return;
        }

        let label_width = 4;
        let bar_start = area.x + label_width;
        let bar_width = area.width.saturating_sub(label_width + 1);

        // Draw label
        let label = format!("{:<4}", &self.label);
        buf.set_string(area.x, area.y, &label, Style::default().fg(self.color));

        // Draw bar background
        for x in bar_start..bar_start + bar_width {
            buf.set_string(x, area.y, "░", Style::default().fg(theme::DIM));
        }

        // Draw level
        let level_width = (self.level * bar_width as f32) as u16;
        for x in bar_start..bar_start + level_width.min(bar_width) {
            let frac = (x - bar_start) as f32 / bar_width as f32;
            let color = if frac < 0.6 {
                theme::VU_GREEN
            } else if frac < 0.85 {
                theme::VU_YELLOW
            } else {
                theme::VU_RED
            };
            buf.set_string(x, area.y, "█", Style::default().fg(color));
        }

        // Draw peak hold
        let peak_pos = (self.peak * bar_width as f32) as u16;
        if peak_pos > 0 && peak_pos <= bar_width {
            buf.set_string(
                bar_start + peak_pos - 1,
                area.y,
                "│",
                Style::default().fg(Color::White),
            );
        }
    }
}
