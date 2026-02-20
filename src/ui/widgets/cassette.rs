use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::canvas::{Canvas, Circle, Line};
use ratatui::widgets::Widget;

use crate::messages::TransportDisplay;
use crate::ui::theme;

pub struct CassetteWidget {
    /// Current tape position as fraction 0.0 - 1.0
    pub tape_position: f64,
    /// Transport state for color
    pub transport: TransportDisplay,
    /// Animation frame counter (for reel rotation)
    pub frame: u64,
    /// Is transport active (reels spinning)
    pub spinning: bool,
}

impl Widget for CassetteWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let tape_pos = self.tape_position.clamp(0.0, 1.0);

        // Reel radii: left reel shrinks as tape advances, right grows
        let max_radius = 12.0;
        let min_radius = 4.0;
        let range = max_radius - min_radius;
        let left_radius = max_radius - (tape_pos * range);
        let right_radius = min_radius + (tape_pos * range);

        // Canvas coordinate system
        let cx = area.width as f64;
        let cy = area.height as f64 * 2.0; // braille doubles vertical res

        let left_cx = cx * 0.30;
        let right_cx = cx * 0.70;
        let reel_cy = cy * 0.45;

        // Reel rotation angle
        let angle = if self.spinning {
            (self.frame as f64) * 0.15
        } else {
            0.0
        };

        let reel_color = match self.transport {
            TransportDisplay::Recording => theme::RECORD_RED,
            TransportDisplay::Playing => theme::PLAYING_GREEN,
            TransportDisplay::Paused => theme::DIM,
            TransportDisplay::Stopped => theme::DIM,
        };

        let tape_color = match self.transport {
            TransportDisplay::Recording => Color::Rgb(180, 40, 40),
            TransportDisplay::Playing => Color::Rgb(40, 160, 80),
            _ => Color::Rgb(60, 60, 70),
        };

        let canvas = Canvas::default()
            .x_bounds([0.0, cx])
            .y_bounds([0.0, cy])
            .marker(ratatui::symbols::Marker::Braille)
            .paint(move |ctx| {
                // Cassette body outline
                let body_x1 = cx * 0.08;
                let body_x2 = cx * 0.92;
                let body_y1 = cy * 0.10;
                let body_y2 = cy * 0.85;

                // Top edge
                ctx.draw(&Line {
                    x1: body_x1, y1: body_y2,
                    x2: body_x2, y2: body_y2,
                    color: theme::DIM,
                });
                // Bottom edge
                ctx.draw(&Line {
                    x1: body_x1, y1: body_y1,
                    x2: body_x2, y2: body_y1,
                    color: theme::DIM,
                });
                // Left edge
                ctx.draw(&Line {
                    x1: body_x1, y1: body_y1,
                    x2: body_x1, y2: body_y2,
                    color: theme::DIM,
                });
                // Right edge
                ctx.draw(&Line {
                    x1: body_x2, y1: body_y1,
                    x2: body_x2, y2: body_y2,
                    color: theme::DIM,
                });

                // Left reel (circle)
                ctx.draw(&Circle {
                    x: left_cx, y: reel_cy,
                    radius: left_radius,
                    color: reel_color,
                });

                // Right reel (circle)
                ctx.draw(&Circle {
                    x: right_cx, y: reel_cy,
                    radius: right_radius,
                    color: reel_color,
                });

                // Hub circles (inner)
                ctx.draw(&Circle {
                    x: left_cx, y: reel_cy,
                    radius: 2.5,
                    color: tape_color,
                });
                ctx.draw(&Circle {
                    x: right_cx, y: reel_cy,
                    radius: 2.5,
                    color: tape_color,
                });

                // Spokes on left reel (3 lines)
                for i in 0..3 {
                    let spoke_angle = angle + (i as f64) * std::f64::consts::TAU / 3.0;
                    let dx = spoke_angle.cos() * left_radius * 0.8;
                    let dy = spoke_angle.sin() * left_radius * 0.8;
                    ctx.draw(&Line {
                        x1: left_cx, y1: reel_cy,
                        x2: left_cx + dx, y2: reel_cy + dy,
                        color: reel_color,
                    });
                }

                // Spokes on right reel
                for i in 0..3 {
                    let spoke_angle = angle + (i as f64) * std::f64::consts::TAU / 3.0;
                    let dx = spoke_angle.cos() * right_radius * 0.8;
                    let dy = spoke_angle.sin() * right_radius * 0.8;
                    ctx.draw(&Line {
                        x1: right_cx, y1: reel_cy,
                        x2: right_cx + dx, y2: reel_cy + dy,
                        color: reel_color,
                    });
                }

                // Tape path: left reel → head → right reel
                let head_y = cy * 0.18;
                let head_x = cx * 0.50;

                // Tape from left reel to head guide (left)
                ctx.draw(&Line {
                    x1: left_cx, y1: reel_cy - left_radius,
                    x2: cx * 0.25, y2: head_y,
                    color: tape_color,
                });
                // Guide to head
                ctx.draw(&Line {
                    x1: cx * 0.25, y1: head_y,
                    x2: head_x, y2: head_y,
                    color: tape_color,
                });
                // Head to right guide
                ctx.draw(&Line {
                    x1: head_x, y1: head_y,
                    x2: cx * 0.75, y2: head_y,
                    color: tape_color,
                });
                // Right guide to right reel
                ctx.draw(&Line {
                    x1: cx * 0.75, y1: head_y,
                    x2: right_cx, y2: reel_cy - right_radius,
                    color: tape_color,
                });

                // Head marker
                ctx.draw(&Line {
                    x1: head_x - 1.0, y1: head_y - 1.5,
                    x2: head_x + 1.0, y2: head_y - 1.5,
                    color: theme::ACCENT,
                });
                ctx.draw(&Line {
                    x1: head_x - 1.0, y1: head_y + 1.5,
                    x2: head_x + 1.0, y2: head_y + 1.5,
                    color: theme::ACCENT,
                });

                // Label
                ctx.print(cx * 0.38, cy * 0.92, ratatui::text::Line::from("TAPEDECK").style(
                    ratatui::style::Style::default().fg(theme::ACCENT),
                ));
            });

        canvas.render(area, buf);
    }
}
