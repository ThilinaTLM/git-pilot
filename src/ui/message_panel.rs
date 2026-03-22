use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

use crate::app::state::{AppState, MessageLevel};
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let Some(message) = &state.message else {
        return;
    };

    theme::render_backdrop(frame, area);
    let modal = centered_rect(60, 40, area);
    frame.render_widget(Clear, modal);

    let title = match message.level {
        MessageLevel::Error => "Error",
        MessageLevel::Info => "Info",
    };
    let block = theme::modal_elevated_block(title);
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // message body
            Constraint::Length(1), // gap
            Constraint::Length(1), // shortcut bar
        ])
        .split(inner);

    let text_style = match message.level {
        MessageLevel::Error => theme::error_text_style(),
        MessageLevel::Info => theme::modal_text_style(),
    };
    let body = Paragraph::new(message.text.as_str())
        .style(text_style)
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(body, layout[0]);

    let shortcuts = Line::from(vec![
        Span::styled("esc ", theme::modal_accent_style()),
        Span::styled("close", theme::modal_muted_style()),
        Span::styled("  enter ", theme::modal_accent_style()),
        Span::styled("close", theme::modal_muted_style()),
    ]);
    frame.render_widget(Paragraph::new(shortcuts), layout[2]);
}
