use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::Instant;

use anyhow::Result;

use crate::app::state::RepositoryState;
use crate::domain::ai::{GeneratedCommitMessage, GeneratedPrDescription};
use crate::domain::pull_request::{PrCheckInfo, PrInfo};

static NEXT_JOB_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JobId(u64);

impl JobId {
    pub fn next() -> Self {
        Self(NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JobKind {
    AiBranchName,
    AiCommitMessage,
    CopilotDeviceCode,
    CopilotLogin,
    Fetch,
    Push,
    Pull,
    ListPrs,
    PrChecks,
    RefreshRepos,
    ReloadRepo,
    CreateRepo,
    AiPrDescription,
    CreatePr,
}

impl JobKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AiBranchName => "Generating branch name",
            Self::AiCommitMessage => "Generating commit message",
            Self::CopilotDeviceCode => "Starting Copilot login",
            Self::CopilotLogin => "Completing Copilot login",
            Self::Fetch => "Fetching",
            Self::Push => "Pushing",
            Self::Pull => "Pulling",
            Self::ListPrs => "Loading PRs",
            Self::PrChecks => "Loading PR checks",
            Self::RefreshRepos => "Refreshing repositories",
            Self::ReloadRepo => "Reloading repository",
            Self::CreateRepo => "Creating repository",
            Self::AiPrDescription => "Generating PR description",
            Self::CreatePr => "Creating pull request",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ActiveJob {
    pub id: JobId,
    pub kind: JobKind,
    pub started_at: Instant,
}

#[derive(Clone, Debug)]
pub struct DeviceCodeInfo {
    pub user_code: String,
    pub verification_uri: String,
}

pub enum BackgroundResult {
    BranchNameGenerated(JobId, Result<String>),
    CommitMessageGenerated(JobId, Result<GeneratedCommitMessage>),
    DeviceCodeReceived(JobId, Result<DeviceCodeInfo>),
    LoginCompleted(JobId, Result<()>),
    FetchCompleted(JobId, Result<()>),
    PushCompleted(JobId, Result<()>),
    PullCompleted(JobId, Result<()>),
    PrsLoaded(JobId, Result<Vec<PrInfo>>),
    PrChecksLoaded(JobId, Result<Vec<PrCheckInfo>>),
    ReposRefreshed(JobId, Result<Vec<RepositoryState>>),
    RepoCreated(JobId, Result<String>),
    PrDescriptionGenerated(JobId, Result<GeneratedPrDescription>),
    PrCreated(JobId, Result<PrInfo>),
}

pub fn create_channel() -> (
    mpsc::Sender<BackgroundResult>,
    mpsc::Receiver<BackgroundResult>,
) {
    mpsc::channel()
}
