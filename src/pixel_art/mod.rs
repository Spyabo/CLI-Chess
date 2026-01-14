mod board_widget;
mod captured_bar;
mod colours;
mod game_over_modal;
mod move_history;
mod sprites;

pub use board_widget::{calculate_board_layout, PixelArtBoard};
pub use captured_bar::{calculate_material, CapturedPiecesBar};
pub use game_over_modal::{centered_rect, GameOverModal};
pub use move_history::MoveHistoryPanel;
pub use sprites::PieceSprites;

use ratatui::style::Color;
use crate::pieces::Color as PieceColour;

/// Represents a single pixel colour for the piece artwork
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pixel {
    /// Transparent - shows the square background colour
    Transparent,
    /// Primary piece colour (white or black depending on the piece)
    Primary,
    /// Secondary/outline colour for contrast
    Outline,
    /// Accent colour for details (e.g., cross on king)
    Accent,
}

/// Resolves a Pixel enum to an actual RGB colour
pub fn resolve_pixel_colour(
    pixel: Pixel,
    piece_colour: PieceColour,
    square_bg: Color,
) -> Color {
    match pixel {
        Pixel::Transparent => square_bg,
        Pixel::Primary => match piece_colour {
            PieceColour::White => Color::Rgb(255, 255, 255),
            PieceColour::Black => Color::Rgb(40, 40, 40),
        },
        Pixel::Outline => match piece_colour {
            // Outline provides contrast - opposite of primary
            PieceColour::White => Color::Rgb(80, 80, 80),
            PieceColour::Black => Color::Rgb(200, 200, 200),
        },
        Pixel::Accent => Color::Rgb(218, 165, 32), // Gold for crown details
    }
}

/// Converts two vertical pixels into a single character with appropriate styling
/// Returns (character, foreground colour, background colour)
pub fn pixels_to_char(
    upper: Pixel,
    lower: Pixel,
    piece_colour: PieceColour,
    square_bg: Color,
) -> (char, Color, Color) {
    let upper_colour = resolve_pixel_colour(upper, piece_colour, square_bg);
    let lower_colour = resolve_pixel_colour(lower, piece_colour, square_bg);

    if upper_colour == lower_colour {
        // Both same colour - use full block or space
        if upper == Pixel::Transparent {
            (' ', Color::Reset, upper_colour) // (char, fg, bg)
        } else {
            ('\u{2588}', upper_colour, square_bg) // Full block
        }
    } else {
        // Different colours - use upper half block
        // Upper half block with fg=upper, bg=lower
        ('\u{2580}', upper_colour, lower_colour)
    }
}
