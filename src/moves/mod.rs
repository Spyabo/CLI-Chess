use crate::board::{Board, Position};
use crate::pieces::{Color, PieceType};

pub fn get_valid_moves(board: &Board, from: Position) -> Vec<Position> {
    let mut moves = Vec::new();
    
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

fn get_pawn_moves(board: &Board, from: Position, color: Color, moves: &mut Vec<Position>) {
    let direction = match color {
        Color::White => 1,
        Color::Black => -1,
        _ => return,
    };
    
    let one_forward = from + (0, direction);
    if one_forward.is_valid() && board.is_square_empty(one_forward) {
        moves.push(one_forward);
        
        // Check for double move from starting position
        let starting_rank = match color {
            Color::White => 1,
            Color::Black => 6,
            _ => return,
        };
        
        if from.y == starting_rank {
            let two_forward = from + (0, 2 * direction);
            if two_forward.is_valid() && board.is_square_empty(two_forward) {
                moves.push(two_forward);
            }
        }
    }
    
    // Check captures
    for dx in [-1, 1].iter() {
        let capture_pos = from + (*dx, direction);
        if !capture_pos.is_valid() {
            continue;
        }
        
        if let Some(target_piece) = board.get_piece(capture_pos) {
            if target_piece.color != color {
                moves.push(capture_pos);
            }
        }
        // TODO: Handle en passant
    }
}

fn get_rook_moves(board: &Board, from: Position, color: Color, moves: &mut Vec<Position>) {
    let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    get_sliding_moves(board, from, color, &directions, moves);
}

fn get_knight_moves(board: &Board, from: Position, color: Color, moves: &mut Vec<Position>) {
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
                moves.push(to);
            }
        } else {
            moves.push(to);
        }
    }
}

fn get_bishop_moves(board: &Board, from: Position, color: Color, moves: &mut Vec<Position>) {
    let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
    get_sliding_moves(board, from, color, &directions, moves);
}

fn get_queen_moves(board: &Board, from: Position, color: Color, moves: &mut Vec<Position>) {
    let directions = [
        (1, 0), (-1, 0), (0, 1), (0, -1),  // Rook moves
        (1, 1), (1, -1), (-1, 1), (-1, -1), // Bishop moves
    ];
    get_sliding_moves(board, from, color, &directions, moves);
}

fn get_king_moves(board: &Board, from: Position, color: Color, moves: &mut Vec<Position>) {
    let king_moves = [
        (1, 0), (-1, 0), (0, 1), (0, -1),
        (1, 1), (1, -1), (-1, 1), (-1, -1),
    ];
    
    for (dx, dy) in &king_moves {
        let to = from + (*dx, *dy);
        if !to.is_valid() {
            continue;
        }
        
        if let Some(piece) = board.get_piece(to) {
            if piece.color != color {
                moves.push(to);
            }
        } else {
            moves.push(to);
        }
    }
    
    // TODO: Add castling
}

fn get_sliding_moves(
    board: &Board,
    from: Position,
    color: Color,
    directions: &[(i8, i8)],
    moves: &mut Vec<Position>,
) {
    for &(dx, dy) in directions {
        let mut current = from + (dx, dy);
        while current.is_valid() {
            if let Some(piece) = board.get_piece(current) {
                if piece.color != color {
                    moves.push(current);
                }
                break;
            } else {
                moves.push(current);
            }
            current = current + (dx, dy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use std::str::FromStr;

    #[test]
    fn test_pawn_moves() {
        let board = Board::new();
        let pos = Position::from_str("e2").unwrap();
        let moves = get_valid_moves(&board, pos);
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Position::from_str("e3").unwrap()));
        assert!(moves.contains(&Position::from_str("e4").unwrap()));
    }

    #[test]
    fn test_knight_moves() {
        let board = Board::new();
        let pos = Position::from_str("g1").unwrap();
        let moves = get_valid_moves(&board, pos);
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Position::from_str("f3").unwrap()));
        assert!(moves.contains(&Position::from_str("h3").unwrap()));
    }
}
