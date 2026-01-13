use ratatui::style::Color;

/// Square background colours for different states
pub struct SquareColours {
    pub light: Color,
    pub dark: Color,
    pub cursor: Color,
    pub selected: Color,
    pub legal_move_light: Color,
    pub legal_move_dark: Color,
    pub check: Color,
}

impl Default for SquareColours {
    fn default() -> Self {
        Self {
            // Standard board colours (warm tones)
            light: Color::Rgb(240, 217, 181),  // Warm cream
            dark: Color::Rgb(181, 136, 99),    // Warm brown

            // Cursor highlight
            cursor: Color::Rgb(100, 149, 237), // Cornflower blue

            // Selected piece
            selected: Color::Rgb(70, 130, 180), // Steel blue

            // Legal move highlights (tinted versions of light/dark)
            legal_move_light: Color::Rgb(170, 210, 170), // Light green tint
            legal_move_dark: Color::Rgb(130, 170, 100),  // Dark green tint

            // King in check
            check: Color::Rgb(220, 80, 80), // Red
        }
    }
}
