// src/tui/mod.rs

use anyhow::{Result, Context};
use std::time::Instant;

use crossterm::{
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

use crate::{
    board::{GameState, Position, Move},
};

// Declare the modules in this crate
pub(crate) mod draw;
pub(crate) mod input;
pub(crate) mod key;
pub(crate) mod selection;

type TuiResult<T> = Result<T, anyhow::Error>;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<std::io::Stderr>>,
    pub(crate) mouse_enabled: bool,
    pub(crate) status_message: String,
    pub(crate) status_timer: Option<Instant>,
    pub(crate) cursor_position: Position,
    pub(crate) selected_piece: Option<Position>, // Keep selection state here
    pub(crate) possible_moves: Vec<Move>,      // Keep possible moves here
    should_quit: bool,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
        terminal.hide_cursor()?;

        Ok(Self {
            terminal,
            mouse_enabled: false,
            status_message: String::new(),
            status_timer: None,
            cursor_position: Position::new(0, 0).expect("Invalid initial cursor position"),
            selected_piece: None, // Managed by selection module, state held here
            possible_moves: Vec::new(), // Managed by selection module, state held here
            // No need to store game state reference
            should_quit: false,
        })
    }

    pub fn run(&mut self, game_state: &mut GameState) -> TuiResult<()> {
        self.setup()?;

        while !self.should_quit {
            self.draw(game_state)?;
            self.handle_input(game_state)?;
        }

        self.cleanup()
    }

    fn setup(&mut self) -> TuiResult<()> {
        enable_raw_mode().context("Failed to enable raw mode")?;
        execute!(std::io::stderr(), EnterAlternateScreen)
            .context("Failed to enter alternate screen")?;
        self.terminal.clear().context("Failed to clear terminal")?;
        Ok(())
    }

    pub fn cleanup(&mut self) -> TuiResult<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor().context("Failed to show cursor")?;
        Ok(())
    }

    fn draw(&mut self, game_state: &GameState) -> TuiResult<()> {
        // Clear expired status message
        if let Some(timer) = self.status_timer {
            if timer.elapsed().as_secs() >= 5 {
                self.status_message.clear();
                self.status_timer = None;
            }
        }

        // Call drawing logic from the draw module
        draw::draw_ui(self, game_state)
    }

    fn handle_input(&mut self, game_state: &mut GameState) -> TuiResult<()> {
        // Call input handling logic from the input module
        input::handle_input(self, game_state)
    }

    // General Tui state update methods
    pub(crate) fn set_status(&mut self, message: String) {
        self.status_message = message;
        self.status_timer = Some(Instant::now());
    }

    pub(crate) fn set_should_quit(&mut self, quit: bool) {
        self.should_quit = quit;
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
