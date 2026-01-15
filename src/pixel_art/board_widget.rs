use std::time::Instant;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

use crate::{
    board::{GameState, Move, Position},
    pieces::{Color as PieceColour, PieceType},
};

use super::{
    colours::SquareColours,
    pixels_to_char,
    sprites::{PieceSprite, PieceSprites, SPRITE_HEIGHT, SPRITE_WIDTH},
};

/// Minimum square dimensions to show pixel art sprites
/// Sprites are 5 chars wide × 4 chars tall, so we need at least this much space
const MIN_SQUARE_WIDTH_FOR_SPRITES: usize = 5;  // Exact sprite width
const MIN_SQUARE_HEIGHT_FOR_SPRITES: usize = 4; // Exact sprite height

/// Ideal square dimensions for perfect sprite display
/// Width chosen so (width - sprite_width) is even for perfect centring
/// Sprite is 5 wide, so width of 9 gives 4 chars padding (2 each side)
const IDEAL_SQUARE_HEIGHT: usize = 4;
const IDEAL_SQUARE_WIDTH: usize = 9; // (9-5)/2 = 2 chars padding each side

/// Integer division with rounding to nearest (not toward zero)
/// This ensures proper centring: e.g., 1/2 = 1, -1/2 = 0, 3/2 = 2
fn div_round_nearest(numerator: i32, denominator: i32) -> i32 {
    if numerator >= 0 {
        (numerator + denominator / 2) / denominator
    } else {
        (numerator - denominator / 2) / denominator
    }
}

/// Get Unicode chess symbol for a piece (fallback mode)
fn get_piece_char(piece_type: PieceType, colour: PieceColour) -> char {
    match (piece_type, colour) {
        (PieceType::King, PieceColour::White) => '♔',
        (PieceType::Queen, PieceColour::White) => '♕',
        (PieceType::Rook, PieceColour::White) => '♖',
        (PieceType::Bishop, PieceColour::White) => '♗',
        (PieceType::Knight, PieceColour::White) => '♘',
        (PieceType::Pawn, PieceColour::White) => '♙',
        (PieceType::King, PieceColour::Black) => '♚',
        (PieceType::Queen, PieceColour::Black) => '♛',
        (PieceType::Rook, PieceColour::Black) => '♜',
        (PieceType::Bishop, PieceColour::Black) => '♝',
        (PieceType::Knight, PieceColour::Black) => '♞',
        (PieceType::Pawn, PieceColour::Black) => '♟',
        (PieceType::Empty, _) => ' ',
    }
}

/// A custom Ratatui widget that renders a chess board with pixel art pieces
pub struct PixelArtBoard<'a> {
    game_state: &'a GameState,
    cursor_position: Position,
    selected_piece: Option<Position>,
    possible_moves: &'a [Move],
    sprites: &'a PieceSprites,
    colours: SquareColours,
    capture_animation: Option<(Position, Instant)>,
    last_move: Option<(Position, Position)>, // (from, to) of the last move
    flipped: bool, // true = black at bottom, false = white at bottom (default)
}

impl<'a> PixelArtBoard<'a> {
    pub fn new(
        game_state: &'a GameState,
        cursor_position: Position,
        selected_piece: Option<Position>,
        possible_moves: &'a [Move],
        sprites: &'a PieceSprites,
        capture_animation: Option<(Position, Instant)>,
        last_move: Option<(Position, Position)>,
        flipped: bool,
    ) -> Self {
        Self {
            game_state,
            cursor_position,
            selected_piece,
            possible_moves,
            sprites,
            colours: SquareColours::default(),
            capture_animation,
            last_move,
            flipped,
        }
    }

    /// Get the sprite for a piece type
    fn get_sprite(&self, piece_type: PieceType) -> &PieceSprite {
        match piece_type {
            PieceType::Pawn => &self.sprites.pawn,
            PieceType::Knight => &self.sprites.knight,
            PieceType::Bishop => &self.sprites.bishop,
            PieceType::Rook => &self.sprites.rook,
            PieceType::Queen => &self.sprites.queen,
            PieceType::King => &self.sprites.king,
            PieceType::Empty => &self.sprites.pawn, // Won't be used
        }
    }

