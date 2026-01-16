use std::fs;
use std::path::Path;
use chrono::Local;

/// Directory for storing PGN files
const PGN_DIR: &str = "pgn";

use crate::board::{GameState, Position};
use crate::pieces::{Color, PieceType};

/// Ensure the PGN directory exists, creating it if necessary
fn ensure_pgn_dir() -> Result<(), String> {
    let path = Path::new(PGN_DIR);
    if !path.exists() {
        fs::create_dir(path).map_err(|e| format!("Failed to create pgn directory: {}", e))?;
    }
    Ok(())
}

/// Generate a unique filename for saving a game (includes pgn/ directory)
pub fn generate_save_filename(white_name: &str, black_name: &str) -> String {
    let timestamp = Local::now().format("%Y-%m-%d_%H%M%S");
    // Sanitize names for filename (remove/replace invalid characters)
    let white_safe = sanitize_filename(white_name);
    let black_safe = sanitize_filename(black_name);
    format!("{}/{}-{}-{}.pgn", PGN_DIR, white_safe, black_safe, timestamp)
}

/// Sanitize a string for use in a filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// List all PGN files in the pgn directory
pub fn list_pgn_files() -> Vec<String> {
    let pgn_path = Path::new(PGN_DIR);
    if !pgn_path.exists() {
        return Vec::new();
    }

    let mut files: Vec<String> = fs::read_dir(pgn_path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let path = e.path();
                    if path.extension().map(|ext| ext == "pgn").unwrap_or(false) {
                        // Return full path including pgn/ directory
                        path.to_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    // Sort by modification time (newest first)
    files.sort_by(|a, b| {
        let time_a = fs::metadata(a).and_then(|m| m.modified()).ok();
        let time_b = fs::metadata(b).and_then(|m| m.modified()).ok();
        time_b.cmp(&time_a)
    });

    files
}

/// Fuzzy match a query against a filename (case-insensitive substring match)
pub fn fuzzy_match(filename: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let filename_lower = filename.to_lowercase();
    let query_lower = query.to_lowercase();
    filename_lower.contains(&query_lower)
}

/// Parse player names from PGN content
/// Returns (white_name, black_name)
pub fn parse_player_names(path: &str) -> Result<(String, String), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read PGN file: {}", e))?;

    let mut white_name = "White".to_string();
    let mut black_name = "Black".to_string();

    for line in content.lines() {
        if line.starts_with("[White \"") {
            if let Some(name) = line.strip_prefix("[White \"").and_then(|s| s.strip_suffix("\"]")) {
                white_name = name.to_string();
            }
        } else if line.starts_with("[Black \"") {
            if let Some(name) = line.strip_prefix("[Black \"").and_then(|s| s.strip_suffix("\"]")) {
                black_name = name.to_string();
            }
        }
    }

    Ok((white_name, black_name))
}

/// Export the current game state to a PGN file
pub fn export_pgn(game_state: &GameState, path: &str, white_name: &str, black_name: &str) -> Result<(), String> {
    // Ensure pgn directory exists if saving to pgn folder
    if path.starts_with(PGN_DIR) {
        ensure_pgn_dir()?;
    }

    let mut pgn = String::new();

    // Write headers
    pgn.push_str("[Event \"CLI Chess Game\"]\n");
    pgn.push_str("[Site \"Terminal\"]\n");
    pgn.push_str(&format!("[Date \"{}\"]\n", Local::now().format("%Y.%m.%d")));
    pgn.push_str("[Round \"1\"]\n");
    pgn.push_str(&format!("[White \"{}\"]\n", white_name));
    pgn.push_str(&format!("[Black \"{}\"]\n", black_name));

    // Determine result
    let result = if game_state.checkmate {
        match game_state.active_color {
            Color::White => "0-1", // Black wins
            Color::Black => "1-0", // White wins
        }
    } else if game_state.stalemate {
        "1/2-1/2"
    } else {
        "*" // Game in progress
    };
    pgn.push_str(&format!("[Result \"{}\"]\n", result));
    pgn.push_str("\n");

    // Write moves
    let moves = &game_state.move_history;
    for (i, chunk) in moves.chunks(2).enumerate() {
        let move_num = i + 1;
        pgn.push_str(&format!("{}. ", move_num));

        // White's move
        pgn.push_str(&chunk[0].to_algebraic(false));

        // Black's move (if exists)
        if let Some(black_move) = chunk.get(1) {
            pgn.push_str(" ");
            pgn.push_str(&black_move.to_algebraic(false));
        }

        pgn.push_str(" ");
    }

    // Add result at end
    pgn.push_str(result);
    pgn.push('\n');

    fs::write(path, pgn).map_err(|e| format!("Failed to write PGN file: {}", e))
}

/// Import a game from a PGN file
pub fn import_pgn(path: &str) -> Result<GameState, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read PGN file: {}", e))?;

    let mut game_state = GameState::new();

    // Skip header lines (lines starting with '[')
    // Find the moves section
    let moves_section: String = content
        .lines()
        .filter(|line| !line.starts_with('[') && !line.trim().is_empty())
        .collect::<Vec<&str>>()
        .join(" ");

    // Parse moves - remove move numbers and result markers
    let tokens: Vec<&str> = moves_section
        .split_whitespace()
        .filter(|token| {
            // Skip move numbers (e.g., "1.", "2.")
            if token.ends_with('.') && token[..token.len()-1].parse::<u32>().is_ok() {
                return false;
            }
            // Skip results
            if *token == "1-0" || *token == "0-1" || *token == "1/2-1/2" || *token == "*" {
                return false;
            }
            true
        })
        .collect();

    // Parse and execute each move
    for notation in tokens {
        let (from, to, promotion) = parse_algebraic_move(&game_state, notation)?;
        game_state.make_move(from, to, promotion)
            .map_err(|e| format!("Invalid move '{}': {}", notation, e))?;
    }

    Ok(game_state)
}

