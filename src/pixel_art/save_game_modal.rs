use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, BorderType, Clear, Widget},
};

/// Which input field is currently active
#[derive(Clone, Copy, PartialEq)]
pub enum SaveModalField {
    White,
    Black,
}

/// A modal dialog for saving games with player names
#[derive(Clone)]
pub struct SaveGameModal {
    white_name: String,
    black_name: String,
    active_field: SaveModalField,
    white_is_default: bool,  // True if still showing default, clears on first keystroke
    black_is_default: bool,
}

impl SaveGameModal {
    /// Create a save modal with default placeholder names
    pub fn new() -> Self {
        Self {
            white_name: "P1".to_string(),
            black_name: "P2".to_string(),
            active_field: SaveModalField::White,
            white_is_default: true,
            black_is_default: true,
        }
    }

    /// Create a save modal with player names pre-filled (e.g., from a loaded game)
    pub fn new_with_names(white: &str, black: &str) -> Self {
        Self {
            white_name: white.to_string(),
            black_name: black.to_string(),
            active_field: SaveModalField::White,
            white_is_default: false,
            black_is_default: false,
        }
    }

    /// Get the white player name
    pub fn white_name(&self) -> &str {
        &self.white_name
    }

    /// Get the black player name
    pub fn black_name(&self) -> &str {
        &self.black_name
    }

    /// Switch to the next input field
    pub fn next_field(&mut self) {
        self.active_field = match self.active_field {
            SaveModalField::White => SaveModalField::Black,
            SaveModalField::Black => SaveModalField::White,
        };
    }

    /// Add a character to the active field
    pub fn add_char(&mut self, c: char) {
        match self.active_field {
            SaveModalField::White => {
                if self.white_is_default {
                    self.white_name.clear();
                    self.white_is_default = false;
                }
                self.white_name.push(c);
            }
            SaveModalField::Black => {
                if self.black_is_default {
                    self.black_name.clear();
                    self.black_is_default = false;
                }
                self.black_name.push(c);
            }
        }
    }

    /// Remove the last character from the active field
    pub fn backspace(&mut self) {
        match self.active_field {
            SaveModalField::White => {
                if self.white_is_default {
                    self.white_name.clear();
                    self.white_is_default = false;
                } else {
                    self.white_name.pop();
                }
            }
            SaveModalField::Black => {
                if self.black_is_default {
                    self.black_name.clear();
                    self.black_is_default = false;
                } else {
                    self.black_name.pop();
                }
            }
        }
    }
}

impl Widget for SaveGameModal {
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
            .title(" Save Game ")
            .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Rgb(30, 30, 40)));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 6 || inner.width < 25 {
            return; // Too small
        }

        let label_style = Style::default().fg(Color::Rgb(150, 150, 150));
        let input_style = Style::default().fg(Color::White);
        let default_style = Style::default().fg(Color::Rgb(100, 100, 100));
        let active_style = Style::default().fg(Color::Yellow);
        let cursor_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::SLOW_BLINK);

        // White player label and input
        let white_y = inner.y + 1;
        let label = "White: ";
        for (i, ch) in label.chars().enumerate() {
            if inner.x + (i as u16) < inner.x + inner.width {
                buf.get_mut(inner.x + (i as u16), white_y)
                    .set_char(ch)
                    .set_style(label_style);
            }
        }

        let input_x = inner.x + label.len() as u16;
        let is_white_active = self.active_field == SaveModalField::White;
        let white_style = if self.white_is_default { default_style } else if is_white_active { active_style } else { input_style };

        for (i, ch) in self.white_name.chars().enumerate() {
            if input_x + (i as u16) < inner.x + inner.width {
                buf.get_mut(input_x + (i as u16), white_y)
                    .set_char(ch)
                    .set_style(white_style);
            }
        }

        // Cursor for white field
        if is_white_active {
            let cursor_x = input_x + self.white_name.len() as u16;
            if cursor_x < inner.x + inner.width {
                buf.get_mut(cursor_x, white_y)
                    .set_char('_')
                    .set_style(cursor_style);
            }
        }

        // Black player label and input
        let black_y = inner.y + 3;
        let label = "Black: ";
        for (i, ch) in label.chars().enumerate() {
            if inner.x + (i as u16) < inner.x + inner.width {
                buf.get_mut(inner.x + (i as u16), black_y)
                    .set_char(ch)
                    .set_style(label_style);
            }
        }

        let is_black_active = self.active_field == SaveModalField::Black;
        let black_style = if self.black_is_default { default_style } else if is_black_active { active_style } else { input_style };

        for (i, ch) in self.black_name.chars().enumerate() {
            if input_x + (i as u16) < inner.x + inner.width {
                buf.get_mut(input_x + (i as u16), black_y)
                    .set_char(ch)
                    .set_style(black_style);
            }
        }

        // Cursor for black field
        if is_black_active {
            let cursor_x = input_x + self.black_name.len() as u16;
            if cursor_x < inner.x + inner.width {
                buf.get_mut(cursor_x, black_y)
                    .set_char('_')
                    .set_style(cursor_style);
            }
        }

        // Instructions at the bottom
        let instructions = "[Tab] Switch  [Enter] Save  [Esc]";
        let instructions_y = inner.y + inner.height - 1;
        let instructions_x = inner.x + (inner.width.saturating_sub(instructions.len() as u16)) / 2;
        let instructions_style = Style::default().fg(Color::Rgb(120, 120, 120));

        for (i, ch) in instructions.chars().enumerate() {
            if instructions_x + (i as u16) < inner.x + inner.width {
                buf.get_mut(instructions_x + (i as u16), instructions_y)
                    .set_char(ch)
                    .set_style(instructions_style);
            }
        }
    }
}
