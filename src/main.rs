mod board;
mod moves;
mod pgn;
mod pieces;
mod pixel_art;
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
    
    // Initialize the terminal
    let mut tui = Tui::new()?;
    
    // Initialize the game state
    let mut game_state = match args.fen {
        Some(fen) => GameState::from_fen(&fen).map_err(|e| anyhow::anyhow!("Failed to parse FEN: {}", e))?,
        None => GameState::new(),
    };
    
    // Run the TUI main loop
    tui.run(&mut game_state)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::pieces::PieceType;

    #[test]
    fn test_board_initialization() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let pos = crate::board::Position::new(0, 0).unwrap();
        assert_eq!(board.get_piece(pos).unwrap().piece_type, PieceType::Rook);
    }
}
