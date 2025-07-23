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
fn test_turn_switching() {
    // Create a new game state
    let mut game_state = GameState::new();
    
    // Initially it should be White's turn
    assert_eq!(game_state.active_color, Color::White);
    
    // Make a move with White (e2 to e4)
    let from = Position::from_notation("e2").unwrap();
    let to = Position::from_notation("e4").unwrap();
    game_state.make_move(from, to).unwrap();
    
    // After White's move, it should be Black's turn
    assert_eq!(game_state.active_color, Color::Black);
    
    // Make a move with Black (e7 to e5)
    let from = Position::from_notation("e7").unwrap();
    let to = Position::from_notation("e5").unwrap();
    game_state.make_move(from, to).unwrap();
    
    // After Black's move, it should be White's turn again
    assert_eq!(game_state.active_color, Color::White);
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
