use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

use crate::app::actions::AppAction;
use crate::app::state::{AppState, Modal, View};
use crate::ui::layout;

/// Map a mouse event to an `AppAction` based on current layout and state.
pub fn map_mouse_event(mouse: MouseEvent, terminal_area: Rect, state: &AppState) -> AppAction {
    if state.modal != Modal::None {
        return AppAction::Noop;
    }

    let screen = layout::build_layout(terminal_area);

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            map_left_click(mouse.column, mouse.row, &screen, state)
        }
        MouseEventKind::ScrollDown => map_scroll_down(mouse.column, mouse.row, &screen, state),
        MouseEventKind::ScrollUp => map_scroll_up(mouse.column, mouse.row, &screen, state),
        _ => AppAction::Noop,
    }
}

fn map_left_click(
    col: u16,
    row: u16,
    screen: &layout::ScreenLayout,
    state: &AppState,
) -> AppAction {
    if in_rect(col, row, screen.header_row1) {
        return hit_repo_tab(col, screen.header_row1, state);
    }

    if in_rect(col, row, screen.header_row2) {
        return hit_view_tab(col, screen.header_row2);
    }

    if in_rect(col, row, screen.view_area) {
        return hit_view_area(col, row, screen.view_area, state);
    }

    AppAction::Noop
}

fn map_scroll_down(
    col: u16,
    row: u16,
    screen: &layout::ScreenLayout,
    state: &AppState,
) -> AppAction {
    if !in_rect(col, row, screen.view_area) {
        return AppAction::Noop;
    }

    match state.active_view {
        View::Changes => {
            let (left, right) = layout::split_changes_view(screen.view_area);
            if in_rect(col, row, left) {
                AppAction::SelectNextFile
            } else if in_rect(col, row, right) {
                AppAction::ScrollDiffDown
            } else {
                AppAction::Noop
            }
        }
        View::Branches => AppAction::SelectNextBranch,
        View::Commits => {
            let (left, right) = layout::split_log_view(screen.view_area);
            if in_rect(col, row, left) {
                AppAction::SelectNextLogEntry
            } else if in_rect(col, row, right) {
                AppAction::ScrollLogDown
            } else {
                AppAction::Noop
            }
        }
        View::Pr => {
            let (left, right) = layout::split_pr_view(screen.view_area);
            if in_rect(col, row, left) {
                AppAction::SelectNextPr
            } else if in_rect(col, row, right) {
                AppAction::ScrollPrDetailDown
            } else {
                AppAction::Noop
            }
        }
        View::Settings => AppAction::SelectNextSettingsItem,
    }
}

fn map_scroll_up(col: u16, row: u16, screen: &layout::ScreenLayout, state: &AppState) -> AppAction {
    if !in_rect(col, row, screen.view_area) {
        return AppAction::Noop;
    }

    match state.active_view {
        View::Changes => {
            let (left, right) = layout::split_changes_view(screen.view_area);
            if in_rect(col, row, left) {
                AppAction::SelectPreviousFile
            } else if in_rect(col, row, right) {
                AppAction::ScrollDiffUp
            } else {
                AppAction::Noop
            }
        }
        View::Branches => AppAction::SelectPreviousBranch,
        View::Commits => {
            let (left, right) = layout::split_log_view(screen.view_area);
            if in_rect(col, row, left) {
                AppAction::SelectPreviousLogEntry
            } else if in_rect(col, row, right) {
                AppAction::ScrollLogUp
            } else {
                AppAction::Noop
            }
        }
        View::Pr => {
            let (left, right) = layout::split_pr_view(screen.view_area);
            if in_rect(col, row, left) {
                AppAction::SelectPreviousPr
            } else if in_rect(col, row, right) {
                AppAction::ScrollPrDetailUp
            } else {
                AppAction::Noop
            }
        }
        View::Settings => AppAction::SelectPreviousSettingsItem,
    }
}

fn hit_repo_tab(col: u16, area: Rect, state: &AppState) -> AppAction {
    if state.repos.is_empty() {
        return AppAction::Noop;
    }

    let mut x = area.x;
    for (i, repo) in state.repos.iter().enumerate() {
        if i > 0 {
            x += 2; // "  " separator
        }
        let pill_width = (repo.summary.name.len() as u16) + 2; // " name "
        if col >= x && col < x + pill_width {
            return AppAction::SelectRepo(i);
        }
        x += pill_width;
    }

    AppAction::Noop
}

fn hit_view_tab(col: u16, area: Rect) -> AppAction {
    let tabs: &[(&str, View)] = &[
        ("Changes", View::Changes),
        ("Branches", View::Branches),
        ("Commits", View::Commits),
        ("PR", View::Pr),
        ("Settings", View::Settings),
    ];

    let mut x = area.x;
    for (i, (label, view)) in tabs.iter().enumerate() {
        if i > 0 {
            x += 3; // " │ " separator
        }
        let tab_width = (label.len() as u16) + 2; // " label "
        if col >= x && col < x + tab_width {
            return AppAction::SwitchView(view.clone());
        }
        x += tab_width;
    }

    AppAction::Noop
}

