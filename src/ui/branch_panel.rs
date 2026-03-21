use ratatui::prelude::*;
use ratatui::widgets::{Clear, List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let content = if let Some(repo) = state.selected_repo_ref() {
        let branch = repo.current_branch.as_deref().unwrap_or("(detached)");
        let branch_count = repo.branches.len();
        let file_count = repo.status_files.len();
        Line::from(vec![
            Span::styled(format!("  {branch}"), theme::accent_text_style()),
            Span::styled(" • ", theme::muted_text_style()),
            Span::styled(
                format!(
                    "{branch_count} branch{}",
                    if branch_count == 1 { "" } else { "es" }
                ),
                theme::text_style(),
            ),
            Span::styled(" • ", theme::muted_text_style()),
            Span::styled(
                format!(
                    "{file_count} changed file{}",
                    if file_count == 1 { "" } else { "s" }
                ),
                theme::text_style(),
            ),
        ])
    } else {
        Line::from(Span::styled(
            "  No repository selected.",
            theme::muted_text_style(),
        ))
    };

    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, area);
}

pub fn render_switch_modal(frame: &mut Frame, area: Rect, state: &AppState) {
    let modal = centered_rect(50, 60, area);
    frame.render_widget(Clear, modal);

    let total_count = state
        .selected_repo_ref()
        .map(|r| r.branches.len())
        .unwrap_or(0);
    let filtered_count = state.filtered_branches.len();
    let title = if state.branch_filter.is_empty() {
        format!("Switch Branch ({total_count})")
    } else {
        format!("Switch Branch ({filtered_count} of {total_count})")
    };
    let block = theme::modal_elevated_block(title);
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // current branch
            Constraint::Length(1), // gap
            Constraint::Length(3), // filter input
            Constraint::Length(1), // gap
            Constraint::Min(3),    // branch list
            Constraint::Length(1), // gap
            Constraint::Length(1), // shortcut bar
        ])
        .split(inner);

    // Current branch
    let current = state
        .selected_repo_ref()
        .and_then(|r| r.current_branch.as_deref())
        .unwrap_or("none");
    let desc = Line::from(vec![
        Span::styled("Current: ", theme::modal_muted_style()),
        Span::styled(current.to_string(), theme::modal_accent_style()),
    ]);
    frame.render_widget(Paragraph::new(desc), layout[0]);

    // Filter input
    let filter_block = theme::input_block_focused("Filter");
    let filter_inner = filter_block.inner(layout[2]);
    frame.render_widget(filter_block, layout[2]);

    let filter_display = if state.branch_filter.is_empty() {
        Line::from(Span::styled(
            "type to filter...",
            Style::default()
                .fg(Color::Rgb(71, 85, 105))
                .bg(Color::Rgb(15, 23, 42)),
        ))
    } else {
        Line::from(vec![
            Span::styled(
                state.branch_filter.clone(),
                Style::default()
                    .fg(Color::Rgb(226, 232, 240))
                    .bg(Color::Rgb(15, 23, 42)),
            ),
            Span::styled(
                "_",
                Style::default()
                    .fg(Color::Rgb(34, 211, 238))
                    .bg(Color::Rgb(15, 23, 42)),
            ),
        ])
    };
    frame.render_widget(Paragraph::new(filter_display), filter_inner);

    // Branch list (filtered)
    let items = state
        .selected_repo_ref()
        .map(|repo| {
            state
                .filtered_branches
                .iter()
                .map(|&real_idx| {
                    let branch = &repo.branches[real_idx];
                    let is_current = repo
                        .current_branch
                        .as_deref()
                        .is_some_and(|current| current == branch);
                    let prefix = if is_current { " ● " } else { "   " };
                    let style = if is_current {
                        theme::modal_accent_style()
                    } else {
                        theme::modal_text_style()
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(branch.clone(), style),
                    ]))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut list_state = ListState::default();
    if !items.is_empty() {
        list_state.select(Some(state.selected_branch.min(items.len() - 1)));
    }

    let list = List::new(items)
        .style(theme::modal_text_style())
        .highlight_style(
            Style::default()
                .fg(Color::Rgb(226, 232, 240))
                .bg(Color::Rgb(22, 78, 99))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ▸ ");

    frame.render_stateful_widget(list, layout[4], &mut list_state);

    // Shortcut bar
    let shortcuts = Line::from(vec![
        Span::styled("↑/↓ ", theme::modal_accent_style()),
        Span::styled("navigate", theme::modal_muted_style()),
        Span::styled("  Enter ", theme::modal_accent_style()),
        Span::styled("switch", theme::modal_muted_style()),
        Span::styled("  Esc ", theme::modal_accent_style()),
        Span::styled("cancel", theme::modal_muted_style()),
    ]);
    frame.render_widget(Paragraph::new(shortcuts), layout[6]);
}

pub fn render_create_modal(frame: &mut Frame, area: Rect, state: &AppState) {
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

    let input_display = if state.ai_branch_loading() {
        Line::from(Span::styled(
            "Generating with AI...",
            Style::default()
                .fg(Color::Rgb(34, 211, 238))
                .bg(Color::Rgb(15, 23, 42)),
        ))
    } else if state.branch_name_input.is_empty() {
        Line::from(Span::styled(
            "feature/my-branch...",
            Style::default()
                .fg(Color::Rgb(71, 85, 105))
                .bg(Color::Rgb(15, 23, 42)),
        ))
    } else {
        Line::from(vec![
            Span::styled(
                state.branch_name_input.clone(),
                Style::default()
                    .fg(Color::Rgb(226, 232, 240))
                    .bg(Color::Rgb(15, 23, 42)),
            ),
            Span::styled(
                "_",
                Style::default()
                    .fg(Color::Rgb(34, 211, 238))
                    .bg(Color::Rgb(15, 23, 42)),
            ),
        ])
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
        Span::styled("create", theme::modal_muted_style()),
    ];
    shortcut_spans.extend(ai_shortcut);
    shortcut_spans.extend(vec![
        Span::styled("  Esc ", theme::modal_accent_style()),
        Span::styled("cancel", theme::modal_muted_style()),
    ]);
    let shortcuts = Line::from(shortcut_spans);
    frame.render_widget(Paragraph::new(shortcuts), layout[6]);
}
