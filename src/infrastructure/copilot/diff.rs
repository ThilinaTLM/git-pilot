use std::path::Path;

use anyhow::Result;

use crate::infrastructure::git_cli::GitRepositoryService;

const MAX_TOTAL_CHARS: usize = 64_000;
const MAX_PER_FILE_CHARS: usize = 4_000;

const NOISE_FILES: &[&str] = &[
    "Cargo.lock",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Gemfile.lock",
    "poetry.lock",
    "composer.lock",
    "go.sum",
    "flake.lock",
];

fn is_noise_file(path: &str) -> bool {
    let filename = path.rsplit('/').next().unwrap_or(path);
    NOISE_FILES.contains(&filename)
}

fn truncate_with_marker(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }
    let truncated = &text[..max_chars];
    let end = truncated.rfind('\n').unwrap_or(max_chars);
    format!("{}\n[...truncated]", &text[..end])
}

pub fn prepare_diff_context(git: &impl GitRepositoryService, repo_path: &Path) -> Result<String> {
    let stat = git.diff_staged_stat(repo_path)?;
    let file_names = git.diff_staged_file_names(repo_path)?;

    let mut context = format!("## Overview (git diff --stat)\n{stat}\n");
    let budget = MAX_TOTAL_CHARS.saturating_sub(context.len());
    let mut remaining = budget;

    for file_name in &file_names {
        if is_noise_file(file_name) {
            continue;
        }

        let diff = match git.diff_staged_file(repo_path, file_name) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if diff.trim().is_empty() {
            continue;
        }

        let diff = truncate_with_marker(&diff, MAX_PER_FILE_CHARS);
        let section = format!("## File: {file_name}\n{diff}\n\n");

        if section.len() > remaining {
            break;
        }

        remaining -= section.len();
        context.push_str(&section);
    }

    Ok(context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_noise_files() {
        assert!(is_noise_file("Cargo.lock"));
        assert!(is_noise_file("path/to/package-lock.json"));
        assert!(is_noise_file("deep/nested/yarn.lock"));
        assert!(!is_noise_file("src/main.rs"));
        assert!(!is_noise_file("Cargo.toml"));
    }

    #[test]
    fn truncates_long_text() {
        let text = "line1\nline2\nline3\nline4\n";
        let result = truncate_with_marker(text, 12);
        assert!(result.ends_with("[...truncated]"));
        assert!(result.len() < text.len() + 20);
    }

    #[test]
    fn short_text_not_truncated() {
        let text = "short";
        assert_eq!(truncate_with_marker(text, 100), "short");
    }
}
