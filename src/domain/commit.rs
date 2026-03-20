use crate::domain::errors::ValidationError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogEntry {
    pub hash: String,
    pub subject: String,
    pub author: String,
    pub date: String,
    pub full_message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitMessage {
    raw: String,
}

impl CommitMessage {
    pub fn subject(&self) -> &str {
        self.raw
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or_default()
            .trim()
    }

    pub fn body(&self) -> Option<String> {
        let mut lines = self.raw.lines();
        let _ = lines.find(|line| !line.trim().is_empty())?;
        let body = lines.collect::<Vec<_>>().join("\n").trim().to_string();
        if body.is_empty() { None } else { Some(body) }
    }

    pub fn git_message_args(&self) -> Vec<String> {
        let mut args = vec![self.subject().to_string()];
        if let Some(body) = self.body() {
            for paragraph in body.split("\n\n") {
                let paragraph = paragraph.trim();
                if !paragraph.is_empty() {
                    args.push(paragraph.to_string());
                }
            }
        }
        args
    }
}

impl TryFrom<String> for CommitMessage {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.lines().any(|line| !line.trim().is_empty()) {
            Ok(Self { raw: value })
        } else {
            Err(ValidationError::EmptyCommitMessage)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CommitMessage;

    #[test]
    fn builds_git_message_args_from_subject_and_body() {
        let message = CommitMessage::try_from("Subject\n\nBody one\n\nBody two".to_string())
            .expect("valid commit message");

        assert_eq!(
            message.git_message_args(),
            vec![
                "Subject".to_string(),
                "Body one".to_string(),
                "Body two".to_string()
            ]
        );
    }
}
