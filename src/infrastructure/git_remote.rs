use std::path::Path;

use anyhow::Result;

use crate::domain::remote::TrackingStatus;
use crate::infrastructure::process::{base_git_command, run_command};

pub trait GitRemoteService {
    fn fetch(&self, repo_path: &Path) -> Result<()>;
    fn push(&self, repo_path: &Path, branch: &str) -> Result<()>;
    fn pull(&self, repo_path: &Path) -> Result<()>;
    fn tracking_status(&self, repo_path: &Path, branch: &str) -> Result<TrackingStatus>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GitCliRemoteService;

impl GitRemoteService for GitCliRemoteService {
    fn fetch(&self, repo_path: &Path) -> Result<()> {
        let mut command = base_git_command(repo_path);
        command.arg("fetch");
        run_command(&mut command).map(|_| ())
    }

    fn push(&self, repo_path: &Path, branch: &str) -> Result<()> {
        let mut command = base_git_command(repo_path);
        command.arg("push").arg("origin").arg(branch);
        run_command(&mut command).map(|_| ())
    }

    fn pull(&self, repo_path: &Path) -> Result<()> {
        let mut command = base_git_command(repo_path);
        command.arg("pull");
        run_command(&mut command).map(|_| ())
    }

    fn tracking_status(&self, repo_path: &Path, branch: &str) -> Result<TrackingStatus> {
        // Get the upstream ref name
        let mut upstream_cmd = base_git_command(repo_path);
        upstream_cmd
            .arg("config")
            .arg(format!("branch.{branch}.remote"));
        let remote_name = match run_command(&mut upstream_cmd) {
            Ok(output) => {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if name.is_empty() { None } else { Some(name) }
            }
            Err(_) => None,
        };

        // Get ahead/behind counts
        let mut command = base_git_command(repo_path);
        command
            .arg("rev-list")
            .arg("--left-right")
            .arg("--count")
            .arg(format!("{branch}...@{{upstream}}"));

        match run_command(&mut command) {
            Ok(output) => {
                let raw = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = raw.trim().split('\t').collect();
                let ahead = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                let behind = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                Ok(TrackingStatus {
                    ahead,
                    behind,
                    remote_name,
                })
            }
            Err(_) => Ok(TrackingStatus {
                ahead: 0,
                behind: 0,
                remote_name: None,
            }),
        }
    }
}
