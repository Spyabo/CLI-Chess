mod position;
mod tests;

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;

use crate::moves;
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
    pub check: bool,
    pub checkmate: bool,
    pub stalemate: bool,
    pub selected_square: Option<Position>,
    pub valid_moves: HashSet<Position>,
    pub position_history: HashMap<String, u8>,
}

impl Default for GameState {
    fn default() -> Self {
        let mut board = Board::default();
        board.load_fen(STARTING_FEN).expect("Failed to load starting position");
        
        let mut game_state = GameState {
            board,
            active_color: Color::White,
            check: false,
            checkmate: false,
            stalemate: false,
            selected_square: None,
            valid_moves: HashSet::new(),
            position_history: HashMap::new(),
        };
        
        game_state.record_position();
        game_state
    }
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
            position_history: HashMap::new(),
        };
        
        // Record the initial position
        game_state.record_position();
        
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
            position_history: HashMap::new(),
        };
        
        // Update the game state based on the FEN position
        game_state.update_state();
        Ok(game_state)
    }
    
    /// Updates the game state (check, checkmate, stalemate, threefold repetition)
    fn update_state(&mut self) {
        // Update check status
        self.check = self.board.is_in_check(self.active_color);
        
        // Update valid moves for selected piece using the moves module
        if let Some(pos) = self.selected_square {
            use crate::moves::get_valid_moves;
            self.valid_moves = get_valid_moves(&self.board, pos);
            
            // Filter out moves that would put the king in check
            let current_moves = self.valid_moves.clone();
            self.valid_moves = current_moves.into_iter()
                .filter(|&to| {
                    let mut board_clone = self.board.clone();
                    board_clone.move_piece(pos, to).is_ok()
                })
                .collect();
        } else {
            self.valid_moves.clear();
        }
        
        // Check for checkmate/stalemate/threefold repetition
        if !self.has_any_legal_moves() {
            if self.board.is_in_check(self.active_color) {
                self.checkmate = true;
            } else {
                self.stalemate = true;
            }
        } else if self.is_threefold_repetition() {
            self.stalemate = true;
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
        
        // Check if this is a capture or pawn move (which reset the position history)
        let is_reset_move = self.board.get_piece(to).is_some() || 
                           matches!(self.board.get_piece(from), Some(p) if p.piece_type == PieceType::Pawn);
        
        // Try to make the move
        if let Err(e) = self.board.move_piece(from, to) {
            return Err(e);
        }
        
        // Toggle the active color
        self.active_color = !self.active_color;
        
        // Update the position history
        if is_reset_move {
            self.position_history.clear();
        }
        self.record_position();
        
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
    
    /// Records the current position in the position history
    fn record_position(&mut self) {
        let fen = self.board.to_fen();
        *self.position_history.entry(fen).or_insert(0) += 1;
    }
    
    /// Checks if the current position has occurred three times
    pub fn is_threefold_repetition(&self) -> bool {
        self.position_history.values().any(|&count| count >= 3)
    }
}

const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone)]
pub struct Board {
    pub squares: HashMap<Position, Piece>,
    pub active_color: Color,
    pub castling_rights: String,
    pub en_passant_target: Option<Position>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
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
    /// Convert the current board state to FEN notation
    pub fn to_fen(&self) -> String {
        let mut fen_parts = Vec::new();
        
        // 1. Piece placement data
        let mut fen_placement = String::new();
        for rank in (0..8).rev() {
            let mut empty_squares = 0;
            let mut rank_str = String::new();
            
            for file in 0..8 {
                if let Some(pos) = Position::from_xy(file, rank) {
                    if let Some(piece) = self.get_piece(pos) {
                        if empty_squares > 0 {
                            rank_str.push_str(&empty_squares.to_string());
                            empty_squares = 0;
                        }
                        rank_str.push(piece.to_char());
                    } else {
                        empty_squares += 1;
                    }
                }
            }
            
            if empty_squares > 0 {
                rank_str.push_str(&empty_squares.to_string());
            }
            
            fen_placement.push_str(&rank_str);
            if rank > 0 {
                fen_placement.push('/');
            }
        }
        fen_parts.push(fen_placement);
        
        // 2. Active color
        fen_parts.push(if self.active_color == Color::White { "w".to_string() } else { "b".to_string() });
        
        // 3. Castling availability
        fen_parts.push(if self.castling_rights.is_empty() { "-".to_string() } else { self.castling_rights.clone() });
        
        // 4. En passant target square
        fen_parts.push(
            self.en_passant_target
                .map(|pos| pos.to_string())
                .unwrap_or_else(|| "-".to_string())
        );
        
        // 5. Halfmove clock (50-move rule)
        fen_parts.push(self.halfmove_clock.to_string());
        
        // 6. Fullmove number
        fen_parts.push(self.fullmove_number.to_string());
        
        fen_parts.join(" ")
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
        // Check for pawn attacks
        let direction = if by_color == Color::White { 1 } else { -1 };
        for dx in [-1, 1] {
            if let Some(attack_pos) = Position::new(
                (pos.file() as i8 + dx) as i8,
                (pos.rank() as i8 - direction) as i8
            ) {
                if let Some(piece) = self.get_piece(attack_pos) {
                    if piece.color == by_color && piece.piece_type == PieceType::Pawn {
                        return true;
                    }
                }
            }
        }

        // Check for knight attacks
        let knight_moves = [
            (1, 2), (2, 1), (2, -1), (1, -2),
            (-1, -2), (-2, -1), (-2, 1), (-1, 2)
        ];
        for &(dx, dy) in &knight_moves {
            if let Some(attack_pos) = Position::new(
                (pos.file() as i8 + dx) as i8,
                (pos.rank() as i8 + dy) as i8
            ) {
                if let Some(piece) = self.get_piece(attack_pos) {
                    if piece.color == by_color && piece.piece_type == PieceType::Knight {
                        return true;
                    }
                }
            }
        }

        // Check for sliding pieces (rook, bishop, queen, king)
        let directions = [
            // Rook/Queen directions
            (1, 0), (-1, 0), (0, 1), (0, -1),
            // Bishop/Queen directions
            (1, 1), (1, -1), (-1, 1), (-1, -1)
        ];

        for &(dx, dy) in &directions {
            for step in 1..8 {
                let x = (pos.file() as i8 + dx * step) as i8;
                let y = (pos.rank() as i8 + dy * step) as i8;
                
                if let Some(attack_pos) = Position::new(x, y) {
                    if let Some(piece) = self.get_piece(attack_pos) {
                        if piece.color != by_color {
                            break; // Blocked by opponent's piece
                        }
                        
                        // Check if this is an attacking piece
                        match piece.piece_type {
                            PieceType::Queen => return true,
                            PieceType::Rook if dx == 0 || dy == 0 => return true,
                            PieceType::Bishop if dx != 0 && dy != 0 => return true,
                            PieceType::King if step == 1 => return true,
                            _ => break, // Not an attacking piece
                        }
                    }
                } else {
                    break; // Out of board
                }
            }
        }

        false
    }
    
