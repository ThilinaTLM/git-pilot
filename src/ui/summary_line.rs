use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::app::state::AppState;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let Some(repo) = state.selected_repo_ref() else {
        let line = Line::from(Span::styled(
            "No repository selected.",
            theme::muted_text_style(),
        ));
        frame.render_widget(Paragraph::new(line), area);
        return;
    };

    let mut spans: Vec<Span> = Vec::new();

    // Branch name
    let branch = repo.current_branch.as_deref().unwrap_or("(detached)");
    spans.push(Span::styled(
        format!(" {branch}"),
        theme::accent_text_style(),
    ));

    // Branch count
    let branch_count = repo.branches.len();
    spans.push(Span::styled(" • ", theme::muted_text_style()));
    spans.push(Span::styled(
        format!(
            "{branch_count} branch{}",
            if branch_count == 1 { "" } else { "es" }
        ),
        theme::text_style(),
    ));

    // PR count (only if > 0)
    let pr_count = repo.pull_requests.len();
    if pr_count > 0 {
        spans.push(Span::styled(" • ", theme::muted_text_style()));
        spans.push(Span::styled(
            format!("{pr_count} PR{}", if pr_count == 1 { "" } else { "s" }),
            theme::text_style(),
        ));
    }

    // Staged count
    let staged_count = repo.status_files.iter().filter(|f| f.staged).count();
    spans.push(Span::styled(" • ", theme::muted_text_style()));
    spans.push(Span::styled(
        format!("{staged_count} staged"),
        theme::success_text_style(),
    ));

    // Unstaged count (including untracked)
    let unstaged_count = repo.status_files.iter().filter(|f| f.unstaged).count();
    spans.push(Span::styled(" • ", theme::muted_text_style()));
    spans.push(Span::styled(
        format!("{unstaged_count} unstaged"),
        theme::warning_text_style(),
    ));

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}
