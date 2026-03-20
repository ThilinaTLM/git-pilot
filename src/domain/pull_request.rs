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

pub struct CreatePrParams {
    pub title: String,
    pub body: String,
    pub base: String,
    pub head: String,
    pub draft: bool,
}
