# CLI Chess

A command-line chess game implemented in Rust, featuring a terminal-based interface with Unicode piece symbols and move highlighting.

## Features

- Terminal-based chess game with Unicode piece symbols
- Move validation for all chess pieces
- Turn-based gameplay
- Move input in algebraic notation (e.g., "e2e4")
- Visual highlighting of selected pieces and valid moves
- Clean terminal interface with ANSI colors

## Installation

Make sure you have [Rust and Cargo](https://www.rust-lang.org/tools/install) installed on your system.

```bash
# Clone the repository
git clone https://github.com/yourusername/CLI-Chess.git
cd CLI-Chess

# Build and run
cargo run --release
```

## How to Play

- Use the keyboard to enter moves in algebraic notation (e.g., "e2e4" to move from e2 to e4)
- Press 'q' or ESC to quit the game
- Press 'r' to reset the game
- The current player is indicated at the bottom of the board
- Selected pieces are highlighted in green
- Valid moves are highlighted in blue

## Controls

- `e2e4` - Move piece from e2 to e4 (algebraic notation)
- `q` or `ESC` - Quit the game
- `r` - Reset the game

## Dependencies

- [crossterm](https://crates.io/crates/crossterm) - For cross-platform terminal handling
- [strum](https://crates.io/crates/strum) - For enum utilities
- [thiserror](https://crates.io/crates/thiserror) - For error handling
- [lazy_static](https://crates.io/crates/lazy_static) - For static initialization

## Project Structure

- `src/main.rs` - Main game loop and terminal interface
- `src/board/` - Board representation and game state
  - `mod.rs` - Board implementation
  - `position.rs` - Position/coordinate handling
- `src/pieces/` - Chess piece definitions and logic
  - `mod.rs` - Piece types and their properties
- `src/moves/` - Move generation and validation
  - `mod.rs` - Move generation for all piece types

## License

MIT

## Credits

Inspired by the Python implementation from [ArjanCodes](https://github.com/ArjanCodes/2022-chessroast)
