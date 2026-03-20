use std::path::Path;
use std::process::Command;

use anyhow::{Result, anyhow};
use serde::Deserialize;

use crate::domain::pull_request::{
    CheckConclusion, CheckStatus, CreatePrParams, PrCheckInfo, PrInfo, PrState,
};
use crate::domain::remote::{CreateRepoParams, RepoVisibility};
use crate::infrastructure::process::run_command;

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

#[derive(Debug, Default, Clone, Copy)]
pub struct GhCliGitHubService;

impl GhCliGitHubService {
    pub fn check_gh_auth(&self) -> Result<()> {
        let mut command = Command::new("gh");
        command.arg("auth").arg("status");
        run_command(&mut command)?;
        Ok(())
    }

    pub fn create_repo(&self, params: &CreateRepoParams) -> Result<String> {
        let visibility_flag = match params.visibility {
            RepoVisibility::Public => "--public",
            RepoVisibility::Private => "--private",
        };
        let repo_name = format!("{}/{}", params.owner, params.name);

        let mut command = Command::new("gh");
        command
            .arg("repo")
            .arg("create")
            .arg(&repo_name)
            .arg(visibility_flag)
            .arg(format!("--source={}", params.source_dir.display()))
            .arg(format!("--remote={}", params.remote_name));

        let output = run_command(&mut command)?;
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if url.is_empty() {
            return Err(anyhow!("gh repo create succeeded but returned no URL"));
        }
        Ok(url)
    }

    pub fn pr_checks(&self, repo_path: &Path, pr_number: u32) -> Result<Vec<PrCheckInfo>> {
        let mut command = Command::new("gh");
        command
            .current_dir(repo_path)
            .arg("pr")
            .arg("checks")
            .arg(pr_number.to_string())
            .arg("--json")
            .arg("name,state,conclusion");

        let output = match run_command(&mut command) {
            Ok(o) => o,
            Err(_) => return Ok(Vec::new()),
        };

        let raw = String::from_utf8_lossy(&output.stdout);
        let items: Vec<GhCheckJson> = serde_json::from_str(&raw).unwrap_or_default();
        Ok(items
            .into_iter()
            .map(|c| PrCheckInfo {
                name: c.name,
                status: match c.state.to_uppercase().as_str() {
                    "QUEUED" | "PENDING" | "WAITING" => CheckStatus::Queued,
                    "IN_PROGRESS" => CheckStatus::InProgress,
                    _ => CheckStatus::Completed,
                },
                conclusion: c
                    .conclusion
                    .as_deref()
                    .map(|s| match s.to_uppercase().as_str() {
                        "FAILURE" | "ACTION_REQUIRED" => CheckConclusion::Failure,
                        "CANCELLED" => CheckConclusion::Cancelled,
                        "SKIPPED" | "NEUTRAL" => CheckConclusion::Skipped,
                        "TIMED_OUT" | "STARTUP_FAILURE" => CheckConclusion::TimedOut,
                        _ => CheckConclusion::Success,
                    }),
            })
            .collect())
    }

    pub fn list_prs(&self, repo_path: &Path) -> Result<Vec<PrInfo>> {
        let mut command = Command::new("gh");
        command
            .current_dir(repo_path)
            .arg("pr")
            .arg("list")
            .arg("--json")
            .arg("number,title,state,url,headRefName")
            .arg("--state")
            .arg("open")
            .arg("--limit")
            .arg("50");

        let output = match run_command(&mut command) {
            Ok(o) => o,
            Err(_) => return Ok(Vec::new()),
        };

        let raw = String::from_utf8_lossy(&output.stdout);
        let items: Vec<GhPrJson> = serde_json::from_str(&raw).unwrap_or_default();
        Ok(items
            .into_iter()
            .map(|p| PrInfo {
                number: p.number,
                title: p.title,
                state: match p.state.to_uppercase().as_str() {
                    "MERGED" => PrState::Merged,
                    "CLOSED" => PrState::Closed,
                    _ => PrState::Open,
                },
                url: p.url,
                head_branch: p.head_ref_name,
                checks_passed: None,
            })
            .collect())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhPrJson {
    number: u32,
    title: String,
    state: String,
    url: String,
    head_ref_name: String,
}

#[derive(Deserialize)]
struct GhCheckJson {
    name: String,
    state: String,
    conclusion: Option<String>,
}
