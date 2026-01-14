use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, BorderType, Clear, Widget},
};

use crate::pieces::PieceType;

/// Available promotion choices
const PROMOTION_CHOICES: [PieceType; 4] = [
    PieceType::Queen,
    PieceType::Rook,
    PieceType::Bishop,
    PieceType::Knight,
];

/// A modal dialog for pawn promotion piece selection
pub struct PromotionModal {
    selected_index: usize,
    use_unicode: bool,
}

impl PromotionModal {
    pub fn new(use_unicode: bool) -> Self {
        Self {
            selected_index: 0, // Queen selected by default
            use_unicode,
        }
    }

    /// Move selection to the next option
    pub fn next(&mut self) {
        self.selected_index = (self.selected_index + 1) % PROMOTION_CHOICES.len();
    }

    /// Get display character for a piece type
    fn piece_char(&self, piece_type: PieceType) -> &'static str {
        if self.use_unicode {
            match piece_type {
                PieceType::Queen => "\u{265B}",  // ♛
                PieceType::Rook => "\u{265C}",   // ♜
                PieceType::Bishop => "\u{265D}", // ♝
                PieceType::Knight => "\u{265E}", // ♞
                _ => "?",
            }
        } else {
            match piece_type {
                PieceType::Queen => "Q",
                PieceType::Rook => "R",
                PieceType::Bishop => "B",
                PieceType::Knight => "N",
                _ => "?",
            }
        }
    }

    /// Get the label for a piece type
    fn piece_label(&self, piece_type: PieceType) -> &'static str {
        match piece_type {
            PieceType::Queen => "Queen",
            PieceType::Rook => "Rook",
            PieceType::Bishop => "Bishop",
            PieceType::Knight => "Knight",
            _ => "?",
        }
    }
}

impl Widget for PromotionModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area first
        Clear.render(area, buf);

        // Fill background
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf.get_mut(x, y).set_bg(Color::Rgb(30, 30, 40));
            }
        }

        // Create the block with borders
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Rgb(100, 180, 255)))
            .title(" Promote Pawn ")
            .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Rgb(30, 30, 40)));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 3 || inner.width < 20 {
            return; // Too small
        }

        // Render the four choices horizontally
        let choice_width = 8; // Width for each choice cell (increased for "Knight")
        let total_width = choice_width * 4;
        let start_x = inner.x + (inner.width.saturating_sub(total_width as u16)) / 2;
        let choice_y = inner.y + 1; // Start near top to leave room for instructions

        for (i, piece_type) in PROMOTION_CHOICES.iter().enumerate() {
            let is_selected = i == self.selected_index;
            let x = start_x + (i * choice_width) as u16;

            // Background highlight for selected
            if is_selected {
                for dy in 0..3 {
                    for dx in 0..choice_width as u16 {
                        if x + dx < inner.x + inner.width && choice_y + dy < inner.y + inner.height {
                            buf.get_mut(x + dx, choice_y + dy)
                                .set_bg(Color::Rgb(60, 60, 100));
                        }
                    }
                }
            }

            // Piece symbol (centered in cell)
            let symbol = self.piece_char(*piece_type);
            let symbol_x = x + (choice_width as u16 - 1) / 2;
            let symbol_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            if symbol_x < inner.x + inner.width && choice_y < inner.y + inner.height {
                buf.get_mut(symbol_x, choice_y)
                    .set_char(symbol.chars().next().unwrap_or('?'))
                    .set_style(symbol_style);
            }

            // Piece label below symbol
            let label = self.piece_label(*piece_type);
            let label_x = x + (choice_width as u16).saturating_sub(label.len() as u16) / 2;
            let label_y = choice_y + 2;
            let label_style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Rgb(150, 150, 150))
            };

            if label_y < inner.y + inner.height {
                for (j, ch) in label.chars().enumerate() {
                    if label_x + (j as u16) < inner.x + inner.width {
                        buf.get_mut(label_x + (j as u16), label_y)
                            .set_char(ch)
                            .set_style(label_style);
                    }
                }
            }
        }

        // Instructions at the bottom
        let instructions = "[</>] Select  [Enter] Confirm";
        let instructions_y = inner.y + inner.height.saturating_sub(1);
        let instructions_x = inner.x + (inner.width.saturating_sub(instructions.len() as u16)) / 2;
        let instructions_style = Style::default().fg(Color::Rgb(120, 120, 120));

        if instructions_y < inner.y + inner.height {
            for (i, ch) in instructions.chars().enumerate() {
                if instructions_x + (i as u16) < inner.x + inner.width {
                    buf.get_mut(instructions_x + (i as u16), instructions_y)
                        .set_char(ch)
                        .set_style(instructions_style);
                }
            }
        }
    }
}
