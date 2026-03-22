use ratatui::prelude::*;
use ratatui::widgets::{Clear, Paragraph};

use crate::app::state::{AppState, CreateRepoStep};
use crate::ui::layout::centered_rect;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let Some(ref rs) = state.create_repo_state else {
        return;
    };

    theme::render_backdrop(frame, area);
    let modal = centered_rect(55, 50, area);
    frame.render_widget(Clear, modal);

    let step_num = match rs.step {
        CreateRepoStep::Owner => 1,
        CreateRepoStep::RepoName => 2,
        CreateRepoStep::Visibility => 3,
        CreateRepoStep::Confirm => 4,
    };

    let block =
        theme::modal_elevated_block(format!("Create GitHub Repository — Step {step_num} of 4"));
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    match rs.step {
        CreateRepoStep::Owner => render_owner_step(frame, inner, &rs.owner_input),
        CreateRepoStep::RepoName => {
            render_repo_name_step(frame, inner, &rs.owner_input, &rs.repo_name_input)
        }
        CreateRepoStep::Visibility => render_visibility_step(frame, inner, rs.is_public),
        CreateRepoStep::Confirm => render_confirm_step(frame, inner, rs),
    }
}

fn render_owner_step(frame: &mut Frame, area: Rect, owner: &str) {
    let lines = vec![
        Line::from(Span::styled(
            "GitHub username or organization:",
            theme::modal_text_style(),
        )),
        Line::default(),
        Line::from(vec![
            Span::styled("> ", theme::modal_accent_style()),
            Span::styled(owner, theme::modal_text_style()),
            Span::styled("█", theme::modal_accent_style()),
        ]),
        Line::default(),
        Line::default(),
        shortcut_line(&[("Enter", "next"), ("Esc", "cancel")]),
    ];

    let paragraph = Paragraph::new(Text::from(lines));
    frame.render_widget(paragraph, area);
}

fn render_repo_name_step(frame: &mut Frame, area: Rect, owner: &str, name: &str) {
    let lines = vec![
        Line::from(Span::styled("Repository name:", theme::modal_text_style())),
        Line::default(),
        Line::from(vec![
            Span::styled("> ", theme::modal_accent_style()),
            Span::styled(name, theme::modal_text_style()),
            Span::styled("█", theme::modal_accent_style()),
        ]),
        Line::default(),
        Line::from(vec![
            Span::styled("Preview: ", theme::modal_muted_style()),
            Span::styled(
                format!("github.com/{owner}/{name}"),
                theme::modal_accent_style(),
            ),
        ]),
        Line::default(),
        shortcut_line(&[("Enter", "next"), ("Esc", "back")]),
    ];

    let paragraph = Paragraph::new(Text::from(lines));
    frame.render_widget(paragraph, area);
}

fn render_visibility_step(frame: &mut Frame, area: Rect, is_public: bool) {
    let (pub_marker, priv_marker) = if is_public {
        ("● ", "  ")
    } else {
        ("  ", "● ")
    };

    let lines = vec![
        Line::from(Span::styled("Visibility:", theme::modal_text_style())),
        Line::default(),
        Line::from(vec![
            Span::styled(
                pub_marker,
                if is_public {
                    theme::modal_accent_style()
                } else {
                    theme::modal_muted_style()
                },
            ),
            Span::styled(
                "Public",
                if is_public {
                    theme::modal_text_style()
                } else {
                    theme::modal_muted_style()
                },
            ),
            Span::styled(
                "  — anyone can see this repository",
                theme::modal_muted_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                priv_marker,
                if !is_public {
                    theme::modal_accent_style()
                } else {
                    theme::modal_muted_style()
                },
            ),
            Span::styled(
                "Private",
                if !is_public {
                    theme::modal_text_style()
                } else {
                    theme::modal_muted_style()
                },
            ),
            Span::styled(
                " — only you and collaborators can see",
                theme::modal_muted_style(),
            ),
        ]),
        Line::default(),
        shortcut_line(&[("Space/j/k", "toggle"), ("Enter", "next"), ("Esc", "back")]),
    ];

    let paragraph = Paragraph::new(Text::from(lines));
    frame.render_widget(paragraph, area);
}

fn render_confirm_step(frame: &mut Frame, area: Rect, rs: &crate::app::state::CreateRepoState) {
    let visibility = if rs.is_public { "public" } else { "private" };
    let lines = vec![
        Line::from(Span::styled("Summary:", theme::modal_text_style())),
        Line::default(),
        Line::from(vec![
            Span::styled("  Repository: ", theme::modal_muted_style()),
            Span::styled(
                format!("{}/{}", rs.owner_input, rs.repo_name_input),
                theme::modal_accent_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Visibility: ", theme::modal_muted_style()),
            Span::styled(visibility, theme::modal_text_style()),
        ]),
        Line::default(),
        Line::from(Span::styled("Command preview:", theme::modal_muted_style())),
        Line::from(Span::styled(
            format!(
                "  gh repo create {}/{} --{visibility} --source=. --remote=origin",
                rs.owner_input, rs.repo_name_input
            ),
            theme::modal_muted_style(),
        )),
        Line::default(),
        shortcut_line(&[("Enter", "create"), ("Esc", "back")]),
    ];

    let paragraph = Paragraph::new(Text::from(lines));
    frame.render_widget(paragraph, area);
}

fn shortcut_line(entries: &[(&str, &str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, (key, label)) in entries.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", theme::modal_muted_style()));
        }
        spans.push(Span::styled(key.to_string(), theme::modal_accent_style()));
        spans.push(Span::styled(
            format!(" {label}"),
            theme::modal_muted_style(),
        ));
    }
    Line::from(spans)
}
