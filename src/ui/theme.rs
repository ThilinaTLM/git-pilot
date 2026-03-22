use ratatui::prelude::*;
use ratatui::symbols;
use ratatui::widgets::{Block, Borders, Padding};

const BACKDROP_BG: Color = Color::Rgb(2, 6, 14);

pub const PAGE_MARGIN_X: u16 = 2;
pub const PAGE_MARGIN_Y: u16 = 1;
pub const SECTION_GAP: u16 = 1;
pub const PANE_GAP: u16 = 2;

const APP_BG: Color = Color::Rgb(15, 23, 42);
const SURFACE_BG: Color = Color::Rgb(15, 23, 42);
const TEXT_PRIMARY: Color = Color::Rgb(226, 232, 240);
const TEXT_MUTED: Color = Color::Rgb(148, 163, 184);
const BORDER: Color = Color::Rgb(71, 85, 105);
const ACCENT: Color = Color::Rgb(34, 211, 238);
const ACCENT_SOFT: Color = Color::Rgb(22, 78, 99);
const SUCCESS: Color = Color::Rgb(74, 222, 128);
const WARNING: Color = Color::Rgb(250, 204, 21);
const ERROR: Color = Color::Rgb(248, 113, 113);

pub fn screen_style() -> Style {
    Style::default().bg(APP_BG).fg(TEXT_PRIMARY)
}

pub fn pane_block(title: impl Into<String>) -> Block<'static> {
    chrome_block(title.into(), Padding::new(1, 1, 0, 0))
}

pub fn render_backdrop(frame: &mut ratatui::Frame, area: Rect) {
    let backdrop = Block::default().style(Style::default().bg(BACKDROP_BG));
    frame.render_widget(backdrop, area);
}

pub fn footer_block() -> Block<'static> {
    Block::default()
        .style(surface_style())
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(border_style())
        .padding(Padding::horizontal(1))
}

pub fn header_separator_style() -> Style {
    Style::default().fg(BORDER).bg(SURFACE_BG)
}

pub fn render_header_rule(frame: &mut ratatui::Frame, area: Rect) {
    let rule = "─".repeat(area.width as usize);
    let line = Line::from(Span::styled(
        rule,
        Style::default().fg(BORDER).bg(SURFACE_BG),
    ));
    frame.render_widget(ratatui::widgets::Paragraph::new(line), area);
}

pub fn repo_name_style() -> Style {
    Style::default()
        .fg(TEXT_PRIMARY)
        .bg(SURFACE_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn text_style() -> Style {
    Style::default().fg(TEXT_PRIMARY).bg(SURFACE_BG)
}

pub fn muted_text_style() -> Style {
    Style::default().fg(TEXT_MUTED).bg(SURFACE_BG)
}

pub fn accent_text_style() -> Style {
    Style::default()
        .fg(ACCENT)
        .bg(SURFACE_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn success_text_style() -> Style {
    Style::default().fg(SUCCESS).bg(SURFACE_BG)
}

pub fn warning_text_style() -> Style {
    Style::default().fg(WARNING).bg(SURFACE_BG)
}

pub fn error_text_style() -> Style {
    Style::default().fg(ERROR).bg(SURFACE_BG)
}

pub fn selected_list_style() -> Style {
    Style::default()
        .fg(TEXT_PRIMARY)
        .bg(ACCENT_SOFT)
        .add_modifier(Modifier::BOLD)
}

pub fn repo_pill_active_style() -> Style {
    Style::default()
        .fg(TEXT_PRIMARY)
        .bg(ACCENT_SOFT)
        .add_modifier(Modifier::BOLD)
}

pub fn repo_pill_inactive_style() -> Style {
    Style::default().fg(TEXT_MUTED).bg(SURFACE_BG)
}

pub fn active_tab_style() -> Style {
    Style::default()
        .fg(APP_BG)
        .bg(ACCENT)
        .add_modifier(Modifier::BOLD)
}

pub fn inactive_tab_style() -> Style {
    Style::default().fg(TEXT_MUTED).bg(SURFACE_BG)
}

pub fn section_header_style() -> Style {
    Style::default()
        .fg(ACCENT)
        .bg(SURFACE_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn status_symbol_style(staged: bool, unstaged: bool, untracked: bool) -> Style {
    if untracked {
        warning_text_style()
    } else if staged && unstaged {
        accent_text_style()
    } else if staged {
        success_text_style()
    } else if unstaged {
        warning_text_style()
    } else {
        muted_text_style()
    }
}

pub fn title_line(title: impl Into<String>) -> Line<'static> {
    Line::from(vec![Span::styled(
        format!(" {} ", title.into()),
        Style::default()
            .fg(ACCENT)
            .bg(SURFACE_BG)
            .add_modifier(Modifier::BOLD),
    )])
}

pub const MODAL_BG: Color = Color::Rgb(22, 33, 52);

pub fn modal_elevated_block(title: impl Into<String>) -> Block<'static> {
    Block::default()
        .style(Style::default().fg(TEXT_PRIMARY).bg(MODAL_BG))
        .title(Line::from(vec![Span::styled(
            format!(" {} ", title.into()),
            Style::default()
                .fg(ACCENT)
                .bg(MODAL_BG)
                .add_modifier(Modifier::BOLD),
        )]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(BORDER).bg(MODAL_BG))
        .padding(Padding::new(2, 2, 1, 1))
}

pub fn input_block(label: impl Into<String>) -> Block<'static> {
    Block::default()
        .style(Style::default().fg(TEXT_PRIMARY).bg(Color::Rgb(15, 23, 42)))
        .title(Line::from(vec![Span::styled(
            format!(" {} ", label.into()),
            Style::default().fg(TEXT_MUTED).bg(Color::Rgb(15, 23, 42)),
        )]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(BORDER).bg(Color::Rgb(15, 23, 42)))
        .padding(Padding::horizontal(1))
}

pub fn input_block_focused(label: impl Into<String>) -> Block<'static> {
    Block::default()
        .style(Style::default().fg(TEXT_PRIMARY).bg(Color::Rgb(15, 23, 42)))
        .title(Line::from(vec![Span::styled(
            format!(" {} ", label.into()),
            Style::default()
                .fg(ACCENT)
                .bg(Color::Rgb(15, 23, 42))
                .add_modifier(Modifier::BOLD),
        )]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(ACCENT).bg(Color::Rgb(15, 23, 42)))
        .padding(Padding::horizontal(1))
}

pub fn modal_text_style() -> Style {
    Style::default().fg(TEXT_PRIMARY).bg(MODAL_BG)
}

pub fn modal_muted_style() -> Style {
    Style::default().fg(TEXT_MUTED).bg(MODAL_BG)
}

pub fn modal_accent_style() -> Style {
    Style::default()
        .fg(ACCENT)
        .bg(MODAL_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn separator_style() -> Style {
    Style::default().fg(BORDER).bg(MODAL_BG)
}

fn chrome_block(title: String, padding: Padding) -> Block<'static> {
    Block::default()
        .style(surface_style())
        .title(title_line(title))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(border_style())
        .padding(padding)
}

fn border_style() -> Style {
    Style::default().fg(BORDER).bg(SURFACE_BG)
}

fn surface_style() -> Style {
    Style::default().fg(TEXT_PRIMARY).bg(SURFACE_BG)
}