/// Parse algebraic notation (e.g., "Nf3", "exd5", "O-O") into from/to positions
fn parse_algebraic_move(game_state: &GameState, notation: &str) -> Result<(Position, Position, Option<PieceType>), String> {
    let notation = notation.trim();

    // Handle castling
    if notation == "O-O" || notation == "0-0" {
        let rank = if game_state.active_color == Color::White { 0 } else { 7 };
        let from = Position::new(4, rank).unwrap(); // King's position
        let to = Position::new(6, rank).unwrap();   // Kingside castle destination
        return Ok((from, to, None));
    }
    if notation == "O-O-O" || notation == "0-0-0" {
        let rank = if game_state.active_color == Color::White { 0 } else { 7 };
        let from = Position::new(4, rank).unwrap();
        let to = Position::new(2, rank).unwrap();   // Queenside castle destination
        return Ok((from, to, None));
    }

    // Remove check/checkmate indicators
    let notation = notation.trim_end_matches('+').trim_end_matches('#');

    // Parse promotion (e.g., "e8=Q")
    let (notation, promotion) = if notation.contains('=') {
        let parts: Vec<&str> = notation.split('=').collect();
        let promo_piece = match parts.get(1).and_then(|s| s.chars().next()) {
            Some('Q') => Some(PieceType::Queen),
            Some('R') => Some(PieceType::Rook),
            Some('B') => Some(PieceType::Bishop),
            Some('N') => Some(PieceType::Knight),
            _ => return Err(format!("Invalid promotion in '{}'", notation)),
        };
        (parts[0], promo_piece)
    } else {
        (notation, None)
    };

    // Determine piece type from first character
    let (piece_type, notation) = match notation.chars().next() {
        Some('K') => (PieceType::King, &notation[1..]),
        Some('Q') => (PieceType::Queen, &notation[1..]),
        Some('R') => (PieceType::Rook, &notation[1..]),
        Some('B') => (PieceType::Bishop, &notation[1..]),
        Some('N') => (PieceType::Knight, &notation[1..]),
        Some(c) if c.is_lowercase() => (PieceType::Pawn, notation),
        _ => return Err(format!("Invalid notation '{}'", notation)),
    };

    // Remove capture indicator
    let notation = notation.replace('x', "");

    // Parse destination square (last 2 characters)
    if notation.len() < 2 {
        return Err(format!("Invalid notation '{}'", notation));
    }
    let dest_str = &notation[notation.len()-2..];
    let to = Position::from_notation(dest_str)
        .map_err(|_| format!("Invalid destination square '{}'", dest_str))?;

    // Parse disambiguation (characters before destination)
    let disambig = &notation[..notation.len()-2];

    // Find the piece that can make this move
    let from = find_piece_for_move(game_state, piece_type, to, disambig)?;

    Ok((from, to, promotion))
}

