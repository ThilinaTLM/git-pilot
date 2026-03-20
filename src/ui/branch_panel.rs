use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout::centered_rect;

pub fn render_summary(frame: &mut Frame, area: Rect, state: &AppState) {
    let content = if let Some(repo) = state.selected_repo_ref() {
        vec![
            Line::from(format!("Path: {}", repo.summary.relative_path.display())),
            Line::from(format!(
                "Branch: {}",
                repo.current_branch.as_deref().unwrap_or("(detached)")
            )),
            Line::from(format!("Branches: {}", repo.branches.len())),
            Line::from(format!("Files: {}", repo.status_files.len())),
        ]
    } else {
        vec![Line::from("No repository selected.")]
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().title("Repository").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

pub fn render_switch_modal(frame: &mut Frame, area: Rect, state: &AppState) {
    let modal = centered_rect(50, 60, area);
    frame.render_widget(Clear, modal);

    let items = state
        .selected_repo_ref()
        .map(|repo| {
            repo.branches
                .iter()
                .map(|branch| ListItem::new(branch.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut list_state = ListState::default();
    if !items.is_empty() {
        list_state.select(Some(state.selected_branch.min(items.len() - 1)));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title("Switch Branch")
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().bg(Color::Green).fg(Color::Black))
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, modal, &mut list_state);
}

pub fn render_create_modal(frame: &mut Frame, area: Rect, state: &AppState) {
    let modal = centered_rect(50, 25, area);
    frame.render_widget(Clear, modal);
    let paragraph = Paragraph::new(state.branch_name_input.as_str()).block(
        Block::default()
            .title("Create Branch (Enter to confirm, Esc to cancel)")
            .borders(Borders::ALL),
    );
    frame.render_widget(paragraph, modal);
}
