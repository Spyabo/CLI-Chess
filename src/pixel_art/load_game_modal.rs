use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, BorderType, Clear, Widget},
};

use crate::pgn;

/// A modal dialog for loading saved games
#[derive(Clone)]
pub struct LoadGameModal {
    files: Vec<String>,
    filtered_files: Vec<String>,
    selected_index: usize,
    search_query: String,
    scroll_offset: usize,
}

impl LoadGameModal {
    pub fn new() -> Self {
        let files = pgn::list_pgn_files();
        let filtered_files = files.clone();
        Self {
            files,
            filtered_files,
            selected_index: 0,
            search_query: String::new(),
            scroll_offset: 0,
        }
    }

    /// Refresh the file list
    pub fn refresh(&mut self) {
        self.files = pgn::list_pgn_files();
        self.update_filtered_files();
    }

    /// Get the currently selected filename (if any)
    pub fn selected_file(&self) -> Option<&str> {
        self.filtered_files.get(self.selected_index).map(|s| s.as_str())
    }

    /// Move selection up
    pub fn prev(&mut self) {
        if !self.filtered_files.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.filtered_files.len() - 1;
            }
            self.adjust_scroll();
        }
    }

    /// Move selection down
    pub fn next(&mut self) {
        if !self.filtered_files.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_files.len();
            self.adjust_scroll();
        }
    }

    /// Add a character to the search query
    pub fn add_char(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filtered_files();
    }

    /// Remove the last character from the search query
    pub fn backspace(&mut self) {
        self.search_query.pop();
        self.update_filtered_files();
    }

    fn update_filtered_files(&mut self) {
        self.filtered_files = self.files
            .iter()
            .filter(|f| pgn::fuzzy_match(f, &self.search_query))
            .cloned()
            .collect();

        // Reset selection if out of bounds
        if self.selected_index >= self.filtered_files.len() {
            self.selected_index = self.filtered_files.len().saturating_sub(1);
        }
        self.scroll_offset = 0;
        self.adjust_scroll();
    }

    fn adjust_scroll(&mut self) {
        // Keep selected item visible (assuming max 8 visible items)
        let visible_items = 8;
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_items {
            self.scroll_offset = self.selected_index - visible_items + 1;
        }
    }
}

impl Widget for LoadGameModal {
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
            .title(" Load Game ")
            .title_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Rgb(30, 30, 40)));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 4 || inner.width < 20 {
            return; // Too small
        }

        let mut y = inner.y;

        // Search box
        let search_label = "Search: ";
        let search_style = Style::default().fg(Color::Rgb(150, 150, 150));
        for (i, ch) in search_label.chars().enumerate() {
            if inner.x + (i as u16) < inner.x + inner.width {
                buf.get_mut(inner.x + (i as u16), y)
                    .set_char(ch)
                    .set_style(search_style);
            }
        }

        // Search query with cursor
        let query_x = inner.x + search_label.len() as u16;
        let query_style = Style::default().fg(Color::White);
        for (i, ch) in self.search_query.chars().enumerate() {
            if query_x + (i as u16) < inner.x + inner.width {
                buf.get_mut(query_x + (i as u16), y)
                    .set_char(ch)
                    .set_style(query_style);
            }
        }
        // Cursor
        let cursor_x = query_x + self.search_query.len() as u16;
        if cursor_x < inner.x + inner.width {
            buf.get_mut(cursor_x, y)
                .set_char('_')
                .set_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::SLOW_BLINK));
        }

        y += 2;

        // File list
        if self.filtered_files.is_empty() {
            let msg = if self.files.is_empty() {
                "No PGN files found"
            } else {
                "No matching files"
            };
            let msg_style = Style::default().fg(Color::Rgb(120, 120, 120));
            let msg_x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
            for (i, ch) in msg.chars().enumerate() {
                if msg_x + (i as u16) < inner.x + inner.width {
                    buf.get_mut(msg_x + (i as u16), y)
                        .set_char(ch)
                        .set_style(msg_style);
                }
            }
        } else {
            let visible_items = (inner.height - 4) as usize; // Leave room for search and instructions
            let end_idx = (self.scroll_offset + visible_items).min(self.filtered_files.len());

            for (i, file) in self.filtered_files[self.scroll_offset..end_idx].iter().enumerate() {
                let actual_idx = self.scroll_offset + i;
                let is_selected = actual_idx == self.selected_index;

                let line_y = y + (i as u16);
                if line_y >= inner.y + inner.height - 2 {
                    break;
                }

                // Background highlight for selected
                if is_selected {
                    for x in inner.x..inner.x + inner.width {
                        buf.get_mut(x, line_y).set_bg(Color::Rgb(60, 60, 100));
                    }
                }

                // Selection indicator
                let indicator = if is_selected { "> " } else { "  " };
                let indicator_style = Style::default().fg(Color::Yellow);
                for (j, ch) in indicator.chars().enumerate() {
                    if inner.x + (j as u16) < inner.x + inner.width {
                        buf.get_mut(inner.x + (j as u16), line_y)
                            .set_char(ch)
                            .set_style(if is_selected { indicator_style } else { Style::default() });
                    }
                }

                // Filename (truncate if too long)
                let max_len = (inner.width as usize).saturating_sub(4);
                let display_name: String = if file.len() > max_len {
                    format!("{}...", &file[..max_len - 3])
                } else {
                    file.clone()
                };

                let file_style = if is_selected {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(200, 200, 200))
                };

                for (j, ch) in display_name.chars().enumerate() {
                    let x = inner.x + 2 + (j as u16);
                    if x < inner.x + inner.width {
                        buf.get_mut(x, line_y)
                            .set_char(ch)
                            .set_style(file_style);
                    }
                }
            }

            // Show scroll indicators if needed
            if self.scroll_offset > 0 {
                buf.get_mut(inner.x + inner.width - 2, y)
                    .set_char('^')
                    .set_style(Style::default().fg(Color::Rgb(100, 100, 100)));
            }
            if end_idx < self.filtered_files.len() {
                let bottom_y = y + (visible_items as u16).min(inner.height - 4) - 1;
                if bottom_y < inner.y + inner.height - 2 {
                    buf.get_mut(inner.x + inner.width - 2, bottom_y)
                        .set_char('v')
                        .set_style(Style::default().fg(Color::Rgb(100, 100, 100)));
                }
            }
        }

        // Instructions at the bottom
        let instructions = "[Enter] Load  [Esc] Cancel";
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
