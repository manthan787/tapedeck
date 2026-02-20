use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::messages::TrackDisplay;
use crate::ui::theme;

pub struct TrackSelectorWidget {
    pub tracks: [TrackDisplay; 4],
    pub selected: usize,
}

impl Widget for TrackSelectorWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 1 {
            return;
        }

        let y = area.y;
        let mut x = area.x + 1;

        for (i, track) in self.tracks.iter().enumerate() {
            let is_sel = i == self.selected;

            // Track number
            let num_style = if is_sel {
                Style::default().fg(theme::TRACK_COLORS[i])
            } else {
                Style::default().fg(theme::DIM)
            };
            buf.set_string(x, y, &format!("T{}", i + 1), num_style);
            x += 2;

            // Status indicators
            if track.armed {
                buf.set_string(x, y, "●", Style::default().fg(theme::RECORD_RED));
            }
            x += 1;
            if track.muted {
                buf.set_string(x, y, "M", Style::default().fg(theme::MUTE_YELLOW));
            }
            x += 1;
            if track.solo {
                buf.set_string(x, y, "S", Style::default().fg(theme::SOLO_BLUE));
            }
            x += 2;
        }
    }
}
