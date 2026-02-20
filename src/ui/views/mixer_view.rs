use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::Paragraph;

use crate::app::AppState;
use crate::constants::TRACK_COUNT;
use crate::ui::layout::MixerLayout;
use crate::ui::theme;
use crate::ui::views::View;
use crate::ui::widgets::vu_meter::VuMeterWidget;

pub struct MixerView;

impl View for MixerView {
    fn render(&self, state: &AppState, frame: &mut Frame, area: Rect) {
        let layout = MixerLayout::new(area);

        for i in 0..TRACK_COUNT {
            render_channel_strip(state, frame, layout.channels[i], i);
        }

        // Master section
        render_master(state, frame, layout.master);
    }
}

fn render_channel_strip(state: &AppState, frame: &mut Frame, area: Rect, track: usize) {
    let is_selected = track == state.selected_track;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Track header
            Constraint::Min(4),   // Fader
            Constraint::Length(1), // Pan
            Constraint::Length(1), // VU meter
            Constraint::Length(1), // Mute/Solo
        ])
        .split(area);

    // Header
    let header_style = if is_selected {
        Style::default().fg(theme::TRACK_COLORS[track]).bg(theme::SELECTED_BG)
    } else {
        Style::default().fg(theme::TRACK_COLORS[track])
    };
    frame.render_widget(
        Paragraph::new(format!("  Track {}", track + 1)).style(header_style),
        chunks[0],
    );

    // Fader (vertical bar)
    let fader_area = chunks[1];
    let level = state.track_displays[track].level;
    let fader_height = fader_area.height;
    let filled = (level * fader_height as f32) as u16;

    for y in 0..fader_height {
        let from_bottom = fader_height - 1 - y;
        let ch = if from_bottom < filled { "█" } else { "░" };
        let color = if from_bottom < filled {
            theme::TRACK_COLORS[track]
        } else {
            theme::DIM
        };
        let x = fader_area.x + fader_area.width / 2;
        frame.render_widget(
            Paragraph::new(ch).style(Style::default().fg(color)),
            Rect::new(x, fader_area.y + y, 1, 1),
        );
    }

    // Level value
    let level_str = format!("{:.0}%", level * 100.0);
    frame.render_widget(
        Paragraph::new(level_str).style(Style::default().fg(theme::FG)),
        Rect::new(fader_area.x + 1, fader_area.y + fader_height / 2, 5, 1),
    );

    // Pan
    let pan = state.track_displays[track].pan;
    let pan_str = if pan < -0.05 {
        format!("◄{:.0}", pan.abs() * 100.0)
    } else if pan > 0.05 {
        format!("{:.0}►", pan * 100.0)
    } else {
        "  C  ".to_string()
    };
    frame.render_widget(
        Paragraph::new(format!(" {}", pan_str)).style(Style::default().fg(theme::FG)),
        chunks[2],
    );

    // VU
    let meter = VuMeterWidget {
        label: "".to_string(),
        level: state.levels[track],
        peak: state.peaks[track],
        color: theme::TRACK_COLORS[track],
    };
    frame.render_widget(meter, chunks[3]);

    // Mute/Solo
    let td = &state.track_displays[track];
    let mut status = String::from("  ");
    if td.muted {
        status.push_str("M ");
    }
    if td.solo {
        status.push_str("S");
    }
    let status_color = if td.muted {
        theme::MUTE_YELLOW
    } else if td.solo {
        theme::SOLO_BLUE
    } else {
        theme::DIM
    };
    frame.render_widget(
        Paragraph::new(status).style(Style::default().fg(status_color)),
        chunks[4],
    );
}

fn render_master(state: &AppState, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(2),
            Constraint::Length(2),
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new("  MASTER").style(Style::default().fg(theme::ACCENT)),
        chunks[0],
    );

    // Master VU L/R
    let vu_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(chunks[1]);

    let meter_l = VuMeterWidget {
        label: "L ".to_string(),
        level: state.master_level.0,
        peak: state.master_level.0 * 1.2,
        color: theme::ACCENT,
    };
    frame.render_widget(meter_l, vu_chunks[0]);

    if vu_chunks.len() > 1 {
        let meter_r = VuMeterWidget {
            label: "R ".to_string(),
            level: state.master_level.1,
            peak: state.master_level.1 * 1.2,
            color: theme::ACCENT,
        };
        frame.render_widget(meter_r, vu_chunks[1]);
    }
}
