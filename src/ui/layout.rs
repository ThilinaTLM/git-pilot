use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};

use crate::ui::theme;

pub struct ScreenLayout {
    pub tabs: Rect,
    pub view_tabs: Rect,
    pub status_bar: Rect,
    pub view_area: Rect,
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
            Constraint::Length(3),                  // tabs
            Constraint::Length(theme::SECTION_GAP), // gap
            Constraint::Length(1),                  // view_tabs
            Constraint::Length(theme::SECTION_GAP), // gap
            Constraint::Length(1),                  // status_bar
            Constraint::Length(theme::SECTION_GAP), // gap
            Constraint::Min(8),                     // view_area
            Constraint::Length(theme::SECTION_GAP), // gap
            Constraint::Length(3),                  // footer
        ])
        .split(page);

    ScreenLayout {
        tabs: vertical[0],
        view_tabs: vertical[2],
        status_bar: vertical[4],
        view_area: vertical[6],
        footer: vertical[8],
    }
}

pub fn split_changes_view(area: Rect) -> (Rect, Rect) {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(theme::PANE_GAP),
            Constraint::Percentage(60),
        ])
        .split(area);
    (horizontal[0], horizontal[2])
}

pub fn split_branches_view(area: Rect) -> (Rect, Rect) {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(theme::PANE_GAP),
            Constraint::Percentage(60),
        ])
        .split(area);
    (horizontal[0], horizontal[2])
}

pub fn split_log_view(area: Rect) -> (Rect, Rect) {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(theme::PANE_GAP),
            Constraint::Percentage(60),
        ])
        .split(area);
    (horizontal[0], horizontal[2])
}

pub fn split_remotes_view(area: Rect) -> (Rect, Rect) {
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(theme::PANE_GAP),
            Constraint::Percentage(60),
        ])
        .split(area);
    (horizontal[0], horizontal[2])
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
