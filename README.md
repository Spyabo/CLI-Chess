# Pixel Chess

A terminal-based chess game built in Rust with pixel art pieces, mouse support, and full PGN save/load functionality.

## Quickstart

```bash
# Install from crates.io
cargo install pixel-chess
pixel-chess

# Or clone and build
git clone https://github.com/Spyabo/pixel-chess.git
cd pixel-chess
cargo run --release
```

Requires [Rust](https://www.rust-lang.org/tools/install) (1.70+).

## Features

- **Pixel art pieces** - Detailed 8x8 pixel sprites rendered using Unicode half-blocks
- **Mouse & keyboard** - Click to select/move pieces or use arrow keys + Enter
- **Move history panel** - Scrollable list with algebraic notation, click to rewind
- **Board flipping** - Manual flip (`f`) or auto-flip on turn change (`F`)
- **Pawn promotion** - Modal to choose Q/R/B/N with keyboard shortcuts
- **Save/Load PGN** - Games saved with player names and timestamps
- **Last move highlight** - Yellow squares show the previous move
- **Check/checkmate/stalemate detection** - With game over modal

## Controls

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor |
| Enter | Select/move piece |
| Mouse click | Select/move piece |
| `H` | Toggle move history panel |
| `S` | Save game (enter player names) |
| `L` | Load game (fuzzy search) |
| `f` | Flip board |
| `F` | Toggle auto-flip |
| `R` | Reset game |
| `Q` / `Esc` | Quit |

## Project Structure

```
src/
├── main.rs              # Entry point, CLI args
├── tui.rs               # Terminal UI, input handling, rendering
├── board/
│   ├── mod.rs           # Board state, move execution, game logic
│   └── position.rs      # Square coordinates (e.g., e4 -> (4,3))
├── pieces/mod.rs        # Piece types, colors, FEN parsing
├── moves/mod.rs         # Move struct with from/to positions
├── pgn.rs               # PGN export/import, player names
└── pixel_art/
    ├── board_widget.rs  # Main board renderer with sprites
    ├── sprites.rs       # 8x8 pixel art for each piece
    ├── colours.rs       # Square colors (light/dark/highlight)
    ├── captured_bar.rs  # Shows captured pieces + material
    ├── move_history.rs  # Scrollable move list widget
    ├── promotion_modal.rs
    ├── save_game_modal.rs
    ├── load_game_modal.rs
    └── game_over_modal.rs
```

## Key Implementation Details

**Pixel Art Rendering** (`pixel_art/sprites.rs`):
Each piece is an 8x8 grid of `Pixel` enums (Transparent/Primary/Outline/Accent). Two vertical pixels are combined into one terminal character using Unicode half-blocks (`▀`), giving 8x8 pixel resolution in 8x4 character cells.

**Move Validation** (`board/mod.rs`):
Legal moves are computed by generating pseudo-legal moves, then filtering out those that would leave the king in check. Special moves (castling, en passant, promotion) are handled explicitly.

**PGN Parsing** (`pgn.rs`):
Algebraic notation like `Nf3` is parsed by finding which knight can legally move to f3. Disambiguation (`Rab1`) is supported for ambiguous moves.

## Building for Release

```bash
cargo build --release
```

The optimized binary will be at `target/release/cli-chess` (~3MB). This runs significantly faster than debug builds.

## License

MIT
