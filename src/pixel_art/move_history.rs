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
    scroll_offset: usize,           // First visible move pair index
    selected_move: Option<usize>,   // Which move is highlighted (0-based index into moves)
    is_focused: bool,               // Show focus indicator
    viewing_history: bool,          // Whether we're viewing a historical state
}

impl<'a> MoveHistoryPanel<'a> {
    pub fn new(moves: &'a [MoveRecord], use_unicode: bool) -> Self {
        Self {
            moves,
            use_unicode,
            scroll_offset: 0,
            selected_move: None,
            is_focused: false,
            viewing_history: false,
        }
    }

    pub fn scroll_offset(mut self, offset: usize) -> Self {
        self.scroll_offset = offset;
        self
    }

    pub fn selected_move(mut self, index: Option<usize>) -> Self {
        self.selected_move = index;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    pub fn viewing_history(mut self, viewing: bool) -> Self {
        self.viewing_history = viewing;
        self
    }
}

impl Widget for MoveHistoryPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Determine title and border color based on state
        let (title, border_color) = if self.viewing_history {
            (" Moves [VIEW] ", Color::Yellow)
        } else if self.is_focused {
            (" Moves ", Color::Cyan)
        } else {
            (" Moves ", Color::Rgb(100, 100, 100))
        };

        // Create the border block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title)
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

        let total_pairs = move_pairs.len();

        // Determine which moves to show based on scroll_offset
        // If not focused/scrolling manually, auto-scroll to bottom
        let start_idx = if self.is_focused || self.viewing_history {
            self.scroll_offset.min(total_pairs.saturating_sub(visible_lines))
        } else {
            total_pairs.saturating_sub(visible_lines)
        };

        // Calculate which pair contains the selected move (for highlighting)
        let selected_pair_idx = self.selected_move.map(|m| m / 2);
        let selected_is_white = self.selected_move.map(|m| m % 2 == 0);

        // Render each move pair
        for (i, (white_move, black_move)) in move_pairs.iter().skip(start_idx).enumerate() {
            if i >= visible_lines {
                break;
            }

            let y = inner.y + i as u16;
            let pair_idx = start_idx + i;
            let move_num = pair_idx + 1;

            // Check if this row contains the selected move
            let row_selected = selected_pair_idx == Some(pair_idx);
            let white_selected = row_selected && selected_is_white == Some(true);
            let black_selected = row_selected && selected_is_white == Some(false);

            // Row background for selected row
            if row_selected && self.is_focused {
                let bg_style = Style::default().bg(Color::Rgb(40, 40, 60));
                for x in inner.x..(inner.x + inner.width) {
                    buf.get_mut(x, y).set_style(bg_style);
                }
            }

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
            let white_style = if white_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
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
                let black_style = if black_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(180, 180, 180))
                };
                for (j, ch) in black_notation.chars().enumerate() {
                    if black_x + (j as u16) < inner.x + inner.width {
                        buf.get_mut(black_x + (j as u16), y)
                            .set_char(ch)
                            .set_style(black_style);
                    }
                }
            }
        }

        // Show scroll indicators if there's more content
        if total_pairs > visible_lines {
            let indicator_style = Style::default().fg(Color::Rgb(100, 100, 100));

            // Up arrow if we can scroll up
            if start_idx > 0 {
                let x = inner.x + inner.width.saturating_sub(2);
                if x < inner.x + inner.width && inner.y > area.y {
                    buf.get_mut(x, inner.y).set_char('▲').set_style(indicator_style);
                }
            }

            // Down arrow if we can scroll down
            if start_idx + visible_lines < total_pairs {
                let x = inner.x + inner.width.saturating_sub(2);
                let y = inner.y + inner.height.saturating_sub(1);
                if x < inner.x + inner.width && y < inner.y + inner.height {
                    buf.get_mut(x, y).set_char('▼').set_style(indicator_style);
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
