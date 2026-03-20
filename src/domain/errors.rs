use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("branch name cannot be empty")]
    EmptyBranchName,
    #[error("commit message must include a non-empty subject line")]
    EmptyCommitMessage,
}
