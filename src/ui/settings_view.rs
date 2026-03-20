use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let (list_area, detail_area) = layout::split_settings_view(area);
    render_remote_list(frame, list_area, state);
    render_remote_details(frame, detail_area, state);
}

fn render_remote_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::pane_block("Remotes");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(repo) = state.selected_repo_ref() else {
        let empty = Paragraph::new("No repository selected.").style(theme::muted_text_style());
        frame.render_widget(empty, inner);
        return;
    };

    if repo.remotes.is_empty() {
        let lines = vec![
            Line::from(Span::styled(
                "No remotes configured.",
                theme::muted_text_style(),
            )),
            Line::default(),
            Line::from(vec![
                Span::styled("Press ", theme::muted_text_style()),
                Span::styled("R", theme::accent_text_style()),
                Span::styled(" to create a GitHub repository", theme::muted_text_style()),
            ]),
        ];
        let paragraph = Paragraph::new(Text::from(lines));
        frame.render_widget(paragraph, inner);
        return;
    }

    let mut items = Vec::new();
    for remote in &repo.remotes {
        items.push(ListItem::new(vec![
            Line::from(Span::styled(&remote.name, theme::text_style())),
            Line::from(Span::styled(&remote.fetch_url, theme::muted_text_style())),
        ]));
    }

    let list = List::new(items)
        .style(theme::text_style())
        .highlight_style(theme::selected_list_style())
        .highlight_symbol("▸ ");

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_remote));
    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn render_remote_details(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::pane_block("Details");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(repo) = state.selected_repo_ref() else {
        return;
    };

    let Some(remote) = repo.remotes.get(state.selected_remote) else {
        return;
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Name:      ", theme::muted_text_style()),
            Span::styled(&remote.name, theme::accent_text_style()),
        ]),
        Line::default(),
        Line::from(vec![
            Span::styled("Fetch URL: ", theme::muted_text_style()),
            Span::styled(&remote.fetch_url, theme::text_style()),
        ]),
        Line::default(),
        Line::from(vec![
            Span::styled("Push URL:  ", theme::muted_text_style()),
            Span::styled(&remote.push_url, theme::text_style()),
        ]),
    ];

    let paragraph = Paragraph::new(Text::from(lines)).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}
