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

/// A recorded move with all details needed for algebraic notation
#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub piece: PieceType,
    pub color: Color,
    pub from: Position,
    pub to: Position,
    pub captured: Option<PieceType>,
    pub is_check: bool,
    pub is_checkmate: bool,
    pub is_castling: Option<bool>,  // Some(true) = kingside, Some(false) = queenside
    pub promotion: Option<PieceType>,
}

impl MoveRecord {
    /// Convert to algebraic notation (e.g., "Nf3", "Qxd7+", "O-O")
    pub fn to_algebraic(&self, use_unicode: bool) -> String {
        // Castling
        if let Some(kingside) = self.is_castling {
            return if kingside { "O-O".to_string() } else { "O-O-O".to_string() };
        }

        let mut notation = String::new();

        // Piece letter/symbol (pawns omitted)
        if self.piece != PieceType::Pawn {
            if use_unicode {
                notation.push(match (self.piece, self.color) {
                    (PieceType::King, Color::White) => '♔',
                    (PieceType::Queen, Color::White) => '♕',
                    (PieceType::Rook, Color::White) => '♖',
                    (PieceType::Bishop, Color::White) => '♗',
                    (PieceType::Knight, Color::White) => '♘',
                    (PieceType::King, Color::Black) => '♚',
                    (PieceType::Queen, Color::Black) => '♛',
                    (PieceType::Rook, Color::Black) => '♜',
                    (PieceType::Bishop, Color::Black) => '♝',
                    (PieceType::Knight, Color::Black) => '♞',
                    _ => ' ',
                });
            } else {
                notation.push(match self.piece {
                    PieceType::King => 'K',
                    PieceType::Queen => 'Q',
                    PieceType::Rook => 'R',
                    PieceType::Bishop => 'B',
                    PieceType::Knight => 'N',
                    _ => ' ',
                });
            }
        }

        // Capture indicator
        if self.captured.is_some() {
            if self.piece == PieceType::Pawn {
                // Pawn captures show file of origin
                let file = (b'a' + self.from.x as u8) as char;
                notation.push(file);
            }
            notation.push('x');
        }

        // Destination square
        notation.push_str(&self.to.to_notation());

        // Promotion
        if let Some(promo) = self.promotion {
            notation.push('=');
            notation.push(match promo {
                PieceType::Queen => 'Q',
                PieceType::Rook => 'R',
                PieceType::Bishop => 'B',
                PieceType::Knight => 'N',
                _ => '?',
            });
        }

        // Check/checkmate
        if self.is_checkmate {
            notation.push('#');
        } else if self.is_check {
            notation.push('+');
        }

        notation
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub board: Board,
    pub active_color: Color,
    pub check: bool,
    pub checkmate: bool,
    pub stalemate: bool,
    pub position_history: HashMap<String, u8>,
    pub current_pieces: HashMap<Color, HashSet<(PieceType, Position)>>,
    pub captured_by_white: Vec<Piece>,  // Black pieces captured by white
    pub captured_by_black: Vec<Piece>,  // White pieces captured by black
    pub move_history: Vec<MoveRecord>,  // All moves played in this game
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
            position_history: HashMap::new(),
            current_pieces: HashMap::new(),
            captured_by_white: Vec::new(),
            captured_by_black: Vec::new(),
            move_history: Vec::new(),
        };

