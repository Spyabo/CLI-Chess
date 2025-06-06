use anyhow::{Result, Context};
use std::time::Instant;

use crossterm::{
    event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
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
    
    pub fn draw(&mut self, game_state: &GameState) -> TuiResult<()> {
        // Clear expired status message
        if let Some(timer) = self.status_timer {
            if timer.elapsed().as_secs() >= 5 {
                self.status_message.clear();
                self.status_timer = None;
            }
        }
        
        // Extract the data we need before borrowing terminal mutably
        let cursor_position = self.cursor_position;
        let selected_piece = self.selected_piece;
        let possible_moves = self.possible_moves.clone();
        let status_text = self.get_status_text(game_state);
        
        self.terminal.draw(|f| {
            // Create widgets inside the draw closure using extracted data
            let board = Self::create_board_widget_static(
                game_state, 
                cursor_position, 
                selected_piece, 
                &possible_moves
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

            // Render all widgets
            f.render_widget(title, chunks[0]);
            f.render_widget(board, chunks[1]);
            f.render_widget(status_bar, chunks[2]);
        })?;
        Ok(())
    }

    fn handle_input(&mut self, game_state: &mut GameState) -> TuiResult<()> {
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            match crossterm::event::read()? {
                Event::Key(key) => self.handle_key_event(key, game_state)?,
                Event::Mouse(mouse) => self.handle_mouse_event(mouse, game_state)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn create_board_widget_static<'a>(
        game_state: &'a GameState,
        cursor_position: Position,
        selected_piece: Option<Position>,
        possible_moves: &'a [Move],
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
            cells.push(Cell::from((row + 1).to_string())
                .style(Style::default().add_modifier(Modifier::BOLD)));
            
            for col in 0..8 {
                let pos = Position::from_xy(col, row).unwrap();
                let cell = Self::create_board_cell_static(
                    pos, 
                    game_state, 
                    cursor_position, 
                    selected_piece, 
                    possible_moves
                );
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

    fn create_board_cell_static<'a>(
        pos: Position,
        game_state: &'a GameState,
        cursor_position: Position,
        selected_piece: Option<Position>,
        possible_moves: &'a [Move],
    ) -> Cell<'a> {
        let is_light_square = (pos.x + pos.y) % 2 == 1;
        let mut cell_style = if is_light_square {
            Style::default().bg(Color::Rgb(245, 222, 179)) // Light squares
        } else {
            Style::default().bg(Color::Rgb(139, 69, 19))   // Dark squares
        };

        // Highlight cursor position
        if pos == cursor_position {
            cell_style = Style::default()
                .bg(Color::Rgb(80, 80, 200))
                .add_modifier(Modifier::BOLD);
        }

        // Get piece symbol and apply styling
        if let Some(piece) = game_state.board.get_piece(pos) {
            let symbol = Self::get_piece_symbol(piece.piece_type, piece.color);
            let mut piece_style = cell_style
                .fg(if piece.color == PieceColor::White { Color::White } else { Color::Black });
            
            // Highlight king in check
            if piece.piece_type == PieceType::King 
                && game_state.check 
                && piece.color == game_state.active_color {
                piece_style = Style::default()
                    .bg(Color::Rgb(200, 50, 50))
                    .fg(if piece.color == PieceColor::White { Color::White } else { Color::Black })
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

    fn get_piece_symbol(piece_type: PieceType, color: PieceColor) -> &'static str {
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

    fn get_status_text(&self, game_state: &GameState) -> String {
        let mut s = format!("Cursor: {}", self.cursor_position);
        
        if !self.status_message.is_empty() {
            s.push_str(" | ");
            s.push_str(&self.status_message);
        }
        
        // Add game state information
        if game_state.checkmate {
            s.push_str(" | Checkmate! ");
            s.push_str(match game_state.active_color {
                PieceColor::White => "Black wins!",
                PieceColor::Black => "White wins!",
            });
        } else if game_state.stalemate {
            s.push_str(" | Stalemate! Game drawn.");
        } else if game_state.check {
            s.push_str(" | Check!");
        }
        
        // Add current turn
        s.push_str(" | ");
        s.push_str(match game_state.active_color {
            PieceColor::White => "White to move",
            PieceColor::Black => "Black to move",
        });
        
        s
    }
    
    fn handle_key_event(&mut self, key: KeyEvent, game_state: &mut GameState) -> TuiResult<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.selected_piece.is_some() {
                    self.deselect_piece();
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Char('r') => {
                self.reset_game(game_state);
            }
            KeyCode::Char('m') => {
                self.toggle_mouse();
            }
            KeyCode::Up => self.move_cursor(0, 1),
            KeyCode::Down => self.move_cursor(0, -1),
            KeyCode::Left => self.move_cursor(-1, 0),
            KeyCode::Right => self.move_cursor(1, 0),
            KeyCode::Enter => {
                self.handle_enter_key(game_state)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn deselect_piece(&mut self) {
        self.selected_piece = None;
        self.possible_moves.clear();
        self.set_status("Deselected piece".to_string());
    }

    fn reset_game(&mut self, game_state: &mut GameState) {
        *game_state = GameState::new();
        self.selected_piece = None;
        self.possible_moves.clear();
        self.set_status("Game reset".to_string());
    }

    fn toggle_mouse(&mut self) {
        self.mouse_enabled = !self.mouse_enabled;
        self.set_status(format!(
            "Mouse {}", 
            if self.mouse_enabled { "enabled" } else { "disabled" }
        ));
    }

    fn move_cursor(&mut self, dx: i8, dy: i8) {
        let new_x = (self.cursor_position.x as i8 + dx).clamp(0, 7);
        let new_y = (self.cursor_position.y as i8 + dy).clamp(0, 7);
        
        if let Some(new_pos) = Position::new(new_x, new_y) {
            self.cursor_position = new_pos;
        }
    }

    fn handle_enter_key(&mut self, game_state: &mut GameState) -> TuiResult<()> {
        if let Some(_selected_pos) = self.selected_piece {
            // Try to make a move
            if let Some(mv) = self.possible_moves.iter()
                .find(|m| m.to == self.cursor_position) {
                if game_state.board.move_piece(mv.from, mv.to).is_ok() {
                    self.set_status(format!("Moved {}", mv));
                    self.switch_turn(game_state);
                }
            }
            self.deselect_piece();
        } else {
            // Select piece at cursor
            self.try_select_piece_at_cursor(game_state);
        }
        Ok(())
    }

    fn try_select_piece_at_cursor(&mut self, game_state: &GameState) {
        if let Some(piece) = game_state.board.get_piece(self.cursor_position) {
            if piece.color == game_state.active_color {
                self.selected_piece = Some(self.cursor_position);
                self.possible_moves = game_state.board.get_legal_moves(self.cursor_position)
                    .into_iter()
                    .map(|to| Move {
                        from: self.cursor_position,
                        to,
                        promotion: None,
                    })
                    .collect();
                self.set_status(format!("Selected {} at {}", piece, self.cursor_position));
            }
        }
    }

    fn switch_turn(&mut self, game_state: &mut GameState) {
        game_state.active_color = match game_state.active_color {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        };
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent, game_state: &mut GameState) -> TuiResult<()> {
        if game_state.checkmate || game_state.stalemate || !self.mouse_enabled {
            return Ok(());
        }

        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return Ok(());
        }

        if let Some(pos) = self.calculate_board_position(mouse) {
            self.handle_square_click(pos, game_state)?;
        }

        self.status_timer = Some(Instant::now());
        Ok(())
    }

    fn calculate_board_position(&self, mouse: MouseEvent) -> Option<Position> {
        let term_size = self.terminal.size().ok()?;
        let board_width = 8 * 4;
        let board_height = 8 * 2;
        
        let board_x = (term_size.width.saturating_sub(board_width as u16)) / 2;
        let board_y = (term_size.height.saturating_sub(board_height as u16)) / 2;
        
        let clicked_col = (mouse.column.saturating_sub(board_x)) / 4;
        let clicked_row = 7 - (mouse.row.saturating_sub(board_y)) / 2;
        
        if clicked_col >= 8 || clicked_row >= 8 {
            return None;
        }
        
        Position::new(clicked_col as i8, clicked_row as i8)
    }

    fn handle_square_click(&mut self, pos: Position, game_state: &mut GameState) -> TuiResult<()> {
        if let Some(selected_pos) = game_state.selected_square {
            if game_state.valid_moves.contains(&pos) {
                // Create move manually since Move::new doesn't exist
                let mv = Move {
                    from: selected_pos,
                    to: pos,
                    promotion: None,
                };
                // Use make_move to ensure proper game state management
                if game_state.make_move(mv.from, mv.to).is_ok() {
                    self.switch_turn(game_state);
                }
            } else {
                self.try_select_new_piece(pos, game_state);
            }
        } else {
            self.try_select_piece(pos, game_state);
        }
        Ok(())
    }

    fn try_select_new_piece(&mut self, pos: Position, game_state: &mut GameState) {
        if let Some(piece) = game_state.board.get_piece(pos) {
            if piece.color == game_state.active_color {
                game_state.selected_square = Some(pos);
                game_state.valid_moves = game_state.board.get_legal_moves(pos);
                
                if game_state.valid_moves.is_empty() {
                    self.set_status("No legal moves for selected piece".to_string());
                    game_state.selected_square = None;
                }
            }
        } else {
            game_state.selected_square = None;
            game_state.valid_moves.clear();
        }
    }

    fn try_select_piece(&mut self, pos: Position, game_state: &mut GameState) {
        if let Some(piece) = game_state.board.get_piece(pos) {
            if piece.color == game_state.active_color {
                use crate::moves::get_valid_moves;
                
                game_state.selected_square = Some(pos);
                game_state.valid_moves = get_valid_moves(&game_state.board, pos);
                
                // Filter out moves that would put the king in check
                let current_moves = game_state.valid_moves.clone();
                game_state.valid_moves = current_moves.into_iter()
                    .filter(|&to| {
                        let mut board_clone = game_state.board.clone();
                        board_clone.move_piece(pos, to).is_ok()
                    })
                    .collect();
                
                if game_state.valid_moves.is_empty() {
                    self.set_status("No legal moves for selected piece".to_string());
                    game_state.selected_square = None;
                }
            } else {
                self.set_status("It's not your turn to move that piece".to_string());
            }
        }
    }
        
    fn set_status(&mut self, message: String) {
        self.status_message = message;
        self.status_timer = Some(Instant::now());
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}