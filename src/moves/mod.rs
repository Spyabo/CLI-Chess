use crate::board::{Board, Position};
use crate::pieces::{Color, PieceType};

#[allow(dead_code)]
pub fn get_valid_moves(board: &Board, from: Position) -> std::collections::HashSet<Position> {
    let mut moves = std::collections::HashSet::new();
    
    if let Some(piece) = board.get_piece(from) {
        match piece.piece_type {
            PieceType::Pawn => get_pawn_moves(board, from, piece.color, &mut moves),
            PieceType::Rook => get_rook_moves(board, from, piece.color, &mut moves),
            PieceType::Knight => get_knight_moves(board, from, piece.color, &mut moves),
            PieceType::Bishop => get_bishop_moves(board, from, piece.color, &mut moves),
            PieceType::Queen => get_queen_moves(board, from, piece.color, &mut moves),
            PieceType::King => get_king_moves(board, from, piece.color, &mut moves),
            PieceType::Empty => {}
        }
    }
    
    moves
}

pub fn get_pawn_moves(board: &Board, from: Position, color: Color, moves: &mut std::collections::HashSet<Position>) {
    let direction = match color {
        Color::White => 1,
        Color::Black => -1,
    };
    
    let one_forward = from + (0, direction);
    if one_forward.is_valid() && board.is_square_empty(one_forward) {
        moves.insert(one_forward);
        
        // Check for double move from starting position
        let starting_rank = match color {
            Color::White => 1,
            Color::Black => 6,
        };
        
        if from.y == starting_rank {
            let two_forward = from + (0, 2 * direction);
            if two_forward.is_valid() && board.is_square_empty(two_forward) {
                moves.insert(two_forward);
            }
        }
    }
    
    // Check captures
    for dx in [-1, 1].iter() {
        let capture_pos = from + (*dx, direction);
        if !capture_pos.is_valid() {
            continue;
        }
        
        // Regular capture
        if let Some(target_piece) = board.get_piece(capture_pos) {
            if target_piece.color != color {
                moves.insert(capture_pos);
            }
        }
        // En passant capture
        else if let Some(ep_target) = board.en_passant_target() {
            if capture_pos == ep_target {
                let ep_capture_pos = Position::new(capture_pos.file(), from.rank()).unwrap();
                if let Some(target_piece) = board.get_piece(ep_capture_pos) {
                    if target_piece.piece_type == PieceType::Pawn && target_piece.color != color {
                        moves.insert(capture_pos);
                    }
                }
            }
        }
    }
}

pub fn get_rook_moves(board: &Board, from: Position, color: Color, moves: &mut std::collections::HashSet<Position>) {
    let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    get_sliding_moves(board, from, color, &directions, moves);
}

pub fn get_knight_moves(board: &Board, from: Position, color: Color, moves: &mut std::collections::HashSet<Position>) {
    let knight_moves = [
        (2, 1), (2, -1), (-2, 1), (-2, -1),
        (1, 2), (1, -2), (-1, 2), (-1, -2),
    ];
    
    for (dx, dy) in &knight_moves {
        let to = from + (*dx, *dy);
        if !to.is_valid() {
            continue;
        }
        
        if let Some(piece) = board.get_piece(to) {
            if piece.color != color {
                moves.insert(to);
            }
        } else {
            moves.insert(to);
        }
    }
}

pub fn get_bishop_moves(board: &Board, from: Position, color: Color, moves: &mut std::collections::HashSet<Position>) {
    let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
    get_sliding_moves(board, from, color, &directions, moves);
}

pub fn get_queen_moves(board: &Board, from: Position, color: Color, moves: &mut std::collections::HashSet<Position>) {
    let directions = [
        (1, 0), (-1, 0), (0, 1), (0, -1),  // Rook moves
        (1, 1), (1, -1), (-1, 1), (-1, -1), // Bishop moves
    ];
    get_sliding_moves(board, from, color, &directions, moves);
}

pub fn get_king_moves(board: &Board, from: Position, color: Color, moves: &mut std::collections::HashSet<Position>) {
    let king_moves = [
        (1, 0), (-1, 0), (0, 1), (0, -1),
        (1, 1), (1, -1), (-1, 1), (-1, -1),
    ];
    
    for (dx, dy) in &king_moves {
        let to = from + (*dx, *dy);
        if !to.is_valid() || to == from {
            continue;
        }
        
        // Only allow the move if the destination square is not under attack
        if !board.is_square_under_attack(to, !color) {
            if let Some(piece) = board.get_piece(to) {
                if piece.color != color {
                    moves.insert(to);
                }
            } else {
                moves.insert(to);
            }
        }
    }
    
    // Castling - only possible if king hasn't moved and is not in check
    if !board.is_in_check(color) && from == Position::new(4, if color == Color::White { 0 } else { 7 }).unwrap() {
        let rank = if color == Color::White { 0 } else { 7 };
        
        // Kingside castling (O-O)
        if board.castling_rights.contains(if color == Color::White { 'K' } else { 'k' }) {
            // Check if squares between king and rook are empty
            let f_pos = Position::new(5, rank).unwrap();
            let g_pos = Position::new(6, rank).unwrap();
            
            if board.is_square_empty(f_pos) && 
               board.is_square_empty(g_pos) &&
               !board.is_square_under_attack(f_pos, !color) &&
               !board.is_square_under_attack(g_pos, !color) {
                moves.insert(g_pos);
            }
        }
        
        // Queenside castling (O-O-O)
        if board.castling_rights.contains(if color == Color::White { 'Q' } else { 'q' }) {
            // Check if squares between king and rook are empty
            let b_pos = Position::new(1, rank).unwrap();
            let c_pos = Position::new(2, rank).unwrap();
            let d_pos = Position::new(3, rank).unwrap();
            
            if board.is_square_empty(b_pos) && 
               board.is_square_empty(c_pos) && 
               board.is_square_empty(d_pos) &&
               !board.is_square_under_attack(c_pos, !color) &&
               !board.is_square_under_attack(d_pos, !color) {
                moves.insert(c_pos);
            }
        }
    }
}

fn get_sliding_moves(
    board: &Board,
    from: Position,
    color: Color,
    directions: &[(i8, i8)],
    moves: &mut std::collections::HashSet<Position>,
) {
    for &(dx, dy) in directions {
        let mut current = from + (dx, dy);
        while current.is_valid() {
            if let Some(piece) = board.get_piece(current) {
                if piece.color != color {
                    moves.insert(current);
                }
                break;
            } else {
                moves.insert(current);
            }
            current = current + (dx, dy);
        }
    }
}