    pub fn get_pseudo_legal_moves(&self, from: Position) -> HashSet<Position> {
        let mut moves = HashSet::new();
        if let Some(piece) = self.get_piece(from) {
            match piece.piece_type {
                PieceType::Pawn => moves::get_pawn_moves(self, from, piece.color, &mut moves),
                PieceType::Rook => moves::get_rook_moves(self, from, piece.color, &mut moves),
                PieceType::Knight => moves::get_knight_moves(self, from, piece.color, &mut moves),
                PieceType::Bishop => moves::get_bishop_moves(self, from, piece.color, &mut moves),
                PieceType::Queen => moves::get_queen_moves(self, from, piece.color, &mut moves),
                PieceType::King => moves::get_king_moves(self, from, piece.color, &mut moves),
                PieceType::Empty => {}
            }
        }
        moves
    }

    pub fn get_legal_moves(&self, from: Position) -> HashSet<Position> {
        let mut legal_moves = HashSet::new();
        let piece = match self.get_piece(from) {
            Some(p) => p,
            None => return legal_moves,
        };
        
        let pseudo_legal_moves = self.get_pseudo_legal_moves(from);
        
        // Get the current king position before making any moves
        let king_pos = if piece.piece_type == PieceType::King {
            // If the piece is the king, the new position after move would be 'to'
            // We'll handle this case specially in the loop
            None
        } else {
            self.get_king_position(piece.color)
        };
        
        for &to in &pseudo_legal_moves {
            // Skip castling moves for now, they're handled separately
            if piece.piece_type == PieceType::King && (from.file() as i8 - to.file() as i8).abs() > 1 {
                legal_moves.insert(to);
                continue;
            }
            
            // Create a temporary board for this move
            let mut board_copy = self.clone();
            
            // Make the move on the copy
            if board_copy.move_piece(from, to).is_err() {
                continue;
            }
            
            // Check if the king is in check after the move
            let check_pos = if piece.piece_type == PieceType::King {
                to  // King moved to 'to' position
            } else {
                // King didn't move, use original position
                king_pos.unwrap_or_else(|| {
                    // If we can't find the king, something is wrong
                    panic!("King not found for color {:?}", piece.color);
                })
            };
            
            if !board_copy.is_square_under_attack(check_pos, !piece.color) {
                legal_moves.insert(to);
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
        let mut piece = match self.get_piece(from) {
            Some(p) => p.clone(),
            None => return Err("No piece at source position".to_string()),
        };
        
        // Initialize rook_move for castling
        let mut rook_move = None;

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

        // Update castling rights if the king moves
        if piece.piece_type == PieceType::King {
            self.update_castling_rights_after_king_move(piece.color);
            
            // Handle castling move
            if (from.file() as i8 - to.file() as i8).abs() > 1 {
                // This is a castling move
                let (rook_from_file, rook_to_file) = if to.file() > from.file() {
                    // Kingside castle (O-O)
                    (7, 5)
                } else {
                    // Queenside castle (O-O-O)
                    (0, 3)
                };
                
                let rank = from.rank();
                if let Some(rook_pos) = Position::new(rook_from_file, rank) {
                    if let Some(rook) = self.remove_piece(rook_pos) {
                        let new_rook_pos = Position::new(rook_to_file, rank).unwrap();
                        rook_move = Some((new_rook_pos, rook));
                    }
                }
            }
        }

        // Update castling rights if a rook moves
        if piece.piece_type == PieceType::Rook {
            self.update_castling_rights_after_rook_move(from, piece.color);
        }

        // Handle en passant capture
        if let Some(ep_target) = self.en_passant_target {
            if piece.piece_type == PieceType::Pawn && to == ep_target {
                // The captured pawn is on the same file as the destination, but on the rank we came from
                let capture_pos = Position::new(to.file(), from.rank()).unwrap();
                self.remove_piece(capture_pos);
            }
        }
        
        // Reset en passant target at the start of each move
        self.en_passant_target = None;
        
        // Set en passant target if a pawn moves two squares
        if piece.piece_type == PieceType::Pawn && (from.rank() as i8 - to.rank() as i8).abs() == 2 {
            let direction = match piece.color {
                Color::White => 1,
                Color::Black => -1,
            };
            let ep_rank = from.rank() as i8 + direction;
            if ep_rank >= 0 && ep_rank < 8 {
                self.en_passant_target = Position::new(from.file() as i8, ep_rank);
            }
        }

        // Handle pawn promotion before moving the piece
        if piece.piece_type == PieceType::Pawn && (to.rank() == 0 || to.rank() == 7) {
            // Auto-promote to queen
            piece.piece_type = PieceType::Queen;
        }
        
        // Execute the move
        self.remove_piece(from);
        
        // If this is a castling move, place the rook
        if let Some((rook_pos, rook)) = rook_move {
            self.set_piece(rook_pos, rook);
        }
        
        // Update the piece's moved status before placing it
        piece.has_moved = true;
        piece.moves_made += 1;
        
        self.set_piece(to, piece);

        // Toggle active color
        self.active_color = !self.active_color;

        Ok(())
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
    
    /// Returns the current en passant target square, if any
    pub fn en_passant_target(&self) -> Option<Position> {
        self.en_passant_target
    }
    
    fn update_castling_rights_after_king_move(&mut self, color: Color) {
        // When the king moves, remove all castling rights for that color
        match color {
            Color::White => {
                self.castling_rights = self.castling_rights.chars()
                    .filter(|&c| c != 'K' && c != 'Q')
                    .collect();
            },
            Color::Black => {
                self.castling_rights = self.castling_rights.chars()
                    .filter(|&c| c != 'k' && c != 'q')
                    .collect();
            }
        }
    }
    
    fn update_castling_rights_after_rook_move(&mut self, from: Position, color: Color) {
        // If a rook moves from its starting position, remove the corresponding castling right
        match (color, from.file()) {
            (Color::White, 0) => self.castling_rights.retain(|c| c != 'Q'),  // Queenside rook
            (Color::White, 7) => self.castling_rights.retain(|c| c != 'K'),  // Kingside rook
            (Color::Black, 0) => self.castling_rights.retain(|c| c != 'q'),  // Queenside rook
            (Color::Black, 7) => self.castling_rights.retain(|c| c != 'k'),  // Kingside rook
            _ => {}
        }
    }
}