    /// Determine the background colour for a square based on its state
    fn get_square_colour(&self, pos: Position) -> Color {
        let is_light = (pos.x + pos.y) % 2 == 1;

        // Priority order: capture_animation > check > selected > cursor > legal_move > base

        // Check for capture animation (highest priority)
        if let Some((anim_pos, start_time)) = self.capture_animation {
            if anim_pos == pos {
                let elapsed_ms = start_time.elapsed().as_millis();
                if elapsed_ms < 250 {
                    return self.colours.capture_flash;  // Bright red flash
                } else if elapsed_ms < 500 {
                    return self.colours.capture_fade;   // Orange fade
                }
                // After 500ms, fall through to normal colour
            }
        }

        // Check if this square has king in check
        if let Some(piece) = self.game_state.board.get_piece(pos) {
            if piece.piece_type == PieceType::King
                && self.game_state.check
                && piece.color == self.game_state.active_color
            {
                return self.colours.check;
            }
        }

        // Last move highlight (from and to squares)
        if let Some((from, to)) = self.last_move {
            if pos == from || pos == to {
                return if is_light {
                    self.colours.last_move_light
                } else {
                    self.colours.last_move_dark
                };
            }
        }

        // Selected piece
        if self.selected_piece == Some(pos) {
            return self.colours.selected;
        }

        // Cursor position
        if pos == self.cursor_position {
            return self.colours.cursor;
        }

        // Legal move destination
        if self.possible_moves.iter().any(|m| m.to == pos) {
            return if is_light {
                self.colours.legal_move_light
            } else {
                self.colours.legal_move_dark
            };
        }

        // Default square colour
        if is_light {
            self.colours.light
        } else {
            self.colours.dark
        }
    }

    /// Render a single square with its piece (if any) - centred version
    fn render_square_centred(
        &self,
        buf: &mut Buffer,
        board_area: Rect,
        clip_area: Rect,
        pos: Position,
        square_width: usize,
        square_height: usize,
    ) {
        // When flipped, black is at bottom (rank 8 at bottom, rank 1 at top)
        let display_row = if self.flipped { pos.y as usize } else { 7 - pos.y as usize };
        let display_col = if self.flipped { 7 - pos.x as usize } else { pos.x as usize };

        // Calculate pixel position in buffer
        let x_offset = board_area.x + (display_col * square_width) as u16;
        let y_offset = board_area.y + (display_row * square_height) as u16;

        // Determine background colour
        let bg_colour = self.get_square_colour(pos);

        // Fill square with background
        for dy in 0..square_height {
            for dx in 0..square_width {
                let x = x_offset + dx as u16;
                let y = y_offset + dy as u16;
                if x < clip_area.right() && y < clip_area.bottom() {
                    buf.get_mut(x, y).set_char(' ').set_bg(bg_colour);
                }
            }
        }

        // Render piece if present
        let has_piece = if let Some(piece) = self.game_state.board.get_piece(pos) {
            if piece.piece_type != PieceType::Empty {
                let sprite = self.get_sprite(piece.piece_type);
                self.render_sprite_clipped(
                    buf,
                    sprite,
                    piece.color,
                    bg_colour,
                    x_offset,
                    y_offset,
                    square_width,
                    square_height,
                    clip_area,
                );
                true
            } else {
                false
            }
        } else {
            false
        };

        // Check if this is a legal move destination
        let is_legal_move = self.possible_moves.iter().any(|m| m.to == pos);

        // Render corner markers for empty legal move squares
        if !has_piece && is_legal_move {
            let marker_color = Color::Rgb(60, 60, 60); // Dark gray corners
            let tr_x = x_offset + square_width as u16 - 1;
            let bl_y = y_offset + square_height as u16 - 1;
            // Top-left
            if x_offset < clip_area.right() && y_offset < clip_area.bottom() {
                buf.get_mut(x_offset, y_offset)
                    .set_char('┌')
                    .set_fg(marker_color)
                    .set_bg(bg_colour);
            }
            // Top-right
            if tr_x < clip_area.right() && y_offset < clip_area.bottom() {
                buf.get_mut(tr_x, y_offset)
                    .set_char('┐')
                    .set_fg(marker_color)
                    .set_bg(bg_colour);
            }
            // Bottom-left
            if x_offset < clip_area.right() && bl_y < clip_area.bottom() {
                buf.get_mut(x_offset, bl_y)
                    .set_char('└')
                    .set_fg(marker_color)
                    .set_bg(bg_colour);
            }
            // Bottom-right
            if tr_x < clip_area.right() && bl_y < clip_area.bottom() {
                buf.get_mut(tr_x, bl_y)
                    .set_char('┘')
                    .set_fg(marker_color)
                    .set_bg(bg_colour);
            }
        }

        // Render border for capturable enemy pieces
        if has_piece && is_legal_move {
            let border_color = Color::Rgb(200, 60, 60); // Red border for captures
            // Draw corner markers to indicate capture
            // Top-left
            if x_offset < clip_area.right() && y_offset < clip_area.bottom() {
                buf.get_mut(x_offset, y_offset)
                    .set_char('▛')
                    .set_fg(border_color)
                    .set_bg(bg_colour);
            }
            // Top-right
            let tr_x = x_offset + square_width as u16 - 1;
            if tr_x < clip_area.right() && y_offset < clip_area.bottom() {
                buf.get_mut(tr_x, y_offset)
                    .set_char('▜')
                    .set_fg(border_color)
                    .set_bg(bg_colour);
            }
            // Bottom-left
            let bl_y = y_offset + square_height as u16 - 1;
            if x_offset < clip_area.right() && bl_y < clip_area.bottom() {
                buf.get_mut(x_offset, bl_y)
                    .set_char('▙')
                    .set_fg(border_color)
                    .set_bg(bg_colour);
            }
            // Bottom-right
            if tr_x < clip_area.right() && bl_y < clip_area.bottom() {
                buf.get_mut(tr_x, bl_y)
                    .set_char('▟')
                    .set_fg(border_color)
                    .set_bg(bg_colour);
            }
        }
    }

