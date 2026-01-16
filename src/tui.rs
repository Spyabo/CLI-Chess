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
    pgn,
    pieces::{Color as PieceColor, PieceType},
    pixel_art::{calculate_board_layout, calculate_material, centered_rect, CapturedPiecesBar, GameOverModal, LoadGameModal, MoveHistoryPanel, PixelArtBoard, PieceSprites, PromotionModal, SaveGameModal},
};

type TuiResult<T> = Result<T, anyhow::Error>;

/// Result of clicking on the move panel
enum PanelClickResult {
    SpecificMove(usize),  // Clicked on a specific move (index into move_history)
    PanelArea,            // Clicked on panel but not on a specific move
}

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
    // History panel navigation state
    history_focused: bool,              // true = arrow keys control history, not board
    history_scroll_offset: usize,       // First visible move pair index
    selected_move_index: Option<usize>, // Which move is highlighted (0-based into move_history)
    viewing_history: bool,              // true = showing historical board state
    // Board flip state
    board_flipped: bool,                // true = black at bottom, false = white at bottom
    auto_flip_enabled: bool,            // true = flip automatically based on active color
    // Promotion modal state
    pending_promotion: Option<(Position, Position)>, // (from, to) of pending promotion move
    promotion_selection: usize,         // Currently selected piece in modal (0-3)
    // Load game modal state
    load_modal: Option<LoadGameModal>,
    // Save game modal state
    save_modal: Option<SaveGameModal>,
    // Player names (from save or load)
    white_player: String,
    black_player: String,
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
            show_history_panel: true,    // Show by default
            use_unicode_notation: false, // Use letters by default
            history_focused: false,
            history_scroll_offset: 0,
            selected_move_index: None,
            viewing_history: false,
            board_flipped: false,
            auto_flip_enabled: false,
            pending_promotion: None,
            promotion_selection: 0,
            load_modal: None,
            save_modal: None,
            white_player: "White".to_string(),
            black_player: "Black".to_string(),
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

        // History navigation state
        let history_focused = self.history_focused;
        let history_scroll_offset = self.history_scroll_offset;
        let selected_move_index = self.selected_move_index;
        let viewing_history = self.viewing_history;

        // Board flip state
        let board_flipped = self.board_flipped;
        let auto_flip_enabled = self.auto_flip_enabled;

        // Promotion modal state
        let pending_promotion = self.pending_promotion;
        let promotion_selection = self.promotion_selection;

        // Load modal state (clone for rendering since Widget consumes self)
        let load_modal = self.load_modal.clone();

        // Save modal state (clone for rendering since Widget consumes self)
        let save_modal = self.save_modal.clone();

        // Player names for captured pieces bars
        let white_player = self.white_player.clone();
        let black_player = self.black_player.clone();

        // Get captured pieces for the capture bars
        let captured_by_white = game_state.captured_by_white.clone();
        let captured_by_black = game_state.captured_by_black.clone();
        let move_history = game_state.move_history.clone();

        // Get the board to display (current or historical)
        let display_board = if viewing_history {
            if let Some(move_idx) = selected_move_index {
                // board_history[0] = initial, board_history[N] = after move N-1
                // So to see state AFTER move_idx, use board_history[move_idx + 1]
                let board_index = (move_idx + 1).min(game_state.board_history.len().saturating_sub(1));
                game_state.board_history.get(board_index).cloned()
            } else {
                None
            }
        } else {
            None
        };

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

            // Create title with hints and flip indicator
            let flip_indicator = if auto_flip_enabled {
                "[Auto-flip]"
            } else if board_flipped {
                "[Flipped]"
            } else {
                ""
            };

            let title_text = if !layout.use_sprites && !layout.too_small {
                format!(
                    "CLI Chess (Q quit, R reset, H history, S save, L load, f/F flip) {} | Fullscreen recommended",
                    flip_indicator
                )
            } else {
                format!(
                    "CLI Chess (Q quit, R reset, H history, S save, L load, f/F flip) {}",
                    flip_indicator
                )
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
                &black_player,
                black_material.saturating_sub(white_material).max(0),
            );

            // White's captures are shown at the bottom (black pieces white has taken)
            let white_captures_bar = CapturedPiecesBar::new(
                &captured_by_white,
                &white_player,
                white_material.saturating_sub(black_material).max(0),
            );

            // Create board widget - use historical board if viewing history
            let temp_game_state;
            let board_game_state = if let Some(ref hist_board) = display_board {
                // Create a temporary game state with the historical board
                temp_game_state = GameState {
                    board: hist_board.clone(),
                    check: false,  // Don't show check indicator when viewing history
                    checkmate: false,
                    stalemate: false,
                    ..game_state.clone()
                };
                &temp_game_state
            } else {
                game_state
            };

            // When viewing history, don't show selection or possible moves
            let (shown_selected, shown_moves): (Option<Position>, Vec<Move>) = if viewing_history {
                (None, Vec::new())
            } else {
                (selected_piece, possible_moves.clone())
            };

            // Calculate last move to highlight
            let last_move = if viewing_history {
                // When viewing history, highlight the selected historical move
                selected_move_index.and_then(|idx| {
                    move_history.get(idx).map(|m| (m.from, m.to))
                })
            } else {
                // When viewing current position, highlight the most recent move
                move_history.last().map(|m| (m.from, m.to))
            };

            // Calculate effective flip state
            // Auto-flip: flip when it's black's turn so current player is always at bottom
            let effective_flip = if auto_flip_enabled {
                game_state.active_color == PieceColor::Black
            } else {
                board_flipped
            };

            let board = PixelArtBoard::new(
                board_game_state,
                cursor_position,
                shown_selected,
                &shown_moves,
                sprites,
                if viewing_history { None } else { capture_animation },
                last_move,
                effective_flip,
            );

            let status_bar = Paragraph::new(status_text.clone())
                .style(Style::default())
                .alignment(ratatui::layout::Alignment::Left);

            // Render main content widgets
            f.render_widget(title, chunks[0]);
            // Swap capture bars when board is flipped so bottom player's captures stay at bottom
            if effective_flip {
                f.render_widget(white_captures_bar, chunks[1]);
                f.render_widget(black_captures_bar, chunks[3]);
            } else {
                f.render_widget(black_captures_bar, chunks[1]);
                f.render_widget(white_captures_bar, chunks[3]);
            }
            f.render_widget(board, chunks[2]);
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

                    let history_panel = MoveHistoryPanel::new(&move_history, use_unicode)
                        .scroll_offset(history_scroll_offset)
                        .selected_move(selected_move_index)
                        .focused(history_focused)
                        .viewing_history(viewing_history);
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

            // Render promotion modal if pending
            if pending_promotion.is_some() {
                let mut modal = PromotionModal::new(use_unicode);
                // Set the selection to match our state
                for _ in 0..promotion_selection {
                    modal.next();
                }
                let popup_area = centered_rect(40, 8, f.size());
                f.render_widget(modal, popup_area);
            }

            // Render save game modal if active
            if let Some(modal) = save_modal {
                let popup_area = centered_rect(40, 9, f.size());
                f.render_widget(modal, popup_area);
            }

            // Render load game modal if active
            if let Some(modal) = load_modal {
                let popup_area = centered_rect(50, 16, f.size());
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
        // Handle promotion modal if active
        if let Some((from, to)) = self.pending_promotion {
            match key.code {
                KeyCode::Left => {
                    self.promotion_selection = (self.promotion_selection + 3) % 4; // prev
                }
                KeyCode::Right => {
                    self.promotion_selection = (self.promotion_selection + 1) % 4; // next
                }
                KeyCode::Enter => {
                    // Execute the move with the selected promotion piece
                    let promotion_piece = self.get_promotion_piece();
                    let is_capture = game_state.board.get_piece(to).is_some();
                    let is_en_passant = game_state.board.get_piece(from)
                        .map(|p| p.piece_type == PieceType::Pawn)
                        .unwrap_or(false)
                        && game_state.board.en_passant_target() == Some(to);

                    if game_state.make_move(from, to, Some(promotion_piece)).is_ok() {
                        if is_capture || is_en_passant {
                            self.capture_animation = Some((to, Instant::now()));
                        }
                        let piece_name = match promotion_piece {
                            PieceType::Queen => "Queen",
                            PieceType::Rook => "Rook",
                            PieceType::Bishop => "Bishop",
                            PieceType::Knight => "Knight",
                            _ => "piece",
                        };
                        self.set_status(format!("Promoted to {}", piece_name));
                    }
                    self.pending_promotion = None;
                    self.promotion_selection = 0;
                    self.deselect_piece();
                }
                KeyCode::Esc => {
                    // Cancel promotion
                    self.pending_promotion = None;
                    self.promotion_selection = 0;
                    self.set_status("Promotion cancelled".to_string());
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.promotion_selection = 0; // Queen
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.promotion_selection = 1; // Rook
                }
                KeyCode::Char('b') | KeyCode::Char('B') => {
                    self.promotion_selection = 2; // Bishop
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.promotion_selection = 3; // Knight
                }
                _ => {}
            }
            return Ok(());
        }

        // Handle save game modal if active
        if let Some(ref mut modal) = self.save_modal {
            match key.code {
                KeyCode::Esc => {
                    self.save_modal = None;
                    self.set_status("Save cancelled".to_string());
                }
                KeyCode::Tab | KeyCode::Down | KeyCode::Up => {
                    modal.next_field();
                }
                KeyCode::Enter => {
                    let white_name = modal.white_name().to_string();
                    let black_name = modal.black_name().to_string();
                    let filename = pgn::generate_save_filename(&white_name, &black_name);
                    match pgn::export_pgn(game_state, &filename, &white_name, &black_name) {
                        Ok(()) => {
                            // Store player names for display
                            self.white_player = white_name;
                            self.black_player = black_name;
                            self.set_status(format!("Game saved to {}", filename));
                        }
                        Err(e) => self.set_status(format!("Save failed: {}", e)),
                    }
                    self.save_modal = None;
                }
                KeyCode::Backspace => {
                    modal.backspace();
                }
                KeyCode::Char(c) => {
                    modal.add_char(c);
                }
                _ => {}
            }
            return Ok(());
        }

        // Handle load game modal if active
        if let Some(ref mut modal) = self.load_modal {
            match key.code {
                KeyCode::Esc => {
                    self.load_modal = None;
                    self.set_status("Load cancelled".to_string());
                }
                KeyCode::Enter => {
                    if let Some(filename) = modal.selected_file() {
                        let filename = filename.to_string();
                        match pgn::import_pgn(&filename) {
                            Ok(loaded) => {
                                *game_state = loaded;
                                // Parse and store player names from the PGN
                                if let Ok((white, black)) = pgn::parse_player_names(&filename) {
                                    self.white_player = white;
                                    self.black_player = black;
                                }
                                self.deselect_piece();
                                self.exit_history_focus();
                                self.set_status(format!("Loaded {}", filename));
                            }
                            Err(e) => {
                                self.set_status(format!("Load failed: {}", e));
                            }
                        }
                    }
                    self.load_modal = None;
                }
                KeyCode::Up => {
                    modal.prev();
                }
                KeyCode::Down => {
                    modal.next();
                }
                KeyCode::Backspace => {
                    modal.backspace();
                }
                KeyCode::Char(c) => {
                    modal.add_char(c);
                }
                _ => {}
            }
            return Ok(());
        }

        // Handle history-focused mode separately
        if self.history_focused {
            match key.code {
                KeyCode::Esc | KeyCode::Char('h') => {
                    self.exit_history_focus();
                }
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                // Arrow navigation in history mode
                // Up/Down: move by 2 (same color's previous/next move)
                // Left/Right: move by 1 (previous/next move)
                KeyCode::Up => self.navigate_history(-2, game_state),
                KeyCode::Down => self.navigate_history(2, game_state),
                KeyCode::Left => self.navigate_history(-1, game_state),
                KeyCode::Right => self.navigate_history(1, game_state),
                KeyCode::Home => self.jump_to_start(game_state),
                KeyCode::End => self.jump_to_end(game_state),
                KeyCode::Char('n') => {
                    self.use_unicode_notation = !self.use_unicode_notation;
                    self.set_status(format!(
                        "Notation: {}",
                        if self.use_unicode_notation { "Unicode pieces" } else { "Letters" }
                    ));
                }
                KeyCode::Char('r') => {
                    self.exit_history_focus();
                    self.reset_game(game_state);
                }
                _ => {}
            }
            return Ok(());
        }

        // Normal (board-focused) mode
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.viewing_history {
                    // Exit viewing history first
                    self.exit_history_focus();
                } else if self.selected_piece.is_some() {
                    self.deselect_piece();
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Char('r') => {
                self.reset_game(game_state);
            }
            KeyCode::Char('h') => {
                // Toggle history focus (entering history navigation mode)
                if self.history_focused {
                    self.exit_history_focus();
                } else {
                    self.enter_history_focus(game_state);
                }
            }
            KeyCode::Char('n') => {
                self.use_unicode_notation = !self.use_unicode_notation;
                self.set_status(format!(
                    "Notation: {}",
                    if self.use_unicode_notation { "Unicode pieces" } else { "Letters" }
                ));
            }
            KeyCode::Char('f') => {
                // Manual flip toggle - also disables auto-flip
                if self.auto_flip_enabled {
                    // Get current visual state from auto-flip, then flip to opposite
                    let current_auto_flip = game_state.active_color == PieceColor::Black;
                    self.board_flipped = !current_auto_flip; // Flip to opposite of current
                    self.auto_flip_enabled = false;
                    self.set_status(format!(
                        "Auto-flip disabled. {} at bottom",
                        if self.board_flipped { "Black" } else { "White" }
                    ));
                } else {
                    self.board_flipped = !self.board_flipped;
                    self.set_status(format!(
                        "Board flipped - {} at bottom",
                        if self.board_flipped { "Black" } else { "White" }
                    ));
                }
            }
            KeyCode::Char('F') => {
                // Auto-flip toggle (Shift+F)
                self.auto_flip_enabled = !self.auto_flip_enabled;
                self.set_status(format!(
                    "Auto-flip {} (f=manual, F=auto)",
                    if self.auto_flip_enabled { "enabled" } else { "disabled" }
                ));
            }
            KeyCode::Char('s') => {
                // Open save game modal
                self.save_modal = Some(SaveGameModal::new());
            }
            KeyCode::Char('l') => {
                // Open load game modal
                let mut modal = LoadGameModal::new();
                modal.refresh();
                self.load_modal = Some(modal);
            }
            KeyCode::Up => self.move_cursor(0, 1, game_state),
            KeyCode::Down => self.move_cursor(0, -1, game_state),
            KeyCode::Left => self.move_cursor(-1, 0, game_state),
            KeyCode::Right => self.move_cursor(1, 0, game_state),
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

    /// Check if a move from `from` to `to` is a pawn promotion
    fn is_promotion_move(&self, from: Position, to: Position, game_state: &GameState) -> bool {
        if let Some(piece) = game_state.board.get_piece(from) {
            if piece.piece_type == PieceType::Pawn {
                let promo_rank = if piece.color == PieceColor::White { 7 } else { 0 };
                return to.rank() == promo_rank;
            }
        }
        false
    }

    /// Get the currently selected promotion piece based on selection index
    fn get_promotion_piece(&self) -> PieceType {
        match self.promotion_selection {
            0 => PieceType::Queen,
            1 => PieceType::Rook,
            2 => PieceType::Bishop,
            3 => PieceType::Knight,
            _ => PieceType::Queen,
        }
    }

    fn reset_game(&mut self, game_state: &mut GameState) {
        *game_state = GameState::new();
        self.selected_piece = None;
        self.possible_moves.clear();
        self.capture_animation = None;
        // Reset history navigation state
        self.history_focused = false;
        self.history_scroll_offset = 0;
        self.selected_move_index = None;
        self.viewing_history = false;
        // Reset promotion state
        self.pending_promotion = None;
        self.promotion_selection = 0;
        // Reset player names
        self.white_player = "White".to_string();
        self.black_player = "Black".to_string();
        self.set_status("Game reset".to_string());
    }

    fn move_cursor(&mut self, dx: i8, dy: i8, game_state: &GameState) {
        // Calculate effective flip state
        let effective_flip = if self.auto_flip_enabled {
            game_state.active_color == PieceColor::Black
        } else {
            self.board_flipped
        };

        // When board is flipped, invert directions so arrow keys move visually
        let (actual_dx, actual_dy) = if effective_flip {
            (-dx, -dy)
        } else {
            (dx, dy)
        };

        let new_x = (self.cursor_position.x as i8 + actual_dx).clamp(0, 7);
        let new_y = (self.cursor_position.y as i8 + actual_dy).clamp(0, 7);

        if let Some(new_pos) = Position::new(new_x, new_y) {
            self.cursor_position = new_pos;
        }
    }

    fn handle_enter_key(&mut self, game_state: &mut GameState) -> TuiResult<()> {
        // If viewing history, exit history mode first (don't make moves)
        if self.viewing_history {
            self.exit_history_focus();
            self.set_status("Returned to current position".to_string());
            return Ok(());
        }

        if let Some(_selected_pos) = self.selected_piece {
            // Try to make a move
            if let Some(mv) = self.possible_moves.iter()
                .find(|m| m.to == self.cursor_position) {
                // Check if this is a promotion move
                if self.is_promotion_move(mv.from, mv.to, game_state) {
                    // Show promotion modal instead of executing immediately
                    self.pending_promotion = Some((mv.from, mv.to));
                    self.promotion_selection = 0; // Start with Queen selected
                    self.set_status("Choose promotion piece".to_string());
                    return Ok(());
                }

                // Check if this is a capture before making the move
                let is_capture = game_state.board.get_piece(mv.to).is_some();
                // Also check for en passant capture
                let is_en_passant = game_state.board.get_piece(mv.from)
                    .map(|p| p.piece_type == PieceType::Pawn)
                    .unwrap_or(false)
                    && game_state.board.en_passant_target() == Some(mv.to);

                if game_state.make_move(mv.from, mv.to, None).is_ok() {
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
        // Don't allow selection when viewing history
        if self.viewing_history {
            return;
        }

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
        // Handle mouse scroll for move panel
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                if self.show_history_panel {
                    self.history_scroll_offset = self.history_scroll_offset.saturating_sub(1);
                }
                return Ok(());
            }
            MouseEventKind::ScrollDown => {
                if self.show_history_panel {
                    let total_pairs = (game_state.move_history.len() + 1) / 2;
                    // Allow scrolling up to total_pairs (will be clamped in widget)
                    self.history_scroll_offset = (self.history_scroll_offset + 1).min(total_pairs);
                }
                return Ok(());
            }
            MouseEventKind::Down(MouseButton::Left) => {
                // Continue to handle click below
            }
            _ => return Ok(()),
        }

        // Check if click is on the move panel
        if self.show_history_panel {
            if let Some(click_result) = self.calculate_panel_click(mouse, game_state) {
                let total_moves = game_state.move_history.len();

                // Determine which move to select
                let move_index = match click_result {
                    PanelClickResult::SpecificMove(idx) => idx,
                    PanelClickResult::PanelArea => {
                        // Clicked on panel but not a specific move - select most recent
                        if total_moves > 0 {
                            total_moves - 1
                        } else {
                            // No moves yet, just focus the panel
                            self.history_focused = true;
                            self.set_status("Move panel focused - make moves to navigate".to_string());
                            return Ok(());
                        }
                    }
                };

                // Click on move panel - select that move
                self.history_focused = true;
                self.selected_move_index = Some(move_index);
                self.viewing_history = move_index < total_moves.saturating_sub(1);
                self.ensure_move_visible(move_index, game_state);

                let move_num = (move_index / 2) + 1;
                let is_white = move_index % 2 == 0;
                self.set_status(format!(
                    "Selected move {}{} - use arrows to navigate",
                    move_num,
                    if is_white { "." } else { "..." }
                ));
                return Ok(());
            }
        }

        // Don't allow board clicks when game is over
        if game_state.checkmate || game_state.stalemate {
            return Ok(());
        }

        // If viewing history, exit on board click
        if self.viewing_history {
            self.exit_history_focus();
            return Ok(());
        }

        if let Some(pos) = self.calculate_board_position(mouse, game_state) {
            self.handle_square_click(pos, game_state)?;
        }

        self.status_timer = Some(Instant::now());
        Ok(())
    }

    fn calculate_panel_click(&self, mouse: MouseEvent, game_state: &GameState) -> Option<PanelClickResult> {
        let term_size = self.terminal.size().ok()?;

        // Calculate layout to find panel position (matching draw logic)
        let margin = 1u16;
        let title_height = 2u16;
        let captures_bar_height = 1u16;
        let status_height = 2u16;

        let total_height = term_size.height.saturating_sub(margin * 2);
        let board_area_y = margin + title_height + captures_bar_height;
        let board_area_height = total_height.saturating_sub(title_height + captures_bar_height * 2 + status_height);
        let board_area_width = term_size.width.saturating_sub(margin * 2);

        let available_width = board_area_width.saturating_sub(3) as usize;
        let available_height = board_area_height.saturating_sub(2) as usize;

        let layout = calculate_board_layout(available_width, available_height);
        if layout.too_small {
            return None;
        }

        let board_pixel_width = (layout.square_width * 8) as u16;
        let board_pixel_height = (layout.square_height * 8) as u16;

        // Calculate panel bounds
        let board_x_start = margin + 2 + ((available_width as u16).saturating_sub(board_pixel_width)) / 2;
        let board_x_end = board_x_start + board_pixel_width;
        let board_y_start = board_area_y + 1;

        let panel_width = 20u16;
        let panel_x = board_x_end + 1;

        // Check if click is within panel bounds (including border)
        if mouse.column < panel_x || mouse.column >= panel_x + panel_width {
            return None;
        }
        if mouse.row < board_y_start || mouse.row >= board_y_start + board_pixel_height {
            return None;
        }

        // Panel inner area (accounting for border)
        let panel_inner_y_start = board_y_start + 1;
        let panel_inner_y_end = board_y_start + board_pixel_height - 1;

        // If click is on the border or outside inner content area, return PanelArea
        if mouse.row < panel_inner_y_start || mouse.row >= panel_inner_y_end {
            return Some(PanelClickResult::PanelArea);
        }

        // Calculate which row was clicked
        let clicked_row = (mouse.row - panel_inner_y_start) as usize;

        // Calculate which move pair this row represents
        let pair_index = self.history_scroll_offset + clicked_row;
        let total_moves = game_state.move_history.len();
        let total_pairs = (total_moves + 1) / 2;

        // If clicked beyond the moves list, return PanelArea
        if pair_index >= total_pairs {
            return Some(PanelClickResult::PanelArea);
        }

        // Determine if click was on white's move or black's move based on x position
        // White's move: columns 4-9 (relative to panel inner x)
        // Black's move: columns 10+ (relative to panel inner x)
        let rel_x = mouse.column.saturating_sub(panel_x + 1); // +1 for border

        let move_index = if rel_x >= 10 {
            // Click on black's move column
            let black_move_index = pair_index * 2 + 1;
            if black_move_index < total_moves {
                black_move_index
            } else {
                // Black hasn't moved yet, select white's move instead
                pair_index * 2
            }
        } else {
            // Click on white's move column (or move number)
            pair_index * 2
        };

        if move_index < total_moves {
            Some(PanelClickResult::SpecificMove(move_index))
        } else {
            Some(PanelClickResult::PanelArea)
        }
    }

    fn calculate_board_position(&self, mouse: MouseEvent, game_state: &GameState) -> Option<Position> {
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

        let screen_col = rel_x / square_width;
        let screen_row = rel_y / square_height;

        if screen_col >= 8 || screen_row >= 8 {
            return None;
        }

        // Calculate effective flip state (same as in draw)
        let effective_flip = if self.auto_flip_enabled {
            game_state.active_color == PieceColor::Black
        } else {
            self.board_flipped
        };

        // Convert screen coordinates to board coordinates
        // Normal: screen_col 0 = file a, screen_row 0 = rank 8
        // Flipped: screen_col 0 = file h, screen_row 0 = rank 1
        let clicked_col = if effective_flip { 7 - screen_col } else { screen_col };
        let clicked_row = if effective_flip { screen_row } else { 7 - screen_row };

        Position::new(clicked_col as i8, clicked_row as i8)
    }

    fn handle_square_click(&mut self, pos: Position, game_state: &mut GameState) -> TuiResult<()> {
        // Update cursor to clicked position for keyboard/mouse sync
        self.cursor_position = pos;

        if let Some(selected_pos) = self.selected_piece {
            // Check if clicked position is a valid move destination
            let is_valid_move = self.possible_moves.iter().any(|m| m.to == pos);

            if is_valid_move {
                // Check if this is a promotion move
                if self.is_promotion_move(selected_pos, pos, game_state) {
                    // Show promotion modal instead of executing immediately
                    self.pending_promotion = Some((selected_pos, pos));
                    self.promotion_selection = 0;
                    self.set_status("Choose promotion piece".to_string());
                    return Ok(());
                }

                // Check if this is a capture before making the move
                let is_capture = game_state.board.get_piece(pos).is_some();
                // Also check for en passant capture
                let is_en_passant = game_state.board.get_piece(selected_pos)
                    .map(|p| p.piece_type == PieceType::Pawn)
                    .unwrap_or(false)
                    && game_state.board.en_passant_target() == Some(pos);

                // Use make_move to ensure proper game state management
                if game_state.make_move(selected_pos, pos, None).is_ok() {
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

    // History navigation helpers

    fn enter_history_focus(&mut self, game_state: &GameState) {
        if !self.show_history_panel {
            self.show_history_panel = true;
        }
        self.history_focused = true;
        // Select last move by default
        let total = game_state.move_history.len();
        self.selected_move_index = if total > 0 { Some(total - 1) } else { None };
        self.viewing_history = false; // Still showing current position initially
        self.set_status("Move panel: use arrows to navigate, Esc to exit".to_string());
    }

    fn exit_history_focus(&mut self) {
        self.history_focused = false;
        self.selected_move_index = None;
        self.viewing_history = false;
        self.history_scroll_offset = 0;
        self.set_status("Exited move panel".to_string());
    }

    fn navigate_history(&mut self, delta: i32, game_state: &GameState) {
        let total_moves = game_state.move_history.len();
        if total_moves == 0 {
            return;
        }

        let current = self.selected_move_index.unwrap_or(total_moves.saturating_sub(1));
        let new_index = (current as i32 + delta).clamp(0, total_moves as i32 - 1) as usize;

        self.selected_move_index = Some(new_index);
        // We're viewing history if we're not at the latest move
        self.viewing_history = new_index < total_moves.saturating_sub(1);

        // Auto-scroll to keep selection visible
        self.ensure_move_visible(new_index, game_state);

        // Update status with move info
        let move_num = (new_index / 2) + 1;
        let is_white = new_index % 2 == 0;
        self.set_status(format!(
            "Move {}{} of {}",
            move_num,
            if is_white { "." } else { "..." },
            (total_moves + 1) / 2
        ));
    }

    fn ensure_move_visible(&mut self, move_index: usize, _game_state: &GameState) {
        // Calculate which pair this move is in
        let pair_index = move_index / 2;

        // Estimate visible lines (we'll use a reasonable default; actual is calculated in draw)
        // The history panel is typically around 20-30 lines visible
        let estimated_visible_lines = 15usize;

        // Scroll up if selection is above visible area
        if pair_index < self.history_scroll_offset {
            self.history_scroll_offset = pair_index;
        }
        // Scroll down if selection is below visible area
        else if pair_index >= self.history_scroll_offset + estimated_visible_lines {
            self.history_scroll_offset = pair_index.saturating_sub(estimated_visible_lines - 1);
        }
    }

    fn jump_to_start(&mut self, game_state: &GameState) {
        if game_state.move_history.is_empty() {
            return;
        }
        self.selected_move_index = Some(0);
        self.viewing_history = true;
        self.history_scroll_offset = 0;
        self.set_status("Jumped to first move".to_string());
    }

    fn jump_to_end(&mut self, game_state: &GameState) {
        let total = game_state.move_history.len();
        if total == 0 {
            return;
        }
        self.selected_move_index = Some(total - 1);
        self.viewing_history = false;
        self.ensure_move_visible(total - 1, game_state);
        self.set_status("Jumped to current position".to_string());
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}