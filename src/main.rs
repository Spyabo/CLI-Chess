mod board;
mod moves;
mod pieces;
mod tui;

use anyhow::Result;
use clap::Parser;

use crate::{
    board::GameState,
    tui::Tui,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// FEN string to load the board from
    #[arg(short, long)]
    fen: Option<String>,
}

fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Initialize the game state first
    let mut game_state = match args.fen {
        Some(fen) => GameState::from_fen(&fen).map_err(|e| anyhow::anyhow!("Failed to parse FEN: {}", e))?,
        None => GameState::new(),
    };
    
    // Initialize the terminal UI
    let mut tui = Tui::new()?;
    
    // Run the TUI main loop
    tui.run(&mut game_state)?;
    
    Ok(())
}
