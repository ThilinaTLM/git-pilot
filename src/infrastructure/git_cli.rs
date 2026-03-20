use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::domain::branch::{BranchInfo, BranchName};
use crate::domain::commit::{CommitMessage, LogEntry};
use crate::domain::remote::RemoteInfo;
use crate::domain::repo::{RepositoryDetails, RepositorySummary};
use crate::domain::status::{ChangedFile, FileSection, RepositoryStatus};
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
    fn delete_branch(&self, repo_path: &Path, branch_name: &BranchName) -> Result<()>;
    fn merge_branch(&self, repo_path: &Path, branch_name: &BranchName) -> Result<()>;
    fn diff_file(&self, repo_path: &Path, file_path: &Path, section: FileSection)
    -> Result<String>;
    fn diff_staged(&self, repo_path: &Path) -> Result<String>;
    fn diff_staged_stat(&self, repo_path: &Path) -> Result<String>;
    fn diff_staged_file_names(&self, repo_path: &Path) -> Result<Vec<String>>;
    fn diff_staged_file(&self, repo_path: &Path, file_path: &str) -> Result<String>;
    fn log_entries(&self, repo_path: &Path, limit: usize) -> Result<Vec<LogEntry>>;
    fn list_remotes(&self, repo_path: &Path) -> Result<Vec<RemoteInfo>>;
    fn has_remote(&self, repo_path: &Path, name: &str) -> bool;
    fn amend_commit(&self, repo_path: &Path, message: &CommitMessage) -> Result<()>;
    fn last_commit_message(&self, repo_path: &Path) -> Result<String>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GitCliRepositoryService;

