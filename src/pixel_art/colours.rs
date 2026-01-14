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
    pub capture_flash: Color,  // Bright flash on capture
    pub capture_fade: Color,   // Fade out after flash
    pub last_move_light: Color, // Highlight for last move (light square)
    pub last_move_dark: Color,  // Highlight for last move (dark square)
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

            // Capture animation colours
            capture_flash: Color::Rgb(255, 80, 80),  // Bright red flash
            capture_fade: Color::Rgb(255, 160, 80),  // Orange fade

            // Last move highlight (pale yellow tones)
            last_move_light: Color::Rgb(205, 210, 106), // Pale yellow for light squares
            last_move_dark: Color::Rgb(170, 162, 58),   // Darker yellow for dark squares
        }
    }
}
