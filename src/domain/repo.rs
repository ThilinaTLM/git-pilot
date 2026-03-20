use std::path::PathBuf;

use crate::domain::branch::BranchInfo;
use crate::domain::commit::LogEntry;
use crate::domain::remote::RemoteInfo;
use crate::domain::status::RepositoryStatus;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RepositoryId(pub PathBuf);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositorySummary {
    pub id: RepositoryId,
    pub name: String,
    pub path: PathBuf,
    pub relative_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryDetails {
    pub summary: RepositorySummary,
    pub current_branch: Option<String>,
    pub branches: Vec<BranchInfo>,
    pub status: RepositoryStatus,
    pub log_entries: Vec<LogEntry>,
    pub remotes: Vec<RemoteInfo>,
}
