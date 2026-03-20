use std::fmt;

use crate::domain::errors::ValidationError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BranchName(String);

impl BranchName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for BranchName {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyBranchName);
        }

        Ok(Self(trimmed.to_string()))
    }
}

impl fmt::Display for BranchName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchInfo {
    pub name: BranchName,
    pub is_current: bool,
}
