use anyhow::{Result, Context};
use std::time::Instant;

use crossterm::{
    event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::Paragraph,
    Terminal,
};

use crate::{
    board::{GameState, Position, Move},
    pieces::Color as PieceColor,
    pixel_art::{calculate_board_layout, calculate_material, centered_rect, CapturedPiecesBar, GameOverModal, MoveHistoryPanel, PixelArtBoard, PieceSprites},
};

type TuiResult<T> = Result<T, anyhow::Error>;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<std::io::Stderr>>,
    status_message: String,
    status_timer: Option<Instant>,
    cursor_position: Position,
    selected_piece: Option<Position>,
    possible_moves: Vec<Move>,
    should_quit: bool,
    sprites: PieceSprites,
    capture_animation: Option<(Position, Instant)>,  // Position and start time of capture flash
    show_history_panel: bool,    // Toggle with 'H'
    use_unicode_notation: bool,  // Toggle with 'N'
}

impl Tui {
    pub fn new() -> Result<Self> {
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
        terminal.hide_cursor()?;
        
        Ok(Self {
            terminal,
            status_message: String::new(),
            status_timer: None,
            cursor_position: Position::new(0, 0).expect("Invalid initial cursor position"),
            selected_piece: None,
            possible_moves: Vec::new(),
            should_quit: false,
            sprites: PieceSprites::default(),
            capture_animation: None,
            show_history_panel: true,   // Show by default
            use_unicode_notation: false, // Use letters by default
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
        execute!(std::io::stderr(), EnterAlternateScreen, EnableMouseCapture)
            .context("Failed to enter alternate screen")?;
        self.terminal.clear().context("Failed to clear terminal")?;
        Ok(())
    }
    
    pub fn cleanup(&mut self) -> TuiResult<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
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

        // Clear expired capture animation (500ms duration)
        if let Some((_, start_time)) = self.capture_animation {
            if start_time.elapsed().as_millis() >= 500 {
                self.capture_animation = None;
            }
        }

        // Extract the data we need before borrowing terminal mutably
        let cursor_position = self.cursor_position;
        let selected_piece = self.selected_piece;
        let possible_moves = self.possible_moves.clone();
        let status_text = self.get_status_text(game_state);
        let sprites = &self.sprites;
        let capture_animation = self.capture_animation;
        let show_history = self.show_history_panel;
        let use_unicode = self.use_unicode_notation;

        // Get captured pieces for the capture bars
        let captured_by_white = game_state.captured_by_white.clone();
        let captured_by_black = game_state.captured_by_black.clone();
        let move_history = game_state.move_history.clone();

        self.terminal.draw(|f| {
            // Main vertical layout (always uses full screen width - no horizontal split)
            // This keeps the board position stable for mouse calculations
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(2),      // Title
                    Constraint::Length(1),      // Black's captures (white pieces they took)
                    Constraint::Min(10),        // Board (flexible)
                    Constraint::Length(1),      // White's captures (black pieces they took)
                    Constraint::Length(2),      // Status bar
                ])
                .split(f.size());

            // Calculate board layout to determine if we're in sprite mode
            let board_area_width = chunks[2].width.saturating_sub(3) as usize;
            let board_area_height = chunks[2].height.saturating_sub(2) as usize;
            let layout = calculate_board_layout(board_area_width, board_area_height);

            // Create title with hints
            let title_text = if !layout.use_sprites && !layout.too_small {
                "CLI Chess (Q quit, R reset, H history, N notation) | Fullscreen recommended"
            } else {
                "CLI Chess (Q quit, R reset, H history, N notation)"
            };

            let title = Paragraph::new(title_text)
                .style(Style::default().add_modifier(Modifier::BOLD))
                .alignment(ratatui::layout::Alignment::Center);

            // Calculate material advantage
            let white_material = calculate_material(&captured_by_white);
            let black_material = calculate_material(&captured_by_black);

            // Create captured pieces bars
            // Black's captures are shown at the top (white pieces black has taken)
            let black_captures_bar = CapturedPiecesBar::new(
                &captured_by_black,
                "Black",
                black_material.saturating_sub(white_material).max(0),
            );

            // White's captures are shown at the bottom (black pieces white has taken)
            let white_captures_bar = CapturedPiecesBar::new(
                &captured_by_white,
                "White",
                white_material.saturating_sub(black_material).max(0),
            );

            // Create board widget
            let board = PixelArtBoard::new(
                game_state,
                cursor_position,
                selected_piece,
                &possible_moves,
                sprites,
                capture_animation,
            );

            let status_bar = Paragraph::new(status_text.clone())
                .style(Style::default())
                .alignment(ratatui::layout::Alignment::Left);

            // Render main content widgets
            f.render_widget(title, chunks[0]);
            f.render_widget(black_captures_bar, chunks[1]);
            f.render_widget(board, chunks[2]);
            f.render_widget(white_captures_bar, chunks[3]);
            f.render_widget(status_bar, chunks[4]);

            // Render move history panel to the right of the board (within the board area)
            if show_history {
                // Calculate actual board dimensions (not the chunk, but the rendered board)
                let board_pixel_width = (layout.square_width * 8) as u16;
                let board_pixel_height = (layout.square_height * 8) as u16;
                let board_area = chunks[2];

                // Board is centered horizontally, but starts at y + 1 (matching board_widget.rs)
                let board_x_start = board_area.x + 2 + ((board_area.width.saturating_sub(4)).saturating_sub(board_pixel_width)) / 2;
                let board_x_end = board_x_start + board_pixel_width;
                let board_y_start = board_area.y + 1;  // Matches board_widget.rs line 460

                // History panel: positioned right after the board, SAME HEIGHT as actual board
                let panel_width = 20u16;
                let panel_x = board_x_end + 1;  // 1 char gap from board

                // Only render if there's room
                if panel_x + panel_width <= board_area.x + board_area.width {
                    let history_area = ratatui::layout::Rect {
                        x: panel_x,
                        y: board_y_start,
                        width: panel_width.min(board_area.x + board_area.width - panel_x),
                        height: board_pixel_height,  // Match actual board height
                    };

                    let history_panel = MoveHistoryPanel::new(&move_history, use_unicode);
                    f.render_widget(history_panel, history_area);
                }
            }

            // Render game-over modal if applicable
            if game_state.checkmate || game_state.stalemate {
                let modal = if game_state.checkmate {
                    // The winner is the opposite of active_color (who is in checkmate)
                    let winner = match game_state.active_color {
                        PieceColor::White => "Black",
                        PieceColor::Black => "White",
                    };
                    GameOverModal::checkmate(winner)
                } else {
                    GameOverModal::stalemate()
                };

                // Centre the modal on screen
                let popup_area = centered_rect(36, 9, f.size());
                f.render_widget(modal, popup_area);
            }
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
            KeyCode::Char('h') => {
                self.show_history_panel = !self.show_history_panel;
                self.set_status(format!(
                    "Move history {}",
                    if self.show_history_panel { "shown" } else { "hidden" }
                ));
            }
            KeyCode::Char('n') => {
                self.use_unicode_notation = !self.use_unicode_notation;
                self.set_status(format!(
                    "Notation: {}",
                    if self.use_unicode_notation { "Unicode pieces" } else { "Letters" }
                ));
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
        self.capture_animation = None;
        self.set_status("Game reset".to_string());
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
                // Check if this is a capture before making the move
                let is_capture = game_state.board.get_piece(mv.to).is_some();
                // Also check for en passant capture
                let is_en_passant = game_state.board.get_piece(mv.from)
                    .map(|p| p.piece_type == crate::pieces::PieceType::Pawn)
                    .unwrap_or(false)
                    && game_state.board.en_passant_target() == Some(mv.to);

                if game_state.make_move(mv.from, mv.to).is_ok() {
                    // Trigger capture animation if it was a capture
                    if is_capture || is_en_passant {
                        self.capture_animation = Some((mv.to, Instant::now()));
                    }
                    self.set_status(format!("Moved {}", mv));
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

    fn handle_mouse_event(&mut self, mouse: MouseEvent, game_state: &mut GameState) -> TuiResult<()> {
        if game_state.checkmate || game_state.stalemate {
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

        // Calculate the board area (matching the draw layout)
        // Layout: margin(1), title(2), captures_bar(1), board(flexible), captures_bar(1), status(2)
        let margin = 1u16;
        let title_height = 2u16;
        let captures_bar_height = 1u16; // Black's captures bar above the board
        let status_height = 2u16;

        // Calculate board area dimensions
        let total_height = term_size.height.saturating_sub(margin * 2);
        let board_area_height = total_height.saturating_sub(title_height + captures_bar_height * 2 + status_height);
        let board_area_width = term_size.width.saturating_sub(margin * 2);

        // Calculate available space (matching board_widget.rs)
        let available_width = board_area_width.saturating_sub(3) as usize;
        let available_height = board_area_height.saturating_sub(2) as usize;

        // Use the same layout calculation as the board widget
        let layout = calculate_board_layout(available_width, available_height);

        if layout.too_small {
            return None;
        }

        let square_width = layout.square_width;
        let square_height = layout.square_height;

        // Calculate board position (centred horizontally)
        let board_pixel_width = (square_width * 8) as u16;
        let board_x_offset = margin + 2 + ((available_width as u16).saturating_sub(board_pixel_width)) / 2;
        // Y offset: margin + title + captures_bar + board internal offset (1)
        let board_y_offset = margin + title_height + captures_bar_height + 1;

        // Check if click is within board bounds
        if mouse.column < board_x_offset || mouse.row < board_y_offset {
            return None;
        }

        // Calculate which square was clicked
        let rel_x = mouse.column.saturating_sub(board_x_offset) as usize;
        let rel_y = mouse.row.saturating_sub(board_y_offset) as usize;

        let clicked_col = rel_x / square_width;
        let clicked_row = 7usize.saturating_sub(rel_y / square_height);

        if clicked_col >= 8 || clicked_row >= 8 {
            return None;
        }

        Position::new(clicked_col as i8, clicked_row as i8)
    }

    fn handle_square_click(&mut self, pos: Position, game_state: &mut GameState) -> TuiResult<()> {
        // Update cursor to clicked position for keyboard/mouse sync
        self.cursor_position = pos;

        if let Some(selected_pos) = self.selected_piece {
            // Check if clicked position is a valid move destination
            let is_valid_move = self.possible_moves.iter().any(|m| m.to == pos);

            if is_valid_move {
                // Check if this is a capture before making the move
                let is_capture = game_state.board.get_piece(pos).is_some();
                // Also check for en passant capture
                let is_en_passant = game_state.board.get_piece(selected_pos)
                    .map(|p| p.piece_type == crate::pieces::PieceType::Pawn)
                    .unwrap_or(false)
                    && game_state.board.en_passant_target() == Some(pos);

                // Use make_move to ensure proper game state management
                if game_state.make_move(selected_pos, pos).is_ok() {
                    // Trigger capture animation if it was a capture
                    if is_capture || is_en_passant {
                        self.capture_animation = Some((pos, Instant::now()));
                    }
                    self.deselect_piece();
                }
            } else {
                // Try to select a different piece
                self.try_select_piece_at_position(pos, game_state);
            }
        } else {
            self.try_select_piece_at_position(pos, game_state);
        }
        Ok(())
    }

    fn try_select_piece_at_position(&mut self, pos: Position, game_state: &GameState) {
        if let Some(piece) = game_state.board.get_piece(pos) {
            if piece.color == game_state.active_color {
                self.selected_piece = Some(pos);
                self.possible_moves = game_state.board.get_legal_moves(pos)
                    .into_iter()
                    .map(|to| Move {
                        from: pos,
                        to,
                        promotion: None,
                    })
                    .collect();

                if self.possible_moves.is_empty() {
                    self.set_status("No legal moves for selected piece".to_string());
                    self.selected_piece = None;
                } else {
                    self.set_status(format!("Selected {} at {}", piece, pos));
                }
            } else {
                self.set_status("It's not your turn to move that piece".to_string());
            }
        } else {
            self.deselect_piece();
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