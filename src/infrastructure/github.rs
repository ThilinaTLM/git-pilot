use std::path::Path;

use anyhow::Result;

use crate::domain::pull_request::{CreatePrParams, PrInfo};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MergeStrategy {
    Merge,
    Squash,
    Rebase,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckRun {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
}

pub trait GitHubService {
    fn create_pr(&self, repo_path: &Path, params: &CreatePrParams) -> Result<PrInfo>;
    fn list_prs(&self, repo_path: &Path) -> Result<Vec<PrInfo>>;
    fn pr_checks(&self, repo_path: &Path, pr_number: u32) -> Result<Vec<CheckRun>>;
    fn merge_pr(&self, repo_path: &Path, pr_number: u32, strategy: &MergeStrategy) -> Result<()>;
}
