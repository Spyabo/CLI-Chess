use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::pieces::{Color as PieceColour, Piece, PieceType};

/// Get Unicode chess symbol for a piece
fn get_piece_char(piece: &Piece) -> char {
    match (piece.piece_type, piece.color) {
        (PieceType::King, PieceColour::White) => '♔',
        (PieceType::Queen, PieceColour::White) => '♕',
        (PieceType::Rook, PieceColour::White) => '♖',
        (PieceType::Bishop, PieceColour::White) => '♗',
        (PieceType::Knight, PieceColour::White) => '♘',
        (PieceType::Pawn, PieceColour::White) => '♙',
        (PieceType::King, PieceColour::Black) => '♚',
        (PieceType::Queen, PieceColour::Black) => '♛',
        (PieceType::Rook, PieceColour::Black) => '♜',
        (PieceType::Bishop, PieceColour::Black) => '♝',
        (PieceType::Knight, PieceColour::Black) => '♞',
        (PieceType::Pawn, PieceColour::Black) => '♟',
        (PieceType::Empty, _) => ' ',
    }
}

/// Calculate the material value of a piece
fn piece_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 1,
        PieceType::Knight => 3,
        PieceType::Bishop => 3,
        PieceType::Rook => 5,
        PieceType::Queen => 9,
        PieceType::King => 0,  // Kings can't be captured
        PieceType::Empty => 0,
    }
}

/// A widget that displays captured pieces for one side
pub struct CapturedPiecesBar<'a> {
    captured: &'a [Piece],
    label: &'a str,
    material_advantage: i32,
}

impl<'a> CapturedPiecesBar<'a> {
    pub fn new(captured: &'a [Piece], label: &'a str, material_advantage: i32) -> Self {
        Self {
            captured,
            label,
            material_advantage,
        }
    }
}

impl Widget for CapturedPiecesBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width < 10 {
            return;
        }

        // Sort pieces by value for nicer display (queens first, then rooks, etc.)
        let mut sorted_pieces: Vec<&Piece> = self.captured.iter().collect();
        sorted_pieces.sort_by(|a, b| {
            piece_value(b.piece_type).cmp(&piece_value(a.piece_type))
        });

        // Build the display string
        let mut display = format!("{}: ", self.label);

        for piece in &sorted_pieces {
            display.push(get_piece_char(piece));
        }

        // Add material advantage if positive
        if self.material_advantage > 0 {
            display.push_str(&format!(" +{}", self.material_advantage));
        }

        // Render the string
        let style = Style::default().fg(Color::White);
        let x = area.x;
        let y = area.y;

        for (i, ch) in display.chars().enumerate() {
            if x + i as u16 >= area.x + area.width {
                break;
            }
            buf.get_mut(x + i as u16, y).set_char(ch).set_style(style);
        }
    }
}

/// Calculate total material value for a list of pieces
pub fn calculate_material(pieces: &[Piece]) -> i32 {
    pieces.iter().map(|p| piece_value(p.piece_type)).sum()
}
