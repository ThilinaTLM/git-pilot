use ratatui::prelude::*;
use ratatui::widgets::{Clear, List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    theme::render_backdrop(frame, area);
    let modal = centered_rect(50, 70, area);
    frame.render_widget(Clear, modal);

    let total_count = state
        .selected_repo_ref()
        .map(|r| r.branches.len())
        .unwrap_or(0);
    let filtered_count = state.filtered_branches.len();
    let title = if state.branch_filter.is_empty() {
        format!("Branches ({total_count})")
    } else {
        format!("Branches ({filtered_count} of {total_count})")
    };
    let block = theme::modal_elevated_block(title);
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let filter_height = if state.branch_filter_active { 3 } else { 0 };
    let filter_gap = if state.branch_filter_active { 1 } else { 0 };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),             // current branch + tracking
            Constraint::Length(1),             // gap
            Constraint::Length(filter_height), // filter input
            Constraint::Length(filter_gap),    // gap after filter
            Constraint::Min(3),                // branch list
            Constraint::Length(1),             // gap
            Constraint::Length(2),             // shortcut hints
        ])
        .split(inner);

    // Current branch + tracking status
    render_header(frame, layout[0], state);

    // Filter input
    if state.branch_filter_active {
        render_filter(frame, layout[2], state);
    }

    // Branch list
    render_branch_list(frame, layout[4], state);

    // Shortcut hints
    render_shortcuts(frame, layout[6]);
}

pub fn render_merge_confirm(frame: &mut Frame, area: Rect, state: &AppState) {
    theme::render_backdrop(frame, area);
    let modal = centered_rect(50, 20, area);
    frame.render_widget(Clear, modal);
    let block = theme::modal_elevated_block("Confirm Merge");
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let branch_name = state.merge_confirm_branch.as_deref().unwrap_or("unknown");
    let current = state
        .selected_repo_ref()
        .and_then(|r| r.current_branch.as_deref())
        .unwrap_or("unknown");

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(inner);

    let question = Line::from(vec![
        Span::styled("Merge ", theme::modal_text_style()),
        Span::styled(branch_name, theme::modal_accent_style()),
        Span::styled(" into ", theme::modal_text_style()),
        Span::styled(current, theme::modal_accent_style()),
        Span::styled("?", theme::modal_text_style()),
    ]);
    frame.render_widget(Paragraph::new(question), layout[0]);

    let shortcuts = Line::from(vec![
        Span::styled("y ", theme::modal_accent_style()),
        Span::styled("confirm", theme::modal_muted_style()),
        Span::styled("  n ", theme::modal_accent_style()),
        Span::styled("cancel", theme::modal_muted_style()),
    ]);
    frame.render_widget(Paragraph::new(shortcuts), layout[4]);
}

fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let current = state
        .selected_repo_ref()
        .and_then(|r| r.current_branch.as_deref())
        .unwrap_or("none");

    let mut spans = vec![
        Span::styled("On: ", theme::modal_muted_style()),
        Span::styled(current, theme::modal_accent_style()),
    ];

    if let Some(tracking) = &state.branch_tracking {
        if tracking.ahead == 0 && tracking.behind == 0 {
            spans.push(Span::styled("  ✓ up to date", theme::modal_muted_style()));
        } else {
            if tracking.ahead > 0 {
                spans.push(Span::styled(
                    format!("  ↑{}", tracking.ahead),
                    theme::modal_accent_style(),
                ));
            }
            if tracking.behind > 0 {
                spans.push(Span::styled(
                    format!("  ↓{}", tracking.behind),
                    theme::modal_muted_style(),
                ));
            }
        }
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_filter(frame: &mut Frame, area: Rect, state: &AppState) {
    let filter_block = theme::input_block_focused("Filter");
    let filter_inner = filter_block.inner(area);
    frame.render_widget(filter_block, area);

    let filter_display = if state.branch_filter.is_empty() {
        Line::from(Span::styled(
            "type to filter...",
            Style::default()
                .fg(Color::Rgb(71, 85, 105))
                .bg(Color::Rgb(15, 23, 42)),
        ))
    } else {
        Line::from(vec![
            Span::styled(
                state.branch_filter.clone(),
                Style::default()
                    .fg(Color::Rgb(226, 232, 240))
                    .bg(Color::Rgb(15, 23, 42)),
            ),
            Span::styled(
                "_",
                Style::default()
                    .fg(Color::Rgb(34, 211, 238))
                    .bg(Color::Rgb(15, 23, 42)),
            ),
        ])
    };
    frame.render_widget(Paragraph::new(filter_display), filter_inner);
}

fn render_branch_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let Some(repo) = state.selected_repo_ref() else {
        let empty = Paragraph::new("No repository selected.").style(theme::modal_muted_style());
        frame.render_widget(empty, area);
        return;
    };

    if repo.branches.is_empty() {
        let empty = Paragraph::new("No branches found.").style(theme::modal_muted_style());
        frame.render_widget(empty, area);
        return;
    }

    let mut items = Vec::new();
    let mut row_to_branch: Vec<Option<usize>> = Vec::new();

    for (filtered_idx, &real_idx) in state.filtered_branches.iter().enumerate() {
        let branch = &repo.branches[real_idx];
        let is_current = repo
            .current_branch
            .as_deref()
            .is_some_and(|current| current == branch);
        let prefix = if is_current { "● " } else { "  " };
        let style = if is_current {
            theme::modal_accent_style()
        } else {
            theme::modal_text_style()
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(branch.clone(), style),
        ])));
        row_to_branch.push(Some(filtered_idx));
    }

    let visual_row = row_to_branch
        .iter()
        .position(|r| *r == Some(state.selected_branch));

    let list = List::new(items)
        .style(theme::modal_text_style())
        .highlight_style(theme::selected_list_style())
        .highlight_symbol("▸ ");

    let mut list_state = ListState::default();
    list_state.select(visual_row);
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_shortcuts(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(vec![
            Span::styled("Enter ", theme::modal_accent_style()),
            Span::styled("switch", theme::modal_muted_style()),
            Span::styled("  n ", theme::modal_accent_style()),
            Span::styled("create", theme::modal_muted_style()),
            Span::styled("  d ", theme::modal_accent_style()),
            Span::styled("delete", theme::modal_muted_style()),
            Span::styled("  m ", theme::modal_accent_style()),
            Span::styled("merge", theme::modal_muted_style()),
        ]),
        Line::from(vec![
            Span::styled("f ", theme::modal_accent_style()),
            Span::styled("fetch", theme::modal_muted_style()),
            Span::styled("  p ", theme::modal_accent_style()),
            Span::styled("push", theme::modal_muted_style()),
            Span::styled("  P ", theme::modal_accent_style()),
            Span::styled("pull", theme::modal_muted_style()),
            Span::styled("  / ", theme::modal_accent_style()),
            Span::styled("filter", theme::modal_muted_style()),
            Span::styled("  Esc ", theme::modal_accent_style()),
            Span::styled("close", theme::modal_muted_style()),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}
