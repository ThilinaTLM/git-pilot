use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let modal = centered_rect(50, 30, area);
    frame.render_widget(Clear, modal);

    let block = theme::modal_elevated_block("GitHub Copilot Login");
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // spacer
            Constraint::Length(1), // instruction
            Constraint::Length(1), // spacer
            Constraint::Length(1), // user code
            Constraint::Length(1), // spacer
            Constraint::Length(1), // url
            Constraint::Length(1), // spacer
            Constraint::Length(1), // waiting
            Constraint::Min(0),    // fill
            Constraint::Length(1), // shortcut
        ])
        .split(inner);

    let instruction = Paragraph::new(Line::from(Span::styled(
        "Enter this code on GitHub:",
        theme::modal_muted_style(),
    )));
    frame.render_widget(instruction, layout[1]);

    let code_text = state
        .device_code
        .as_ref()
        .map(|d| d.user_code.as_str())
        .unwrap_or("...");
    let code = Paragraph::new(Line::from(Span::styled(
        code_text,
        theme::modal_accent_style(),
    )));
    frame.render_widget(code, layout[3]);

    let uri = state
        .device_code
        .as_ref()
        .map(|d| d.verification_uri.as_str())
        .unwrap_or("https://github.com/login/device");
    let url = Paragraph::new(Line::from(Span::styled(uri, theme::modal_text_style())));
    frame.render_widget(url, layout[5]);

    let waiting = Paragraph::new(Line::from(Span::styled(
        "Waiting for authorization...",
        theme::modal_muted_style(),
    )));
    frame.render_widget(waiting, layout[7]);

    let shortcuts = Line::from(vec![
        Span::styled("Esc ", theme::modal_accent_style()),
        Span::styled("cancel", theme::modal_muted_style()),
    ]);
    frame.render_widget(Paragraph::new(shortcuts), layout[9]);
}
