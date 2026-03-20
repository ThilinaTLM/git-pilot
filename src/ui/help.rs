use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

use crate::app::state::{ActivePanel, AppState, MessageLevel};
use crate::shared::shortcuts::HELP_SECTIONS;
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render_footer(frame: &mut Frame, area: Rect, state: &AppState) {
    let content = match state.message.as_ref().map(|message| &message.level) {
        Some(MessageLevel::Error) => {
            let text = state
                .message
                .as_ref()
                .map(|message| message.text.as_str())
                .unwrap_or("");
            Line::from(vec![
                Span::styled(
                    "Error  ",
                    theme::error_text_style().add_modifier(Modifier::BOLD),
                ),
                Span::styled(text, theme::error_text_style()),
            ])
        }
        Some(MessageLevel::Info) => {
            let text = state
                .message
                .as_ref()
                .map(|message| message.text.as_str())
                .unwrap_or("");
            let hints = context_hint_text(state);
            Line::from(vec![
                Span::styled("Info  ", theme::accent_text_style()),
                Span::styled(text, theme::text_style()),
                Span::styled("  •  ", theme::muted_text_style()),
                Span::styled(hints, theme::muted_text_style()),
            ])
        }
        None => Line::from(vec![
            Span::styled(
                "Hints  ",
                theme::muted_text_style().add_modifier(Modifier::BOLD),
            ),
            Span::styled(context_hint_text(state), theme::muted_text_style()),
        ]),
    };

    let block = theme::footer_block();
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let paragraph = Paragraph::new(content).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

pub fn render_overlay(frame: &mut Frame, area: Rect) {
    let modal = centered_rect(74, 78, area);
    frame.render_widget(Clear, modal);
    let block = theme::modal_block("Keyboard Shortcuts");
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let paragraph = Paragraph::new(help_text())
        .style(theme::text_style())
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

fn help_text() -> Text<'static> {
    let mut lines = Vec::new();

    lines.push(Line::from(vec![Span::styled(
        "Press ? again to close this view.",
        theme::warning_text_style().add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::default());

    for section in HELP_SECTIONS {
        lines.push(Line::from(vec![Span::styled(
            section.title,
            theme::accent_text_style(),
        )]));

        for entry in section.entries {
            lines.push(Line::from(vec![
                Span::styled(format!("{:<14}", entry.keys), theme::success_text_style()),
                Span::styled(entry.description, theme::text_style()),
            ]));
        }

        lines.push(Line::default());
    }

    Text::from(lines)
}

fn context_hint_text(state: &AppState) -> String {
    let hints = match state.active_panel {
        ActivePanel::BranchSwitch => {
            vec!["j/k move", "Enter switch", "Esc cancel", "? full help"]
        }
        ActivePanel::BranchCreate => {
            vec!["type name", "Enter create", "Backspace edit", "Esc cancel"]
        }
        ActivePanel::Commit => {
            vec![
                "type message",
                "Ctrl+n newline",
                "Enter commit",
                "Esc cancel",
            ]
        }
        ActivePanel::None if state.repos.is_empty() => {
            vec!["r refresh", "? full help", "q quit"]
        }
        ActivePanel::None
            if state
                .selected_repo_ref()
                .is_some_and(|repo| repo.status_files.is_empty()) =>
        {
            vec![
                "h/l repos",
                "n new branch",
                "b switch branch",
                "r refresh",
                "? full help",
            ]
        }
        ActivePanel::None => {
            vec![
                "h/l repos",
                "j/k select file",
                "J/K scroll diff",
                "s/u stage",
                "c commit",
                "b branches",
                "? full help",
            ]
        }
    };

    hints.join("  |  ")
}
