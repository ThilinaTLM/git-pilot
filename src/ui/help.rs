use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::state::{AppState, MessageLevel};
use crate::shared::shortcuts::{HELP_SECTIONS, SHORT_HELP};
use crate::ui::layout::centered_rect;

pub fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let message = state
        .message
        .as_ref()
        .map(|message| message.text.as_str())
        .unwrap_or(SHORT_HELP);
    let color = match state.message.as_ref().map(|message| &message.level) {
        Some(MessageLevel::Error) => Color::Red,
        Some(MessageLevel::Info) => Color::Green,
        None => Color::White,
    };

    let paragraph = Paragraph::new(message)
        .style(Style::default().fg(color))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

pub fn render_overlay(frame: &mut Frame, area: Rect) {
    let modal = centered_rect(74, 78, area);
    frame.render_widget(Clear, modal);
    let paragraph = Paragraph::new(help_text())
        .block(
            Block::default()
                .title("Keyboard Shortcuts")
                .borders(Borders::ALL),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(paragraph, modal);
}

fn help_text() -> Text<'static> {
    let mut lines = Vec::new();

    lines.push(Line::from(vec![
        Span::styled(
            "Press ? again to close this view.",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::default());

    for section in HELP_SECTIONS {
        lines.push(Line::from(vec![Span::styled(
            section.title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));

        for entry in section.entries {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:<14}", entry.keys),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(entry.description),
            ]));
        }

        lines.push(Line::default());
    }

    Text::from(lines)
}
