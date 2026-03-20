use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let (list_area, detail_area) = layout::split_settings_view(area);
    render_settings_list(frame, list_area, state);
    render_settings_detail(frame, detail_area, state);
}

fn render_settings_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::pane_block("Settings");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut items = Vec::new();

    // Preferences section
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

    // Remotes section
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

    // Map selected_settings_item to visual row:
    // Row 0 = header "Preferences" (not selectable)
    // Row 1 = auto-fetch toggle (item 0)
    // Row 2 = interval (item 1)
    let visual_row = state.selected_settings_item + 1; // offset by header

    let list = List::new(items)
        .style(theme::text_style())
        .highlight_style(theme::selected_list_style())
        .highlight_symbol("▸ ");

    let mut list_state = ListState::default();
    list_state.select(Some(visual_row));
    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn render_settings_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::pane_block("Details");
    let inner = block.inner(area);
    frame.render_widget(block, area);

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

    if let Some(repo) = state.selected_repo_ref()
        && !repo.has_origin_remote
    {
        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::styled("R ", theme::accent_text_style()),
            Span::styled("create GitHub repository", theme::muted_text_style()),
        ]));
    }

    let paragraph = Paragraph::new(Text::from(lines)).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}
