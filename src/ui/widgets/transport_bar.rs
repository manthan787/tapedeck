use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

use crate::messages::{RecordSource, TransportDisplay};
use crate::ui::theme;

pub struct TransportBarWidget {
    pub state: TransportDisplay,
    pub position: String,
    pub armed_track: Option<usize>,
    pub record_source: RecordSource,
}

impl Widget for TransportBarWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 {
            return;
        }

        let y = area.y;

        // Transport icon
        let (icon, icon_color) = match self.state {
            TransportDisplay::Stopped => ("■ STOP", theme::DIM),
            TransportDisplay::Playing => ("▶ PLAY", theme::PLAYING_GREEN),
            TransportDisplay::Recording => ("● REC ", theme::RECORD_RED),
            TransportDisplay::Paused => ("❚❚PAUSE", theme::MUTE_YELLOW),
        };

        buf.set_string(area.x + 1, y, icon, Style::default().fg(icon_color));

        // Position counter
        let pos_str = format!("  {}  ", self.position);
        buf.set_string(
            area.x + 12,
            y,
            &pos_str,
            Style::default().fg(theme::ACCENT),
        );

        // Armed track + source indicator
        let mut x = area.x + 28;
        if let Some(track) = self.armed_track {
            let arm_str = format!("ARM:T{}", track + 1);
            buf.set_string(x, y, &arm_str, Style::default().fg(theme::RECORD_RED));
            x += arm_str.len() as u16 + 1;
        }

        // Recording source
        let src_str = format!("SRC:{}", self.record_source.label());
        buf.set_string(x, y, &src_str, Style::default().fg(theme::ACCENT));
    }
}