    /// Render a piece sprite within a square, with precise centring and clipping
    fn render_sprite_clipped(
        &self,
        buf: &mut Buffer,
        sprite: &PieceSprite,
        piece_colour: PieceColour,
        square_bg: Color,
        square_x: u16,
        square_y: u16,
        square_width: usize,
        square_height: usize,
        clip_area: Rect,
    ) {
        // Sprite dimensions in character cells
        let sprite_char_height = SPRITE_HEIGHT / 2; // 6 chars (12 pixels / 2)
        let sprite_char_width = SPRITE_WIDTH;       // 7 chars

        // Use signed integers for precise centring calculations
        let sq_w = square_width as i32;
        let sq_h = square_height as i32;
        let sp_w = sprite_char_width as i32;
        let sp_h = sprite_char_height as i32;

        // Calculate centred position using proper rounding
        // This ensures sprites are truly centred, not biased left/right
        let x_offset = div_round_nearest(sq_w - sp_w, 2);
        // Vertical: align sprite to bottom of square (pieces "stand" on the square)
        let y_offset = sq_h - sp_h;

        // For each character position in the square, determine what to render
        for sq_row in 0..square_height {
            for sq_col in 0..square_width {
                // Calculate screen position
                let screen_x = square_x as i32 + sq_col as i32;
                let screen_y = square_y as i32 + sq_row as i32;

                // Bounds check against clip area
                if screen_x < clip_area.x as i32
                    || screen_x >= clip_area.right() as i32
                    || screen_y < clip_area.y as i32
                    || screen_y >= clip_area.bottom() as i32
                {
                    continue;
                }

                // Calculate corresponding sprite position
                let sprite_char_col = sq_col as i32 - x_offset;
                let sprite_char_row = sq_row as i32 - y_offset;

                // Check if this position maps to a valid sprite location
                if sprite_char_col >= 0
                    && sprite_char_col < sp_w
                    && sprite_char_row >= 0
                    && sprite_char_row < sp_h
                {
                    // Convert char row to pixel rows (each char = 2 vertical pixels)
                    let upper_pixel_row = (sprite_char_row * 2) as usize;
                    let lower_pixel_row = upper_pixel_row + 1;
                    let pixel_col = sprite_char_col as usize;

                    if upper_pixel_row < SPRITE_HEIGHT && lower_pixel_row < SPRITE_HEIGHT {
                        let upper_pixel = sprite[upper_pixel_row][pixel_col];
                        let lower_pixel = sprite[lower_pixel_row][pixel_col];

                        let (ch, fg, bg) =
                            pixels_to_char(upper_pixel, lower_pixel, piece_colour, square_bg);

                        buf.get_mut(screen_x as u16, screen_y as u16)
                            .set_char(ch)
                            .set_fg(fg)
                            .set_bg(bg);
                    }
                }
                // If outside sprite bounds, the background was already filled
            }
        }
    }

