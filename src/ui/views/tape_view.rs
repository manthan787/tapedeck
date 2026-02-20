use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::AppState;
use crate::constants::TRACK_COUNT;
use crate::messages::TransportDisplay;
use crate::ui::layout::TapeLayout;
use crate::ui::theme;
use crate::ui::views::View;
use crate::ui::widgets::cassette::CassetteWidget;
use crate::ui::widgets::track_selector::TrackSelectorWidget;
use crate::ui::widgets::transport_bar::TransportBarWidget;
use crate::ui::widgets::vu_meter::VuMeterWidget;

pub struct TapeView {
    pub frame_count: u64,
}

impl TapeView {
    pub fn new() -> Self {
        Self { frame_count: 0 }
    }
}

impl View for TapeView {
    fn render(&self, state: &AppState, frame: &mut Frame, area: Rect) {
        let layout = TapeLayout::new(area);

        // Cassette animation
        let tape_pos = if state.position > 0 {
            state.position as f64 / (crate::constants::TRACK_SAMPLES as f64)
        } else {
            0.0
        };

        let spinning = matches!(
            state.transport,
            TransportDisplay::Playing | TransportDisplay::Recording
        );

        let cassette = CassetteWidget {
            tape_position: tape_pos,
            transport: state.transport,
            frame: self.frame_count,
            spinning,
        };
        frame.render_widget(cassette, layout.cassette);

        // VU meters area - split into track selector + meters
        let vu_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Track selector
                Constraint::Min(1),   // VU meters
            ])
            .split(layout.vu_meters);

        // Track selector
        let track_sel = TrackSelectorWidget {
            tracks: state.track_displays,
            selected: state.selected_track,
        };
        frame.render_widget(track_sel, vu_chunks[0]);

        // VU meters for each track
        let meter_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(vu_chunks[1]);

        for i in 0..TRACK_COUNT {
            if i < meter_chunks.len() {
                let meter = VuMeterWidget {
                    label: format!("T{} ", i + 1),
                    level: state.levels[i],
                    peak: state.peaks[i],
                    color: theme::TRACK_COLORS[i],
                };
                frame.render_widget(meter, meter_chunks[i]);
            }
        }

        // Transport bar
        let armed = state
            .track_displays
            .iter()
            .position(|t| t.armed);
        let transport = TransportBarWidget {
            state: state.transport,
            position: state.position_display(),
            armed_track: armed,
            record_source: state.record_source,
        };
        frame.render_widget(transport, layout.transport);
    }
}
