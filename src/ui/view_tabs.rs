use ratatui::prelude::*;

use crate::app::state::{AppState, View};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let views = [("Changes", View::Changes), ("PR", View::Pr)];

    let mut spans = Vec::new();
    for (i, (label, view)) in views.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", theme::header_separator_style()));
        }

        let style = if state.active_view == *view {
            theme::active_tab_style()
        } else {
            theme::muted_text_style()
        };

        spans.push(Span::styled(format!(" {label} "), style));
    }

    let line = Line::from(spans);
    let paragraph = ratatui::widgets::Paragraph::new(line);
    frame.render_widget(paragraph, area);
}
