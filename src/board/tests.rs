#[allow(unused_imports)]
use super::*;

#[test]
fn test_initial_board_setup() {
    let board = Board::from_fen(STARTING_FEN).unwrap();
    
    // Test that all pieces are in their starting positions
    assert_eq!(board.get_piece(Position::from_notation("a1").unwrap()).unwrap().piece_type, PieceType::Rook);
    assert_eq!(board.get_piece(Position::from_notation("e1").unwrap()).unwrap().piece_type, PieceType::King);
    assert_eq!(board.get_piece(Position::from_notation("e8").unwrap()).unwrap().piece_type, PieceType::King);
    assert_eq!(board.get_piece(Position::from_notation("a2").unwrap()).unwrap().piece_type, PieceType::Pawn);
    assert_eq!(board.get_piece(Position::from_notation("b1").unwrap()).unwrap().piece_type, PieceType::Knight);
    
    // Test empty squares
    for rank in 2..6 {
        for file in 0..8 {
            let pos = Position::new(file, rank).unwrap();
            if rank >= 2 && rank <= 5 {
                assert!(board.get_piece(pos).is_none());
            }
        }
    }
}

#[test]
fn test_pawn_moves() {
    // Test initial pawn moves
    let board = Board::from_fen(STARTING_FEN).unwrap();
    let e2 = Position::from_notation("e2").unwrap();
    let e4 = Position::from_notation("e4").unwrap();
    let e3 = Position::from_notation("e3").unwrap();
    
    let moves = board.get_legal_moves(e2);
    assert_eq!(moves.len(), 2);
    assert!(moves.contains(&e3));
    assert!(moves.contains(&e4));
    
    // Test pawn capture
    let board = Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2").unwrap();
    let e4 = Position::from_notation("e4").unwrap();
    let d5 = Position::from_notation("d5").unwrap();
    
    let moves = board.get_legal_moves(e4);
    assert_eq!(moves.len(), 2);
    assert!(moves.contains(&d5));  // Should be able to capture
}

#[test]
fn test_castling() {
    // Test kingside castling
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1").unwrap();
    let e1 = Position::from_notation("e1").unwrap();
    let g1 = Position::from_notation("g1").unwrap();
    
    let moves = board.get_legal_moves(e1);
    assert!(moves.contains(&g1));  // Should be able to castle
    
    // Move the king
    board.move_piece(e1, g1).unwrap();
    
    // Verify castling happened
    assert_eq!(board.get_piece(g1).unwrap().piece_type, PieceType::King);
    assert_eq!(board.get_piece(Position::from_notation("f1").unwrap()).unwrap().piece_type, PieceType::Rook);
}

#[test]
fn test_en_passant() {
    // Set up en passant position
    let mut board = Board::from_fen("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3").unwrap();
    let e5 = Position::from_notation("e5").unwrap();
    let f6 = Position::from_notation("f6").unwrap();
    
    // White pawn should be able to capture en passant
    let moves = board.get_legal_moves(e5);
    assert!(moves.contains(&f6));
    
    // Perform en passant
    board.move_piece(e5, f6).unwrap();
    
    // Verify the captured pawn is removed
    assert!(board.get_piece(Position::from_notation("f5").unwrap()).is_none());
}

#[test]
fn test_check_detection() {
    // Position where black is in check
    let board = Board::from_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1").unwrap();
    assert!(!board.is_in_check(Color::Black));  // Not in check yet
    
    // Position where black is in check
    let board = Board::from_fen("rnbq2rk/ppppbNp1/5n1p/4p3/4P3/8/PPPP1PPP/RNBQKB1R b KQ - 1 2").unwrap();
    assert!(board.is_in_check(Color::Black));  // Should be in check from knight
}

#[test]
fn test_checkmate() {
    // Fool's mate position
    let game = GameState::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3").unwrap();
    
    // Verify checkmate is detected
    assert!(game.checkmate);
    assert!(game.board.is_in_check(Color::White));
    
    // Verify no legal moves
    let king_pos = game.board.get_king_position(Color::White).unwrap();
    assert!(game.board.get_legal_moves(king_pos).is_empty());
}

