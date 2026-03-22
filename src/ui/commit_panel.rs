use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    theme::render_backdrop(frame, area);
    let modal = centered_rect(60, 55, area);
    frame.render_widget(Clear, modal);

    let title = if state.amend_mode {
        "Amend Commit"
    } else {
        "Commit"
    };
    let block = theme::modal_elevated_block(title);
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    // Layout: description, subject input, separator, body input, shortcut bar
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // description
            Constraint::Length(1), // gap
            Constraint::Length(3), // subject input field
            Constraint::Length(1), // gap
            Constraint::Min(4),    // body input field
            Constraint::Length(1), // gap
            Constraint::Length(1), // shortcut bar
        ])
        .split(inner);

    // Description
    let staged_count = state
        .selected_repo_ref()
        .map(|r| r.status_files.iter().filter(|f| f.staged).count())
        .unwrap_or(0);
    let desc = if state.amend_mode {
        Line::from(vec![
            Span::styled("Amending last commit", theme::modal_accent_style()),
            if staged_count > 0 {
                Span::styled(
                    format!(
                        " with {staged_count} staged file{}",
                        if staged_count == 1 { "" } else { "s" }
                    ),
                    theme::modal_muted_style(),
                )
            } else {
                Span::styled(" (message only)", theme::modal_muted_style())
            },
        ])
    } else {
        Line::from(vec![
            Span::styled("Committing ", theme::modal_muted_style()),
            Span::styled(
                format!(
                    "{staged_count} staged file{}",
                    if staged_count == 1 { "" } else { "s" }
                ),
                theme::modal_accent_style(),
            ),
        ])
    };
    frame.render_widget(Paragraph::new(desc), layout[0]);

    // Parse subject and body from input
    let content = state.commit_message_input.content();
    let cursor_pos = state.commit_message_input.cursor();
    let (subject, body) = split_subject_body(content);

    // Determine if cursor is in subject or body
    let newline_pos = content.find('\n');
    let cursor_in_subject = newline_pos.is_none() || cursor_pos <= newline_pos.unwrap();

    let text_style = Style::default()
        .fg(Color::Rgb(226, 232, 240))
        .bg(Color::Rgb(15, 23, 42));
    let cursor_style = Style::default()
        .fg(Color::Rgb(15, 23, 42))
        .bg(Color::Rgb(34, 211, 238));
    let placeholder_style = Style::default()
        .fg(Color::Rgb(71, 85, 105))
        .bg(Color::Rgb(15, 23, 42));

    // Subject input field
    let subject_label = if state.ai_loading() {
        "Subject (generating with AI...)"
    } else if subject.is_empty() {
        "Subject (required)"
    } else {
        "Subject"
    };
    let subject_block = if cursor_in_subject {
        theme::input_block_focused(subject_label)
    } else {
        theme::input_block(subject_label)
    };
    let subject_inner = subject_block.inner(layout[2]);
    frame.render_widget(subject_block, layout[2]);

    let subject_display = if state.ai_loading() {
        Line::from(Span::styled(
            "Generating with AI...",
            Style::default()
                .fg(Color::Rgb(34, 211, 238))
                .bg(Color::Rgb(15, 23, 42)),
        ))
    } else if subject.is_empty() && cursor_in_subject {
        Line::from(vec![
            Span::styled(" ", cursor_style),
            Span::styled("Write a short summary of changes...", placeholder_style),
        ])
    } else if subject.is_empty() {
        Line::from(Span::styled(
            "Write a short summary of changes...",
            placeholder_style,
        ))
    } else {
        let char_count = subject.len();
        let count_style = if char_count > 72 {
            theme::error_text_style()
        } else if char_count > 50 {
            theme::warning_text_style()
        } else {
            placeholder_style
        };

        let mut spans = if cursor_in_subject {
            render_with_cursor(subject, cursor_pos, text_style, cursor_style)
        } else {
            vec![Span::styled(subject.to_string(), text_style)]
        };
        spans.push(Span::styled("  ", count_style));
        spans.push(Span::styled(format!("{char_count}/72"), count_style));
        Line::from(spans)
    };
    frame.render_widget(Paragraph::new(subject_display), subject_inner);

    // Body input field
    let body_block = if !cursor_in_subject {
        theme::input_block_focused("Body (optional)")
    } else {
        theme::input_block("Body (optional)")
    };
    let body_inner = body_block.inner(layout[4]);
    frame.render_widget(body_block, layout[4]);

    let body_text = body.unwrap_or("");
    let body_display = if body_text.is_empty() && body.is_some() {
        if !cursor_in_subject {
            Text::from(Line::from(vec![
                Span::styled(" ", cursor_style),
                Span::styled("Add a detailed description...", placeholder_style),
            ]))
        } else {
            Text::from(Line::from(Span::styled(
                "Add a detailed description...",
                placeholder_style,
            )))
        }
    } else if body_text.is_empty() {
        Text::from(Line::from(Span::styled(
            "ctrl+n to add body",
            placeholder_style,
        )))
    } else {
        let body_cursor = if !cursor_in_subject {
            cursor_pos - newline_pos.unwrap() - 1
        } else {
            usize::MAX
        };
        let lines = render_multiline_with_cursor(body_text, body_cursor, text_style, cursor_style);
        Text::from(lines)
    };
    frame.render_widget(
        Paragraph::new(body_display).wrap(ratatui::widgets::Wrap { trim: false }),
        body_inner,
    );

    // Shortcut bar at bottom
    let ai_shortcut = if state.copilot_authenticated {
        vec![
            Span::styled("  ctrl+g ", theme::modal_accent_style()),
            Span::styled("generate", theme::modal_muted_style()),
        ]
    } else {
        vec![
            Span::styled("  ctrl+l ", theme::modal_accent_style()),
            Span::styled("login", theme::modal_muted_style()),
        ]
    };
    let mut shortcut_spans = vec![
        Span::styled("enter ", theme::modal_accent_style()),
        Span::styled("commit", theme::modal_muted_style()),
        Span::styled("  ctrl+n ", theme::modal_accent_style()),
        Span::styled("newline", theme::modal_muted_style()),
    ];
    shortcut_spans.extend(ai_shortcut);
    shortcut_spans.extend(vec![
        Span::styled("  esc ", theme::modal_accent_style()),
        Span::styled("cancel", theme::modal_muted_style()),
    ]);
    let shortcuts = Line::from(shortcut_spans);
    frame.render_widget(Paragraph::new(shortcuts), layout[6]);
}