fn hit_view_area(col: u16, row: u16, view_area: Rect, state: &AppState) -> AppAction {
    match state.active_view {
        View::Changes => {
            let (left, _) = layout::split_changes_view(view_area);
            if in_rect(col, row, left) {
                return hit_file_item(row, left, state);
            }
        }
        View::Branches => {
            let (left, _) = layout::split_branches_view(view_area);
            if in_rect(col, row, left) {
                return hit_branch_item(row, left, state);
            }
        }
        View::Commits => {
            let (left, _) = layout::split_log_view(view_area);
            if in_rect(col, row, left) {
                return hit_log_item(row, left, state);
            }
        }
        View::Pr => {
            let (left, _) = layout::split_pr_view(view_area);
            if in_rect(col, row, left) {
                return hit_pr_item(row, left, state);
            }
        }
        View::Settings => {
            let (left, _) = layout::split_settings_view(view_area);
            if in_rect(col, row, left) {
                return hit_settings_item(row, left);
            }
        }
    }

    AppAction::Noop
}

/// Click on a file in the Changes list (accounts for section headers).
fn hit_file_item(row: u16, list_area: Rect, state: &AppState) -> AppAction {
    let inner_y = list_area.y + 1; // block border
    if row < inner_y {
        return AppAction::Noop;
    }
    let clicked_row = (row - inner_y) as usize;

    let mut row_idx: usize = 0;
    let mut current_section = None;

    for (entry_idx, entry) in state.grouped_files.entries.iter().enumerate() {
        if current_section != Some(entry.section) {
            current_section = Some(entry.section);
            if row_idx == clicked_row {
                return AppAction::Noop; // section header
            }
            row_idx += 1;
        }
        if row_idx == clicked_row {
            return AppAction::SelectFile(entry_idx);
        }
        row_idx += 1;
    }

    AppAction::Noop
}

/// Click on a branch (row 0 = header, rows 1..N = branches).
fn hit_branch_item(row: u16, list_area: Rect, state: &AppState) -> AppAction {
    let inner_y = list_area.y + 1;
    if row < inner_y {
        return AppAction::Noop;
    }
    let clicked_row = (row - inner_y) as usize;

    if clicked_row == 0 {
        return AppAction::Noop; // section header
    }
    let branch_idx = clicked_row - 1;
    if branch_idx < state.filtered_branches.len() {
        return AppAction::SelectBranch(branch_idx);
    }

    AppAction::Noop
}

/// Click on a commit (direct mapping, no headers).
fn hit_log_item(row: u16, list_area: Rect, state: &AppState) -> AppAction {
    let inner_y = list_area.y + 1;
    if row < inner_y {
        return AppAction::Noop;
    }
    let clicked_row = (row - inner_y) as usize;

    if let Some(repo) = state.selected_repo_ref()
        && clicked_row < repo.log_entries.len()
    {
        return AppAction::SelectLogEntry(clicked_row);
    }

    AppAction::Noop
}

/// Click on a PR (direct mapping, no headers).
fn hit_pr_item(row: u16, list_area: Rect, state: &AppState) -> AppAction {
    let inner_y = list_area.y + 1;
    if row < inner_y {
        return AppAction::Noop;
    }
    let clicked_row = (row - inner_y) as usize;

    if let Some(repo) = state.selected_repo_ref()
        && clicked_row < repo.pull_requests.len()
    {
        return AppAction::SelectPr(clicked_row);
    }

    AppAction::Noop
}

/// Click on a settings item (row 0 = header, row 1 = auto-fetch, row 2 = interval).
fn hit_settings_item(row: u16, list_area: Rect) -> AppAction {
    let inner_y = list_area.y + 1;
    if row < inner_y {
        return AppAction::Noop;
    }
    let clicked_row = (row - inner_y) as usize;

    match clicked_row {
        1 => AppAction::SelectSettingsItem(0),
        2 => AppAction::SelectSettingsItem(1),
        _ => AppAction::Noop,
    }
}

fn in_rect(col: u16, row: u16, rect: Rect) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}

#[cfg(test)]
mod tests {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    use ratatui::layout::Rect;

    use crate::app::actions::AppAction;
    use crate::app::state::{AppState, View};

    use super::map_mouse_event;

    fn make_click(col: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::NONE,
        }
    }

    #[test]
    fn click_on_view_tab_switches_view() {
        use crate::ui::layout;

        let state = AppState::default();
        let terminal = Rect::new(0, 0, 80, 24);
        let screen = layout::build_layout(terminal);
        let row2 = screen.header_row2;

        // " Changes " = 9 chars, " │ " = 3 chars, then " Branches " starts
        let branches_x = row2.x + 9 + 3;
        let action = map_mouse_event(make_click(branches_x + 1, row2.y), terminal, &state);
        assert_eq!(action, AppAction::SwitchView(View::Branches));
    }

    #[test]
    fn click_outside_tabs_is_noop() {
        let state = AppState::default();
        let terminal = Rect::new(0, 0, 80, 24);

        // Click far right where no tab exists
        let action = map_mouse_event(make_click(79, 3), terminal, &state);
        assert_eq!(action, AppAction::Noop);
    }

    fn make_scroll_down(col: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::NONE,
        }
    }

    #[test]
    fn scroll_in_view_area_navigates() {
        let state = AppState::default();
        let terminal = Rect::new(0, 0, 80, 24);

        // Scroll in the view area (below header, above footer)
        let action = map_mouse_event(make_scroll_down(10, 10), terminal, &state);
        // Default view is Changes, left panel → SelectNextFile
        assert_eq!(action, AppAction::SelectNextFile);
    }
}
