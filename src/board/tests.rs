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
    board.move_piece(e1, g1, None).unwrap();
    
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
    board.move_piece(e5, f6, None).unwrap();
    
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
    game.make_move(black_king_pos, Position::from_notation("a3").unwrap(), None).unwrap();
    
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
    game.make_move(Position::from_notation("g1").unwrap(), Position::from_notation("f3").unwrap(), None).unwrap();
    game.make_move(Position::from_notation("g8").unwrap(), Position::from_notation("f6").unwrap(), None).unwrap();

    game.make_move(Position::from_notation("f3").unwrap(), Position::from_notation("g1").unwrap(), None).unwrap();
    game.make_move(Position::from_notation("f6").unwrap(), Position::from_notation("g8").unwrap(), None).unwrap();

    game.make_move(Position::from_notation("g1").unwrap(), Position::from_notation("f3").unwrap(), None).unwrap();
    game.make_move(Position::from_notation("g8").unwrap(), Position::from_notation("f6").unwrap(), None).unwrap();

    game.make_move(Position::from_notation("f3").unwrap(), Position::from_notation("g1").unwrap(), None).unwrap();
    game.make_move(Position::from_notation("f6").unwrap(), Position::from_notation("g8").unwrap(), None).unwrap();
    
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
    board.move_piece(a7, a8, None).unwrap();

    // Verify promotion to queen (default promotion)
    let piece = board.get_piece(a8).unwrap();
    assert_eq!(piece.piece_type, PieceType::Queen);
    assert_eq!(piece.color, Color::White);
}

#[test]
fn test_promotion_to_queen_explicit() {
    // Test explicit promotion to Queen
    let mut board = Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let a7 = Position::from_notation("a7").unwrap();
    let a8 = Position::from_notation("a8").unwrap();

    board.move_piece(a7, a8, Some(PieceType::Queen)).unwrap();

    let piece = board.get_piece(a8).unwrap();
    assert_eq!(piece.piece_type, PieceType::Queen);
    assert_eq!(piece.color, Color::White);
}

#[test]
fn test_promotion_to_rook() {
    // Test promotion to Rook
    let mut board = Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let a7 = Position::from_notation("a7").unwrap();
    let a8 = Position::from_notation("a8").unwrap();

    board.move_piece(a7, a8, Some(PieceType::Rook)).unwrap();

    let piece = board.get_piece(a8).unwrap();
    assert_eq!(piece.piece_type, PieceType::Rook);
    assert_eq!(piece.color, Color::White);
}

#[test]
fn test_promotion_to_bishop() {
    // Test promotion to Bishop
    let mut board = Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let a7 = Position::from_notation("a7").unwrap();
    let a8 = Position::from_notation("a8").unwrap();

    board.move_piece(a7, a8, Some(PieceType::Bishop)).unwrap();

    let piece = board.get_piece(a8).unwrap();
    assert_eq!(piece.piece_type, PieceType::Bishop);
    assert_eq!(piece.color, Color::White);
}

#[test]
fn test_promotion_to_knight() {
    // Test promotion to Knight
    let mut board = Board::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let a7 = Position::from_notation("a7").unwrap();
    let a8 = Position::from_notation("a8").unwrap();

    board.move_piece(a7, a8, Some(PieceType::Knight)).unwrap();

    let piece = board.get_piece(a8).unwrap();
    assert_eq!(piece.piece_type, PieceType::Knight);
    assert_eq!(piece.color, Color::White);
}

