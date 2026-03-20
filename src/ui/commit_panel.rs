use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let modal = centered_rect(60, 55, area);
    frame.render_widget(Clear, modal);

    let block = theme::modal_elevated_block("Commit");
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
    let desc = Line::from(vec![
        Span::styled("Committing ", theme::modal_muted_style()),
        Span::styled(
            format!(
                "{staged_count} staged file{}",
                if staged_count == 1 { "" } else { "s" }
            ),
            theme::modal_accent_style(),
        ),
    ]);
    frame.render_widget(Paragraph::new(desc), layout[0]);

    // Parse subject and body from input
    let input = &state.commit_message_input;
    let (subject, body) = split_subject_body(input);

    // Subject input field
    let subject_label = if state.ai_loading {
        "Subject (generating with AI...)"
    } else if subject.is_empty() {
        "Subject (required)"
    } else {
        "Subject"
    };
    let subject_block = if body.is_none() {
        theme::input_block_focused(subject_label)
    } else {
        theme::input_block(subject_label)
    };
    let subject_inner = subject_block.inner(layout[2]);
    frame.render_widget(subject_block, layout[2]);

    let subject_display = if state.ai_loading {
        Line::from(Span::styled(
            "Generating with AI...",
            Style::default()
                .fg(Color::Rgb(34, 211, 238))
                .bg(Color::Rgb(15, 23, 42)),
        ))
    } else if subject.is_empty() {
        Line::from(Span::styled(
            "Write a short summary of changes...",
            Style::default()
                .fg(Color::Rgb(71, 85, 105))
                .bg(Color::Rgb(15, 23, 42)),
        ))
    } else {
        let char_count = subject.len();
        let count_style = if char_count > 72 {
            theme::error_text_style()
        } else if char_count > 50 {
            theme::warning_text_style()
        } else {
            Style::default()
                .fg(Color::Rgb(71, 85, 105))
                .bg(Color::Rgb(15, 23, 42))
        };
        Line::from(vec![
            Span::styled(
                subject.to_string(),
                Style::default()
                    .fg(Color::Rgb(226, 232, 240))
                    .bg(Color::Rgb(15, 23, 42)),
            ),
            Span::styled(
                if body.is_none() { "_" } else { "" },
                Style::default()
                    .fg(Color::Rgb(34, 211, 238))
                    .bg(Color::Rgb(15, 23, 42)),
            ),
            Span::styled("  ", count_style),
            Span::styled(format!("{char_count}/72"), count_style),
        ])
    };
    frame.render_widget(Paragraph::new(subject_display), subject_inner);

    // Body input field
    let body_block = if body.is_some() {
        theme::input_block_focused("Body (optional)")
    } else {
        theme::input_block("Body (optional)")
    };
    let body_inner = body_block.inner(layout[4]);
    frame.render_widget(body_block, layout[4]);

    let body_text = body.unwrap_or("");
    let body_display = if body_text.is_empty() && body.is_some() {
        Text::from(Line::from(Span::styled(
            "Add a detailed description..._",
            Style::default()
                .fg(Color::Rgb(71, 85, 105))
                .bg(Color::Rgb(15, 23, 42)),
        )))
    } else if body_text.is_empty() {
        Text::from(Line::from(Span::styled(
            "Ctrl+n to add body",
            Style::default()
                .fg(Color::Rgb(71, 85, 105))
                .bg(Color::Rgb(15, 23, 42)),
        )))
    } else {
        let mut lines: Vec<Line> = body_text
            .lines()
            .map(|l| {
                Line::from(Span::styled(
                    l.to_string(),
                    Style::default()
                        .fg(Color::Rgb(226, 232, 240))
                        .bg(Color::Rgb(15, 23, 42)),
                ))
            })
            .collect();
        // Cursor on the last line
        if let Some(last) = lines.last_mut() {
            last.spans.push(Span::styled(
                "_",
                Style::default()
                    .fg(Color::Rgb(34, 211, 238))
                    .bg(Color::Rgb(15, 23, 42)),
            ));
        }
        Text::from(lines)
    };
    frame.render_widget(
        Paragraph::new(body_display).wrap(ratatui::widgets::Wrap { trim: false }),
        body_inner,
    );

    // Shortcut bar at bottom
    let ai_shortcut = if state.copilot_authenticated {
        vec![
            Span::styled("  Ctrl+g ", theme::modal_accent_style()),
            Span::styled("generate", theme::modal_muted_style()),
        ]
    } else {
        vec![
            Span::styled("  Ctrl+l ", theme::modal_accent_style()),
            Span::styled("login", theme::modal_muted_style()),
        ]
    };
    let mut shortcut_spans = vec![
        Span::styled("Enter ", theme::modal_accent_style()),
        Span::styled("commit", theme::modal_muted_style()),
        Span::styled("  Ctrl+n ", theme::modal_accent_style()),
        Span::styled("newline", theme::modal_muted_style()),
    ];
    shortcut_spans.extend(ai_shortcut);
    shortcut_spans.extend(vec![
        Span::styled("  Esc ", theme::modal_accent_style()),
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