#[test]
fn test_king_stalemate() {
    // Stalemate position - Black to move
    let mut game = GameState::from_fen("8/8/8/8/8/1k6/p7/K7 b - - 0 1").unwrap();
    
    // Black moves their king to a3
    let black_king_pos = game.board.get_king_position(Color::Black).unwrap();
    game.make_move(black_king_pos, Position::from_notation("a3").unwrap()).unwrap();
    
    // Verify no legal moves for white
    let white_king_pos = game.board.get_king_position(Color::White).unwrap();
    let king_moves = game.board.get_legal_moves(white_king_pos);
    assert!(king_moves.is_empty(), "King should have no legal moves");
    
    // Verify stalemate is detected
    assert!(!game.has_any_legal_moves(), "White should have no legal moves");
    assert!(!game.board.is_in_check(Color::White), "White king should not be in check");
    assert!(!game.checkmate, "Should not be checkmate");
    assert!(game.stalemate, "Should be stalemate");
}

#[test]
fn test_threefold_repetition() {
    let mut game = GameState::new();
    
    // Sequence of moves that will lead to the same position three times
    game.make_move(Position::from_notation("g1").unwrap(), Position::from_notation("f3").unwrap()).unwrap();
    game.make_move(Position::from_notation("g8").unwrap(), Position::from_notation("f6").unwrap()).unwrap();
    
    game.make_move(Position::from_notation("f3").unwrap(), Position::from_notation("g1").unwrap()).unwrap();
    game.make_move(Position::from_notation("f6").unwrap(), Position::from_notation("g8").unwrap()).unwrap();
    
    game.make_move(Position::from_notation("g1").unwrap(), Position::from_notation("f3").unwrap()).unwrap();
    game.make_move(Position::from_notation("g8").unwrap(), Position::from_notation("f6").unwrap()).unwrap();
    
    game.make_move(Position::from_notation("f3").unwrap(), Position::from_notation("g1").unwrap()).unwrap();
    game.make_move(Position::from_notation("f6").unwrap(), Position::from_notation("g8").unwrap()).unwrap();
    
    // At this point, the position has been repeated three times
    assert!(game.is_threefold_repetition(), "Should detect threefold repetition");
    assert!(game.stalemate, "Should be stalemate due to threefold repetition");
    assert!(!game.check, "Should not be in check");
    assert!(!game.checkmate, "Should not be checkmate");
}

#[test]
fn test_promotion() {
    // Position where pawn can promote
    let mut board = Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let a7 = Position::from_notation("a7").unwrap();
    let a8 = Position::from_notation("a8").unwrap();
    
    // Move pawn to promote
    board.move_piece(a7, a8).unwrap();
    
    // Verify promotion to queen (default promotion)
    let piece = board.get_piece(a8).unwrap();
    assert_eq!(piece.piece_type, PieceType::Queen);
    assert_eq!(piece.color, Color::White);
}

#[test]
fn test_castling_rights_after_king_move() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
    let e1 = Position::from_notation("e1").unwrap();
    let f1 = Position::from_notation("f1").unwrap();
    
    // Move king
    board.move_piece(e1, f1).unwrap();
    
    // Verify castling rights are lost after king moves
    let game = GameState { board, ..Default::default() };
    assert!(!game.board.castling_rights.contains('K'));
    assert!(!game.board.castling_rights.contains('Q'));
}

#[test]
fn test_pin_detection() {
    // Position where a piece is pinned to the king
    let board = Board::from_fen("r1bqkbnr/ppp1pppp/2n5/1B1p4/3P4/4P3/PPP2PPP/RNBQK1NR b KQkq - 0 1").unwrap();
    
    // Black knight on c6 is pinned to the king by the bishop on b5
    let moves = board.get_legal_moves(Position::from_notation("c6").unwrap());
    assert!(moves.is_empty());
}