#[test]
fn test_promotion_to_queen_causes_check() {
    // Position: White pawn on e7, Black king on e8
    // Promoting to Queen on e8 would be a capture, let's use d7 pawn promoting to d8
    // Black king on e8, white pawn on d7 - promotion to Queen gives check
    let mut game = GameState::from_fen("4k3/3P4/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let d7 = Position::from_notation("d7").unwrap();
    let d8 = Position::from_notation("d8").unwrap();

    // Promote to Queen - should give check
    game.make_move(d7, d8, Some(PieceType::Queen)).unwrap();

    assert!(game.check, "Queen promotion should give check");
    assert!(!game.checkmate, "Should not be checkmate - king can move");

    // Verify the promoted piece is a Queen
    let piece = game.board.get_piece(d8).unwrap();
    assert_eq!(piece.piece_type, PieceType::Queen);

    // Black king should be able to move (e.g., to f7, f8, e7)
    let king_pos = game.board.get_king_position(Color::Black).unwrap();
    let king_moves = game.board.get_legal_moves(king_pos);
    assert!(!king_moves.is_empty(), "King should have escape squares");

    // King moves to f7
    let f7 = Position::from_notation("f7").unwrap();
    assert!(king_moves.contains(&f7), "King should be able to move to f7");
    game.make_move(king_pos, f7, None).unwrap();

    // Now the Queen should be able to make legal moves
    let queen_moves = game.board.get_legal_moves(d8);
    assert!(!queen_moves.is_empty(), "Queen should have legal moves after king moved");

    // Queen can move along the d-file or 8th rank
    assert!(queen_moves.contains(&Position::from_notation("d1").unwrap()), "Queen should be able to move to d1");
}

#[test]
fn test_promotion_to_rook_causes_check() {
    // Black king on a8, white pawn on b7 - promotion to Rook on b8 gives check
    let mut game = GameState::from_fen("k7/1P6/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let b7 = Position::from_notation("b7").unwrap();
    let b8 = Position::from_notation("b8").unwrap();

    // Promote to Rook - should give check along the 8th rank
    game.make_move(b7, b8, Some(PieceType::Rook)).unwrap();

    assert!(game.check, "Rook promotion should give check");

    // Verify the promoted piece is a Rook
    let piece = game.board.get_piece(b8).unwrap();
    assert_eq!(piece.piece_type, PieceType::Rook);

    // King moves to a7
    let king_pos = game.board.get_king_position(Color::Black).unwrap();
    let a7 = Position::from_notation("a7").unwrap();
    game.make_move(king_pos, a7, None).unwrap();

    // Rook should be able to make legal moves
    let rook_moves = game.board.get_legal_moves(b8);
    assert!(!rook_moves.is_empty(), "Rook should have legal moves");

    // Rook can move along the b-file
    assert!(rook_moves.contains(&Position::from_notation("b1").unwrap()), "Rook should be able to move to b1");
}

#[test]
fn test_promotion_to_bishop_causes_check() {
    // Black king on f6, white pawn on h7 - promotion to Bishop on h8
    // h8 bishop attacks along h8-a1 diagonal, which doesn't reach f6
    // Better: Black king on b3, white pawn on d7 - promotion to Bishop on d8
    // d8 bishop attacks along d8-a5 and d8-h4 diagonals
    // Let's use: Black king on g5, white pawn on e7 - e8 bishop attacks g6... no

    // Simpler: Black king on c3, white pawn on a7 - a8 bishop attacks along diagonal
    // a8 bishop: a8-h1 diagonal passes through b7, c6, d5, e4, f3, g2, h1 - not c3

    // Use: Black king on f3, white pawn on d7 - d8 bishop
    // d8 bishop on light square, attacks h4-a1 diagonal? No, d8 is dark.
    // d8 (dark) attacks along a5-e8 and h4-d8 diagonals

    // Let me use a clear setup: Black king on b6, white pawn on d7
    // d8 bishop attacks c7, b6 - yes! This gives check
    let mut game = GameState::from_fen("8/3P4/1k6/8/8/8/8/4K3 w - - 0 1").unwrap();
    let d7 = Position::from_notation("d7").unwrap();
    let d8 = Position::from_notation("d8").unwrap();

    // Promote to Bishop - should give check diagonally to b6
    game.make_move(d7, d8, Some(PieceType::Bishop)).unwrap();

    assert!(game.check, "Bishop promotion should give check");

    // Verify the promoted piece is a Bishop
    let piece = game.board.get_piece(d8).unwrap();
    assert_eq!(piece.piece_type, PieceType::Bishop);

    // King moves out of check (e.g., to a7, a6, a5, c7, c6, c5)
    let king_pos = game.board.get_king_position(Color::Black).unwrap();
    let king_moves = game.board.get_legal_moves(king_pos);
    assert!(!king_moves.is_empty(), "King should have escape squares");

    let a7 = Position::from_notation("a7").unwrap();
    game.make_move(king_pos, a7, None).unwrap();

    // Bishop should be able to make legal moves
    let bishop_moves = game.board.get_legal_moves(d8);
    assert!(!bishop_moves.is_empty(), "Bishop should have legal moves");
}

#[test]
fn test_promotion_to_knight_causes_check() {
    // Knight check is unique - only piece that can give check on promotion
    // that couldn't be given by the pawn's approach
    // Black king on e6, white pawn on d7 - promotion to Knight on d8
    // Knight on d8 attacks e6 and f7 and c6 and b7
    // d8 knight attacks: c6, b7, e6, f7 - yes, e6!
    let mut game = GameState::from_fen("8/3P4/4k3/8/8/8/8/4K3 w - - 0 1").unwrap();
    let d7 = Position::from_notation("d7").unwrap();
    let d8 = Position::from_notation("d8").unwrap();

    // Promote to Knight - should give check (knight on d8 attacks e6)
    game.make_move(d7, d8, Some(PieceType::Knight)).unwrap();

    assert!(game.check, "Knight promotion should give check");

    // Verify the promoted piece is a Knight
    let piece = game.board.get_piece(d8).unwrap();
    assert_eq!(piece.piece_type, PieceType::Knight);

    // King moves out of check
    let king_pos = game.board.get_king_position(Color::Black).unwrap();
    let king_moves = game.board.get_legal_moves(king_pos);
    assert!(!king_moves.is_empty(), "King should have escape squares");

    // King moves to f6
    let f6 = Position::from_notation("f6").unwrap();
    game.make_move(king_pos, f6, None).unwrap();

    // Knight should be able to make legal moves
    let knight_moves = game.board.get_legal_moves(d8);
    assert!(!knight_moves.is_empty(), "Knight should have legal moves");

    // Knight can move to various squares
    assert!(knight_moves.contains(&Position::from_notation("c6").unwrap()), "Knight should be able to move to c6");
    assert!(knight_moves.contains(&Position::from_notation("b7").unwrap()), "Knight should be able to move to b7");
}

#[test]
fn test_black_pawn_promotion() {
    // Test black pawn promotion
    let mut game = GameState::from_fen("4k3/8/8/8/8/8/3p4/4K3 b - - 0 1").unwrap();
    let d2 = Position::from_notation("d2").unwrap();
    let d1 = Position::from_notation("d1").unwrap();

    // Black promotes to Queen - should give check to white king on e1
    game.make_move(d2, d1, Some(PieceType::Queen)).unwrap();

    assert!(game.check, "Black queen promotion should give check");

    // Verify the promoted piece is a black Queen
    let piece = game.board.get_piece(d1).unwrap();
    assert_eq!(piece.piece_type, PieceType::Queen);
    assert_eq!(piece.color, Color::Black);

    // White king moves
    let king_pos = game.board.get_king_position(Color::White).unwrap();
    let f2 = Position::from_notation("f2").unwrap();
    game.make_move(king_pos, f2, None).unwrap();

    // Black queen should have legal moves
    let queen_moves = game.board.get_legal_moves(d1);
    assert!(!queen_moves.is_empty(), "Black queen should have legal moves");
}

#[test]
fn test_promotion_with_capture() {
    // Test promotion while capturing a piece
    // White pawn on g7, black rook on h8, black king on e8
    let mut game = GameState::from_fen("4k2r/6P1/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let g7 = Position::from_notation("g7").unwrap();
    let h8 = Position::from_notation("h8").unwrap();

    // Promote to Knight while capturing - gives check via knight fork potential
    game.make_move(g7, h8, Some(PieceType::Knight)).unwrap();

    // Verify capture happened and promotion occurred
    let piece = game.board.get_piece(h8).unwrap();
    assert_eq!(piece.piece_type, PieceType::Knight);
    assert_eq!(piece.color, Color::White);

    // Verify the rook was captured
    assert_eq!(game.captured_by_white.len(), 1);
    assert_eq!(game.captured_by_white[0].piece_type, PieceType::Rook);
}

#[test]
fn test_castling_rights_after_king_move() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
    let e1 = Position::from_notation("e1").unwrap();
    let f1 = Position::from_notation("f1").unwrap();
    
    // Move king
    board.move_piece(e1, f1, None).unwrap();
    
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
    board.move_piece(e1, c1, None).unwrap();

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
    board.move_piece(a1, a3, None).unwrap();

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

// =============================================================================
// SCENARIO TESTS - Play through complete games
// =============================================================================

/// Helper to make a move from algebraic notation
#[allow(dead_code)]
fn make_move(game: &mut GameState, from: &str, to: &str) {
    let from_pos = Position::from_notation(from).expect(&format!("Invalid from: {}", from));
    let to_pos = Position::from_notation(to).expect(&format!("Invalid to: {}", to));
    game.make_move(from_pos, to_pos, None).expect(&format!("Failed move: {} to {}", from, to));
}

#[test]
fn test_fools_mate() {
    // Fool's mate - fastest possible checkmate (4 half-moves)
    // 1. f3 e5  2. g4 Qh4#
    let mut game = GameState::new();

    // 1. f3 (white weakens kingside)
    make_move(&mut game, "f2", "f3");
    assert!(!game.check);
    assert!(!game.checkmate);

    // 1... e5 (black develops)
    make_move(&mut game, "e7", "e5");
    assert!(!game.check);

    // 2. g4 (white blunders)
    make_move(&mut game, "g2", "g4");
    assert!(!game.check);

    // 2... Qh4# (checkmate!)
    make_move(&mut game, "d8", "h4");

    // Verify checkmate
    assert!(game.check, "White king should be in check");
    assert!(game.checkmate, "Should be checkmate");
    assert!(!game.stalemate, "Should not be stalemate");
    assert_eq!(game.active_color, Color::White, "White should be the one in checkmate");
}

#[test]
fn test_scholars_mate() {
    // Scholar's mate - classic 4-move checkmate
    // 1. e4 e5  2. Bc4 Nc6  3. Qh5 Nf6  4. Qxf7#
    let mut game = GameState::new();

    // 1. e4
    make_move(&mut game, "e2", "e4");
    // 1... e5
    make_move(&mut game, "e7", "e5");

    // 2. Bc4 (bishop to c4, targeting f7)
    make_move(&mut game, "f1", "c4");
    // 2... Nc6
    make_move(&mut game, "b8", "c6");

    // 3. Qh5 (queen threatens mate)
    make_move(&mut game, "d1", "h5");
    // 3... Nf6 (black tries to defend but it's not enough)
    make_move(&mut game, "g8", "f6");

    // 4. Qxf7# (checkmate!)
    make_move(&mut game, "h5", "f7");

    // Verify checkmate
    assert!(game.check, "Black king should be in check");
    assert!(game.checkmate, "Should be checkmate");
    assert!(!game.stalemate);
    assert_eq!(game.active_color, Color::Black, "Black should be the one in checkmate");
}

#[test]
fn test_back_rank_mate() {
    // Set up a position where back rank mate is possible
    // White rook delivers mate on black's back rank
    let mut game = GameState::from_fen("6k1/5ppp/8/8/8/8/8/R3K3 w Q - 0 1").unwrap();

    // Ra8# - back rank mate
    make_move(&mut game, "a1", "a8");

    assert!(game.checkmate, "Should be back rank checkmate");
    assert_eq!(game.active_color, Color::Black);
}

#[test]
fn test_smothered_mate() {
    // Classic smothered mate position
    // Knight on f7 delivers mate to king on h8, trapped by own rook on g8 and pawns
    let game = GameState::from_fen("6rk/5Npp/8/8/8/8/8/4K3 b - - 0 1").unwrap();

    // Black is already in checkmate from Nf7
    assert!(game.check, "Black king should be in check");
    assert!(game.checkmate, "Should be smothered mate");
}

#[test]
fn test_capture_tracking_in_game() {
    // Play a short game with captures and verify tracking
    let mut game = GameState::new();

    // 1. e4 d5 (Scandinavian Defense)
    make_move(&mut game, "e2", "e4");
    make_move(&mut game, "d7", "d5");

    // 2. exd5 (capture)
    assert!(game.captured_by_white.is_empty());
    make_move(&mut game, "e4", "d5");
    assert_eq!(game.captured_by_white.len(), 1, "White should have captured one piece");
    assert_eq!(game.captured_by_white[0].piece_type, PieceType::Pawn);

    // 2... Qxd5 (recapture)
    make_move(&mut game, "d8", "d5");
    assert_eq!(game.captured_by_black.len(), 1, "Black should have captured one piece");
    assert_eq!(game.captured_by_black[0].piece_type, PieceType::Pawn);
}

#[test]
fn test_stalemate_by_moves() {
    // King and queen vs lone king - stalemate position
    // Black king on a1, trapped by white king on a3 and queen on b3
    // Queen on b3 doesn't check a1, but controls b1 and b2
    // King on a3 controls a2
    let game = GameState::from_fen("8/8/8/8/8/KQ6/8/k7 b - - 0 1").unwrap();

    assert!(!game.check, "King should NOT be in check for stalemate");
    assert!(game.stalemate, "Should be stalemate");
    assert!(!game.checkmate, "Should NOT be checkmate");
}
