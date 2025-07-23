use std::fmt;
use std::ops::Not;
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString)]
pub enum Color {
    #[strum(serialize = "White")]
    White,
    #[strum(serialize = "Black")]
    Black,
}

impl Not for Color {
    type Output = Self;
    
    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, Default)]
pub enum PieceType {
    #[strum(serialize = "p")]
    Pawn,
    #[strum(serialize = "r")]
    Rook,
    #[strum(serialize = "n")]
    Knight,
    #[strum(serialize = "b")]
    Bishop,
    #[strum(serialize = "q")]
    Queen,
    #[strum(serialize = "k")]
    King,
    #[default]
    Empty,
}



#[derive(Debug, Clone, Copy)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
    pub has_moved: bool,
    pub moves_made: u32,
}

impl Default for Piece {
    fn default() -> Self {
        Self {
            piece_type: PieceType::Empty,
            color: Color::White,  // Default color for empty squares, won't be used
            has_moved: false,
            moves_made: 0,
        }
    }
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Self {
            piece_type,
            color,
            has_moved: false,
            moves_made: 0,
        }
    }

    pub fn from_fen(c: char) -> Option<Self> {
        let color = if c.is_uppercase() {
            Color::White
        } else {
            Color::Black
        };

        let piece_type = match c.to_ascii_lowercase() as char {
            'p' => PieceType::Pawn,
            'r' => PieceType::Rook,
            'n' => PieceType::Knight,
            'b' => PieceType::Bishop,
            'q' => PieceType::Queen,
            'k' => PieceType::King,
            _ => return None,
        };

        Some(Self::new(piece_type, color))
    }

    /// Convert the piece to its FEN character representation
    pub fn to_char(&self) -> char {
        let c = match self.piece_type {
            PieceType::Pawn => 'p',
            PieceType::Rook => 'r',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
            PieceType::Empty => ' ',
        };
        
        match self.color {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        }
    }
    
    pub fn to_unicode(&self) -> &'static str {
        match (self.piece_type, self.color) {
            (PieceType::Pawn, Color::White) => "♟",
            (PieceType::Rook, Color::White) => "♜",
            (PieceType::Knight, Color::White) => "♞",
            (PieceType::Bishop, Color::White) => "♝",
            (PieceType::Queen, Color::White) => "♛",
            (PieceType::King, Color::White) => "♚",
            (PieceType::Pawn, Color::Black) => "♙",
            (PieceType::Rook, Color::Black) => "♖",
            (PieceType::Knight, Color::Black) => "♘",
            (PieceType::Bishop, Color::Black) => "♗",
            (PieceType::Queen, Color::Black) => "♕",
            (PieceType::King, Color::Black) => "♔",
            _ => "·",
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_unicode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_creation() {
        let white_pawn = Piece::new(PieceType::Pawn, Color::White);
        assert_eq!(white_pawn.piece_type, PieceType::Pawn);
        assert_eq!(white_pawn.color, Color::White);
        assert!(!white_pawn.has_moved);
        assert_eq!(white_pawn.moves_made, 0);
    }

    #[test]
    fn test_from_fen() {
        let white_king = Piece::from_fen('K').unwrap();
        assert_eq!(white_king.piece_type, PieceType::King);
        assert_eq!(white_king.color, Color::White);

        let black_queen = Piece::from_fen('q').unwrap();
        assert_eq!(black_queen.piece_type, PieceType::Queen);
        assert_eq!(black_queen.color, Color::Black);

        assert!(Piece::from_fen('x').is_none());
    }

    #[test]
    fn test_unicode_display() {
        let white_rook = Piece::new(PieceType::Rook, Color::White);
        assert_eq!(white_rook.to_unicode(), "♜");

        let black_knight = Piece::new(PieceType::Knight, Color::Black);
        assert_eq!(black_knight.to_unicode(), "♘");
    }
}
