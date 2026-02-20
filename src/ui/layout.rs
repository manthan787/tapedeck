use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Main screen layout regions
pub struct ScreenLayout {
    pub header: Rect,
    pub main: Rect,
    pub footer: Rect,
}

impl ScreenLayout {
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header (mode tabs + transport)
                Constraint::Min(10),   // Main content area
                Constraint::Length(2), // Footer (key hints)
            ])
            .split(area);

        Self {
            header: chunks[0],
            main: chunks[1],
            footer: chunks[2],
        }
    }
}

/// Tape view layout: cassette on top, VU meters below
pub struct TapeLayout {
    pub cassette: Rect,
    pub vu_meters: Rect,
    pub transport: Rect,
}

impl TapeLayout {
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // Cassette animation
                Constraint::Percentage(25), // VU meters + track info
                Constraint::Length(3),       // Transport bar
            ])
            .split(area);

        Self {
            cassette: chunks[0],
            vu_meters: chunks[1],
            transport: chunks[2],
        }
    }
}

/// Mixer view layout: 4 channel strips side by side
pub struct MixerLayout {
    pub channels: [Rect; 4],
    pub master: Rect,
}

impl MixerLayout {
    pub fn new(area: Rect) -> Self {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(80),
                Constraint::Percentage(20),
            ])
            .split(area);

        let ch_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(main_chunks[0]);

        Self {
            channels: [ch_chunks[0], ch_chunks[1], ch_chunks[2], ch_chunks[3]],
            master: main_chunks[1],
        }
    }
}
