// src/tui/selection.rs

use crate::{
    board::{GameState, Position, Move},
    moves::get_valid_moves,
    tui::Tui,
};

// Deselects any currently selected piece and clears possible moves.
pub(crate) fn deselect_piece(tui: &mut Tui) {
    tui.selected_piece = None;
    tui.possible_moves.clear();
    // Status message can be set by the caller (e.g., key.rs)
}

// Tries to select the piece at the given position. Updates Tui state.
pub(crate) fn try_select_piece(tui: &mut Tui, game_state: &GameState, pos: Position) {
    if let Some(piece) = game_state.board.get_piece(pos) {
        if piece.color == game_state.active_color {
            tui.selected_piece = Some(pos);
            // Get legal moves for this piece
            // Filter out moves that would leave the king in check - this logic
            // should ideally be part of your `get_legal_moves` or a separate
            // `is_move_legal` function in your `board` or `moves` module.
            // For demonstration, we'll keep the filtering here for now.
            let candidate_moves = get_valid_moves(&game_state.board, pos);

            tui.possible_moves = candidate_moves.into_iter()
                .filter(|&to| {
                    let mut board_clone = game_state.board.clone();
                    // Use move_piece on the clone to test legality
                    board_clone.move_piece(pos, to).is_ok()
                })
                .map(|to| Move { from: pos, to, promotion: None }) // Assuming no promotion handling here yet
                .collect();

            if tui.possible_moves.is_empty() {
                tui.set_status("No legal moves for selected piece".to_string());
                deselect_piece(tui); // Deselect if no legal moves
            } else {
                 tui.set_status(format!("Selected {} at {}", piece, pos));
            }
        } else {
            tui.set_status("It's not your turn to move that piece".to_string());
        }
    } else {
        // Clicked on an empty square when nothing was selected
        deselect_piece(tui); // Ensure nothing is selected
    }
}

// Tries to make a move from the selected piece to the cursor/clicked position.
// Assumes a piece is already selected (`tui.selected_piece` is Some).
// Returns true if a move was successfully made, false otherwise.
pub(crate) fn try_make_move(tui: &mut Tui, game_state: &mut GameState) -> bool {
    let Some(_from_pos) = tui.selected_piece else {
        return false; // No piece selected
    };
    
    let to_pos = tui.cursor_position; // Use cursor position for keyboard input
    
    // Check if the cursor position is one of the possible moves
    let Some(mv) = tui.possible_moves.iter().find(|m| m.to == to_pos).cloned() else {
        tui.set_status("Not a legal move for the selected piece".to_string());
        return false; // Not a legal move
    };

    // Attempt to make the move on the actual game state
    match game_state.make_move(mv.from, mv.to) {
        Ok(()) => {
            // Update the game state after the move
            game_state.update_state();
            
            // Get the piece that was moved (it should exist after a successful move)
            let piece_str = game_state.board.get_piece(mv.to)
                .map(|p| p.to_string())
                .unwrap_or_else(|| "piece".to_string());
                
            tui.set_status(format!("Moved {} to {}", piece_str, mv.to));
            
            // Clear the selection and possible moves after a successful move
            deselect_piece(tui);
            
            true // Move was successful
        },
        Err(e) => {
            // This shouldn't happen if possible_moves contains only legal moves
            tui.set_status(format!("Invalid move: {}", e));
            false // Move failed
        }
    }
}
