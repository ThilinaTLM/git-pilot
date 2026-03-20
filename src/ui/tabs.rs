use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::app::state::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut spans: Vec<Span> = Vec::new();

    if state.repos.is_empty() {
        spans.push(Span::styled("No repos", theme::muted_text_style()));
    } else {
        for (i, repo) in state.repos.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled("  ", theme::muted_text_style()));
            }
            let is_selected = i == state.selected_repo.min(state.repos.len().saturating_sub(1));
            let style = if is_selected {
                theme::repo_pill_active_style()
            } else {
                theme::repo_pill_inactive_style()
            };
            spans.push(Span::styled(format!(" {} ", repo.summary.name), style));
        }
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}
