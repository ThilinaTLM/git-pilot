use ratatui::prelude::*;
use ratatui::widgets::{Clear, List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let modal = centered_rect(60, 60, area);
    frame.render_widget(Clear, modal);
    let block = theme::modal_block("Settings");
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(inner);

    render_settings_list(frame, halves[0], state);
    render_settings_detail(frame, halves[1], state);
}

fn render_settings_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut items = Vec::new();

    items.push(ListItem::new(Line::from(Span::styled(
        "Preferences",
        theme::section_header_style(),
    ))));

    let auto_fetch_label = if state.settings.auto_fetch_enabled {
        "Auto-fetch: ON"
    } else {
        "Auto-fetch: OFF"
    };
    items.push(ListItem::new(Line::from(Span::styled(
        format!("  {auto_fetch_label}"),
        theme::text_style(),
    ))));

    items.push(ListItem::new(Line::from(Span::styled(
        format!("  Interval: {}s", state.settings.auto_fetch_interval_secs),
        theme::text_style(),
    ))));

    items.push(ListItem::new(Line::from(Span::styled(
        "",
        theme::text_style(),
    ))));
    items.push(ListItem::new(Line::from(Span::styled(
        "Remotes",
        theme::section_header_style(),
    ))));

    if let Some(repo) = state.selected_repo_ref() {
        if repo.remotes.is_empty() {
            items.push(ListItem::new(Line::from(Span::styled(
                "  No remotes configured",
                theme::muted_text_style(),
            ))));
        } else {
            for remote in &repo.remotes {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("  ", theme::text_style()),
                    Span::styled(&remote.name, theme::text_style()),
                    Span::styled(" ", theme::text_style()),
                    Span::styled(&remote.fetch_url, theme::muted_text_style()),
                ])));
            }
        }
    }

    let visual_row = state.selected_settings_item + 1;

    let list = List::new(items)
        .style(theme::text_style())
        .highlight_style(theme::selected_list_style())
        .highlight_symbol("▸ ");

    let mut list_state = ListState::default();
    list_state.select(Some(visual_row));
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_settings_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut lines = Vec::new();

    match state.selected_settings_item {
        0 => {
            lines.push(Line::from(Span::styled(
                "Auto-fetch",
                theme::accent_text_style(),
            )));
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                "Automatically fetch from remote at a regular interval.",
                theme::text_style(),
            )));
            lines.push(Line::default());
            let status = if state.settings.auto_fetch_enabled {
                "Enabled"
            } else {
                "Disabled"
            };
            lines.push(Line::from(vec![
                Span::styled("Status: ", theme::muted_text_style()),
                Span::styled(
                    status,
                    if state.settings.auto_fetch_enabled {
                        theme::success_text_style()
                    } else {
                        theme::muted_text_style()
                    },
                ),
            ]));
            lines.push(Line::default());
            lines.push(Line::from(vec![
                Span::styled("Space/Enter ", theme::accent_text_style()),
                Span::styled("toggle", theme::muted_text_style()),
            ]));
        }
        1 => {
            lines.push(Line::from(Span::styled(
                "Auto-fetch Interval",
                theme::accent_text_style(),
            )));
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                "How often to automatically fetch (in seconds).",
                theme::text_style(),
            )));
            lines.push(Line::default());
            lines.push(Line::from(vec![
                Span::styled("Current: ", theme::muted_text_style()),
                Span::styled(
                    format!("{}s", state.settings.auto_fetch_interval_secs),
                    theme::accent_text_style(),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Range:   ", theme::muted_text_style()),
                Span::styled("30s - 600s", theme::text_style()),
            ]));
            lines.push(Line::default());
            lines.push(Line::from(vec![
                Span::styled("+/- ", theme::accent_text_style()),
                Span::styled("adjust by 30s", theme::muted_text_style()),
            ]));
        }
        _ => {}
    }

    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("Esc ", theme::accent_text_style()),
        Span::styled("close", theme::muted_text_style()),
    ]));

    let paragraph = Paragraph::new(Text::from(lines)).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
