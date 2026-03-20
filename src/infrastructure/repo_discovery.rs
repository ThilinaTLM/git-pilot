use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

use crate::domain::repo::{RepositoryId, RepositorySummary};

pub trait RepositoryDiscovery {
    fn discover(&self, root: &Path, max_depth: usize) -> Result<Vec<RepositorySummary>>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WalkDirRepositoryDiscovery;

impl RepositoryDiscovery for WalkDirRepositoryDiscovery {
    fn discover(&self, root: &Path, max_depth: usize) -> Result<Vec<RepositorySummary>> {
        let mut repos = Vec::new();
        let mut seen = HashSet::new();
        let canonical_root = fs::canonicalize(root)?;

        if looks_like_git_repo(root) {
            seen.insert(canonical_root.clone());
            repos.push(to_repository_summary(root, canonical_root.clone())?);
        }

        let mut walker = WalkDir::new(root)
            .min_depth(1)
            .max_depth(max_depth)
            .into_iter();

        while let Some(entry) = walker.next() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            if !entry.file_type().is_dir() {
                continue;
            }

            let path = entry.path();
            if path.file_name().is_some_and(|name| name == ".git") {
                continue;
            }

            if !looks_like_git_repo(path) {
                continue;
            }

            let canonical = fs::canonicalize(path)?;
            if seen.insert(canonical.clone()) {
                repos.push(to_repository_summary(root, canonical.clone())?);
            }
            walker.skip_current_dir();
        }

        repos.sort_by(compare_repositories);
        Ok(repos)
    }
}

fn looks_like_git_repo(path: &Path) -> bool {
    match fs::metadata(path.join(".git")) {
        Ok(metadata) => metadata.is_dir() || metadata.is_file(),
        Err(_) => false,
    }
}

fn to_repository_summary(
    root: &Path,
    canonical_path: std::path::PathBuf,
) -> Result<RepositorySummary> {
    let relative_path = canonical_path
        .strip_prefix(root)
        .map(relative_or_dot)
        .unwrap_or_else(|_| canonical_path.clone());
    let name = canonical_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| canonical_path.display().to_string());

    Ok(RepositorySummary {
        id: RepositoryId(canonical_path.clone()),
        name,
        path: canonical_path,
        relative_path,
    })
}

fn relative_or_dot(path: &Path) -> PathBuf {
    if path.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        path.to_path_buf()
    }
}

fn compare_repositories(left: &RepositorySummary, right: &RepositorySummary) -> Ordering {
    match (
        left.relative_path == Path::new("."),
        right.relative_path == Path::new("."),
    ) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => left.relative_path.cmp(&right.relative_path),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::{RepositoryDiscovery, WalkDirRepositoryDiscovery};

    #[test]
    fn discovers_nested_git_repositories_within_depth_limit() {
        let tempdir = TempDir::new().expect("tempdir");
        create_git_dir(tempdir.path().join("one"));
        create_git_dir(tempdir.path().join("nested/two"));
        create_git_dir(tempdir.path().join("a/b/c/three"));

        let service = WalkDirRepositoryDiscovery;
        let repos = service
            .discover(tempdir.path(), 3)
            .expect("repo discovery succeeds");

        let relative_paths = repos
            .into_iter()
            .map(|repo| repo.relative_path.display().to_string())
            .collect::<Vec<_>>();

        assert_eq!(relative_paths, vec!["nested/two", "one"]);
    }

    #[test]
    fn includes_root_repo_as_single_result() {
        let tempdir = TempDir::new().expect("tempdir");
        create_git_dir(tempdir.path().to_path_buf());

        let service = WalkDirRepositoryDiscovery;
        let repos = service
            .discover(tempdir.path(), 3)
            .expect("repo discovery succeeds");

        let relative_paths = repos
            .into_iter()
            .map(|repo| repo.relative_path.display().to_string())
            .collect::<Vec<_>>();

        assert_eq!(relative_paths, vec!["."]);
    }

    #[test]
    fn includes_root_repo_and_nested_repositories_without_duplicates() {
        let tempdir = TempDir::new().expect("tempdir");
        create_git_dir(tempdir.path().to_path_buf());
        create_git_dir(tempdir.path().join("one"));
        create_git_dir(tempdir.path().join("nested/two"));

        let service = WalkDirRepositoryDiscovery;
        let repos = service
            .discover(tempdir.path(), 3)
            .expect("repo discovery succeeds");

        let relative_paths = repos
            .into_iter()
            .map(|repo| repo.relative_path.display().to_string())
            .collect::<Vec<_>>();

        assert_eq!(relative_paths, vec![".", "nested/two", "one"]);
    }

    #[test]
    fn recognizes_worktree_style_git_metadata_file() {
        let tempdir = TempDir::new().expect("tempdir");
        let repo_root = tempdir.path().join("worktree");
        fs::create_dir_all(&repo_root).expect("create repo dir");
        fs::write(repo_root.join(".git"), "gitdir: /tmp/mock").expect("create git file");

        let service = WalkDirRepositoryDiscovery;
        let repos = service
            .discover(tempdir.path(), 2)
            .expect("repo discovery succeeds");

        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "worktree");
    }

    fn create_git_dir(path: std::path::PathBuf) {
        fs::create_dir_all(path.join(".git")).expect("create fake repo");
    }
}