/// Find which piece of the given type can move to the destination
fn find_piece_for_move(
    game_state: &GameState,
    piece_type: PieceType,
    to: Position,
    disambig: &str,
) -> Result<Position, String> {
    let color = game_state.active_color;
    let mut candidates: Vec<Position> = Vec::new();

    // Iterate over all squares to find pieces of the right type
    for y in 0..8 {
        for x in 0..8 {
            let from = Position::new(x, y).unwrap();

            // Check if there's a piece of the right type and color
            if let Some(piece) = game_state.board.get_piece(from) {
                if piece.piece_type != piece_type || piece.color != color {
                    continue;
                }

                // Check if this piece can legally move to the destination
                let legal_destinations = game_state.board.get_legal_moves(from);
                if !legal_destinations.contains(&to) {
                    continue;
                }

                // Check disambiguation if present
                if !disambig.is_empty() {
                    let from_notation = from.to_notation();
                    let mut matches = true;
                    for c in disambig.chars() {
                        if c.is_ascii_alphabetic() {
                            // File disambiguation (e.g., "Rab1" - 'a' specifies which rook)
                            if from_notation.chars().next() != Some(c) {
                                matches = false;
                                break;
                            }
                        } else if c.is_ascii_digit() {
                            // Rank disambiguation (e.g., "R1a3" - '1' specifies which rook)
                            if from_notation.chars().nth(1) != Some(c) {
                                matches = false;
                                break;
                            }
                        }
                    }
                    if !matches {
                        continue;
                    }
                }

                candidates.push(from);
            }
        }
    }

    match candidates.len() {
        0 => Err(format!("No {} can move to {}", piece_type_name(piece_type), to.to_notation())),
        1 => Ok(candidates[0]),
        _ => Err(format!("Ambiguous move: multiple {}s can move to {}", piece_type_name(piece_type), to.to_notation())),
    }
}

