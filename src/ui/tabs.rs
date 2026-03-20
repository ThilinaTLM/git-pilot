use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Tabs};

use crate::app::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let titles = if state.repos.is_empty() {
        vec![Line::from("No repos")]
    } else {
        state
            .repos
            .iter()
            .map(|repo| Line::from(repo.summary.name.clone()))
            .collect::<Vec<_>>()
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().title("Repositories").borders(Borders::ALL))
        .select(state.selected_repo.min(state.repos.len().saturating_sub(1)))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider("│");

    frame.render_widget(tabs, area);
}
