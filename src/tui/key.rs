// src/tui/key.rs

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use crate::{
    board::{GameState, Position},
    tui::{Tui, selection}
};

type TuiResult<T> = Result<T, anyhow::Error>;

pub(crate) fn handle_key_event(tui: &mut Tui, game_state: &mut GameState, key: KeyEvent) -> TuiResult<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            if tui.selected_piece.is_some() {
                selection::deselect_piece(tui); // Delegate to selection module
            } else {
                tui.set_should_quit(true); // Call method on Tui struct
            }
        }
        KeyCode::Char('r') => {
            reset_game(tui, game_state); // Call local helper
        }
        KeyCode::Char('m') => {
            toggle_mouse(tui); // Call local helper
        }
        KeyCode::Up => move_cursor(tui, 0, 1), // Call local helper
        KeyCode::Down => move_cursor(tui, 0, -1), // Call local helper
        KeyCode::Left => move_cursor(tui, -1, 0), // Call local helper
        KeyCode::Right => move_cursor(tui, 1, 0), // Call local helper
        KeyCode::Enter => {
            handle_enter_key(tui, game_state)?; // Call local helper
        }
        _ => {}
    }
    Ok(())
}

// --- Helper functions for key events ---

fn reset_game(tui: &mut Tui, game_state: &mut GameState) {
    *game_state = GameState::new();
    selection::deselect_piece(tui); // This will clear the status message
    // Don't set status here since deselect_piece already does it
}

fn toggle_mouse(tui: &mut Tui) {
    tui.mouse_enabled = !tui.mouse_enabled;
    tui.set_status(format!(
        "Mouse {}",
        if tui.mouse_enabled { "enabled" } else { "disabled" }
    ));
}

fn move_cursor(tui: &mut Tui, dx: i8, dy: i8) {
    let new_x = (tui.cursor_position.x as i8 + dx).clamp(0, 7);
    let new_y = (tui.cursor_position.y as i8 + dy).clamp(0, 7);

    if let Some(new_pos) = Position::new(new_x, new_y) {
        tui.cursor_position = new_pos;
    }
}

fn handle_enter_key(tui: &mut Tui, game_state: &mut GameState) -> TuiResult<()> {
    if tui.selected_piece.is_some() {
        // Try to make a move using the selection module
        // Note: turn switching is handled within try_make_move via game_state.make_move
        let move_made = selection::try_make_move(tui, game_state);
        if !move_made {
            // Only deselect if move wasn't made (if move was made, it's already handled)
            selection::deselect_piece(tui);
        }
    } else {
        // If no piece is selected, try to select the piece at the cursor position
        selection::try_select_piece(tui, game_state, tui.cursor_position);
    }
    Ok(())
}
