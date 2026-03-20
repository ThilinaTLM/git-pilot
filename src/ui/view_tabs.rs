use ratatui::prelude::*;

use crate::app::state::{AppState, View};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let views = [
        ("1", "Changes", View::Changes),
        ("2", "Branches", View::Branches),
        ("3", "Log", View::Log),
        ("4", "Remotes", View::Remotes),
    ];

    let mut spans = Vec::new();
    for (i, (key, label, view)) in views.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", theme::header_separator_style()));
        }

        let is_active = state.active_view == *view;

        if is_active {
            spans.push(Span::styled(
                format!(" {key} {label} "),
                theme::active_tab_style(),
            ));
        } else {
            spans.push(Span::styled(
                format!(" {key} {label} "),
                theme::muted_text_style(),
            ));
        }
    }

    let line = Line::from(spans);
    let paragraph = ratatui::widgets::Paragraph::new(line);
    frame.render_widget(paragraph, area);
}