#[test]
fn test_check_evasion() {
    // Position where king is in check and must move out of check
    let board = Board::from_fen("rnbq2rk/ppppbNp1/5n1p/4p3/4P3/8/PPPP1PPP/RNBQKB1R b KQ - 1 2").unwrap();
    
    let moves = board.get_legal_moves(board.get_king_position(Color::Black).unwrap());
    assert_eq!(moves.len(), 1);

    // Should not be able to castle out of check
    assert!(!moves.contains(&Position::from_notation("g1").unwrap()));
}

#[test]
fn test_knight_moves() {
    // Knight on e4 with no obstructions
    let board = Board::from_fen("4k3/8/8/8/4N3/8/8/4K3 w - - 0 1").unwrap();
    let e4 = Position::from_notation("e4").unwrap();

    let moves = board.get_legal_moves(e4);
    assert_eq!(moves.len(), 8); // Knight has 8 possible moves from centre

    // Verify L-shaped moves
    assert!(moves.contains(&Position::from_notation("d6").unwrap()));
    assert!(moves.contains(&Position::from_notation("f6").unwrap()));
    assert!(moves.contains(&Position::from_notation("g5").unwrap()));
    assert!(moves.contains(&Position::from_notation("g3").unwrap()));
    assert!(moves.contains(&Position::from_notation("f2").unwrap()));
    assert!(moves.contains(&Position::from_notation("d2").unwrap()));
    assert!(moves.contains(&Position::from_notation("c3").unwrap()));
    assert!(moves.contains(&Position::from_notation("c5").unwrap()));
}

#[test]
fn test_bishop_moves() {
    // Bishop on d4
    let board = Board::from_fen("4k3/8/8/8/3B4/8/8/4K3 w - - 0 1").unwrap();
    let d4 = Position::from_notation("d4").unwrap();

    let moves = board.get_legal_moves(d4);
    assert_eq!(moves.len(), 13); // Bishop has 13 moves from d4

    // Verify diagonal moves
    assert!(moves.contains(&Position::from_notation("a1").unwrap()));
    assert!(moves.contains(&Position::from_notation("h8").unwrap()));
    assert!(moves.contains(&Position::from_notation("a7").unwrap()));
    assert!(moves.contains(&Position::from_notation("g1").unwrap()));
}

#[test]
fn test_rook_moves() {
    // Rook on d4
    let board = Board::from_fen("4k3/8/8/8/3R4/8/8/4K3 w - - 0 1").unwrap();
    let d4 = Position::from_notation("d4").unwrap();

    let moves = board.get_legal_moves(d4);
    assert_eq!(moves.len(), 14); // Rook has 14 moves from d4

    // Verify horizontal and vertical moves
    assert!(moves.contains(&Position::from_notation("d1").unwrap()));
    assert!(moves.contains(&Position::from_notation("d8").unwrap()));
    assert!(moves.contains(&Position::from_notation("a4").unwrap()));
    assert!(moves.contains(&Position::from_notation("h4").unwrap()));
}

#[test]
fn test_queen_moves() {
    // Queen on d4
    let board = Board::from_fen("4k3/8/8/8/3Q4/8/8/4K3 w - - 0 1").unwrap();
    let d4 = Position::from_notation("d4").unwrap();

    let moves = board.get_legal_moves(d4);
    assert_eq!(moves.len(), 27); // Queen has 27 moves from d4 (13 diagonal + 14 straight)
}

#[test]
fn test_queenside_castling() {
    // Position where queenside castling is available
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
    let e1 = Position::from_notation("e1").unwrap();
    let c1 = Position::from_notation("c1").unwrap();

    let moves = board.get_legal_moves(e1);
    assert!(moves.contains(&c1), "Should be able to castle queenside");

    // Perform queenside castling
    board.move_piece(e1, c1).unwrap();

    // Verify castling happened
    assert_eq!(board.get_piece(c1).unwrap().piece_type, PieceType::King);
    assert_eq!(board.get_piece(Position::from_notation("d1").unwrap()).unwrap().piece_type, PieceType::Rook);
}

