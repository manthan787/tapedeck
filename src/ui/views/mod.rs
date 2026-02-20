pub mod tape_view;
pub mod synth_view;
pub mod drum_view;
pub mod mixer_view;

use ratatui::Frame;
use ratatui::layout::Rect;
use crate::app::AppState;

pub trait View {
    fn render(&self, state: &AppState, frame: &mut Frame, area: Rect);
}
