use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

use crate::app::state::{AppState, View};
use crate::app::suggestions::compute_suggestions;
use crate::shared::shortcuts;
use crate::ui::layout::centered_rect;
use crate::ui::summary_line;
use crate::ui::theme;

pub fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let left_spans = match state.message.as_ref().map(|message| &message.level) {
        Some(crate::app::state::MessageLevel::Error) => {
            let text = state
                .message
                .as_ref()
                .map(|message| message.text.as_str())
                .unwrap_or("");
            vec![
                Span::styled(
                    "Error  ",
                    theme::error_text_style().add_modifier(Modifier::BOLD),
                ),
                Span::styled(text, theme::error_text_style()),
            ]
        }
        _ => summary_line::build_spans(state),
    };

    let right_spans = render_suggestion_spans(state);

    // Calculate widths to right-align hints
    let left_width: usize = left_spans.iter().map(|s| s.width()).sum();
    let right_width: usize = right_spans.iter().map(|s| s.width()).sum();
    let area_width = area.width as usize;

    let mut spans = left_spans;

    let gap = area_width.saturating_sub(left_width + right_width);
    if gap > 0 {
        spans.push(Span::raw(" ".repeat(gap)));
    }
    spans.extend(right_spans);

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
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
    theme::render_backdrop(frame, area);
    let modal = centered_rect(74, 78, area);
    frame.render_widget(Clear, modal);
    let block = theme::modal_elevated_block("Keyboard Shortcuts");
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let paragraph = Paragraph::new(help_text(state))
        .style(theme::modal_text_style())
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

fn help_text(state: &AppState) -> Text<'static> {
    let mut lines = Vec::new();

    lines.push(Line::from(vec![Span::styled(
        "Press ? again to close this view.",
        theme::modal_accent_style(),
    )]));
    lines.push(Line::default());

    // Global shortcuts
    render_section(&mut lines, "Global", shortcuts::GLOBAL_SHORTCUTS);

    // View-specific shortcuts
    match state.active_view {
        View::Changes => {
            render_section(&mut lines, "Changes View", shortcuts::CHANGES_SHORTCUTS);
        }
        View::Pr => {
            render_section(&mut lines, "PR View", shortcuts::PR_SHORTCUTS);
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
        theme::modal_accent_style(),
    )]));

    for entry in entries {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<18}", entry.keys),
                Style::default()
                    .fg(Color::Rgb(74, 222, 128))
                    .bg(theme::MODAL_BG),
            ),
            Span::styled(entry.description, theme::modal_text_style()),
        ]));
    }

    lines.push(Line::default());
}
