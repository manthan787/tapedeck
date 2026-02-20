use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::Paragraph;

use crate::app::AppState;
use crate::ui::theme;
use crate::ui::views::View;
use crate::ui::widgets::knob::KnobWidget;

pub struct SynthView;

impl View for SynthView {
    fn render(&self, state: &AppState, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Engine selector
                Constraint::Min(8),    // Knobs
                Constraint::Length(3), // Keyboard visualization
            ])
            .split(area);

        // Engine selector
        let engines = ["SINE", "SAW", "FM", "STRING", "NOISE"];
        let engine_str: String = engines
            .iter()
            .enumerate()
            .map(|(i, name)| {
                if i == state.synth_engine {
                    format!(" [{}] ", name)
                } else {
                    format!("  {}  ", name)
                }
            })
            .collect();

        frame.render_widget(
            Paragraph::new(engine_str).style(Style::default().fg(theme::ACCENT)),
            chunks[0],
        );

        // Parameter knobs
        let knob_area = chunks[1];
        let knob_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(knob_area);

        let param_names = ["FREQ", "RESO", "ATTACK", "DECAY"];
        for i in 0..4 {
            let knob = KnobWidget {
                label: param_names[i].to_string(),
                value: state.synth_params[i],
                selected: i == 0, // TODO: track selected param
            };
            frame.render_widget(knob, knob_chunks[i]);
        }

        // Keyboard hint
        frame.render_widget(
            Paragraph::new("  Z S X D C V G B H N J M  │  Q 2 W 3 E R 5 T 6 Y 7 U")
                .style(Style::default().fg(theme::DIM)),
            chunks[2],
        );
    }
}
