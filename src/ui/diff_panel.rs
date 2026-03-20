use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::app::state::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = theme::pane_block("Diff");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(diff) = &state.diff_content else {
        let placeholder =
            Paragraph::new("Select a file to view diff.").style(theme::muted_text_style());
        frame.render_widget(placeholder, inner);
        return;
    };

    let lines: Vec<Line> = diff
        .lines()
        .map(|line| {
            let style = if line.starts_with('+') && !line.starts_with("+++") {
                theme::success_text_style()
            } else if line.starts_with('-') && !line.starts_with("---") {
                theme::error_text_style()
            } else if line.starts_with("@@") {
                theme::accent_text_style()
            } else if line.starts_with("diff ") || line.starts_with("index ") {
                theme::muted_text_style()
            } else {
                theme::text_style()
            };
            Line::from(Span::styled(line.to_string(), style))
        })
        .collect();

    let paragraph = Paragraph::new(Text::from(lines))
        .style(theme::text_style())
        .scroll((state.diff_scroll, 0));
    frame.render_widget(paragraph, inner);
}
