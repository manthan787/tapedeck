use ratatui::style::Color;

/// OP-1 inspired color palette
pub const BG: Color = Color::Rgb(20, 20, 25);
pub const FG: Color = Color::Rgb(200, 200, 210);
pub const DIM: Color = Color::Rgb(80, 80, 90);
pub const ACCENT: Color = Color::Rgb(0, 200, 150);       // Teal/green
pub const RECORD_RED: Color = Color::Rgb(220, 50, 50);
pub const PLAYING_GREEN: Color = Color::Rgb(50, 220, 100);
pub const MUTE_YELLOW: Color = Color::Rgb(220, 200, 50);
pub const SOLO_BLUE: Color = Color::Rgb(50, 150, 255);
pub const TRACK_COLORS: [Color; 4] = [
    Color::Rgb(100, 200, 255),  // Track 1: Light blue
    Color::Rgb(255, 150, 100),  // Track 2: Coral
    Color::Rgb(150, 255, 100),  // Track 3: Lime
    Color::Rgb(255, 200, 50),   // Track 4: Gold
];
pub const VU_GREEN: Color = Color::Rgb(50, 220, 80);
pub const VU_YELLOW: Color = Color::Rgb(220, 220, 50);
pub const VU_RED: Color = Color::Rgb(220, 50, 50);
pub const HEADER_BG: Color = Color::Rgb(35, 35, 45);
pub const SELECTED_BG: Color = Color::Rgb(40, 45, 55);