        // Initialize current_pieces from the board
        game_state.init_current_pieces();
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
            check: false,
            checkmate: false,
            stalemate: false,
            position_history: HashMap::new(),
            current_pieces: HashMap::new(),
            captured_by_white: Vec::new(),
            captured_by_black: Vec::new(),
            move_history: Vec::new(),
        };
        
        // Initialize current_pieces   
        game_state.init_current_pieces();
        
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
            check: false,
            checkmate: false,
            stalemate: false,
            position_history: HashMap::new(),
            current_pieces: HashMap::new(),
            captured_by_white: Vec::new(),
            captured_by_black: Vec::new(),
            move_history: Vec::new(),
        };

        // Initialize current_pieces from the FEN position
        game_state.init_current_pieces();
        
        // Update the game state based on the FEN position
        game_state.update_state();
        Ok(game_state)
    }
    
    /// Updates the game state (check, checkmate, stalemate, threefold repetition)
    fn update_state(&mut self) {
        // Update check status
        self.check = self.board.is_in_check(self.active_color);

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
    /// Updates the current_pieces map when a piece is captured
    fn remove_piece_from_tracking(&mut self, pos: Position, piece: &Piece) {
        if let Some(pieces) = self.current_pieces.get_mut(&piece.color) {
            pieces.retain(|(_, p)| *p != pos);
        }
    }
    
    /// Updates the current_pieces map when a piece is moved or promoted
    fn update_piece_tracking(&mut self, old_pos: Position, new_pos: Position, new_piece: &Piece) {
        // Remove from old position if it exists
        if let Some(pieces) = self.current_pieces.get_mut(&new_piece.color) {
            pieces.retain(|(_, p)| *p != old_pos);
            pieces.insert((new_piece.piece_type, new_pos));
        } else {
            // Shouldn't happen if the board is in a valid state
            let mut pieces = HashSet::new();
            pieces.insert((new_piece.piece_type, new_pos));
            self.current_pieces.insert(new_piece.color, pieces);
        }
    }
    
    /// Initializes the current_pieces map from the board state
    fn init_current_pieces(&mut self) {
        self.current_pieces.clear();
        
        for (&pos, piece) in &self.board.squares {
            if piece.piece_type != PieceType::Empty {
                self.current_pieces
                    .entry(piece.color)
                    .or_insert_with(HashSet::new)
                    .insert((piece.piece_type, pos));
            }
        }
    }
    
    pub fn make_move(&mut self, from: Position, to: Position) -> Result<(), String> {
        // Save the current state for potential undo
        let original_state = self.board.clone();
        let original_move_history_len = self.move_history.len();

        // Get the moving piece before the move
        let moving_piece = match self.board.get_piece(from) {
            Some(p) => p.clone(),
            None => return Err("No piece at source position".to_string()),
        };

        // Check if this is a capture (including en passant)
        let mut captured_piece = self.board.get_piece(to).cloned();

        // Check for en passant capture
        let is_en_passant = moving_piece.piece_type == PieceType::Pawn
            && self.board.en_passant_target() == Some(to)
            && captured_piece.is_none();

        if is_en_passant {
            // En passant: captured pawn is on same file as destination but same rank as origin
            let ep_capture_pos = Position::new(to.file() as i8, from.rank() as i8).unwrap();
            captured_piece = self.board.get_piece(ep_capture_pos).cloned();
        }

        // Detect castling (king moves 2 squares horizontally)
        let is_castling = moving_piece.piece_type == PieceType::King
            && (from.x - to.x).abs() == 2;
        let castling_side = if is_castling {
            Some(to.x > from.x)  // true = kingside, false = queenside
        } else {
            None
        };

        // Check if this is a pawn move (which resets the position history)
        let is_pawn_move = moving_piece.piece_type == PieceType::Pawn;
        let is_reset_move = captured_piece.is_some() || is_pawn_move;

        // Handle capture tracking
        if let Some(ref captured) = captured_piece {
            if is_en_passant {
                let ep_capture_pos = Position::new(to.file() as i8, from.rank() as i8).unwrap();
                self.remove_piece_from_tracking(ep_capture_pos, captured);
            } else {
                self.remove_piece_from_tracking(to, captured);
            }
            // Track the captured piece for display
            match moving_piece.color {
                Color::White => self.captured_by_white.push(captured.clone()),
                Color::Black => self.captured_by_black.push(captured.clone()),
            }
        }

        // Try to make the move
        if let Err(e) = self.board.move_piece(from, to) {
            return Err(e);
        }

        // Get the piece type after the move (in case of promotion)
        let moved_piece = self.board.get_piece(to).unwrap().clone();

        // Detect promotion
        let promotion = if moving_piece.piece_type == PieceType::Pawn
            && moved_piece.piece_type != PieceType::Pawn
        {
            Some(moved_piece.piece_type)
        } else {
            None
        };

        // Update piece tracking
        self.update_piece_tracking(from, to, &moved_piece);

        // Toggle active color
        self.active_color = !self.active_color;

        // Reset position history on capture or pawn move (50-move rule)
        if is_reset_move {
            self.position_history.clear();
        }

        // Record the position after the move
        self.record_position();

        // Update the game state
        self.update_state();

        // If the move leaves the king in check, it's illegal
        if self.board.is_in_check(!self.active_color) {
            // Revert the move
            self.board = original_state;
            self.active_color = !self.active_color; // Toggle back

            // Rebuild the pieces map since we reverted the board
            self.init_current_pieces();
            // Remove any move we might have added
            self.move_history.truncate(original_move_history_len);
            return Err("Move would leave king in check".to_string());
        }

        // Record the move in history (after confirming it's legal)
        let move_record = MoveRecord {
            piece: moving_piece.piece_type,
            color: moving_piece.color,
            from,
            to,
            captured: captured_piece.map(|p| p.piece_type),
            is_check: self.check,
            is_checkmate: self.checkmate,
            is_castling: castling_side,
            promotion,
        };
        self.move_history.push(move_record);

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
        // Scan outwards from the square in all directions
        let knight_squares = [
            (1, 2), (2, 1), (2, -1), (1, -2),
            (-1, -2), (-2, -1), (-2, 1), (-1, 2),
            ];
            
        for (dx, dy) in knight_squares {
            let current: Position = pos + (dx, dy);
            if current.is_valid() && self.get_piece(current).is_some_and(|p| p.color == by_color && p.piece_type == PieceType::Knight) {
                return true;
            }
        }

        // Check if an opposing pawn is attacking the position
        let pawn_squares = match by_color {
            Color::White => [(-1, -1), (1, -1)],  // Check squares below (pawns attack upward)
            Color::Black => [(-1, 1), (1, 1)],    // Check squares above (pawns attack downward)
        };

        for (dx, dy) in pawn_squares {
            let current: Position = pos + (dx, dy);
            if current.is_valid() && self.get_piece(current).is_some_and(|p| p.color == by_color && p.piece_type == PieceType::Pawn) {
                return true;
            }
        }

        // Check for attacking king (adjacent squares)
        let king_squares = [
            (1, 0), (-1, 0), (0, 1), (0, -1),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
        ];
        for (dx, dy) in king_squares {
            let current: Position = pos + (dx, dy);
            if current.is_valid() && self.get_piece(current).is_some_and(|p| p.color == by_color && p.piece_type == PieceType::King) {
                return true;
            }
        }

        // Check for sliding piece attacks (rooks, bishops, queens)
        let directions = [
            (1, 0), (-1, 0), (0, 1), (0, -1),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
        ];

        for (dx, dy) in directions {
            let mut current: Position = pos + (dx, dy);
            let diagonal = dx != 0 && dy != 0;

            while current.is_valid() {
                if let Some(piece) = self.get_piece(current) {
                    if piece.color != by_color {
                        break; // Blocked by piece of opposite color
                    }

                    // Check if this piece type can attack along this direction
                    if diagonal {
                        if piece.piece_type == PieceType::Bishop || piece.piece_type == PieceType::Queen {
                            return true;
                        }
                    } else {
                        if piece.piece_type == PieceType::Rook || piece.piece_type == PieceType::Queen {
                            return true;
                        }
                    }
                    break; // Blocked by piece that can't attack this way
                }
                // Empty square - continue scanning in this direction
                current = current + (dx, dy);
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
