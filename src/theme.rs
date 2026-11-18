use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Padding},
};

use crate::Surface;

pub const VOID: Color = Color::Rgb(10, 13, 20);
pub const PANEL: Color = Color::Rgb(17, 22, 32);
pub const PANEL_RAISED: Color = Color::Rgb(23, 29, 42);
pub const TEXT: Color = Color::Rgb(225, 231, 239);
pub const MUTED: Color = Color::Rgb(112, 124, 143);
pub const FAINT: Color = Color::Rgb(54, 64, 82);
pub const CYAN: Color = Color::Rgb(83, 214, 214);
pub const BLUE: Color = Color::Rgb(105, 165, 255);
pub const VIOLET: Color = Color::Rgb(190, 145, 255);
pub const PINK: Color = Color::Rgb(255, 126, 182);
pub const GREEN: Color = Color::Rgb(102, 217, 153);
pub const YELLOW: Color = Color::Rgb(242, 194, 100);
pub const RED: Color = Color::Rgb(255, 112, 112);

pub const fn surface_accent(surface: Surface) -> Color {
    match surface {
        Surface::Overview => CYAN,
        Surface::Agenda => PINK,
        Surface::Chronos => BLUE,
        Surface::Vellum => VIOLET,
        Surface::Workspace => BLUE,
        Surface::Agora => VIOLET,
        Surface::Moira => PINK,
        Surface::Daimon => GREEN,
        Surface::Hestia => Color::Rgb(255, 171, 92),
        Surface::Tellus => YELLOW,
        Surface::Aegis => Color::Rgb(255, 150, 92),
        Surface::Hermes => Color::Rgb(99, 200, 255),
        Surface::Operations => Color::Rgb(183, 195, 214),
        Surface::Configuration => Color::Rgb(125, 224, 190),
    }
}

pub const fn base() -> Style {
    Style::new().fg(TEXT).bg(VOID)
}

pub const fn panel() -> Style {
    Style::new().fg(TEXT).bg(PANEL)
}

pub const fn raised() -> Style {
    Style::new().fg(TEXT).bg(PANEL_RAISED)
}

pub const fn muted() -> Style {
    Style::new().fg(MUTED).bg(VOID)
}

pub const fn key() -> Style {
    Style::new().fg(CYAN).bg(VOID).add_modifier(Modifier::BOLD)
}

pub fn card<'a>(title: impl Into<ratatui::text::Line<'a>>, accent: Color) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(FAINT).bg(PANEL))
        .style(panel())
        .padding(Padding::horizontal(1))
        .title(title)
        .title_style(
            Style::new()
                .fg(accent)
                .bg(PANEL)
                .add_modifier(Modifier::BOLD),
        )
}

pub fn modal<'a>(title: impl Into<ratatui::text::Line<'a>>) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(CYAN).bg(PANEL_RAISED))
        .style(raised())
        .padding(Padding::horizontal(1))
        .title(title)
        .title_style(
            Style::new()
                .fg(CYAN)
                .bg(PANEL_RAISED)
                .add_modifier(Modifier::BOLD),
        )
}
