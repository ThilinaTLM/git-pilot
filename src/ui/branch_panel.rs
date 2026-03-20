use ratatui::prelude::*;
use ratatui::widgets::{Clear, List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let content = if let Some(repo) = state.selected_repo_ref() {
        let branch = repo
            .current_branch
            .as_deref()
            .unwrap_or("(detached)");
        let branch_count = repo.branches.len();
        let file_count = repo.status_files.len();
        Line::from(vec![
            Span::styled(format!("  {branch}"), theme::accent_text_style()),
            Span::styled(" • ", theme::muted_text_style()),
            Span::styled(
                format!("{branch_count} branch{}", if branch_count == 1 { "" } else { "es" }),
                theme::text_style(),
            ),
            Span::styled(" • ", theme::muted_text_style()),
            Span::styled(
                format!(
                    "{file_count} changed file{}",
                    if file_count == 1 { "" } else { "s" }
                ),
                theme::text_style(),
            ),
        ])
    } else {
        Line::from(Span::styled(
            "  No repository selected.",
            theme::muted_text_style(),
        ))
    };

    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, area);
}

pub fn render_switch_modal(frame: &mut Frame, area: Rect, state: &AppState) {
    let modal = centered_rect(50, 60, area);
    frame.render_widget(Clear, modal);
    let block = theme::modal_block("Switch Branch");
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let items = state
        .selected_repo_ref()
        .map(|repo| {
            repo.branches
                .iter()
                .map(|branch| {
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
                    ListItem::new(Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(branch.clone(), style),
                    ]))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut list_state = ListState::default();
    if !items.is_empty() {
        list_state.select(Some(state.selected_branch.min(items.len() - 1)));
    }

    let list = List::new(items)
        .style(theme::text_style())
        .highlight_style(theme::selected_list_style())
        .highlight_symbol("▸ ");

    frame.render_stateful_widget(list, inner, &mut list_state);
}

pub fn render_create_modal(frame: &mut Frame, area: Rect, state: &AppState) {
    let modal = centered_rect(50, 25, area);
    frame.render_widget(Clear, modal);
    let block = theme::modal_block("Create Branch");
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let paragraph = Paragraph::new(vec![
        Line::from(Span::styled(
            "Enter a new branch name.",
            theme::muted_text_style(),
        )),
        Line::default(),
        Line::from(Span::styled(
            state.branch_name_input.as_str(),
            theme::text_style(),
        )),
        Line::default(),
        Line::from(Span::styled(
            "Enter to create, Esc to cancel.",
            theme::muted_text_style(),
        )),
    ])
    .wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}
