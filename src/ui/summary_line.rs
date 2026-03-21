use ratatui::prelude::*;

use crate::app::state::AppState;
use crate::ui::theme;

/// Build summary spans (branch, counts) for use in the footer.
pub fn build_spans(state: &AppState) -> Vec<Span<'static>> {
    let Some(repo) = state.selected_repo_ref() else {
        return vec![Span::styled(
            "No repository selected.",
            theme::muted_text_style(),
        )];
    };

    let mut spans: Vec<Span> = Vec::new();

    // Show spinner + job labels when background jobs are active
    if state.has_active_jobs() {
        let labels: Vec<&str> = state.active_jobs.iter().map(|j| j.kind.label()).collect();
        let label = labels.join(", ");
        spans.push(Span::styled(
            format!(" {} ", state.spinner_char()),
            theme::accent_text_style(),
        ));
        spans.push(Span::styled(label, theme::text_style()));
        return spans;
    }

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

    spans
}
