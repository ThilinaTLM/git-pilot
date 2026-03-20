use ratatui::prelude::*;

use crate::app::state::{AppState, View};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let views = [
        ("1", "Changes", View::Changes),
        ("2", "Branches", View::Branches),
        ("3", "Log", View::Changes),     // future — shown dimmed
        ("4", "Remotes", View::Changes), // future — shown dimmed
    ];

    let mut spans = Vec::new();
    for (i, (key, label, view)) in views.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", theme::muted_text_style()));
        }

        let is_active = state.active_view == *view && i < 2;
        let is_future = i >= 2;

        if is_active {
            spans.push(Span::styled(
                format!("{key} {label}"),
                theme::accent_text_style(),
            ));
        } else if is_future {
            spans.push(Span::styled(
                format!("{key} {label}"),
                Style::default()
                    .fg(Color::Rgb(71, 85, 105))
                    .bg(Color::Rgb(15, 23, 42)),
            ));
        } else {
            spans.push(Span::styled(
                format!("{key} {label}"),
                theme::muted_text_style(),
            ));
        }
    }

    let line = Line::from(spans);
    let paragraph = ratatui::widgets::Paragraph::new(line);
    frame.render_widget(paragraph, area);
}
