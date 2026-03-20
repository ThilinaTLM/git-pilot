use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};

use crate::ui::theme;

pub struct ScreenLayout {
    pub header_row1: Rect,
    pub header_row2: Rect,
    pub header_rule: Rect,
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
            Constraint::Length(1), // row1: repo pill tabs
            Constraint::Length(1), // spacer
            Constraint::Length(1), // row2: view tabs
            Constraint::Length(1), // horizontal rule
            Constraint::Min(8),    // view_area
            Constraint::Length(1), // footer (single line)
        ])
        .split(page);

    ScreenLayout {
        header_row1: vertical[0],
        header_row2: vertical[2],
        header_rule: vertical[3],
        view_area: vertical[4],
        footer: vertical[5],
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

pub fn split_pr_view(area: Rect) -> (Rect, Rect) {
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

pub fn split_settings_view(area: Rect) -> (Rect, Rect) {
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
