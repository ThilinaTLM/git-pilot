use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};

use crate::ui::theme;

pub struct ScreenLayout {
    pub tabs: Rect,
    pub status_bar: Rect,
    pub file_list: Rect,
    pub diff_preview: Rect,
    pub footer: Rect,
}

pub fn build_layout(area: Rect) -> ScreenLayout {
    let page = area.inner(Margin {
        horizontal: theme::PAGE_MARGIN_X,
        vertical: theme::PAGE_MARGIN_Y,
    });
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(theme::SECTION_GAP),
            Constraint::Length(1),
            Constraint::Length(theme::SECTION_GAP),
            Constraint::Min(10),
            Constraint::Length(theme::SECTION_GAP),
            Constraint::Length(3),
        ])
        .split(page);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(theme::PANE_GAP),
            Constraint::Percentage(60),
        ])
        .split(vertical[4]);

    ScreenLayout {
        tabs: vertical[0],
        status_bar: vertical[2],
        file_list: horizontal[0],
        diff_preview: horizontal[2],
        footer: vertical[6],
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
