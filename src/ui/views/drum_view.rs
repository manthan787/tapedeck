use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::Paragraph;

use crate::app::AppState;
use crate::ui::theme;
use crate::ui::views::View;
use crate::ui::widgets::step_grid::StepGridWidget;

pub struct DrumView;

impl View for DrumView {
    fn render(&self, state: &AppState, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // BPM + info
                Constraint::Min(8),   // Step grid
                Constraint::Length(2), // Controls hint
            ])
            .split(area);

        // BPM display
        let bpm_str = format!(
            "  BPM: {:.0}  │  Step: {:2}/16  │  Inst: {}",
            state.bpm,
            state.current_step + 1,
            ["KICK", "SNARE", "HAT", "CLAP", "TOM", "RIM"][state.selected_instrument],
        );
        frame.render_widget(
            Paragraph::new(bpm_str).style(Style::default().fg(theme::ACCENT)),
            chunks[0],
        );

        // Step grid
        let grid = StepGridWidget {
            patterns: state.drum_patterns,
            current_step: state.current_step,
            selected_instrument: state.selected_instrument,
            instrument_names: ["KICK", "SNR ", "HAT ", "CLAP", "TOM ", "RIM "],
        };
        frame.render_widget(grid, chunks[1]);

        // Controls
        frame.render_widget(
            Paragraph::new("  Z-K:Toggle Steps  1-4:Instrument  ↑/↓:BPM  R:Record")
                .style(Style::default().fg(theme::DIM)),
            chunks[2],
        );
    }
}
