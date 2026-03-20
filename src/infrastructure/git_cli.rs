use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::domain::branch::{BranchInfo, BranchName};
use crate::domain::commit::CommitMessage;
use crate::domain::repo::{RepositoryDetails, RepositorySummary};
use crate::domain::status::{ChangedFile, RepositoryStatus};
use crate::infrastructure::process::{base_git_command, run_command};

pub trait GitRepositoryService {
    fn load_repository(&self, summary: &RepositorySummary) -> Result<RepositoryDetails>;
    fn stage_file(&self, repo_path: &Path, file_path: &Path) -> Result<()>;
    fn unstage_file(&self, repo_path: &Path, file_path: &Path) -> Result<()>;
    fn stage_all(&self, repo_path: &Path) -> Result<()>;
    fn unstage_all(&self, repo_path: &Path) -> Result<()>;
    fn switch_branch(&self, repo_path: &Path, branch_name: &BranchName) -> Result<()>;
    fn create_branch(&self, repo_path: &Path, branch_name: &BranchName) -> Result<()>;
    fn commit(&self, repo_path: &Path, message: &CommitMessage) -> Result<()>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GitCliRepositoryService;

impl GitRepositoryService for GitCliRepositoryService {
    fn load_repository(&self, summary: &RepositorySummary) -> Result<RepositoryDetails> {
        let current_branch = self.current_branch(&summary.path)?;
        let branches = self.branches(&summary.path, current_branch.as_deref())?;
        let status = self.status(&summary.path)?;

        Ok(RepositoryDetails {
            summary: summary.clone(),
            current_branch,
            branches,
            status,
        })
    }

    fn stage_file(&self, repo_path: &Path, file_path: &Path) -> Result<()> {
        let mut command = base_git_command(repo_path);
        command.arg("add").arg("--").arg(file_path);
        run_command(&mut command).map(|_| ())
    }

    fn unstage_file(&self, repo_path: &Path, file_path: &Path) -> Result<()> {
        let mut command = base_git_command(repo_path);
        command.arg("reset").arg("HEAD").arg("--").arg(file_path);
        run_command(&mut command).map(|_| ())
    }

    fn stage_all(&self, repo_path: &Path) -> Result<()> {
        let mut command = base_git_command(repo_path);
        command.arg("add").arg("--all");
        run_command(&mut command).map(|_| ())
    }

    fn unstage_all(&self, repo_path: &Path) -> Result<()> {
        let mut command = base_git_command(repo_path);
        command.arg("reset").arg("HEAD").arg("--").arg(".");
        run_command(&mut command).map(|_| ())
    }

    fn switch_branch(&self, repo_path: &Path, branch_name: &BranchName) -> Result<()> {
        let mut command = base_git_command(repo_path);
        command.arg("switch").arg(branch_name.as_str());
        run_command(&mut command).map(|_| ())
    }

    fn create_branch(&self, repo_path: &Path, branch_name: &BranchName) -> Result<()> {
        let mut command = base_git_command(repo_path);
        command.arg("switch").arg("-c").arg(branch_name.as_str());
        run_command(&mut command).map(|_| ())
    }

    fn commit(&self, repo_path: &Path, message: &CommitMessage) -> Result<()> {
        let mut command = base_git_command(repo_path);
        command.arg("commit");
        for message_part in message.git_message_args() {
            command.arg("-m").arg(message_part);
        }
        run_command(&mut command).map(|_| ())
    }
}

impl GitCliRepositoryService {
    fn current_branch(&self, repo_path: &Path) -> Result<Option<String>> {
        let mut command = base_git_command(repo_path);
        command.arg("branch").arg("--show-current");
        let output = run_command(&mut command)?;
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if branch.is_empty() {
            Ok(None)
        } else {
            Ok(Some(branch))
        }
    }

    fn branches(&self, repo_path: &Path, current_branch: Option<&str>) -> Result<Vec<BranchInfo>> {
        let mut command = base_git_command(repo_path);
        command.arg("branch").arg("--format=%(refname:short)");
        let output = run_command(&mut command)?;
        let mut branches = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let name = BranchName::try_from(line.trim().to_string())
                    .map_err(|error| anyhow!(error.to_string()))?;
                Ok(BranchInfo {
                    is_current: current_branch.is_some_and(|current| current == line.trim()),
                    name,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        if let Some(current_branch) = current_branch
            && branches
                .iter()
                .all(|branch| branch.name.as_str() != current_branch)
        {
            branches.push(BranchInfo {
                is_current: true,
                name: BranchName::try_from(current_branch.to_string())
                    .map_err(|error| anyhow!(error.to_string()))?,
            });
        }

        branches.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(branches)
    }

    fn status(&self, repo_path: &Path) -> Result<RepositoryStatus> {
        let mut command = base_git_command(repo_path);
        command
            .arg("status")
            .arg("--porcelain=v1")
            .arg("-z")
            .arg("--untracked-files=all");
        let output = run_command(&mut command)?;
        parse_status_output(&output.stdout)
    }
}

pub fn parse_status_output(raw: &[u8]) -> Result<RepositoryStatus> {
    let entries = raw
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<_>>();

    let mut files = Vec::new();
    let mut index = 0;
    while index < entries.len() {
        let entry = entries[index];
        if entry.len() < 4 {
            return Err(anyhow!("unexpected git status entry"));
        }

        let staged_code = entry[0] as char;
        let unstaged_code = entry[1] as char;
        let mut path = parse_path(&entry[3..]);

        let is_rename_or_copy =
            matches!(staged_code, 'R' | 'C') || matches!(unstaged_code, 'R' | 'C');
        if is_rename_or_copy {
            index += 1;
            let renamed_path = entries
                .get(index)
                .context("missing renamed target path in git status output")?;
            path = parse_path(renamed_path);
        }

        files.push(ChangedFile {
            path,
            staged: staged_code != ' ' && staged_code != '?',
            unstaged: unstaged_code != ' ' || staged_code == '?',
            untracked: staged_code == '?' || unstaged_code == '?',
            status_code: format!("{staged_code}{unstaged_code}"),
        });

        index += 1;
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(RepositoryStatus { files })
}

fn parse_path(raw: &[u8]) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(raw).to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_status_output;

    #[test]
    fn parses_staged_and_unstaged_files() {
        let output = b"M  src/main.rs\0 M Cargo.toml\0?? notes.txt\0";
        let status = parse_status_output(output).expect("status parses");

        assert_eq!(status.files.len(), 3);
        assert_eq!(status.files[0].path, PathBuf::from("Cargo.toml"));
        assert!(status.files[0].unstaged);
        assert_eq!(status.files[1].path, PathBuf::from("notes.txt"));
        assert!(status.files[1].untracked);
        assert_eq!(status.files[2].path, PathBuf::from("src/main.rs"));
        assert!(status.files[2].staged);
    }

    #[test]
    fn parses_renamed_entries_with_target_path() {
        let output = b"R  old.txt\0new.txt\0";
        let status = parse_status_output(output).expect("status parses");

        assert_eq!(status.files.len(), 1);
        assert_eq!(status.files[0].path, PathBuf::from("new.txt"));
    }

    use std::path::PathBuf;
}
