use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

use crate::app::state::{AppState, View};
use crate::app::suggestions::compute_suggestions;
use crate::shared::shortcuts;
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let content = match state.message.as_ref().map(|message| &message.level) {
        Some(crate::app::state::MessageLevel::Error) => {
            let text = state
                .message
                .as_ref()
                .map(|message| message.text.as_str())
                .unwrap_or("");
            Line::from(vec![
                Span::styled(
                    "Error  ",
                    theme::error_text_style().add_modifier(Modifier::BOLD),
                ),
                Span::styled(text, theme::error_text_style()),
            ])
        }
        Some(crate::app::state::MessageLevel::Info) => {
            let text = state
                .message
                .as_ref()
                .map(|message| message.text.as_str())
                .unwrap_or("");
            let suggestion_line = render_suggestion_spans(state);
            let mut spans = vec![
                Span::styled("Info  ", theme::accent_text_style()),
                Span::styled(text, theme::text_style()),
                Span::styled("  •  ", theme::muted_text_style()),
            ];
            spans.extend(suggestion_line);
            Line::from(spans)
        }
        None => {
            let mut spans = vec![Span::styled(
                "Hints  ",
                theme::muted_text_style().add_modifier(Modifier::BOLD),
            )];
            spans.extend(render_suggestion_spans(state));
            Line::from(spans)
        }
    };

    let block = theme::footer_block();
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let paragraph = Paragraph::new(content).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

fn render_suggestion_spans(state: &AppState) -> Vec<Span<'static>> {
    let suggestions = compute_suggestions(state);
    let mut spans = Vec::new();

    for (i, suggestion) in suggestions.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" | ", theme::muted_text_style()));
        }
        spans.push(Span::styled(
            suggestion.key_hint.to_string(),
            theme::accent_text_style(),
        ));
        spans.push(Span::styled(
            format!(" {}", suggestion.label),
            theme::muted_text_style(),
        ));
    }

    spans
}

pub fn render_overlay(frame: &mut Frame, area: Rect, state: &AppState) {
    let modal = centered_rect(74, 78, area);
    frame.render_widget(Clear, modal);
    let block = theme::modal_block("Keyboard Shortcuts");
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let paragraph = Paragraph::new(help_text(state))
        .style(theme::text_style())
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

fn help_text(state: &AppState) -> Text<'static> {
    let mut lines = Vec::new();

    lines.push(Line::from(vec![Span::styled(
        "Press ? again to close this view.",
        theme::warning_text_style().add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::default());

    // Global shortcuts
    render_section(&mut lines, "Global", shortcuts::GLOBAL_SHORTCUTS);

    // View-specific shortcuts
    match state.active_view {
        View::Changes => {
            render_section(&mut lines, "Changes View", shortcuts::CHANGES_SHORTCUTS);
        }
        View::Branches => {
            render_section(&mut lines, "Branches View", shortcuts::BRANCHES_SHORTCUTS);
        }
        View::Log => {
            render_section(&mut lines, "Log View", shortcuts::LOG_SHORTCUTS);
        }
        View::Remotes => {
            render_section(&mut lines, "Remotes View", shortcuts::REMOTES_SHORTCUTS);
        }
    }

    // Modal shortcuts
    render_section(&mut lines, "Modals", shortcuts::MODAL_SHORTCUTS);

    Text::from(lines)
}

fn render_section(
    lines: &mut Vec<Line<'static>>,
    title: &'static str,
    entries: &[shortcuts::ShortcutEntry],
) {
    lines.push(Line::from(vec![Span::styled(
        title,
        theme::accent_text_style(),
    )]));

    for entry in entries {
        lines.push(Line::from(vec![
            Span::styled(format!("{:<18}", entry.keys), theme::success_text_style()),
            Span::styled(entry.description, theme::text_style()),
        ]));
    }

    lines.push(Line::default());
}
