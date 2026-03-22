use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

use crate::app::state::{AppState, CreatePrField};
use crate::ui::commit_panel::{render_multiline_with_cursor, render_with_cursor};
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    theme::render_backdrop(frame, area);
    let modal = centered_rect(65, 60, area);
    frame.render_widget(Clear, modal);

    let block = theme::modal_elevated_block("Create Pull Request");
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let Some(pr_state) = &state.create_pr_state else {
        return;
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // branch info
            Constraint::Length(1), // gap
            Constraint::Length(3), // title input
            Constraint::Length(1), // gap
            Constraint::Min(4),    // body input
            Constraint::Length(1), // gap
            Constraint::Length(1), // draft + shortcuts
        ])
        .split(inner);

    // Branch info line
    let branch_info = Line::from(vec![
        Span::styled(&pr_state.head_branch, theme::modal_accent_style()),
        Span::styled(" → ", theme::modal_muted_style()),
        Span::styled(&pr_state.base_branch, theme::modal_accent_style()),
    ]);
    frame.render_widget(Paragraph::new(branch_info), layout[0]);

    let text_style = Style::default()
        .fg(Color::Rgb(226, 232, 240))
        .bg(Color::Rgb(15, 23, 42));
    let cursor_style = Style::default()
        .fg(Color::Rgb(15, 23, 42))
        .bg(Color::Rgb(34, 211, 238));
    let placeholder_style = Style::default()
        .fg(Color::Rgb(71, 85, 105))
        .bg(Color::Rgb(15, 23, 42));

    let title_focused = pr_state.active_field == CreatePrField::Title;
    let body_focused = pr_state.active_field == CreatePrField::Body;

    // Title input
    let title_label = if state.ai_pr_loading() {
        "Title (generating with AI...)"
    } else if pr_state.title_input.content().is_empty() {
        "Title (required)"
    } else {
        "Title"
    };
    let title_block = if title_focused {
        theme::input_block_focused(title_label)
    } else {
        theme::input_block(title_label)
    };
    let title_inner = title_block.inner(layout[2]);
    frame.render_widget(title_block, layout[2]);

    let title = pr_state.title_input.content();
    let title_cursor = pr_state.title_input.cursor();

    let title_display = if state.ai_pr_loading() {
        Line::from(Span::styled(
            "Generating with AI...",
            Style::default()
                .fg(Color::Rgb(34, 211, 238))
                .bg(Color::Rgb(15, 23, 42)),
        ))
    } else if title.is_empty() && title_focused {
        Line::from(vec![
            Span::styled(" ", cursor_style),
            Span::styled("Short PR title...", placeholder_style),
        ])
    } else if title.is_empty() {
        Line::from(Span::styled("Short PR title...", placeholder_style))
    } else {
        let char_count = title.len();
        let count_style = if char_count > 70 {
            theme::error_text_style()
        } else if char_count > 50 {
            theme::warning_text_style()
        } else {
            placeholder_style
        };

        let mut spans = if title_focused {
            render_with_cursor(title, title_cursor, text_style, cursor_style)
        } else {
            vec![Span::styled(title.to_string(), text_style)]
        };
        spans.push(Span::styled("  ", count_style));
        spans.push(Span::styled(format!("{char_count}/70"), count_style));
        Line::from(spans)
    };
    frame.render_widget(Paragraph::new(title_display), title_inner);

    // Body input
    let body_block = if body_focused {
        theme::input_block_focused("Description (optional)")
    } else {
        theme::input_block("Description (optional)")
    };
    let body_inner = body_block.inner(layout[4]);
    frame.render_widget(body_block, layout[4]);

    let body = pr_state.body_input.content();
    let body_cursor = pr_state.body_input.cursor();

    let body_display = if state.ai_pr_loading() {
        Text::from(Line::from(Span::styled(
            "Generating with AI...",
            Style::default()
                .fg(Color::Rgb(34, 211, 238))
                .bg(Color::Rgb(15, 23, 42)),
        )))
    } else if body.is_empty() && body_focused {
        Text::from(Line::from(vec![
            Span::styled(" ", cursor_style),
            Span::styled("Describe your changes...", placeholder_style),
        ]))
    } else if body.is_empty() {
        Text::from(Line::from(Span::styled(
            "Describe your changes...",
            placeholder_style,
        )))
    } else if body_focused {
        let lines = render_multiline_with_cursor(body, body_cursor, text_style, cursor_style);
        Text::from(lines)
    } else {
        let lines: Vec<Line> = body
            .split('\n')
            .map(|l| Line::from(Span::styled(l.to_string(), text_style)))
            .collect();
        Text::from(lines)
    };
    frame.render_widget(
        Paragraph::new(body_display).wrap(ratatui::widgets::Wrap { trim: false }),
        body_inner,
    );

    // Draft indicator + shortcut bar
    let draft_span = if pr_state.draft {
        Span::styled("[Draft] ", theme::warning_text_style())
    } else {
        Span::styled("[Ready] ", theme::success_text_style())
    };

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
        draft_span,
        Span::styled(" enter ", theme::modal_accent_style()),
        Span::styled("create", theme::modal_muted_style()),
        Span::styled("  tab ", theme::modal_accent_style()),
        Span::styled("switch", theme::modal_muted_style()),
    ];
    shortcut_spans.extend(ai_shortcut);
    shortcut_spans.extend(vec![
        Span::styled("  ctrl+d ", theme::modal_accent_style()),
        Span::styled("draft", theme::modal_muted_style()),
        Span::styled("  esc ", theme::modal_accent_style()),
        Span::styled("cancel", theme::modal_muted_style()),
    ]);
    let shortcuts = Line::from(shortcut_spans);
    frame.render_widget(Paragraph::new(shortcuts), layout[6]);
}
