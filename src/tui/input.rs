// src/tui/input.rs

use anyhow::Result;
use crossterm::event::Event;
use crate::{
    board::GameState,
    tui::{Tui, key}
};

type TuiResult<T> = Result<T, anyhow::Error>;

pub(crate) fn handle_input(tui: &mut Tui, game_state: &mut GameState) -> TuiResult<()> {
    if crossterm::event::poll(std::time::Duration::from_millis(100))? {
        match crossterm::event::read()? {
            Event::Key(key) => key::handle_key_event(tui, game_state, key)?, // Delegate to key module with key event
            _ => {}
        }
    }
    Ok(())
}
