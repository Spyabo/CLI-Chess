use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, BorderType, Clear, Widget},
};

/// A modal dialog that displays game-over information
pub struct GameOverModal {
    title: String,
    message: String,
    border_colour: Color,
}

impl GameOverModal {
    /// Create a checkmate modal
    pub fn checkmate(winner: &str) -> Self {
        Self {
            title: "CHECKMATE!".to_string(),
            message: format!("{} wins!", winner),
            border_colour: Color::Rgb(255, 80, 80), // Bright red
        }
    }

    /// Create a stalemate modal
    pub fn stalemate() -> Self {
        Self {
            title: "STALEMATE!".to_string(),
            message: "Game drawn.".to_string(),
            border_colour: Color::Rgb(255, 200, 80), // Yellow
        }
    }
}

impl Widget for GameOverModal {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area first (creates contrast with board behind)
        Clear.render(area, buf);

        // Fill background with dark colour
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                buf.get_mut(x, y)
                    .set_bg(Color::Rgb(30, 30, 30));
            }
        }

        // Create the block with double borders
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(self.border_colour))
            .style(Style::default().bg(Color::Rgb(30, 30, 30)));

        // Get inner area for content
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 5 || inner.width < 10 {
            return; // Too small to render content
        }

        // Calculate vertical positions for centred content
        let content_height = 5; // title + blank + message + blank + instructions
        let start_y = inner.y + (inner.height.saturating_sub(content_height)) / 2;

        // Render title (large, bold)
        let title_style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);

        let title_x = inner.x + (inner.width.saturating_sub(self.title.len() as u16)) / 2;
        if start_y < inner.y + inner.height {
            for (i, ch) in self.title.chars().enumerate() {
                if title_x + (i as u16) < inner.x + inner.width {
                    buf.get_mut(title_x + (i as u16), start_y)
                        .set_char(ch)
                        .set_style(title_style);
                }
            }
        }

        // Render message
        let message_style = Style::default().fg(Color::White);
        let message_y = start_y + 2;
        let message_x = inner.x + (inner.width.saturating_sub(self.message.len() as u16)) / 2;
        if message_y < inner.y + inner.height {
            for (i, ch) in self.message.chars().enumerate() {
                if message_x + (i as u16) < inner.x + inner.width {
                    buf.get_mut(message_x + (i as u16), message_y)
                        .set_char(ch)
                        .set_style(message_style);
                }
            }
        }

        // Render instructions
        let instructions = "[S] Save  [R] New Game  [Q] Quit";
        let instructions_style = Style::default().fg(Color::Rgb(150, 150, 150));
        let instructions_y = start_y + 4;
        let instructions_x = inner.x + (inner.width.saturating_sub(instructions.len() as u16)) / 2;
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

/// Calculate a centred rectangle within an area
pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