impl GitRepositoryService for GitCliRepositoryService {
    fn load_repository(&self, summary: &RepositorySummary) -> Result<RepositoryDetails> {
        let current_branch = self.current_branch(&summary.path)?;
        let branches = self.branches(&summary.path, current_branch.as_deref())?;
        let status = self.status(&summary.path)?;
        let log_entries = self.log_entries(&summary.path, 100).unwrap_or_default();
        let remotes = self.list_remotes(&summary.path).unwrap_or_default();

        Ok(RepositoryDetails {
            summary: summary.clone(),
            current_branch,
            branches,
            status,
            log_entries,
            remotes,
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

    fn delete_branch(&self, repo_path: &Path, branch_name: &BranchName) -> Result<()> {
        let mut command = base_git_command(repo_path);
        command.arg("branch").arg("-d").arg(branch_name.as_str());
        run_command(&mut command).map(|_| ())
    }

    fn merge_branch(&self, repo_path: &Path, branch_name: &BranchName) -> Result<()> {
        let mut command = base_git_command(repo_path);
        command.arg("merge").arg(branch_name.as_str());
        run_command(&mut command).map(|_| ())
    }

    fn diff_staged(&self, repo_path: &Path) -> Result<String> {
        let mut command = base_git_command(repo_path);
        command.arg("diff").arg("--cached");
        let output = run_command(&mut command)?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn diff_staged_stat(&self, repo_path: &Path) -> Result<String> {
        let mut command = base_git_command(repo_path);
        command.arg("diff").arg("--cached").arg("--stat");
        let output = run_command(&mut command)?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn diff_staged_file_names(&self, repo_path: &Path) -> Result<Vec<String>> {
        let mut command = base_git_command(repo_path);
        command.arg("diff").arg("--cached").arg("--name-only");
        let output = run_command(&mut command)?;
        let names = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.to_string())
            .collect();
        Ok(names)
    }

    fn diff_staged_file(&self, repo_path: &Path, file_path: &str) -> Result<String> {
        let mut command = base_git_command(repo_path);
        command.arg("diff").arg("--cached").arg("--").arg(file_path);
        let output = run_command(&mut command)?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn diff_file(
        &self,
        repo_path: &Path,
        file_path: &Path,
        section: FileSection,
    ) -> Result<String> {
        match section {
            FileSection::Untracked => {
                let mut command = base_git_command(repo_path);
                command
                    .arg("diff")
                    .arg("--no-index")
                    .arg("--")
                    .arg("/dev/null")
                    .arg(file_path);
                // --no-index returns exit code 1 when files differ, so we read output directly
                let output = command.output().context("failed to execute git diff")?;
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
            FileSection::Staged => {
                let mut command = base_git_command(repo_path);
                command.arg("diff").arg("--cached").arg("--").arg(file_path);
                let output = run_command(&mut command)?;
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
            FileSection::Unstaged => {
                let mut command = base_git_command(repo_path);
                command.arg("diff").arg("--").arg(file_path);
                let output = run_command(&mut command)?;
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            }
        }
    }

    fn log_entries(&self, repo_path: &Path, limit: usize) -> Result<Vec<LogEntry>> {
        let mut command = base_git_command(repo_path);
        command
            .arg("log")
            .arg("--format=%h%x00%s%x00%an%x00%ar%x00%at%x00%B%x00")
            .arg(format!("-n{limit}"))
            .arg("-z");
        let output = run_command(&mut command)?;
        let raw = String::from_utf8_lossy(&output.stdout);
        parse_log_output(&raw)
    }

    fn list_remotes(&self, repo_path: &Path) -> Result<Vec<RemoteInfo>> {
        let mut command = base_git_command(repo_path);
        command.arg("remote").arg("-v");
        let output = run_command(&mut command)?;
        let raw = String::from_utf8_lossy(&output.stdout);
        Ok(parse_remote_output(&raw))
    }

    fn has_remote(&self, repo_path: &Path, name: &str) -> bool {
        self.list_remotes(repo_path)
            .map(|remotes| remotes.iter().any(|r| r.name == name))
            .unwrap_or(false)
    }

    fn amend_commit(&self, repo_path: &Path, message: &CommitMessage) -> Result<()> {
        let mut command = base_git_command(repo_path);
        command.arg("commit").arg("--amend");
        for message_part in message.git_message_args() {
            command.arg("-m").arg(message_part);
        }
        run_command(&mut command).map(|_| ())
    }

    fn last_commit_message(&self, repo_path: &Path) -> Result<String> {
        let mut command = base_git_command(repo_path);
        command.arg("log").arg("-1").arg("--format=%B");
        let output = run_command(&mut command)?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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

fn parse_log_output(raw: &str) -> Result<Vec<LogEntry>> {
    let mut entries = Vec::new();
    // Records are separated by NUL. Each record has fields separated by NUL too.
    // Format: hash\0subject\0author\0date\0timestamp\0full_message\0 (then -z adds another \0 between records)
    let records: Vec<&str> = raw.split('\0').collect();
    // We get fields in groups of 6, with possible empty strings between records
    let fields: Vec<&str> = records
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut i = 0;
    while i + 5 < fields.len() {
        let hash = fields[i].to_string();
        let subject = fields[i + 1].to_string();
        let author = fields[i + 2].to_string();
        let date = fields[i + 3].to_string();
        let timestamp = fields[i + 4].parse::<i64>().unwrap_or(0);
        let full_message = fields[i + 5].to_string();
        entries.push(LogEntry {
            hash,
            subject,
            author,
            date,
            timestamp,
            full_message,
        });
        i += 6;
    }

    Ok(entries)
}

fn parse_remote_output(raw: &str) -> Vec<RemoteInfo> {
    use std::collections::HashMap;
    let mut fetch_map: HashMap<String, String> = HashMap::new();
    let mut push_map: HashMap<String, String> = HashMap::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Format: name\turl (fetch|push)
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let name = parts[0].to_string();
        let rest = parts[1];
        if let Some(url) = rest.strip_suffix(" (fetch)") {
            fetch_map.insert(name, url.to_string());
        } else if let Some(url) = rest.strip_suffix(" (push)") {
            push_map.insert(name, url.to_string());
        }
    }

    let mut names: Vec<String> = fetch_map.keys().chain(push_map.keys()).cloned().collect();
    names.sort();
    names.dedup();

    names
        .into_iter()
        .map(|name| {
            let fetch_url = fetch_map.get(&name).cloned().unwrap_or_default();
            let push_url = push_map.get(&name).cloned().unwrap_or(fetch_url.clone());
            RemoteInfo {
                name,
                fetch_url,
                push_url,
            }
        })
        .collect()
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
