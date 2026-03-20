use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;
use crate::domain::pull_request::{CheckConclusion, CheckStatus, PrState};
use crate::ui::layout;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let (list_area, detail_area) = layout::split_pr_view(area);
    render_pr_list(frame, list_area, state);
    render_pr_detail(frame, detail_area, state);
}

fn render_pr_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::pane_block("Pull Requests");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(repo) = state.selected_repo_ref() else {
        let empty = Paragraph::new("No repository selected.").style(theme::muted_text_style());
        frame.render_widget(empty, inner);
        return;
    };

    if repo.pull_requests.is_empty() {
        let lines = vec![
            Line::from(Span::styled(
                "No open pull requests.",
                theme::muted_text_style(),
            )),
            Line::default(),
            Line::from(vec![
                Span::styled("Press ", theme::muted_text_style()),
                Span::styled("r", theme::accent_text_style()),
                Span::styled(" to refresh", theme::muted_text_style()),
            ]),
        ];
        let paragraph = Paragraph::new(Text::from(lines));
        frame.render_widget(paragraph, inner);
        return;
    }

    let mut items = Vec::new();
    for pr in &repo.pull_requests {
        let status_icon = match pr.state {
            PrState::Open => Span::styled("● ", theme::success_text_style()),
            PrState::Merged => Span::styled("● ", theme::accent_text_style()),
            PrState::Closed => Span::styled("● ", theme::error_text_style()),
        };
        items.push(ListItem::new(Line::from(vec![
            status_icon,
            Span::styled(format!("#{} ", pr.number), theme::muted_text_style()),
            Span::styled(&pr.title, theme::text_style()),
        ])));
    }

    let list = List::new(items)
        .style(theme::text_style())
        .highlight_style(theme::selected_list_style())
        .highlight_symbol("▸ ");

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected_pr));
    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn render_pr_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::pane_block("Details");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(repo) = state.selected_repo_ref() else {
        return;
    };

    let Some(pr) = repo.pull_requests.get(state.selected_pr) else {
        return;
    };

    let state_style = match pr.state {
        PrState::Open => theme::success_text_style(),
        PrState::Merged => theme::accent_text_style(),
        PrState::Closed => theme::error_text_style(),
    };
    let state_label = match pr.state {
        PrState::Open => "Open",
        PrState::Merged => "Merged",
        PrState::Closed => "Closed",
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Title:   ", theme::muted_text_style()),
            Span::styled(
                &pr.title,
                theme::text_style().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::default(),
        Line::from(vec![
            Span::styled("Number:  ", theme::muted_text_style()),
            Span::styled(format!("#{}", pr.number), theme::accent_text_style()),
        ]),
        Line::from(vec![
            Span::styled("Status:  ", theme::muted_text_style()),
            Span::styled(state_label, state_style),
        ]),
        Line::from(vec![
            Span::styled("Branch:  ", theme::muted_text_style()),
            Span::styled(&pr.head_branch, theme::text_style()),
        ]),
        Line::default(),
        Line::from(vec![
            Span::styled("URL:     ", theme::muted_text_style()),
            Span::styled(&pr.url, theme::text_style()),
        ]),
    ];

    // Checks section
    if !state.pr_checks_cache.is_empty() {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            "Checks",
            theme::accent_text_style(),
        )));

        for check in &state.pr_checks_cache {
            let (icon, style) = match check.status {
                CheckStatus::Queued => ("○ ", theme::muted_text_style()),
                CheckStatus::InProgress => ("● ", theme::warning_text_style()),
                CheckStatus::Completed => match check.conclusion.as_ref() {
                    Some(CheckConclusion::Success) => ("✓ ", theme::success_text_style()),
                    Some(CheckConclusion::Failure) => ("✗ ", theme::error_text_style()),
                    Some(CheckConclusion::Cancelled) => ("⊘ ", theme::muted_text_style()),
                    Some(CheckConclusion::Skipped) => ("⊘ ", theme::muted_text_style()),
                    Some(CheckConclusion::TimedOut) => ("✗ ", theme::error_text_style()),
                    None => ("○ ", theme::muted_text_style()),
                },
            };
            lines.push(Line::from(vec![
                Span::styled(icon, style),
                Span::styled(&check.name, theme::text_style()),
            ]));
        }
    }

    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("Enter ", theme::accent_text_style()),
        Span::styled("open in browser", theme::muted_text_style()),
        Span::styled("  r ", theme::accent_text_style()),
        Span::styled("refresh", theme::muted_text_style()),
    ]));

    let paragraph = Paragraph::new(Text::from(lines))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .scroll((state.pr_detail_scroll, 0));
    frame.render_widget(paragraph, inner);
}
