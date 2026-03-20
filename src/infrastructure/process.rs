use std::path::Path;
use std::process::{Command, Output};

use anyhow::{Context, Result, anyhow, bail};

pub fn run_command(command: &mut Command) -> Result<Output> {
    let output = command.output().context("failed to execute command")?;
    if output.status.success() {
        Ok(output)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let mut message = stderr;
        if message.is_empty() {
            message = stdout;
        }
        if message.is_empty() {
            bail!("command failed with status {}", output.status);
        }
        Err(anyhow!(message))
    }
}

pub fn base_git_command(repo_path: &Path) -> Command {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo_path);
    command
}
