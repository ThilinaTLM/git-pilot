use std::path::Path;

use anyhow::Result;

use crate::domain::remote::TrackingStatus;

pub trait GitRemoteService {
    fn fetch(&self, repo_path: &Path) -> Result<()>;
    fn push(&self, repo_path: &Path, branch: &str) -> Result<()>;
    fn pull(&self, repo_path: &Path) -> Result<()>;
    fn tracking_status(&self, repo_path: &Path, branch: &str) -> Result<TrackingStatus>;
}