    /// Render file and rank labels around the board - centred version
    fn render_labels_centred(
        &self,
        buf: &mut Buffer,
        board_area: Rect,
        clip_area: Rect,
        square_width: usize,
        square_height: usize,
    ) {
        // Rank labels (1-8) on the left of the board
        // When flipped: rank 8 at bottom, rank 1 at top
        // When normal: rank 1 at bottom, rank 8 at top
        for display_row in 0..8 {
            let y = board_area.y + (display_row * square_height) as u16 + (square_height / 2) as u16;
            let x = board_area.x.saturating_sub(2);
            // In normal mode, top row (0) = rank 8, bottom row (7) = rank 1
            // In flipped mode, top row (0) = rank 1, bottom row (7) = rank 8
            let rank_num = if self.flipped { display_row + 1 } else { 8 - display_row };
            let label = (rank_num as u8 + b'0') as char;

            if y < clip_area.bottom() && x >= clip_area.x && x < clip_area.right() {
                buf.get_mut(x, y).set_char(label);
            }
        }

        // File labels (a-h) at the bottom of the board
        // When flipped: h on left, a on right
        // When normal: a on left, h on right
        let bottom_y = board_area.y + board_area.height;
        for display_col in 0..8 {
            let x = board_area.x + (display_col * square_width) as u16 + (square_width / 2) as u16;
            // In normal mode, left col (0) = 'a', right col (7) = 'h'
            // In flipped mode, left col (0) = 'h', right col (7) = 'a'
            let file_idx = if self.flipped { 7 - display_col } else { display_col };
            let label = (b'a' + file_idx as u8) as char;

            if x < clip_area.right() && bottom_y < clip_area.bottom() {
                buf.get_mut(x, bottom_y).set_char(label);
            }
        }
    }

    /// Render a single square with character piece (fallback for small terminals)
    fn render_square_char_mode(
        &self,
        buf: &mut Buffer,
        board_area: Rect,
        clip_area: Rect,
        pos: Position,
        square_width: usize,
        square_height: usize,
    ) {
        // When flipped, black is at bottom (rank 8 at bottom, rank 1 at top)
        let display_row = if self.flipped { pos.y as usize } else { 7 - pos.y as usize };
        let display_col = if self.flipped { 7 - pos.x as usize } else { pos.x as usize };

        let x_offset = board_area.x + (display_col * square_width) as u16;
        let y_offset = board_area.y + (display_row * square_height) as u16;

        let bg_colour = self.get_square_colour(pos);

        // Fill square with background
        for dy in 0..square_height {
            for dx in 0..square_width {
                let x = x_offset + dx as u16;
                let y = y_offset + dy as u16;
                if x < clip_area.right() && y < clip_area.bottom() {
                    buf.get_mut(x, y).set_char(' ').set_bg(bg_colour);
                }
            }
        }

        // Render piece character if present
        if let Some(piece) = self.game_state.board.get_piece(pos) {
            if piece.piece_type != PieceType::Empty {
                let piece_char = get_piece_char(piece.piece_type, piece.color);

                // Centre the character in the square
                let char_x = x_offset + (square_width / 2) as u16;
                let char_y = y_offset + (square_height / 2) as u16;

                if char_x < clip_area.right() && char_y < clip_area.bottom() {
                    // Use contrasting foreground colour
                    let fg_colour = match piece.color {
                        PieceColour::White => Color::White,
                        PieceColour::Black => Color::Rgb(30, 30, 30),
                    };

                    buf.get_mut(char_x, char_y)
                        .set_char(piece_char)
                        .set_fg(fg_colour)
                        .set_style(Style::default().add_modifier(Modifier::BOLD));
                }
            }
        }
    }
}

