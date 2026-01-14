use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
};

use crate::board::MoveRecord;

/// A panel widget that displays move history in PGN scoresheet format
pub struct MoveHistoryPanel<'a> {
    moves: &'a [MoveRecord],
    use_unicode: bool,
}

impl<'a> MoveHistoryPanel<'a> {
    pub fn new(moves: &'a [MoveRecord], use_unicode: bool) -> Self {
        Self { moves, use_unicode }
    }
}

impl Widget for MoveHistoryPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create the border block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(100, 100, 100)))
            .title(" Moves ")
            .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));

        // Get inner area for content
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 10 || inner.height < 1 {
            return; // Too small to render
        }

        // Calculate how many move pairs we can show
        let visible_lines = inner.height as usize;

        // Group moves into pairs (white, black)
        let move_pairs: Vec<(Option<&MoveRecord>, Option<&MoveRecord>)> = self
            .moves
            .chunks(2)
            .map(|chunk| {
                let white = chunk.first();
                let black = chunk.get(1);
                (white, black)
            })
            .collect();

        // Determine which moves to show (auto-scroll to bottom)
        let total_pairs = move_pairs.len();
        let start_idx = total_pairs.saturating_sub(visible_lines);

        // Render each move pair
        for (i, (white_move, black_move)) in move_pairs.iter().skip(start_idx).enumerate() {
            if i >= visible_lines {
                break;
            }

            let y = inner.y + i as u16;
            let move_num = start_idx + i + 1;

            // Format: "1. e4  e5" or "1. e4" if black hasn't moved
            let white_notation = white_move
                .map(|m| m.to_algebraic(self.use_unicode))
                .unwrap_or_default();
            let black_notation = black_move
                .map(|m| m.to_algebraic(self.use_unicode))
                .unwrap_or_default();

            // Move number (dim)
            let num_str = format!("{:2}.", move_num);
            let num_style = Style::default().fg(Color::Rgb(120, 120, 120));
            for (j, ch) in num_str.chars().enumerate() {
                if inner.x + (j as u16) < inner.x + inner.width {
                    buf.get_mut(inner.x + (j as u16), y)
                        .set_char(ch)
                        .set_style(num_style);
                }
            }

            // White's move
            let white_x = inner.x + 4;
            let white_style = Style::default().fg(Color::White);
            for (j, ch) in white_notation.chars().enumerate() {
                if white_x + (j as u16) < inner.x + inner.width {
                    buf.get_mut(white_x + (j as u16), y)
                        .set_char(ch)
                        .set_style(white_style);
                }
            }

            // Black's move (if present)
            if !black_notation.is_empty() {
                let black_x = inner.x + 10; // After white's move column
                let black_style = Style::default().fg(Color::Rgb(180, 180, 180));
                for (j, ch) in black_notation.chars().enumerate() {
                    if black_x + (j as u16) < inner.x + inner.width {
                        buf.get_mut(black_x + (j as u16), y)
                            .set_char(ch)
                            .set_style(black_style);
                    }
                }
            }
        }

        // If no moves yet, show placeholder
        if self.moves.is_empty() && inner.height > 0 {
            let placeholder = "No moves";
            let style = Style::default().fg(Color::Rgb(80, 80, 80));
            let x = inner.x + (inner.width.saturating_sub(placeholder.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            for (i, ch) in placeholder.chars().enumerate() {
                if x + (i as u16) < inner.x + inner.width {
                    buf.get_mut(x + (i as u16), y)
                        .set_char(ch)
                        .set_style(style);
                }
            }
        }
    }
}
