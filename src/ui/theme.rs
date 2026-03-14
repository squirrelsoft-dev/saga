use ratatui::style::{Color, Modifier, Style};

pub const PRIMARY: Color = Color::Rgb(91, 155, 213);    // #5B9BD5
pub const SECONDARY: Color = Color::Rgb(112, 173, 71);  // #70AD47
pub const ACCENT: Color = Color::Rgb(237, 125, 49);     // #ED7D31
pub const DANGER: Color = Color::Rgb(255, 82, 82);      // #FF5252
pub const WARNING: Color = Color::Rgb(255, 193, 7);     // #FFC107
pub const SUCCESS: Color = Color::Rgb(76, 175, 80);     // #4CAF50
pub const MUTED: Color = Color::Rgb(128, 128, 128);     // #808080
pub const BG_DARK: Color = Color::Rgb(30, 30, 46);      // #1E1E2E
pub const BG_SURFACE: Color = Color::Rgb(49, 50, 68);   // #313244
pub const FG: Color = Color::Rgb(205, 214, 244);        // #CDD6F4
pub const FG_DIM: Color = Color::Rgb(147, 153, 178);    // #9399B2

pub const PROJECT_COLORS: &[Color] = &[
    Color::Rgb(91, 155, 213),   // Blue
    Color::Rgb(112, 173, 71),   // Green
    Color::Rgb(237, 125, 49),   // Orange
    Color::Rgb(165, 105, 189),  // Purple
    Color::Rgb(231, 76, 60),    // Red
    Color::Rgb(52, 152, 219),   // Sky
    Color::Rgb(26, 188, 156),   // Teal
    Color::Rgb(241, 196, 15),   // Yellow
];

pub fn title_style() -> Style {
    Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    Style::default().bg(PRIMARY).fg(Color::Black)
}

pub fn header_style() -> Style {
    Style::default().fg(FG).add_modifier(Modifier::BOLD)
}

pub fn muted_style() -> Style {
    Style::default().fg(MUTED)
}

pub fn active_timer_style() -> Style {
    Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD)
}

pub fn danger_style() -> Style {
    Style::default().fg(DANGER)
}