#[test]
fn test_castling_rights_after_rook_move() {
    // Position without pawns so rook can move
    let mut board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
    let a1 = Position::from_notation("a1").unwrap();
    let a3 = Position::from_notation("a3").unwrap();

    // Move queenside rook
    board.move_piece(a1, a3).unwrap();

    // Verify queenside castling rights are lost
    assert!(!board.castling_rights.contains('Q'), "Queenside castling should be lost after rook moves");
    assert!(board.castling_rights.contains('K'), "Kingside castling should still be available");
}

#[test]
fn test_cannot_castle_through_check() {
    // Position where f1 is attacked by bishop on b5, blocking kingside castling
    let board = Board::from_fen("4k3/8/8/1b6/8/8/8/R3K2R w KQ - 0 1").unwrap();
    let e1 = Position::from_notation("e1").unwrap();
    let g1 = Position::from_notation("g1").unwrap();

    let moves = board.get_legal_moves(e1);
    assert!(!moves.contains(&g1), "Should not be able to castle through attacked square");
}

#[test]
fn test_cannot_castle_while_in_check() {
    // Position where king is in check
    let board = Board::from_fen("4k3/8/8/8/4r3/8/8/R3K2R w KQ - 0 1").unwrap();
    let e1 = Position::from_notation("e1").unwrap();
    let g1 = Position::from_notation("g1").unwrap();
    let c1 = Position::from_notation("c1").unwrap();

    let moves = board.get_legal_moves(e1);
    assert!(!moves.contains(&g1), "Should not be able to castle kingside while in check");
    assert!(!moves.contains(&c1), "Should not be able to castle queenside while in check");
}

#[test]
fn test_pawn_blocked_by_piece() {
    // White pawn blocked by black pawn
    let board = Board::from_fen("4k3/8/8/8/4p3/4P3/8/4K3 w - - 0 1").unwrap();
    let e3 = Position::from_notation("e3").unwrap();

    let moves = board.get_legal_moves(e3);
    assert!(moves.is_empty(), "Pawn should have no moves when blocked");
}

#[test]
fn test_black_pawn_moves() {
    // Test black pawn initial double move and single move
    let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();
    let e7 = Position::from_notation("e7").unwrap();
    let e6 = Position::from_notation("e6").unwrap();
    let e5 = Position::from_notation("e5").unwrap();

    let moves = board.get_legal_moves(e7);
    assert_eq!(moves.len(), 2);
    assert!(moves.contains(&e6));
    assert!(moves.contains(&e5));

    // Test black pawn capture
    let board = Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2").unwrap();
    let d5 = Position::from_notation("d5").unwrap();
    let e4 = Position::from_notation("e4").unwrap();

    let moves = board.get_legal_moves(d5);
    assert!(moves.contains(&e4), "Black pawn should be able to capture on e4");
}

#[test]
fn test_double_check() {
    // Position with double check - only king can move
    let board = Board::from_fen("4k3/8/5N2/3b4/8/8/4R3/4K3 b - - 0 1").unwrap();

    // Black is in double check from knight and rook (via discovered check)
    assert!(board.is_in_check(Color::Black));

    // Only king moves should be legal
    let king_pos = board.get_king_position(Color::Black).unwrap();
    let _king_moves = board.get_legal_moves(king_pos);

    // Bishop cannot block double check
    let bishop_pos = Position::from_notation("d5").unwrap();
    let bishop_moves = board.get_legal_moves(bishop_pos);
    assert!(bishop_moves.is_empty(), "Bishop cannot move during double check");
}

#[test]
fn test_discovered_check() {
    // Position where moving the bishop reveals check from rook
    let board = Board::from_fen("4k3/8/8/4B3/8/8/4R3/4K3 w - - 0 1").unwrap();
    let e5 = Position::from_notation("e5").unwrap();

    // Bishop on e5 - some moves will deliver discovered check
    let moves = board.get_legal_moves(e5);

    // Moving bishop off the e-file reveals check from rook
    // Bishop should still be able to move (it's not pinned, the rook is behind it attacking the enemy king)
    assert!(!moves.is_empty());
}
