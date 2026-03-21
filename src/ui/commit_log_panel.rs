use ratatui::prelude::*;
use ratatui::widgets::{Clear, List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let modal = centered_rect(70, 80, area);
    frame.render_widget(Clear, modal);
    let block = theme::modal_block("Commits");
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(inner);

    render_log_list(frame, halves[0], state);
    render_commit_detail(frame, halves[1], state);
}

fn render_log_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let Some(repo) = state.selected_repo_ref() else {
        let empty = Paragraph::new("No repository selected.").style(theme::muted_text_style());
        frame.render_widget(empty, area);
        return;
    };

    if repo.log_entries.is_empty() {
        let empty = Paragraph::new("No commits found.").style(theme::muted_text_style());
        frame.render_widget(empty, area);
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
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_commit_detail(frame: &mut Frame, area: Rect, state: &AppState) {
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

    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("Esc ", theme::accent_text_style()),
        Span::styled("close", theme::muted_text_style()),
    ]));

    let paragraph = Paragraph::new(Text::from(lines))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .scroll((state.log_scroll, 0));
    frame.render_widget(paragraph, area);
}