/// Split input into subject (first line) and body (everything after first newline).
/// Returns (subject, Some(body)) if there's a newline, or (subject, None) if not.
fn split_subject_body(input: &str) -> (&str, Option<&str>) {
    if let Some(pos) = input.find('\n') {
        let subject = &input[..pos];
        let body = &input[pos + 1..];
        (subject, Some(body))
    } else {
        (input, None)
    }
}

/// Render a single-line text with a cursor block at the given byte position.
fn render_with_cursor<'a>(
    text: &str,
    cursor: usize,
    text_style: Style,
    cursor_style: Style,
) -> Vec<Span<'a>> {
    let cursor = cursor.min(text.len());
    let before = &text[..cursor];
    let after = &text[cursor..];

    let mut spans = Vec::new();
    if !before.is_empty() {
        spans.push(Span::styled(before.to_string(), text_style));
    }

    // Cursor character (show the char under cursor, or a space if at end)
    if let Some(ch) = after.chars().next() {
        spans.push(Span::styled(ch.to_string(), cursor_style));
        let rest = &after[ch.len_utf8()..];
        if !rest.is_empty() {
            spans.push(Span::styled(rest.to_string(), text_style));
        }
    } else {
        spans.push(Span::styled(" ", cursor_style));
    }

    spans
}

/// Render multi-line text with a cursor at a byte position within the text.
fn render_multiline_with_cursor<'a>(
    text: &str,
    cursor: usize,
    text_style: Style,
    cursor_style: Style,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let mut offset = 0;

    for line_text in text.split('\n') {
        let line_end = offset + line_text.len();
        if cursor >= offset && cursor <= line_end && cursor != usize::MAX {
            let local_cursor = cursor - offset;
            let spans = render_with_cursor(line_text, local_cursor, text_style, cursor_style);
            lines.push(Line::from(spans));
        } else {
            lines.push(Line::from(Span::styled(line_text.to_string(), text_style)));
        }
        offset = line_end + 1; // +1 for the '\n'
    }

    // Handle cursor at very end after trailing newline
    if cursor == text.len() && text.ends_with('\n') {
        lines.push(Line::from(Span::styled(" ", cursor_style)));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(" ", cursor_style)));
    }

    lines
}