/// Calculate square dimensions and rendering mode based on available space
pub fn calculate_board_layout(available_width: usize, available_height: usize) -> BoardLayout {
    // Priority: piece centering > visual square ratio
    // Sprites are 5 wide × 4 tall, so we want widths where (width - 5) is even

    // Check if we can fit ideal dimensions
    let ideal_total_width = IDEAL_SQUARE_WIDTH * 8;   // 56
    let ideal_total_height = IDEAL_SQUARE_HEIGHT * 8; // 32

    let (square_width, square_height) = if available_width >= ideal_total_width
        && available_height >= ideal_total_height
    {
        // Use ideal dimensions for perfect centering
        (IDEAL_SQUARE_WIDTH, IDEAL_SQUARE_HEIGHT)
    } else {
        // Scale down while keeping piece centering in mind
        // Calculate max height we can use
        let max_square_height = available_height / 8;
        // Calculate max width we can use
        let max_square_width = available_width / 8;

        // For piece centering, prefer odd widths (5, 7, 9...) since sprite is 5 wide
        // This gives even padding: (5-5)=0, (7-5)=2, (9-5)=4
        let mut width = max_square_width;
        if width > IDEAL_SQUARE_WIDTH {
            width = IDEAL_SQUARE_WIDTH;
        }
        // If width is even and > minimum, reduce by 1 for better centering
        if width > MIN_SQUARE_WIDTH_FOR_SPRITES && width % 2 == 0 {
            width -= 1;
        }

        // Height is simpler - just use what fits, capped at ideal
        let height = max_square_height.min(IDEAL_SQUARE_HEIGHT);

        (width, height)
    };

    // Determine rendering mode
    let use_sprites = square_width >= MIN_SQUARE_WIDTH_FOR_SPRITES
        && square_height >= MIN_SQUARE_HEIGHT_FOR_SPRITES;

    // Absolute minimum for any rendering
    let too_small = square_width < 2 || square_height < 1;

    BoardLayout {
        square_width,
        square_height,
        use_sprites,
        too_small,
    }
}

/// Board layout configuration
pub struct BoardLayout {
    pub square_width: usize,
    pub square_height: usize,
    pub use_sprites: bool,
    pub too_small: bool,
}

impl<'a> Widget for PixelArtBoard<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Available space (leaving room for labels)
        let available_width = area.width.saturating_sub(3) as usize;
        let available_height = area.height.saturating_sub(2) as usize;

        let layout = calculate_board_layout(available_width, available_height);

        // Terminal too small to render anything
        if layout.too_small {
            return;
        }

        let square_width = layout.square_width;
        let square_height = layout.square_height;

        // Calculate board dimensions and centre it horizontally
        let board_pixel_width = (square_width * 8) as u16;
        let board_pixel_height = (square_height * 8) as u16;

        let board_x_offset =
            area.x + 2 + ((available_width as u16).saturating_sub(board_pixel_width)) / 2;
        let board_y_offset = area.y + 1;

        let board_area = Rect {
            x: board_x_offset,
            y: board_y_offset,
            width: board_pixel_width,
            height: board_pixel_height,
        };

        // Render each square using appropriate mode
        for row in 0..8 {
            for col in 0..8 {
                if let Some(pos) = Position::from_xy(col, row) {
                    if layout.use_sprites {
                        self.render_square_centred(
                            buf, board_area, area, pos, square_width, square_height,
                        );
                    } else {
                        self.render_square_char_mode(
                            buf, board_area, area, pos, square_width, square_height,
                        );
                    }
                }
            }
        }

        // Render labels
        self.render_labels_centred(buf, board_area, area, square_width, square_height);
    }
}
