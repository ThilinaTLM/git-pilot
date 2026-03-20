use ratatui::prelude::*;
use ratatui::widgets::Tabs;

use crate::app::state::AppState;
use crate::ui::theme;

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

    let rule = theme::tabs_rule_block();
    let inner = rule.inner(area);
    frame.render_widget(rule, area);

    let tabs = Tabs::new(titles)
        .style(theme::inactive_tab_style())
        .select(state.selected_repo.min(state.repos.len().saturating_sub(1)))
        .highlight_style(theme::active_tab_style())
        .divider(" ")
        .padding(" ", " ");

    frame.render_widget(tabs, inner);
}
