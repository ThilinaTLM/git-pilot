#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrState {
    Open,
    Closed,
    Merged,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrInfo {
    pub number: u32,
    pub title: String,
    pub state: PrState,
    pub url: String,
    pub head_branch: String,
    pub checks_passed: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckStatus {
    Queued,
    InProgress,
    Completed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckConclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
    TimedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrCheckInfo {
    pub name: String,
    pub status: CheckStatus,
    pub conclusion: Option<CheckConclusion>,
}

pub struct CreatePrParams {
    pub title: String,
    pub body: String,
    pub base: String,
    pub head: String,
    pub draft: bool,
}
