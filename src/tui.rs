use std::collections::HashSet;
use anyhow::{Result, Context};
use std::time::Instant;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Terminal,
};

use crate::{
    board::{GameState, Position, Move},
    pieces::{Color as PieceColor, PieceType},
};

// Using anyhow::Result consistently throughout the crate
type TuiResult<T> = Result<T, anyhow::Error>;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<std::io::Stderr>>,
    mouse_enabled: bool,
    status_message: String,
    status_timer: Option<Instant>,
    cursor_position: Position,
    selected_piece: Option<Position>,
    possible_moves: Vec<Move>,
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
            selected_piece: None,
            possible_moves: Vec::new(),
            should_quit: false,
        })
    }

    pub fn run(&mut self, game_state: &mut GameState) -> TuiResult<()> {
        self.setup()?;
        
        while !self.should_quit {
            self.draw(game_state)?;
            
            // Handle input
            if let Err(e) = self.handle_input(game_state) {
                if e.to_string() == "User quit" {
                    break;
                } else {
                    return Err(e);
                }
            }
        }
        
        self.cleanup()
    }
    
    fn setup(&mut self) -> TuiResult<()> {
        enable_raw_mode().context("Failed to enable raw mode")?;
        execute!(
            std::io::stderr(),
            EnterAlternateScreen,
        ).context("Failed to enter alternate screen")?;
        self.terminal.clear().context("Failed to clear terminal")?;
        Ok(())
    }
    
    pub fn cleanup(&mut self) -> TuiResult<()> {
        disable_raw_mode().context("Failed to disable raw mode")?;
        execute!(
            std::io::stderr(),
            LeaveAlternateScreen,
        ).context("Failed to leave alternate screen")?;
        self.terminal.show_cursor().context("Failed to show cursor")?;
        Ok(())
    }
    
    pub fn draw(&mut self, game_state: &GameState) -> TuiResult<()> {
        // Clear any expired status message
        if let Some(timer) = self.status_timer {
            if timer.elapsed().as_secs() >= 5 {
                self.status_message.clear();
                self.status_timer = None;
            }
        }
        
        // Create the board widget before the draw closure
        let board = self.create_board_widget(game_state);
        let status = self.status_message.clone();
        
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Percentage(80),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // Title
            let title = Paragraph::new("CLI Chess (Q to quit, R to reset, M to toggle mouse)")
                .style(Style::default().add_modifier(Modifier::BOLD))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(title, chunks[0]);

            // Board
            f.render_widget(board, chunks[1]);

            // Status bar
            // Add current position to status
            let status_with_pos = if !status.is_empty() {
                format!("{} | Cursor: {}", status, self.cursor_position)
            } else {
                format!("Cursor: {}", self.cursor_position)
            };
            
            let status_bar = Paragraph::new(status_with_pos)
                .style(Style::default())
                .alignment(ratatui::layout::Alignment::Left);
            f.render_widget(status_bar, chunks[2]);
        })?;
        Ok(())
    }

    fn create_board_widget(&self, game_state: &GameState) -> impl ratatui::widgets::Widget {
        // Convert the board to a 2D array of cells
        let mut rows = Vec::with_capacity(9);
        
        // Add column labels (a-h)
        let mut header = vec![Cell::from(" ".to_string())];
        for c in b'a'..=b'h' {
            header.push(Cell::from((c as char).to_string()));
        }
        rows.push(Row::new(header).style(Style::default().add_modifier(Modifier::BOLD)));
        
        for row in (0..8).rev() {
            let mut cells = Vec::with_capacity(9);
            // Add row number (1-8)
            cells.push(Cell::from((row + 1).to_string())
                .style(Style::default().add_modifier(Modifier::BOLD)));
            
            for col in 0..8 {
                let pos = Position::from_xy(col, row).unwrap();
                
                let mut cell_style = if (row + col) % 2 == 0 {
                    Style::default().bg(Color::Rgb(180, 140, 100)) // Light squares
                } else {
                    Style::default().bg(Color::Rgb(100, 60, 30))  // Dark squares
                };
                
                // Highlight cursor position with a more visible style
                if pos == self.cursor_position {
                    cell_style = cell_style
                        .bg(Color::Rgb(80, 80, 200))  // Blue background for cursor
                        .add_modifier(Modifier::BOLD);
                }
                
                // Calculate background color based on position (checkerboard pattern)
                let bg_color = if (row + col) % 2 == 0 {
                    Color::Rgb(139, 69, 19) // Dark brown
                } else {
                    Color::Rgb(245, 222, 179) // Light brown (wheat)
                };
                
                // Apply the background color to the cell style
                cell_style = cell_style.bg(bg_color);
                
                let cell = if let Some(p) = game_state.board.get_piece(pos) {
                    let symbol = match p.piece_type {
                        PieceType::Pawn => if p.color == PieceColor::White { "♙" } else { "♟" },
                        PieceType::Knight => if p.color == PieceColor::White { "♘" } else { "♞" },
                        PieceType::Bishop => if p.color == PieceColor::White { "♗" } else { "♝" },
                        PieceType::Rook => if p.color == PieceColor::White { "♖" } else { "♜" },
                        PieceType::Queen => if p.color == PieceColor::White { "♕" } else { "♛" },
                        PieceType::King => if p.color == PieceColor::White { "♔" } else { "♚" },
                        PieceType::Empty => " ",
                    };
                    
                    let mut piece_style = cell_style
                        .fg(if p.color == PieceColor::White { Color::White } else { Color::Black });
                    
                    if let Some(selected_pos) = self.selected_piece {
                        if selected_pos == pos {
                            piece_style = Style::default()
                                .fg(if p.color == PieceColor::White { Color::White } else { Color::Black })
                                .bg(Color::Rgb(70, 130, 180));
                        } else if self.possible_moves.iter().any(|m| m.to == pos) {
                            piece_style = Style::default()
                                .fg(if p.color == PieceColor::White { Color::White } else { Color::Black })
                                .bg(Color::Rgb(0, 100, 0));
                        }
                    }
                    
                    Cell::from(symbol).style(piece_style)
                } else {
                    let mut empty_cell_style = cell_style;
                    if let Some(_) = self.selected_piece {
                        if self.possible_moves.iter().any(|m| m.to == pos) {
                            empty_cell_style = Style::default().bg(Color::Rgb(0, 100, 0));
                        }
                    }
                    Cell::from("  ").style(empty_cell_style)
                };
                cells.push(cell);
            }
            rows.push(Row::new(cells).height(1));
        }
        
        // Add file labels (a-h at the bottom)
        let file_labels = Row::new(
            [" ", "a", "b", "c", "d", "e", "f", "g", "h"]
                .iter()
                .map(|&s| Cell::from(s).style(Style::default().add_modifier(Modifier::BOLD)))
        );
        rows.push(file_labels);

        Table::new(rows)
            .block(Block::default().borders(Borders::ALL).title("Chess"))
            .widths(&[Constraint::Length(2); 9])
            .column_spacing(0)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    }

    pub fn handle_input(&mut self, game_state: &mut GameState) -> TuiResult<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => self.handle_key_event(key, game_state)?,
                Event::Mouse(mouse) if self.mouse_enabled => self.handle_mouse_event(mouse, game_state)?,
                _ => {}
            }
        }
        
        // Clear status message after 3 seconds
        if let Some(timer) = self.status_timer {
            if timer.elapsed() > std::time::Duration::from_secs(3) {
                self.status_message.clear();
                self.status_timer = None;
            }
        }
        
        Ok(())
    }
    
    fn handle_key_event(
        &mut self,
        key: KeyEvent,
        game_state: &mut GameState,
    ) -> TuiResult<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.selected_piece.is_some() {
                    // Deselect piece if one is selected
                    self.selected_piece = None;
                    self.possible_moves.clear();
                    self.set_status("Deselected piece".to_string());
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Char('r') => {
                *game_state = GameState::new();
                self.selected_piece = None;
                self.possible_moves.clear();
                self.set_status("Game reset".to_string());
            }
            KeyCode::Char('m') => {
                self.mouse_enabled = !self.mouse_enabled;
                self.set_status(format!(
                    "Mouse {}", 
                    if self.mouse_enabled { "enabled" } else { "disabled" }
                ));
            }
            KeyCode::Up => {
                if self.cursor_position.y < 7 {
                    self.cursor_position.y += 1;
                }
            }
            KeyCode::Down => {
                if self.cursor_position.y > 0 {
                    self.cursor_position.y -= 1;
                }
            }
            KeyCode::Left => {
                if self.cursor_position.x > 0 {
                    self.cursor_position.x -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position.x < 7 {
                    self.cursor_position.x += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(_selected_pos) = self.selected_piece {
                    // Check if the cursor is on a valid move
                    if let Some(mv) = self.possible_moves.iter()
                        .find(|m| m.to == self.cursor_position) {
                        // Make the move
                        if game_state.board.move_piece(mv.from, mv.to).is_ok() {
                            self.set_status(format!("Moved {}", mv));
                            // Switch player
                            game_state.active_color = match game_state.active_color {
                                PieceColor::White => PieceColor::Black,
                                PieceColor::Black => PieceColor::White,
                            };
                        }
                    }
                    // Deselect the piece after attempting to move
                    self.selected_piece = None;
                    self.possible_moves.clear();
                } else {
                    // Select the piece at cursor
                    if let Some(piece) = game_state.board.get_piece(self.cursor_position) {
                        if piece.color == game_state.active_color {
                            self.selected_piece = Some(self.cursor_position);
                            // Get all legal moves for this piece
                            self.possible_moves = game_state.board.get_legal_moves(self.cursor_position)
                                .into_iter()
                                .map(|to| Move {
                                    from: self.cursor_position,
                                    to,
                                    promotion: None, // Handle promotions later
                                })
                                .collect();
                            self.set_status(format!("Selected {} at {}", piece, self.cursor_position));
                        }
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }    
        
    fn set_status(&mut self, message: String) {
        self.status_message = message;
        self.status_timer = Some(Instant::now());
    }
    
    fn handle_mouse_event(
        &mut self, 
        mouse: MouseEvent, 
        game_state: &mut GameState,
    ) -> TuiResult<()> {
        // Don't process mouse events if the game is over
        if game_state.checkmate || game_state.stalemate {
            return Ok(());
        }

        if !self.mouse_enabled || mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return Ok(());
        }

        // Get terminal size to calculate board position
        let term_size = self.terminal.size()?;
        let board_start_x = (term_size.width.saturating_sub(16)) / 2; // Center the board (16 chars wide)
        let board_start_y = (term_size.height.saturating_sub(8)) / 2; // Center vertically (8 rows)
        
        // Calculate board coordinates (0-7)
        let board_x = (mouse.column.saturating_sub(board_start_x) / 2) as i8; // Each square is 2 chars wide
        let board_y = (7 - (mouse.row.saturating_sub(board_start_y))) as i8; // Invert Y axis (0 is bottom)
        
        // Check if click is within board bounds
        if board_x < 0 || board_x > 7 || board_y < 0 || board_y > 7 {
            return Ok(());
        }
        
        let pos = match Position::from_xy(board_x, board_y) {
            Some(p) => p,
            None => return Ok(()),
        };
        
        // Handle piece selection/move
        if let Some(selected_pos) = game_state.selected_square {
            // Try to make a move
            match game_state.make_move(selected_pos, pos) {
                Ok(_) => {
                    // Update status based on game state
                    if game_state.checkmate {
                        let winner = if game_state.active_color == PieceColor::White { "Black" } else { "White" };
                        self.set_status(format!("Checkmate! {} wins!", winner));
                    } else if game_state.stalemate {
                        self.set_status("Stalemate! It's a draw!".to_string());
                    } else if game_state.check {
                        self.set_status(format!("{} is in check!", 
                            if game_state.active_color == PieceColor::White { "White" } else { "Black" }));
                    } else {
                        self.set_status(format!("{}'s turn", 
                            if game_state.active_color == PieceColor::White { "White" } else { "Black" }));
                    }
                }
                Err(e) => {
                    self.set_status(format!("Invalid move: {}", e));
                    game_state.selected_square = None;
                    game_state.valid_moves.clear();
                }
            }
        } else if let Some(piece) = game_state.board.get_piece(pos) {
            // Select a piece if it's the player's turn and the piece is theirs
            if piece.color == game_state.active_color {
                game_state.selected_square = Some(pos);
                // Get legal moves for the selected piece
                game_state.valid_moves = game_state.board.get_legal_moves(pos);
                
                // Filter out moves that would leave the king in check
                let mut legal_moves = HashSet::new();
                for &target_pos in &game_state.valid_moves {
                    let mut test_board = game_state.board.clone();
                    if test_board.move_piece(pos, target_pos).is_ok() {
                        // Check if the move leaves the king in check
                        if let Some(king_pos) = test_board.get_king_position(piece.color) {
                            if !test_board.is_square_under_attack(king_pos, !piece.color) {
                                legal_moves.insert(target_pos);
                            }
                        }
                    }
                }
                game_state.valid_moves = legal_moves;
                
                if game_state.valid_moves.is_empty() {
                    self.set_status("No legal moves for selected piece".to_string());
                    game_state.selected_square = None;
                }
            } else {
                self.set_status("It's not your turn to move that piece".to_string());
            }
        } else {
            // Clicked on empty square with no piece selected
            game_state.selected_square = None;
            game_state.valid_moves.clear();
        }
        
        self.status_timer = Some(Instant::now());
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
