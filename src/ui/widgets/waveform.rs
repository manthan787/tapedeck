use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::Widget;

use crate::ui::theme;

pub struct WaveformWidget {
    pub data: Vec<f32>,
    pub cursor_pos: f64,
    pub color: Color,
    pub label: String,
    pub selected: bool,
}

impl Widget for WaveformWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height < 2 {
            return;
        }

        // Label
        let label_style = if self.selected {
            Style::default().fg(self.color)
        } else {
            Style::default().fg(theme::DIM)
        };
        buf.set_string(area.x, area.y, &self.label, label_style);

        let wave_x = area.x + 4;
        let wave_width = area.width.saturating_sub(5) as usize;
        let wave_height = area.height.saturating_sub(1) as usize;
        let mid_y = area.y + (area.height / 2);

        if self.data.is_empty() || wave_width == 0 {
            // Draw empty line
            for x in wave_x..wave_x + wave_width as u16 {
                buf.set_string(x, mid_y, "─", Style::default().fg(theme::DIM));
            }
            return;
        }

        let samples_per_col = self.data.len().max(1) / wave_width.max(1);
        let half_h = wave_height as f32 / 2.0;

        for i in 0..wave_width.min(self.data.len()) {
            let idx = if samples_per_col > 0 {
                (i * samples_per_col).min(self.data.len() - 1)
            } else {
                i.min(self.data.len() - 1)
            };
            let sample = self.data[idx].clamp(0.0, 1.0);
            let height = (sample * half_h) as u16;

            let x = wave_x + i as u16;
            if height == 0 {
                buf.set_string(x, mid_y, "·", Style::default().fg(theme::DIM));
            } else {
                for dy in 0..=height {
                    if mid_y >= dy && mid_y - dy >= area.y {
                        buf.set_string(x, mid_y - dy, "│", Style::default().fg(self.color));
                    }
                }
            }
        }

        // Draw cursor
        let cursor_x = (self.cursor_pos * wave_width as f64) as u16;
        if cursor_x < wave_width as u16 {
            let cx = wave_x + cursor_x;
            for y in area.y..area.y + area.height {
                buf.set_string(cx, y, "│", Style::default().fg(Color::White));
            }
        }
    }
}
