use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let (list_area, detail_area) = layout::split_branches_view(area);
    render_branch_list(frame, list_area, state);
    render_branch_details(frame, detail_area, state);
}

fn render_branch_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::pane_block("Branches");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(repo) = state.selected_repo_ref() else {
        let empty = Paragraph::new("No repository selected.").style(theme::muted_text_style());
        frame.render_widget(empty, inner);
        return;
    };

    if repo.branches.is_empty() {
        let empty = Paragraph::new("No branches found.").style(theme::muted_text_style());
        frame.render_widget(empty, inner);
        return;
    }

    let branch_count = repo.branches.len();
    let header = ListItem::new(Line::from(Span::styled(
        format!(
            "{branch_count} branch{}",
            if branch_count == 1 { "" } else { "es" }
        ),
        theme::section_header_style(),
    )));

    let mut items = vec![header];
    let mut row_to_branch: Vec<Option<usize>> = vec![None]; // header row

    for (i, branch) in repo.branches.iter().enumerate() {
        let is_current = repo
            .current_branch
            .as_deref()
            .is_some_and(|current| current == branch);
        let prefix = if is_current { "● " } else { "  " };
        let style = if is_current {
            theme::accent_text_style()
        } else {
            theme::text_style()
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(branch.clone(), style),
        ])));
        row_to_branch.push(Some(i));
    }

    let visual_row = row_to_branch
        .iter()
        .position(|r| *r == Some(state.selected_branch));

    let list = List::new(items)
        .style(theme::text_style())
        .highlight_style(theme::selected_list_style())
        .highlight_symbol("▸ ");

    let mut list_state = ListState::default();
    list_state.select(visual_row);
    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn render_branch_details(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::pane_block("Details");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(repo) = state.selected_repo_ref() else {
        return;
    };

    let mut lines = Vec::new();

    if let Some(branch_name) = state.selected_branch_name() {
        let is_current = repo
            .current_branch
            .as_deref()
            .is_some_and(|current| current == branch_name);

        lines.push(Line::from(vec![
            Span::styled("Branch: ", theme::muted_text_style()),
            Span::styled(branch_name.to_string(), theme::accent_text_style()),
        ]));

        if is_current {
            lines.push(Line::from(Span::styled(
                "  (current branch)",
                theme::success_text_style(),
            )));
        }

        if let Some(tracking) = &state.branch_tracking
            && is_current
        {
            lines.push(Line::default());
            let tracking_text = if tracking.ahead == 0 && tracking.behind == 0 {
                "Up to date with remote".to_string()
            } else {
                let mut parts = Vec::new();
                if tracking.ahead > 0 {
                    parts.push(format!("{}  ahead", tracking.ahead));
                }
                if tracking.behind > 0 {
                    parts.push(format!("{}  behind", tracking.behind));
                }
                parts.join(", ")
            };
            lines.push(Line::from(vec![
                Span::styled("Tracking: ", theme::muted_text_style()),
                Span::styled(tracking_text, theme::text_style()),
            ]));
            if let Some(ref remote_name) = tracking.remote_name {
                lines.push(Line::from(vec![
                    Span::styled("Remote:   ", theme::muted_text_style()),
                    Span::styled(remote_name.clone(), theme::text_style()),
                ]));
            }
        } else {
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                "Tracking: not available",
                theme::muted_text_style(),
            )));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "No branch selected.",
            theme::muted_text_style(),
        )));
    }

    // Action hints at bottom
    lines.push(Line::default());
    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("Enter ", theme::accent_text_style()),
        Span::styled("switch", theme::muted_text_style()),
        Span::styled("  n ", theme::accent_text_style()),
        Span::styled("create", theme::muted_text_style()),
        Span::styled("  d ", theme::accent_text_style()),
        Span::styled("delete", theme::muted_text_style()),
        Span::styled("  m ", theme::accent_text_style()),
        Span::styled("merge", theme::muted_text_style()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("f ", theme::accent_text_style()),
        Span::styled("fetch", theme::muted_text_style()),
        Span::styled("  p ", theme::accent_text_style()),
        Span::styled("push", theme::muted_text_style()),
        Span::styled("  P ", theme::accent_text_style()),
        Span::styled("pull", theme::muted_text_style()),
    ]));

    let paragraph = Paragraph::new(Text::from(lines)).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}
