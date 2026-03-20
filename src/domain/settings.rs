use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub auto_fetch_enabled: bool,
    #[serde(default = "default_auto_fetch_interval")]
    pub auto_fetch_interval_secs: u64,
}

fn default_auto_fetch_interval() -> u64 {
    300
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_fetch_enabled: false,
            auto_fetch_interval_secs: default_auto_fetch_interval(),
        }
    }
}
