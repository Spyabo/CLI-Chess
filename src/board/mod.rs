mod position;

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

use crate::pieces::{Color, Piece, PieceType};
pub use position::Position;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    pub from: Position,
    pub to: Position,
    pub promotion: Option<PieceType>,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} to {}", self.from, self.to)
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub board: Board,
    pub active_color: Color,
    pub selected_square: Option<Position>,
    pub valid_moves: HashSet<Position>,
    pub check: bool,
    pub checkmate: bool,
    pub stalemate: bool,
}

impl GameState {
    pub fn new() -> Self {
        let mut board = Board::default();
        board.load_fen(STARTING_FEN).unwrap();
        
        let mut game_state = Self {
            board,
            active_color: Color::White,
            selected_square: None,
            valid_moves: HashSet::new(),
            check: false,
            checkmate: false,
            stalemate: false,
        };
        
        // Update the game state based on the initial position
        game_state.update_state();
        game_state
    }
    
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let board = Board::from_fen(fen)?;
        let active_color = board.active_color;
        
        let mut game_state = Self {
            board,
            active_color,
            selected_square: None,
            valid_moves: HashSet::new(),
            check: false,
            checkmate: false,
            stalemate: false,
        };
        
        // Update the game state based on the FEN position
        game_state.update_state();
        Ok(game_state)
    }
    
    /// Updates the game state (check, checkmate, stalemate)
    fn update_state(&mut self) {
        // Check if the current player is in check
        self.check = self.board.is_in_check(self.active_color);
        
        // Check for checkmate or stalemate
        let has_legal_moves = self.has_any_legal_moves();
        
        if !has_legal_moves {
            if self.check {
                self.checkmate = true;
            } else {
                self.stalemate = true;
            }
        } else {
            self.checkmate = false;
            self.stalemate = false;
        }
    }
    
    /// Checks if the current player has any legal moves
    fn has_any_legal_moves(&self) -> bool {
        for (pos, piece) in &self.board.squares {
            if piece.color == self.active_color {
                let moves = self.board.get_legal_moves(*pos);
                if !moves.is_empty() {
                    return true;
                }
            }
        }
        false
    }
    
    /// Makes a move and updates the game state
    pub fn make_move(&mut self, from: Position, to: Position) -> Result<(), String> {
        // Save the current state for potential undo
        let original_state = self.board.clone();
        
        // Try to make the move
        if let Err(e) = self.board.move_piece(from, to) {
            return Err(e);
        }
        
        // Toggle the active color
        self.active_color = !self.active_color;
        
        // Update the game state
        self.update_state();
        
        // If the move leaves the king in check, it's illegal
        if self.board.is_in_check(!self.active_color) {
            // Revert the move
            self.board = original_state;
            self.active_color = !self.active_color; // Toggle back
            return Err("Move would leave king in check".to_string());
        }
        
        // Clear the selected square and valid moves
        self.selected_square = None;
        self.valid_moves.clear();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_state() {
        let game_state = GameState::new();
        assert!(game_state.selected_square.is_none());
        assert!(game_state.valid_moves.is_empty());
        assert!(!game_state.check);
        assert!(!game_state.checkmate);
        assert!(!game_state.stalemate);
    }

    #[test]
    fn test_from_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let game_state = GameState::from_fen(fen).unwrap();
        assert!(game_state.selected_square.is_none());
        assert!(game_state.valid_moves.is_empty());
        assert!(!game_state.check);
        assert!(!game_state.checkmate);
        assert!(!game_state.stalemate);
    }
    
    #[test]
    fn test_check_detection() {
        // Fool's mate position
        let fen = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3";
        let game_state = GameState::from_fen(fen).unwrap();
        assert!(game_state.check);
        assert!(!game_state.checkmate);
        assert!(!game_state.stalemate);
    }
    
    #[test]
    fn test_checkmate_detection() {
        // Fool's mate
        let fen = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3";
        let mut game_state = GameState::from_fen(fen).unwrap();
        game_state.update_state();
        assert!(game_state.check);
        assert!(game_state.checkmate);
        assert!(!game_state.stalemate);
    }
    
    #[test]
    fn test_stalemate_detection() {
        // Basic stalemate position
        let fen = "k7/8/8/8/8/8/6q1/7K b - - 0 1";
        let mut game_state = GameState::from_fen(fen).unwrap();
        game_state.update_state();
        assert!(!game_state.check);
        assert!(!game_state.checkmate);
        assert!(game_state.stalemate);
    }
}

