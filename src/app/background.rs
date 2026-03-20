use std::sync::mpsc;

use anyhow::Result;

use crate::domain::ai::GeneratedCommitMessage;

#[derive(Clone, Debug)]
pub struct DeviceCodeInfo {
    pub user_code: String,
    pub verification_uri: String,
}

pub enum BackgroundResult {
    CommitMessageGenerated(Result<GeneratedCommitMessage>),
    DeviceCodeReceived(Result<DeviceCodeInfo>),
    LoginCompleted(Result<()>),
}

pub fn create_channel() -> (
    mpsc::Sender<BackgroundResult>,
    mpsc::Receiver<BackgroundResult>,
) {
    mpsc::channel()
}
