use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let (list_area, detail_area) = layout::split_log_view(area);
    render_log_list(frame, list_area, state);
    render_commit_detail(frame, detail_area, state);
}

fn render_log_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::pane_block("Commits");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(repo) = state.selected_repo_ref() else {
        let empty = Paragraph::new("No repository selected.").style(theme::muted_text_style());
        frame.render_widget(empty, inner);
        return;
    };

    if repo.log_entries.is_empty() {
        let empty = Paragraph::new("No commits found.").style(theme::muted_text_style());
        frame.render_widget(empty, inner);
        return;
    }

    let mut items = Vec::new();
    for entry in &repo.log_entries {
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", entry.hash), theme::accent_text_style()),
            Span::styled(&entry.subject, theme::text_style()),
        ])));
    }

    let list = List::new(items)
        .style(theme::text_style())
        .highlight_style(theme::selected_list_style())
        .highlight_symbol("▸ ");

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_log_entry));
    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn render_commit_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::pane_block("Commit");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(repo) = state.selected_repo_ref() else {
        return;
    };

    let Some(entry) = repo.log_entries.get(state.selected_log_entry) else {
        return;
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Hash:    ", theme::muted_text_style()),
            Span::styled(&entry.hash, theme::accent_text_style()),
        ]),
        Line::from(vec![
            Span::styled("Author:  ", theme::muted_text_style()),
            Span::styled(&entry.author, theme::text_style()),
        ]),
        Line::from(vec![
            Span::styled("Date:    ", theme::muted_text_style()),
            Span::styled(&entry.date, theme::text_style()),
        ]),
        Line::default(),
        Line::from(Span::styled(
            &entry.subject,
            theme::text_style().add_modifier(Modifier::BOLD),
        )),
    ];

    let body = entry.full_message.trim();
    if !body.is_empty() {
        lines.push(Line::default());
        for line in body.lines() {
            lines.push(Line::from(Span::styled(line, theme::text_style())));
        }
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .scroll((state.log_scroll, 0));
    frame.render_widget(paragraph, inner);
}