const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone)]
pub struct Board {
    squares: HashMap<Position, Piece>,
    active_color: Color,
    castling_rights: String,
    en_passant_target: Option<Position>,
    halfmove_clock: u32,
    fullmove_number: u32,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            squares: HashMap::new(),
            active_color: Color::White,
            castling_rights: "KQkq".to_string(),
            en_passant_target: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self::default();
        board.load_fen(STARTING_FEN).expect("Failed to load starting position");
        board
    }
    
    pub fn load_fen(&mut self, fen: &str) -> Result<(), String> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty FEN string".to_string());
        }
        
        // Clear the board
        self.squares.clear();
        
        // Parse piece placement
        let rows: Vec<&str> = parts[0].split('/').collect();
        if rows.len() != 8 {
            return Err("Invalid number of ranks in FEN".to_string());
        }
        
        for (y, row) in rows.iter().enumerate() {
            let mut x = 0;
            for c in row.chars() {
                if c.is_ascii_digit() {
                    x += c.to_digit(10).unwrap() as i8;
                } else {
                    if let Some(piece) = Piece::from_fen(c) {
                        if let Some(pos) = Position::new(x, 7 - y as i8) {
                            self.set_piece(pos, piece);
                        }
                        x += 1;
                    } else {
                        return Err(format!("Invalid piece character: {}", c));
                    }
                }
                
                if x > 8 {
                    return Err("Too many squares in rank".to_string());
                }
            }
            
            if x != 8 {
                return Err("Not enough squares in rank".to_string());
            }
        }
        
        // Parse active color
        if parts.len() > 1 {
            self.active_color = match parts[1] {
                "w" => Color::White,
                "b" => Color::Black,
                _ => return Err("Invalid active color in FEN".to_string()),
            };
        }
        
        // Parse castling rights
        if parts.len() > 2 {
            self.castling_rights = parts[2].to_string();
        }
        
        // Parse en passant target
        if parts.len() > 3 && parts[3] != "-" {
            self.en_passant_target = Some(Position::from_str(parts[3]).map_err(|e| e.to_string())?);
        } else {
            self.en_passant_target = None;
        }
        
        // Parse halfmove clock
        if parts.len() > 4 {
            self.halfmove_clock = parts[4].parse().unwrap_or(0);
        }
        
        // Parse fullmove number
        if parts.len() > 5 {
            self.fullmove_number = parts[5].parse().unwrap_or(1);
        }
        
        Ok(())
    }
    
    pub fn get_king_position(&self, color: Color) -> Option<Position> {
        self.squares.iter()
            .find(|(_, piece)| piece.piece_type == PieceType::King && piece.color == color)
            .and_then(|(&pos, _)| Position::from_xy(pos.x, pos.y))
    }
    
    pub fn is_square_under_attack(&self, pos: Position, by_color: Color) -> bool {
        self.squares
            .iter()
            .filter(|(_, piece)| piece.piece_type != PieceType::Empty && piece.color == by_color)
            .any(|(from, _)| {
                let moves = self.get_pseudo_legal_moves(*from);
                moves.contains(&pos)
            })
    }
    
    pub fn get_pseudo_legal_moves(&self, from: Position) -> HashSet<Position> {
        let mut moves = HashSet::new();
        
        let piece = match self.get_piece(from) {
            Some(p) => p,
            None => return moves,
        };
        
        match piece.piece_type {
            PieceType::Empty => return moves, // No moves for empty squares
            PieceType::Pawn => {
                let dir = if piece.color == Color::White { 1 } else { -1 };
                let start_rank = if piece.color == Color::White { 1 } else { 6 };
                
                // Forward moves
                if let Some(one_forward) = Position::from_xy(from.x, from.y + dir) {
                    if self.is_square_empty(one_forward) {
                        moves.insert(one_forward);
                        
                        // Double move from starting position
                        if from.y == start_rank {
                            if let Some(two_forward) = Position::from_xy(from.x, from.y + 2 * dir) {
                                if self.is_square_empty(two_forward) {
                                    moves.insert(two_forward);
                                }
                            }
                        }
                    }
                    
                    // Capture moves
                    for dx in [-1, 1] {
                        if let Some(capture_pos) = Position::from_xy(from.x + dx, from.y + dir) {
                            if let Some(target_piece) = self.get_piece(capture_pos) {
                                if target_piece.color != piece.color {
                                    moves.insert(capture_pos);
                                }
                            }
                            // TODO: Add en passant
                        }
                    }
                }
            },
            PieceType::Knight => {
                let knight_moves = [
                    (2, 1), (1, 2), (-1, 2), (-2, 1),
                    (-2, -1), (-1, -2), (1, -2), (2, -1)
                ];
                
                for &(dx, dy) in &knight_moves {
                    if let Some(new_pos) = Position::from_xy(from.x + dx, from.y + dy) {
                        if let Some(target_piece) = self.get_piece(new_pos) {
                            if target_piece.color != piece.color {
                                moves.insert(new_pos);
                            }
                        } else {
                            moves.insert(new_pos);
                        }
                    }
                }
            },
            PieceType::Bishop => {
                let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
                for &(dx, dy) in &directions {
                    self.add_sliding_moves(from, dx, dy, &mut moves, piece.color);
                }
            },
            PieceType::Rook => {
                let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
                for &(dx, dy) in &directions {
                    self.add_sliding_moves(from, dx, dy, &mut moves, piece.color);
                }
            },
            PieceType::Queen => {
                let directions = [
                    (1, 0), (-1, 0), (0, 1), (0, -1),  // Rook moves
                    (1, 1), (1, -1), (-1, 1), (-1, -1)   // Bishop moves
                ];
                for &(dx, dy) in &directions {
                    self.add_sliding_moves(from, dx, dy, &mut moves, piece.color);
                }
            },
            PieceType::King => {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        if let Some(new_pos) = Position::from_xy(from.x + dx, from.y + dy) {
                            if let Some(target_piece) = self.get_piece(new_pos) {
                                if target_piece.color != piece.color {
                                    moves.insert(new_pos);
                                }
                            } else {
                                moves.insert(new_pos);
                            }
                        }
                    }
                }
                // TODO: Add castling
            },
        };
        
        moves
    }
    
    fn add_sliding_moves(&self, from: Position, dx: i8, dy: i8, moves: &mut HashSet<Position>, color: Color) {
        let mut x = from.x + dx;
        let mut y = from.y + dy;
        
        while let Some(pos) = Position::from_xy(x, y) {
            match self.get_piece(pos) {
                Some(piece) => {
                    if piece.color != color {
                        moves.insert(pos);
                    }
                    break;
                }
                None => {
                    moves.insert(pos);
                    x += dx;
                    y += dy;
                }
            }
        }
    }
    
    pub fn get_legal_moves(&self, from: Position) -> HashSet<Position> {
        let mut legal_moves = HashSet::new();
        let piece = match self.get_piece(from) {
            Some(p) => p,
            None => return legal_moves,
        };
        
        let pseudo_legal_moves = self.get_pseudo_legal_moves(from);
        
        for &to in &pseudo_legal_moves {
            let mut board_copy = self.clone();
            board_copy.move_piece(from, to).ok();
            
            if let Some(king_pos) = board_copy.get_king_position(piece.color) {
                if !board_copy.is_square_under_attack(king_pos, !piece.color) {
                    legal_moves.insert(to);
                }
            }
        }
        
        legal_moves
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut board = Board::default();
        board.load_fen(fen)?;
        Ok(board)
    }
    
    pub fn get_piece(&self, pos: Position) -> Option<&Piece> {
        if !pos.is_valid() {
            return None;
        }
        self.squares.get(&pos)
    }

    pub fn get_piece_mut(&mut self, pos: Position) -> Option<&mut Piece> {
        if !pos.is_valid() {
            return None;
        }
        self.squares.get_mut(&pos)
    }

    pub fn set_piece(&mut self, pos: Position, piece: Piece) {
        if pos.is_valid() {
            self.squares.insert(pos, piece);
        }
    }

    pub fn remove_piece(&mut self, pos: Position) -> Option<Piece> {
        if !pos.is_valid() {
            return None;
        }
        self.squares.remove(&pos)
    }

    pub fn move_piece(&mut self, from: Position, to: Position) -> Result<(), String> {
        // Get the piece at the source position
        let piece = match self.get_piece(from) {
            Some(p) => p.clone(),
            None => return Err("No piece at source position".to_string()),
        };

        // Check if the move is pseudo-legal
        let pseudo_legal_moves = self.get_pseudo_legal_moves(from);
        if !pseudo_legal_moves.contains(&to) {
            return Err("Illegal move".to_string());
        }

        // Check if the move would leave the king in check
        let mut test_board = self.clone();
        test_board.remove_piece(from);
        test_board.set_piece(to, piece.clone());
        
        if let Some(king_pos) = self.get_king_position(piece.color) {
            let checking_king = if from == king_pos { to } else { king_pos };
            if test_board.is_square_under_attack(checking_king, !piece.color) {
                return Err("Move would leave king in check".to_string());
            }
        }

        // Handle en passant capture
        if let Some(ep_target) = self.en_passant_target {
            if piece.piece_type == PieceType::Pawn && to == ep_target {
                let capture_pos = Position::new(from.file(), to.rank()).unwrap();
                self.remove_piece(capture_pos);
            }
        }

        // Handle castling
        if piece.piece_type == PieceType::King && (from.file() - to.file()).abs() > 1 {
            // This is a castling move
            let (rook_from_file, rook_to_file) = if to.file() > from.file() {
                // Kingside castle
                (7, 5)
            } else {
                // Queenside castle
                (0, 3)
            };
            
            let rank = from.rank();
            if let Some(rook_pos) = Position::new(rook_from_file, rank) {
                if let Some(rook) = self.remove_piece(rook_pos) {
                    let new_rook_pos = Position::new(rook_to_file, rank).unwrap();
                    self.set_piece(new_rook_pos, rook);
                }
            }
        }

        // Execute the move
        self.remove_piece(from);
        self.set_piece(to, piece.clone());
        
        // Handle pawn promotion
        if piece.piece_type == PieceType::Pawn && (to.rank() == 0 || to.rank() == 7) {
            // For now, auto-promote to queen
            let promoted_piece = Piece {
                piece_type: PieceType::Queen,
                color: piece.color,
                has_moved: true,
                moves_made: piece.moves_made + 1,
            };
            self.set_piece(to, promoted_piece);
        } else if let Some(p) = self.get_piece_mut(to) {
            // Update piece has_moved status for non-promoted pieces
            p.has_moved = true;
            p.moves_made += 1;
        }

        // Update en passant target for next move
        if piece.piece_type == PieceType::Pawn && (from.rank() as i8 - to.rank() as i8).abs() == 2 {
            let ep_rank = (from.rank() + to.rank()) / 2;
            self.en_passant_target = Position::new(from.file(), ep_rank);
        } else {
            self.en_passant_target = None;
        }

        // Toggle active color
        self.active_color = !self.active_color;

        Ok(())
    }

    pub fn is_square_attacked(&self, pos: Position, by_color: Color) -> bool {
        // Check all opponent's pieces to see if they attack the given square
        for (square, piece) in &self.squares {
            if piece.color == by_color {
                let moves = self.get_pseudo_legal_moves(*square);
                if moves.contains(&pos) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_square_empty(&self, pos: Position) -> bool {
        self.get_piece(pos).map_or(true, |p| p.piece_type == PieceType::Empty)
    }
    
    pub fn is_in_check(&self, color: Color) -> bool {
        if let Some(king_pos) = self.get_king_position(color) {
            self.is_square_under_attack(king_pos, !color)
        } else {
            false // No king found (shouldn't happen in a valid game)
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                if let Some(pos) = Position::new(file, rank) {
                    if let Some(piece) = self.get_piece(pos) {
                        write!(f, "{} ", piece)?;
                    } else {
                        write!(f, ". ")?;
                    }
                } else {
                    write!(f, "? ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
