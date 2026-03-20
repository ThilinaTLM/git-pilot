use std::path::PathBuf;

pub struct TrackingStatus {
    pub ahead: u32,
    pub behind: u32,
    pub remote_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteInfo {
    pub name: String,
    pub fetch_url: String,
    pub push_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepoVisibility {
    Public,
    Private,
}

#[derive(Clone, Debug)]
pub struct CreateRepoParams {
    pub owner: String,
    pub name: String,
    pub visibility: RepoVisibility,
    pub source_dir: PathBuf,
    pub remote_name: String,
}