fn piece_type_name(piece_type: PieceType) -> &'static str {
    match piece_type {
        PieceType::King => "king",
        PieceType::Queen => "queen",
        PieceType::Rook => "rook",
        PieceType::Bishop => "bishop",
        PieceType::Knight => "knight",
        PieceType::Pawn => "pawn",
        PieceType::Empty => "empty",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_export_pgn_empty_game() {
        let game_state = GameState::new();
        let path = "/tmp/test_empty_game.pgn";

        export_pgn(&game_state, path, "Alice", "Bob").unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("[Event \"CLI Chess Game\"]"));
        assert!(content.contains("[White \"Alice\"]"));
        assert!(content.contains("[Black \"Bob\"]"));
        assert!(content.contains("[Result \"*\"]"));
        // No moves, just result
        assert!(content.ends_with("*\n"));

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_export_pgn_with_moves() {
        let mut game_state = GameState::new();

        // Play e4 e5
        game_state.make_move(
            Position::from_notation("e2").unwrap(),
            Position::from_notation("e4").unwrap(),
            None,
        ).unwrap();
        game_state.make_move(
            Position::from_notation("e7").unwrap(),
            Position::from_notation("e5").unwrap(),
            None,
        ).unwrap();

        let path = "/tmp/test_with_moves.pgn";
        export_pgn(&game_state, path, "P1", "P2").unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("1. e4 e5"));

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_import_pgn_simple() {
        // Create a PGN file with simple moves
        let pgn_content = r#"[Event "Test Game"]
[Result "*"]

1. e4 e5 2. Nf3 *
"#;
        let path = "/tmp/test_import_simple.pgn";
        fs::write(path, pgn_content).unwrap();

        let game_state = import_pgn(path).unwrap();

        // Should have 3 moves played
        assert_eq!(game_state.move_history.len(), 3);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_roundtrip_pgn() {
        let mut original = GameState::new();

        // Play some moves: e4 e5 Nf3 Nc6
        original.make_move(
            Position::from_notation("e2").unwrap(),
            Position::from_notation("e4").unwrap(),
            None,
        ).unwrap();
        original.make_move(
            Position::from_notation("e7").unwrap(),
            Position::from_notation("e5").unwrap(),
            None,
        ).unwrap();
        original.make_move(
            Position::from_notation("g1").unwrap(),
            Position::from_notation("f3").unwrap(),
            None,
        ).unwrap();
        original.make_move(
            Position::from_notation("b8").unwrap(),
            Position::from_notation("c6").unwrap(),
            None,
        ).unwrap();

        let path = "/tmp/test_roundtrip.pgn";
        export_pgn(&original, path, "White", "Black").unwrap();

        let loaded = import_pgn(path).unwrap();

        // Both should have same number of moves
        assert_eq!(original.move_history.len(), loaded.move_history.len());

        // Active color should be the same
        assert_eq!(original.active_color, loaded.active_color);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_import_pgn_with_castling() {
        // Create a PGN file with kingside castling
        let pgn_content = r#"[Event "Castling Test"]
[Result "*"]

1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. O-O *
"#;
        let path = "/tmp/test_castling.pgn";
        fs::write(path, pgn_content).unwrap();

        let game_state = import_pgn(path).unwrap();

        // Should have 7 moves (including castling)
        assert_eq!(game_state.move_history.len(), 7);

        // Check that the king is on g1 after castling
        let king_pos = Position::from_notation("g1").unwrap();
        let piece = game_state.board.get_piece(king_pos);
        assert!(piece.is_some());
        assert_eq!(piece.unwrap().piece_type, PieceType::King);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_save_overwrite_deletes_old_file() {
        // Simulate the save-overwrite behavior:
        // 1. Save a game to a file
        // 2. Delete old file and save to new file with different names
        // 3. Verify old file is gone, new file exists with correct names

        let mut game_state = GameState::new();
        // Play a move
        game_state.make_move(
            Position::from_notation("e2").unwrap(),
            Position::from_notation("e4").unwrap(),
            None,
        ).unwrap();

        // Initial save with original names
        let old_path = "/tmp/test_overwrite_Alice-Bob-old.pgn";
        export_pgn(&game_state, old_path, "Alice", "Bob").unwrap();
        assert!(std::path::Path::new(old_path).exists(), "Old file should exist after initial save");

        // Verify old file has correct player names
        let old_content = fs::read_to_string(old_path).unwrap();
        assert!(old_content.contains("[White \"Alice\"]"));
        assert!(old_content.contains("[Black \"Bob\"]"));

        // Play another move
        game_state.make_move(
            Position::from_notation("e7").unwrap(),
            Position::from_notation("e5").unwrap(),
            None,
        ).unwrap();

        // Simulate overwrite: delete old file, save to new file with different names
        fs::remove_file(old_path).unwrap();
        let new_path = "/tmp/test_overwrite_Charlie-Dave-new.pgn";
        export_pgn(&game_state, new_path, "Charlie", "Dave").unwrap();

        // Verify old file is deleted
        assert!(!std::path::Path::new(old_path).exists(), "Old file should be deleted");

        // Verify new file exists with updated names and moves
        assert!(std::path::Path::new(new_path).exists(), "New file should exist");
        let new_content = fs::read_to_string(new_path).unwrap();
        assert!(new_content.contains("[White \"Charlie\"]"));
        assert!(new_content.contains("[Black \"Dave\"]"));
        assert!(new_content.contains("1. e4 e5"));

        // Cleanup
        fs::remove_file(new_path).ok();
    }

    #[test]
    fn test_generate_save_filename_changes_with_names() {
        // Verify that different player names generate different filenames
        let filename1 = generate_save_filename("Alice", "Bob");
        let filename2 = generate_save_filename("Charlie", "Dave");

        assert!(filename1.contains("Alice-Bob"), "Filename should contain player names");
        assert!(filename2.contains("Charlie-Dave"), "Filename should contain player names");
        assert_ne!(filename1, filename2, "Different names should produce different filenames");
    }
}
