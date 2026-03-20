use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let modal = centered_rect(60, 50, area);
    frame.render_widget(Clear, modal);
    let block = theme::modal_block("Commit Message");
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let mut lines = vec![
        Line::from(Span::styled(
            "Write the commit subject and optional body.",
            theme::muted_text_style(),
        )),
        Line::default(),
    ];

    if state.commit_message_input.is_empty() {
        lines.push(Line::from(Span::styled(
            "Subject line",
            theme::muted_text_style(),
        )));
    } else {
        lines.extend(
            state
                .commit_message_input
                .lines()
                .map(|line| Line::from(line.to_string())),
        );
    }

    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "Enter to commit, Ctrl+n for newline, Esc to cancel.",
        theme::muted_text_style(),
    )));

    let paragraph = Paragraph::new(Text::from(lines))
        .style(theme::text_style())
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(paragraph, inner);
}
