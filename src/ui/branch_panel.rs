use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render_create_modal(frame: &mut Frame, area: Rect, state: &AppState) {
    theme::render_backdrop(frame, area);
    let modal = centered_rect(50, 30, area);
    frame.render_widget(Clear, modal);

    let block = theme::modal_elevated_block("New Branch");
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // description
            Constraint::Length(1), // gap
            Constraint::Length(3), // input field
            Constraint::Length(1), // gap
            Constraint::Length(1), // branching from
            Constraint::Min(0),    // spacer
            Constraint::Length(1), // shortcut bar
        ])
        .split(inner);

    // Description
    let desc = Line::from(Span::styled(
        "Enter a name for the new branch.",
        theme::modal_muted_style(),
    ));
    frame.render_widget(Paragraph::new(desc), layout[0]);

    // Input field
    let input_label = if state.ai_branch_loading() {
        "Branch name (generating with AI...)"
    } else {
        "Branch name"
    };
    let input_block = theme::input_block_focused(input_label);
    let input_inner = input_block.inner(layout[2]);
    frame.render_widget(input_block, layout[2]);

    let text_style = Style::default()
        .fg(Color::Rgb(226, 232, 240))
        .bg(Color::Rgb(15, 23, 42));
    let cursor_style = Style::default()
        .fg(Color::Rgb(15, 23, 42))
        .bg(Color::Rgb(34, 211, 238));
    let placeholder_style = Style::default()
        .fg(Color::Rgb(71, 85, 105))
        .bg(Color::Rgb(15, 23, 42));

    let input_display = if state.ai_branch_loading() {
        Line::from(Span::styled(
            "Generating with AI...",
            Style::default()
                .fg(Color::Rgb(34, 211, 238))
                .bg(Color::Rgb(15, 23, 42)),
        ))
    } else if state.branch_name_input.is_empty() {
        Line::from(vec![
            Span::styled(" ", cursor_style),
            Span::styled("feature/my-branch...", placeholder_style),
        ])
    } else {
        let (before, after) = state.branch_name_input.split_at_cursor();
        let mut spans = Vec::new();
        if !before.is_empty() {
            spans.push(Span::styled(before.to_string(), text_style));
        }
        if let Some(ch) = after.chars().next() {
            spans.push(Span::styled(ch.to_string(), cursor_style));
            let rest = &after[ch.len_utf8()..];
            if !rest.is_empty() {
                spans.push(Span::styled(rest.to_string(), text_style));
            }
        } else {
            spans.push(Span::styled(" ", cursor_style));
        }
        Line::from(spans)
    };
    frame.render_widget(Paragraph::new(input_display), input_inner);

    // Branching from
    let from_branch = state
        .selected_repo_ref()
        .and_then(|r| r.current_branch.as_deref())
        .unwrap_or("unknown");
    let from_line = Line::from(vec![
        Span::styled("From: ", theme::modal_muted_style()),
        Span::styled(from_branch.to_string(), theme::modal_accent_style()),
    ]);
    frame.render_widget(Paragraph::new(from_line), layout[4]);

    // Shortcut bar
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
        Span::styled("create", theme::modal_muted_style()),
    ];
    shortcut_spans.extend(ai_shortcut);
    shortcut_spans.extend(vec![
        Span::styled("  esc ", theme::modal_accent_style()),
        Span::styled("cancel", theme::modal_muted_style()),
    ]);
    let shortcuts = Line::from(shortcut_spans);
    frame.render_widget(Paragraph::new(shortcuts), layout[6]);
}
