use std::fs;
use std::path::Path;
use std::process::Command;

use git_pilot::domain::branch::BranchName;
use git_pilot::domain::commit::CommitMessage;
use git_pilot::domain::repo::{RepositoryId, RepositorySummary};
use git_pilot::infrastructure::git_cli::{GitCliRepositoryService, GitRepositoryService};
use git_pilot::infrastructure::process::run_command;
use tempfile::TempDir;

#[test]
fn git_cli_service_handles_stage_branch_and_commit_flow() {
    let tempdir = TempDir::new().expect("tempdir");
    init_repo(tempdir.path());
    fs::write(tempdir.path().join("hello.txt"), "hello\n").expect("write working file");

    let service = GitCliRepositoryService;

    service
        .stage_file(tempdir.path(), Path::new("hello.txt"))
        .expect("stage file");
    let after_stage = service
        .load_repository(&summary(tempdir.path()))
        .expect("load repository");
    assert!(after_stage.status.files.iter().any(|file| file.staged));

    let branch = BranchName::try_from("feature/demo".to_string()).expect("valid branch");
    service
        .create_branch(tempdir.path(), &branch)
        .expect("create branch");
    let after_branch = service
        .load_repository(&summary(tempdir.path()))
        .expect("load repository");
    assert_eq!(after_branch.current_branch.as_deref(), Some("feature/demo"));

    let message = CommitMessage::try_from("Initial commit".to_string()).expect("commit message");
    service
        .commit(tempdir.path(), &message)
        .expect("commit changes");
    let after_commit = service
        .load_repository(&summary(tempdir.path()))
        .expect("load repository");
    assert!(after_commit.status.files.is_empty());
}

fn init_repo(path: &Path) {
    run_git(path, ["init", "-b", "main"]);
    run_git(path, ["config", "user.name", "Test User"]);
    run_git(path, ["config", "user.email", "test@example.com"]);
    run_git(path, ["config", "commit.gpgsign", "false"]);
}

fn run_git<const N: usize>(path: &Path, args: [&str; N]) {
    let mut command = Command::new("git");
    command.arg("-C").arg(path).args(args);
    run_command(&mut command).expect("git command succeeds");
}

fn summary(path: &Path) -> RepositorySummary {
    let canonical = path.canonicalize().expect("canonical path");
    RepositorySummary {
        id: RepositoryId(canonical.clone()),
        name: canonical
            .file_name()
            .expect("repo dir name")
            .to_string_lossy()
            .to_string(),
        path: canonical.clone(),
        relative_path: canonical.file_name().expect("repo dir name").into(),
    }
}
