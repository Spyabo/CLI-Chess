// src/tui/draw.rs

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::{
    board::{GameState, Position},
    pieces::{Color as PieceColor, PieceType},
    tui::Tui,
};

type TuiResult<T> = Result<T, anyhow::Error>;

// Main drawing function
pub(crate) fn draw_ui(tui: &mut Tui, game_state: &GameState) -> TuiResult<()> {
    let cursor_position = tui.cursor_position;
    let selected_piece = tui.selected_piece;
    let possible_moves = tui.possible_moves.clone();

    let status_text = get_status_text(game_state, tui); // Call helper within this module or from tui::mod

    tui.terminal.draw(|f| {
        let board = create_board_widget(
            game_state,
            cursor_position,
            selected_piece,
            &possible_moves,
        );
        let title = Paragraph::new("CLI Chess (Q to quit, R to reset, M to toggle mouse)")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .alignment(ratatui::layout::Alignment::Center);
        let status_bar = Paragraph::new(status_text.clone())
            .style(Style::default())
            .alignment(ratatui::layout::Alignment::Left);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Percentage(80),
                Constraint::Length(3),
            ])
            .split(f.size());

        f.render_widget(title, chunks[0]);
        f.render_widget(board, chunks[1]);
        f.render_widget(status_bar, chunks[2]);
    })?;
    Ok(())
}

// Creates the main board table widget
pub(crate) fn create_board_widget<'a>(
    game_state: &'a GameState,
    cursor_position: Position,
    selected_piece: Option<Position>,
    possible_moves: &'a [crate::board::Move], // Use full path or import Move
) -> Table<'a> {
    let mut rows = Vec::with_capacity(9);

    // Add column labels (a-h)
    let mut header = vec![Cell::from(" ")];
    for c in b'a'..=b'h' {
        header.push(Cell::from((c as char).to_string()));
    }
    rows.push(Row::new(header).style(Style::default().add_modifier(Modifier::BOLD)));

    for row in (0..8).rev() {
        let mut cells = Vec::with_capacity(9);
        // Add row number (1-8)
        cells.push(
            Cell::from((row + 1).to_string())
                .style(Style::default().add_modifier(Modifier::BOLD)),
        );

        for col in 0..8 {
            let pos = Position::from_xy(col, row).unwrap();
            let cell = create_board_cell(
                pos,
                game_state,
                cursor_position,
                selected_piece,
                possible_moves,
            );
            cells.push(cell);
        }
        rows.push(Row::new(cells).height(1));
    }

    // Add file labels (a-h at the bottom)
    let file_labels = Row::new(
        [" ", "a", "b", "c", "d", "e", "f", "g", "h"]
            .iter()
            .map(|&s| Cell::from(s).style(Style::default().add_modifier(Modifier::BOLD))),
    );
    rows.push(file_labels);

    Table::new(rows)
        .block(Block::default().borders(Borders::ALL).title("Chess"))
        .widths(&[Constraint::Length(2); 9])
        .column_spacing(0)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
}

// Creates a single cell for a board square
fn create_board_cell<'a>(
    pos: Position,
    game_state: &'a GameState,
    cursor_position: Position,
    selected_piece: Option<Position>,
    possible_moves: &'a [crate::board::Move], // Use full path or import Move
) -> Cell<'a> {
    let is_light_square = (pos.x + pos.y) % 2 == 1;
    let mut cell_style = if is_light_square {
        Style::default().bg(Color::Rgb(245, 222, 179)) // Light squares
    } else {
        Style::default().bg(Color::Rgb(139, 69, 19)) // Dark squares
    };

    // Highlight cursor position
    if pos == cursor_position {
        cell_style = Style::default()
            .bg(Color::Rgb(80, 80, 200))
            .add_modifier(Modifier::BOLD);
    }

    // Get piece symbol and apply styling
    if let Some(piece) = game_state.board.get_piece(pos) {
        let symbol = get_piece_symbol(piece.piece_type, piece.color);
        let mut piece_style = cell_style.fg(if piece.color == PieceColor::White {
            Color::White
        } else {
            Color::Black
        });

        // Highlight king in check
        if piece.piece_type == PieceType::King
            && game_state.check
            && piece.color == game_state.active_color
        {
            piece_style = Style::default()
                .bg(Color::Rgb(200, 50, 50))
                .fg(if piece.color == PieceColor::White {
                    Color::White
                } else {
                    Color::Black
                })
                .add_modifier(Modifier::BOLD);
        }

        // Highlight selected piece and possible moves
        if let Some(selected_pos) = selected_piece {
            if selected_pos == pos {
                piece_style = piece_style.bg(Color::Rgb(70, 130, 180));
            } else if possible_moves.iter().any(|m| m.to == pos) {
                piece_style = piece_style.bg(Color::Rgb(0, 100, 0));
            }
        }

        Cell::from(symbol).style(piece_style)
    } else {
        // Empty square
        let mut empty_style = cell_style;
        if selected_piece.is_some() {
            if possible_moves.iter().any(|m| m.to == pos) {
                empty_style = Style::default().bg(Color::Rgb(0, 100, 0));
            }
        }
        Cell::from("  ").style(empty_style)
    }
}

// Helper to get the piece symbol
pub(crate) fn get_piece_symbol(piece_type: PieceType, color: PieceColor) -> &'static str {
    match piece_type {
        PieceType::Pawn => if color == PieceColor::White { "♙" } else { "♟" },
        PieceType::Knight => if color == PieceColor::White { "♘" } else { "♞" },
        PieceType::Bishop => if color == PieceColor::White { "♗" } else { "♝" },
        PieceType::Rook => if color == PieceColor::White { "♖" } else { "♜" },
        PieceType::Queen => if color == PieceColor::White { "♕" } else { "♛" },
        PieceType::King => if color == PieceColor::White { "♔" } else { "♚" },
        PieceType::Empty => " ",
    }
}

// Helper to get the status text
fn get_status_text(game_state: &GameState, tui: &Tui) -> String {
    // Check if there's a game over state
    if game_state.checkmate {
        let winner = match game_state.active_color {
            PieceColor::White => "Black",
            PieceColor::Black => "White",
        };
        return format!("CHECKMATE! {} wins! (Press 'r' to reset)", winner);
    } else if game_state.stalemate {
        return "STALEMATE! Game is a draw. (Press 'r' to reset)".to_string();
    }

    // Normal status message
    if !tui.status_message.is_empty() {
        format!("{} | Cursor: {}", tui.status_message, tui.cursor_position)
    } else {
        format!("Cursor: {}", tui.cursor_position)
    }
}
