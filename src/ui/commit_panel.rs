use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout::centered_rect;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let modal = centered_rect(60, 50, area);
    frame.render_widget(Clear, modal);

    let paragraph = Paragraph::new(state.commit_message_input.as_str())
        .block(
            Block::default()
                .title("Commit Message (Enter confirm, Ctrl+n newline, Esc cancel)")
                .borders(Borders::ALL),
        )
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(paragraph, modal);
}
