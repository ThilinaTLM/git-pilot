use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};

use super::types::{CopilotTokenResponse, DeviceCodeResponse, OAuthTokenResponse, StoredAuth};

const CLIENT_ID: &str = "Iv1.b507a08c87ecfe98";
const SCOPE: &str = "read:user";

pub fn config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir().context("could not determine config directory")?;
    Ok(base.join("git-tui"))
}

fn auth_file_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("copilot_auth.json"))
}

pub fn load_auth() -> Result<StoredAuth> {
    let path = auth_file_path()?;
    let contents = fs::read_to_string(&path).context("no saved auth found")?;
    serde_json::from_str(&contents).context("invalid auth file")
}

pub fn save_auth(auth: &StoredAuth) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)?;
    let path = dir.join("copilot_auth.json");
    let json = serde_json::to_string_pretty(auth)?;
    fs::write(&path, &json)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

pub fn start_device_flow() -> Result<DeviceCodeResponse> {
    let form_data = vec![
        ("client_id".to_string(), CLIENT_ID.to_string()),
        ("scope".to_string(), SCOPE.to_string()),
    ];
    let mut response = ureq::post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .send_form(form_data)?;
    let body: DeviceCodeResponse = response.body_mut().read_json()?;
    Ok(body)
}

pub fn poll_for_oauth_token(device_code: &str, interval: u64) -> Result<String> {
    let poll_interval = Duration::from_secs(interval.max(5));
    loop {
        std::thread::sleep(poll_interval);

        let form_data = vec![
            ("client_id".to_string(), CLIENT_ID.to_string()),
            ("device_code".to_string(), device_code.to_string()),
            (
                "grant_type".to_string(),
                "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            ),
        ];
        let mut response = ureq::post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .send_form(form_data)?;
        let body: OAuthTokenResponse = response.body_mut().read_json()?;

        if let Some(token) = body.access_token {
            return Ok(token);
        }

        match body.error.as_deref() {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
            Some(error) => return Err(anyhow!("OAuth error: {error}")),
            None => return Err(anyhow!("unexpected OAuth response")),
        }
    }
}

pub fn exchange_copilot_token(oauth_token: &str) -> Result<CopilotTokenResponse> {
    let mut response = ureq::get("https://api.github.com/copilot_internal/v2/token")
        .header("Authorization", &format!("token {oauth_token}"))
        .header("Accept", "application/json")
        .header("User-Agent", "git-tui")
        .call()?;
    let body: CopilotTokenResponse = response.body_mut().read_json()?;
    Ok(body)
}

pub struct CopilotTokenManager {
    oauth_token: String,
    cached_jwt: Option<String>,
    expires_at: i64,
}

impl CopilotTokenManager {
    pub fn new(oauth_token: String) -> Self {
        Self {
            oauth_token,
            cached_jwt: None,
            expires_at: 0,
        }
    }

    pub fn get_token(&mut self) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Refresh if expired or within 5-minute buffer
        if self.cached_jwt.is_none() || now >= self.expires_at - 300 {
            let resp = exchange_copilot_token(&self.oauth_token)?;
            self.cached_jwt = Some(resp.token);
            self.expires_at = resp.expires_at;
        }

        self.cached_jwt
            .clone()
            .ok_or_else(|| anyhow!("no cached token"))
    }
}
