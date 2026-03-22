use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

use anyhow::{Result, anyhow};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;

use crate::app::actions::{AppAction, map_key_event};
use crate::app::background::{BackgroundResult, DeviceCodeInfo, JobKind, create_channel};
use crate::app::state::{AppState, Modal, RepositoryState};
use crate::domain::branch::BranchName;
use crate::domain::commit::CommitMessage;
use crate::domain::status::FileSection;
use crate::infrastructure::ai::AiService;
use crate::infrastructure::config;
use crate::infrastructure::copilot::auth::{
    CopilotTokenManager, load_auth, poll_for_oauth_token, save_auth, start_device_flow,
};
use crate::infrastructure::copilot::client::CopilotAiService;
use crate::infrastructure::copilot::diff::prepare_diff_context;
use crate::infrastructure::copilot::types::StoredAuth;
use crate::infrastructure::git_cli::{GitCliRepositoryService, GitRepositoryService};
use crate::infrastructure::git_remote::{GitCliRemoteService, GitRemoteService};
use crate::infrastructure::github::GhCliGitHubService;
use crate::infrastructure::repo_discovery::{RepositoryDiscovery, WalkDirRepositoryDiscovery};
use crate::ui::mouse::map_mouse_event;

pub struct AppController {
    state: AppState,
    discovery: WalkDirRepositoryDiscovery,
    git: GitCliRepositoryService,
    remote: GitCliRemoteService,
    github: GhCliGitHubService,
    root: PathBuf,
    max_depth: usize,
    ai_service: Option<Arc<dyn AiService>>,
    bg_sender: mpsc::Sender<BackgroundResult>,
    bg_receiver: mpsc::Receiver<BackgroundResult>,
}

impl AppController {
    pub fn bootstrap(root: PathBuf) -> Result<Self> {
        let (bg_sender, bg_receiver) = create_channel();

        // Try to load saved auth and construct AI service
        let (ai_service, copilot_authenticated) = match load_auth() {
            Ok(auth) => {
                let token_manager = CopilotTokenManager::new(auth.oauth_token);
                let service = CopilotAiService::new(token_manager, "gpt-4o".to_string());
                (Some(Arc::new(service) as Arc<dyn AiService>), true)
            }
            Err(_) => (None, false),
        };

        let mut controller = Self {
            state: AppState::default(),
            discovery: WalkDirRepositoryDiscovery,
            git: GitCliRepositoryService,
            remote: GitCliRemoteService,
            github: GhCliGitHubService,
            root,
            max_depth: 3,
            ai_service,
            bg_sender,
            bg_receiver,
        };
        controller.state.copilot_authenticated = copilot_authenticated;
        controller.state.settings = config::load_settings();
        controller.refresh_repositories_sync()?;
        controller.load_tracking_status();
        Ok(controller)
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn tick_spinner(&mut self) {
        if self.state.has_active_jobs() {
            self.state.spinner_tick = self.state.spinner_tick.wrapping_add(1) % 10;
        }
    }

    pub fn auto_fetch(&mut self) {
        if self.state.selected_repo_path().is_some() && !self.state.is_job_running(&JobKind::Fetch)
        {
            let _ = self.fetch_remote();
        }
    }

    pub fn check_background_results(&mut self) {
        while let Ok(result) = self.bg_receiver.try_recv() {
            match result {
                BackgroundResult::BranchNameGenerated(job_id, Ok(name)) => {
                    self.state.finish_job(job_id);
                    self.state.branch_name_input = name;
                }
                BackgroundResult::BranchNameGenerated(job_id, Err(e)) => {
                    self.state.finish_job(job_id);
                    self.state.set_error(format!("AI branch name failed: {e}"));
                }
                BackgroundResult::CommitMessageGenerated(job_id, Ok(msg)) => {
                    self.state.finish_job(job_id);
                    let mut text = msg.subject;
                    if let Some(body) = msg.body {
                        text.push('\n');
                        text.push_str(&body);
                    }
                    self.state.commit_message_input = text;
                }
                BackgroundResult::CommitMessageGenerated(job_id, Err(e)) => {
                    self.state.finish_job(job_id);
                    self.state.set_error(format!("AI generation failed: {e}"));
                }
                BackgroundResult::DeviceCodeReceived(job_id, Ok(info)) => {
                    self.state.finish_job(job_id);
                    self.state.device_code = Some(info);
                    self.state.modal = Modal::CopilotLogin;
                }
                BackgroundResult::DeviceCodeReceived(job_id, Err(e)) => {
                    self.state.finish_job(job_id);
                    self.state.set_error(format!("Device flow failed: {e}"));
                }
                BackgroundResult::LoginCompleted(job_id, Ok(())) => {
                    self.state.finish_job(job_id);
                    if let Ok(auth) = load_auth() {
                        let token_manager = CopilotTokenManager::new(auth.oauth_token);
                        let service = CopilotAiService::new(token_manager, "gpt-4o".to_string());
                        self.ai_service = Some(Arc::new(service) as Arc<dyn AiService>);
                        self.state.copilot_authenticated = true;
                    }
                    self.state.device_code = None;
                    self.state.modal = Modal::None;
                    self.state.set_info("Logged in to GitHub Copilot");
                }
                BackgroundResult::LoginCompleted(job_id, Err(e)) => {
                    self.state.finish_job(job_id);
                    self.state.device_code = None;
                    self.state.modal = Modal::None;
                    self.state.set_error(format!("Copilot login failed: {e}"));
                }
                BackgroundResult::FetchCompleted(job_id, result) => {
                    self.state.finish_job(job_id);
                    match result {
                        Ok(()) => {
                            let _ = self.reload_selected_repo();
                            self.load_tracking_status();
                            self.state.set_info("Fetched from remote");
                        }
                        Err(e) => self.state.set_error(format!("Fetch failed: {e}")),
                    }
                }
                BackgroundResult::PushCompleted(job_id, result) => {
                    self.state.finish_job(job_id);
                    match result {
                        Ok(()) => {
                            let _ = self.reload_selected_repo();
                            self.load_tracking_status();
                            self.state.set_info("Pushed to remote");
                        }
                        Err(e) => self.state.set_error(format!("Push failed: {e}")),
                    }
                }
                BackgroundResult::PullCompleted(job_id, result) => {
                    self.state.finish_job(job_id);
                    match result {
                        Ok(()) => {
                            let _ = self.reload_selected_repo();
                            self.load_tracking_status();
                            self.state.set_info("Pulled from remote");
                        }
                        Err(e) => self.state.set_error(format!("Pull failed: {e}")),
                    }
                }
                BackgroundResult::PrsLoaded(job_id, result) => {
                    self.state.finish_job(job_id);
                    match result {
                        Ok(prs) => {
                            let count = prs.len();
                            if let Some(repo) = self.state.repos.get_mut(self.state.selected_repo) {
                                repo.pull_requests = prs;
                            }
                            self.load_pr_checks();
                            self.state.set_info(format!("Loaded {count} pull requests"));
                        }
                        Err(e) => {
                            self.state.set_error(format!("Failed to load PRs: {e}"));
                        }
                    }
                }
                BackgroundResult::PrChecksLoaded(job_id, result) => {
                    self.state.finish_job(job_id);
                    match result {
                        Ok(checks) => {
                            self.state.pr_checks_cache = checks;
                            self.state.pr_detail_scroll = 0;
                        }
                        Err(_) => {
                            self.state.pr_checks_cache.clear();
                        }
                    }
                }
                BackgroundResult::ReposRefreshed(job_id, result) => {
                    self.state.finish_job(job_id);
                    match result {
                        Ok(repos) => {
                            let selected_path = self.state.selected_repo_path();
                            let count = repos.len();
                            self.state.set_repositories(repos, selected_path);
                            if count == 0 {
                                self.state.set_info(
                                    "No Git repositories found in the current directory or descendants",
                                );
                            } else {
                                self.state.set_info(format!("Loaded {count} repositories"));
                            }
                        }
                        Err(e) => {
                            self.state
                                .set_error(format!("Failed to refresh repositories: {e}"));
                        }
                    }
                }
                BackgroundResult::RepoCreated(job_id, result) => {
                    self.state.finish_job(job_id);
                    match result {
                        Ok(url) => {
                            let _ = self.reload_selected_repo();
                            self.state.set_info(format!("Created repository: {url}"));
                        }
                        Err(e) => {
                            self.state
                                .set_error(format!("Failed to create repository: {e}"));
                        }
                    }
                }
            }
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let action = map_key_event(
            &self.state.active_view,
            &self.state.modal,
            key_event,
            self.state.branch_filter_active,
        );
        if let Err(error) = self.dispatch(action) {
            self.state.set_error(error.to_string());
        }
        Ok(())
    }

    pub fn handle_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        terminal_area: Rect,
    ) -> Result<()> {
        let action = map_mouse_event(mouse_event, terminal_area, &self.state);
        if let Err(error) = self.dispatch(action) {
            self.state.set_error(error.to_string());
        }
        Ok(())
    }

    fn dispatch(&mut self, action: AppAction) -> Result<()> {
        match action {
            AppAction::Noop => {}
            AppAction::Quit => self.state.should_quit = true,
            AppAction::RefreshRepos => self.refresh_repositories()?,
            AppAction::SelectNextRepo => self.state.select_next_repo(),
            AppAction::SelectPreviousRepo => self.state.select_previous_repo(),
            AppAction::SelectNextFile => self.state.select_next_file(),
            AppAction::SelectPreviousFile => self.state.select_previous_file(),
            AppAction::SelectNextBranch => self.state.select_next_branch(),
            AppAction::SelectPreviousBranch => self.state.select_previous_branch(),
            AppAction::StageSelected => self.stage_selected()?,
            AppAction::UnstageSelected => self.unstage_selected()?,
            AppAction::StageAll => self.stage_all()?,
            AppAction::UnstageAll => self.unstage_all()?,
            AppAction::ToggleStage => self.toggle_stage()?,
            AppAction::OpenBranches => self.state.open_branches(),
            AppAction::OpenBranchCreate => self.state.open_branch_create(),
            AppAction::OpenCommitLog => {
                self.state.modal = Modal::CommitLog;
                self.state.selected_log_entry = 0;
                self.state.log_scroll = 0;
            }
            AppAction::OpenSettings => {
                self.state.modal = Modal::Settings;
            }
            AppAction::OpenCommitPanel => self.state.open_commit_panel(),
            AppAction::OpenCommitAmend => self.open_commit_amend()?,
            AppAction::ConfirmModal => self.confirm_modal()?,
            AppAction::CloseModal => self.state.close_modal(),
            AppAction::ToggleHelp => self.state.show_help = !self.state.show_help,
            AppAction::InsertChar(ch) => self.insert_char(ch),
            AppAction::Backspace => self.backspace(),
            AppAction::InsertNewline => self.insert_newline(),
            AppAction::ScrollDiffDown => {
                self.state.diff_scroll = self.state.diff_scroll.saturating_add(5);
            }
            AppAction::ScrollDiffUp => {
                self.state.diff_scroll = self.state.diff_scroll.saturating_sub(5);
            }
            AppAction::SwitchView(view) => self.state.switch_view(view),
            AppAction::NextView => self.state.next_view(),
            AppAction::PreviousView => self.state.previous_view(),
            AppAction::DeleteBranch => self.delete_branch()?,
            AppAction::MergeBranch => self.merge_branch()?,
            AppAction::GenerateBranchName => self.generate_branch_name()?,
            AppAction::GenerateCommitMessage => self.generate_commit_message()?,
            AppAction::CopilotLogin => self.copilot_login(),
            AppAction::SelectNextLogEntry => self.state.select_next_log_entry(),
            AppAction::SelectPreviousLogEntry => self.state.select_previous_log_entry(),
            AppAction::SelectNextRemote => self.state.select_next_remote(),
            AppAction::SelectPreviousRemote => self.state.select_previous_remote(),
            AppAction::SelectNextPr => {
                self.state.select_next_pr();
                self.load_pr_checks();
            }
            AppAction::SelectPreviousPr => {
                self.state.select_previous_pr();
                self.load_pr_checks();
            }
            AppAction::OpenPrInBrowser => self.open_pr_in_browser()?,
            AppAction::RefreshPrs => self.refresh_prs()?,
            AppAction::ScrollPrDetailDown => {
                self.state.pr_detail_scroll = self.state.pr_detail_scroll.saturating_add(5);
            }
            AppAction::ScrollPrDetailUp => {
                self.state.pr_detail_scroll = self.state.pr_detail_scroll.saturating_sub(5);
            }
            AppAction::ScrollLogDown => {
                self.state.log_scroll = self.state.log_scroll.saturating_add(5);
            }
            AppAction::ScrollLogUp => {
                self.state.log_scroll = self.state.log_scroll.saturating_sub(5);
            }
            AppAction::OpenCreateRepo => self.open_create_repo()?,
            AppAction::ToggleVisibility => self.toggle_repo_visibility(),
            AppAction::SelectRepo(idx) => self.select_repo_by_index(idx),
            AppAction::CreateRepoNextStep => self.create_repo_advance()?,
            AppAction::CreateRepoPrevStep => self.create_repo_go_back(),
            AppAction::FetchRemote => self.fetch_remote()?,
            AppAction::PushBranch => self.push_branch()?,
            AppAction::PullBranch => self.pull_branch()?,
            AppAction::ToggleAutoFetch => self.toggle_auto_fetch()?,
            AppAction::IncreaseAutoFetchInterval => self.adjust_auto_fetch_interval(30)?,
            AppAction::DecreaseAutoFetchInterval => self.adjust_auto_fetch_interval(-30)?,
            AppAction::SelectNextSettingsItem => {
                // Settings has: 0=auto_fetch toggle, 1=interval
                self.state.selected_settings_item = (self.state.selected_settings_item + 1) % 2;
            }
            AppAction::SelectPreviousSettingsItem => {
                self.state.selected_settings_item = if self.state.selected_settings_item == 0 {
                    1
                } else {
                    self.state.selected_settings_item - 1
                };
            }
            AppAction::SelectFile(idx) => {
                let len = self.state.grouped_files.entries.len();
                if idx < len {
                    self.state.selected_file = idx;
                }
            }
            AppAction::SelectBranch(idx) => {
                if idx < self.state.filtered_branches.len() {
                    self.state.selected_branch = idx;
                }
            }
            AppAction::SelectLogEntry(idx) => {
                if let Some(repo) = self.state.selected_repo_ref()
                    && idx < repo.log_entries.len()
                {
                    self.state.selected_log_entry = idx;
                }
            }
            AppAction::SelectPr(idx) => {
                if let Some(repo) = self.state.selected_repo_ref()
                    && idx < repo.pull_requests.len()
                {
                    self.state.selected_pr = idx;
                    self.load_pr_checks();
                }
            }
            AppAction::SelectSettingsItem(idx) => {
                if idx < 2 {
                    self.state.selected_settings_item = idx;
                }
            }
            AppAction::ActivateBranchFilter => {
                self.state.branch_filter_active = true;
                self.state.branch_filter.clear();
                self.state.recompute_branch_filter();
            }
            AppAction::DeactivateBranchFilter => {
                self.state.branch_filter_active = false;
            }
            AppAction::ConfirmMerge => self.confirm_merge()?,
        }

        self.state.sync_selection();
        self.refresh_diff();
        Ok(())
    }

    fn generate_branch_name(&mut self) -> Result<()> {
        let ai = self
            .ai_service
            .clone()
            .ok_or_else(|| anyhow!("not logged in to Copilot — press Ctrl+l to login"))?;

        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;

        let context = prepare_diff_context(&self.git, &repo_path)?;
        if context.trim().is_empty() {
            return Err(anyhow!("no staged changes to generate a branch name for"));
        }

        let job_id = self.state.start_job(JobKind::AiBranchName);
        let sender = self.bg_sender.clone();

        thread::spawn(move || {
            let result = ai.generate_branch_name(&context);
            let _ = sender.send(BackgroundResult::BranchNameGenerated(job_id, result));
        });

        Ok(())
    }

    fn generate_commit_message(&mut self) -> Result<()> {
        let ai = self
            .ai_service
            .clone()
            .ok_or_else(|| anyhow!("not logged in to Copilot — press Ctrl+l to login"))?;

        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;

        let context = prepare_diff_context(&self.git, &repo_path)?;
        if context.trim().is_empty() {
            return Err(anyhow!("no staged changes to generate a message for"));
        }

        let job_id = self.state.start_job(JobKind::AiCommitMessage);
        let sender = self.bg_sender.clone();

        thread::spawn(move || {
            let result = ai.generate_commit_message(&context);
            let _ = sender.send(BackgroundResult::CommitMessageGenerated(job_id, result));
        });

        Ok(())
    }

    fn copilot_login(&mut self) {
        let device_job_id = self.state.start_job(JobKind::CopilotDeviceCode);
        let login_job_id = self.state.start_job(JobKind::CopilotLogin);
        let sender = self.bg_sender.clone();

        thread::spawn(move || {
            // Step 1: Start device flow
            let device_resp = match start_device_flow() {
                Ok(resp) => resp,
                Err(e) => {
                    let _ =
                        sender.send(BackgroundResult::DeviceCodeReceived(device_job_id, Err(e)));
                    return;
                }
            };

            // Send device code info to UI
            let _ = sender.send(BackgroundResult::DeviceCodeReceived(
                device_job_id,
                Ok(DeviceCodeInfo {
                    user_code: device_resp.user_code.clone(),
                    verification_uri: device_resp.verification_uri.clone(),
                }),
            ));

            // Step 2: Poll for OAuth token
            let oauth_token =
                match poll_for_oauth_token(&device_resp.device_code, device_resp.interval) {
                    Ok(token) => token,
                    Err(e) => {
                        let _ = sender.send(BackgroundResult::LoginCompleted(login_job_id, Err(e)));
                        return;
                    }
                };

            // Step 3: Save auth
            let auth = StoredAuth { oauth_token };
            if let Err(e) = save_auth(&auth) {
                let _ = sender.send(BackgroundResult::LoginCompleted(login_job_id, Err(e)));
                return;
            }

            let _ = sender.send(BackgroundResult::LoginCompleted(login_job_id, Ok(())));
        });
    }

    fn open_create_repo(&mut self) -> Result<()> {
        if let Some(repo) = self.state.selected_repo_ref()
            && repo.has_origin_remote
        {
            return Err(anyhow!("origin remote already exists"));
        }
        self.github.check_gh_auth()?;
        let default_name = self
            .state
            .selected_repo_ref()
            .map(|r| r.summary.name.clone())
            .unwrap_or_default();
        self.state.create_repo_state = Some(crate::app::state::CreateRepoState {
            step: crate::app::state::CreateRepoStep::Owner,
            owner_input: String::new(),
            repo_name_input: default_name,
            is_public: true,
        });
        self.state.modal = Modal::CreateRepo(crate::app::state::CreateRepoStep::Owner);
        Ok(())
    }

    fn create_repo_advance(&mut self) -> Result<()> {
        let Some(ref rs) = self.state.create_repo_state else {
            return Ok(());
        };
        let next_step = match rs.step {
            crate::app::state::CreateRepoStep::Owner => {
                if rs.owner_input.trim().is_empty() {
                    return Err(anyhow!("owner cannot be empty"));
                }
                crate::app::state::CreateRepoStep::RepoName
            }
            crate::app::state::CreateRepoStep::RepoName => {
                if rs.repo_name_input.trim().is_empty() {
                    return Err(anyhow!("repository name cannot be empty"));
                }
                crate::app::state::CreateRepoStep::Visibility
            }
            crate::app::state::CreateRepoStep::Visibility => {
                crate::app::state::CreateRepoStep::Confirm
            }
            crate::app::state::CreateRepoStep::Confirm => {
                return self.confirm_create_repo();
            }
        };
        if let Some(ref mut rs) = self.state.create_repo_state {
            rs.step = next_step.clone();
        }
        self.state.modal = Modal::CreateRepo(next_step);
        Ok(())
    }

    fn create_repo_go_back(&mut self) {
        let Some(ref rs) = self.state.create_repo_state else {
            self.state.close_modal();
            return;
        };
        let prev_step = match rs.step {
            crate::app::state::CreateRepoStep::Owner => {
                self.state.close_modal();
                return;
            }
            crate::app::state::CreateRepoStep::RepoName => crate::app::state::CreateRepoStep::Owner,
            crate::app::state::CreateRepoStep::Visibility => {
                crate::app::state::CreateRepoStep::RepoName
            }
            crate::app::state::CreateRepoStep::Confirm => {
                crate::app::state::CreateRepoStep::Visibility
            }
        };
        if let Some(ref mut rs) = self.state.create_repo_state {
            rs.step = prev_step.clone();
        }
        self.state.modal = Modal::CreateRepo(prev_step);
    }

    fn toggle_repo_visibility(&mut self) {
        if let Some(ref mut rs) = self.state.create_repo_state {
            rs.is_public = !rs.is_public;
        }
    }

    fn confirm_create_repo(&mut self) -> Result<()> {
        let rs = self
            .state
            .create_repo_state
            .clone()
            .ok_or_else(|| anyhow!("no create repo state"))?;
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;

        let params = crate::domain::remote::CreateRepoParams {
            owner: rs.owner_input.trim().to_string(),
            name: rs.repo_name_input.trim().to_string(),
            visibility: if rs.is_public {
                crate::domain::remote::RepoVisibility::Public
            } else {
                crate::domain::remote::RepoVisibility::Private
            },
            source_dir: repo_path,
            remote_name: "origin".to_string(),
        };

        self.state.close_modal();
        let job_id = self.state.start_job(JobKind::CreateRepo);
        let sender = self.bg_sender.clone();
        let github = self.github;
        thread::spawn(move || {
            let result = github.create_repo(&params);
            let _ = sender.send(BackgroundResult::RepoCreated(job_id, result));
        });
        Ok(())
    }

    fn toggle_auto_fetch(&mut self) -> Result<()> {
        self.state.settings.auto_fetch_enabled = !self.state.settings.auto_fetch_enabled;
        config::save_settings(&self.state.settings)?;
        let status = if self.state.settings.auto_fetch_enabled {
            "enabled"
        } else {
            "disabled"
        };
        self.state.set_info(format!("Auto-fetch {status}"));
        Ok(())
    }

    fn adjust_auto_fetch_interval(&mut self, delta: i64) -> Result<()> {
        let current = self.state.settings.auto_fetch_interval_secs as i64;
        let new_val = (current + delta).clamp(30, 600) as u64;
        self.state.settings.auto_fetch_interval_secs = new_val;
        config::save_settings(&self.state.settings)?;
        self.state
            .set_info(format!("Auto-fetch interval: {new_val}s"));
        Ok(())
    }

    fn fetch_remote(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        if self.state.is_job_running(&JobKind::Fetch) {
            return Ok(());
        }
        let job_id = self.state.start_job(JobKind::Fetch);
        let sender = self.bg_sender.clone();
        let remote = self.remote;
        thread::spawn(move || {
            let result = remote.fetch(&repo_path);
            let _ = sender.send(BackgroundResult::FetchCompleted(job_id, result));
        });
        Ok(())
    }

    fn push_branch(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let branch = self
            .state
            .selected_repo_ref()
            .and_then(|r| r.current_branch.clone())
            .ok_or_else(|| anyhow!("no current branch"))?;
        if self.state.is_job_running(&JobKind::Push) {
            return Ok(());
        }
        let job_id = self.state.start_job(JobKind::Push);
        let sender = self.bg_sender.clone();
        let remote = self.remote;
        thread::spawn(move || {
            let result = remote.push(&repo_path, &branch);
            let _ = sender.send(BackgroundResult::PushCompleted(job_id, result));
        });
        Ok(())
    }

    fn pull_branch(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        if self.state.is_job_running(&JobKind::Pull) {
            return Ok(());
        }
        let job_id = self.state.start_job(JobKind::Pull);
        let sender = self.bg_sender.clone();
        let remote = self.remote;
        thread::spawn(move || {
            let result = remote.pull(&repo_path);
            let _ = sender.send(BackgroundResult::PullCompleted(job_id, result));
        });
        Ok(())
    }

    fn load_tracking_status(&mut self) {
        let Some(repo_path) = self.state.selected_repo_path() else {
            self.state.branch_tracking = None;
            return;
        };
        let Some(branch) = self
            .state
            .selected_repo_ref()
            .and_then(|r| r.current_branch.clone())
        else {
            self.state.branch_tracking = None;
            return;
        };
        self.state.branch_tracking = self.remote.tracking_status(&repo_path, &branch).ok();
    }

    fn open_pr_in_browser(&mut self) -> Result<()> {
        let repo = self
            .state
            .selected_repo_ref()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let pr = repo
            .pull_requests
            .get(self.state.selected_pr)
            .ok_or_else(|| anyhow!("no PR selected"))?;
        let repo_path = repo.summary.path.clone();
        let number = pr.number;
        let mut command = std::process::Command::new("gh");
        command
            .current_dir(&repo_path)
            .arg("pr")
            .arg("view")
            .arg(number.to_string())
            .arg("--web");
        crate::infrastructure::process::run_command(&mut command)?;
        self.state
            .set_info(format!("Opened PR #{number} in browser"));
        Ok(())
    }

    fn refresh_prs(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        if self.state.is_job_running(&JobKind::ListPrs) {
            return Ok(());
        }
        let job_id = self.state.start_job(JobKind::ListPrs);
        let sender = self.bg_sender.clone();
        let github = self.github;
        thread::spawn(move || {
            let result = github.list_prs(&repo_path);
            let _ = sender.send(BackgroundResult::PrsLoaded(job_id, result));
        });
        Ok(())
    }

    fn load_pr_checks(&mut self) {
        let Some(repo_path) = self.state.selected_repo_path() else {
            self.state.pr_checks_cache.clear();
            return;
        };
        let Some(repo) = self.state.selected_repo_ref() else {
            self.state.pr_checks_cache.clear();
            return;
        };
        let Some(pr) = repo.pull_requests.get(self.state.selected_pr) else {
            self.state.pr_checks_cache.clear();
            return;
        };
        let number = pr.number;
        if self.state.is_job_running(&JobKind::PrChecks) {
            return;
        }
        let job_id = self.state.start_job(JobKind::PrChecks);
        let sender = self.bg_sender.clone();
        let github = self.github;
        thread::spawn(move || {
            let result = github.pr_checks(&repo_path, number);
            let _ = sender.send(BackgroundResult::PrChecksLoaded(job_id, result));
        });
    }

    fn select_repo_by_index(&mut self, idx: usize) {
        if idx < self.state.repos.len() {
            self.state.selected_repo = idx;
            self.state.selected_file = 0;
            self.state.selected_branch = self.state.current_branch_index().unwrap_or(0);
            self.state.diff_content = None;
            self.state.diff_scroll = 0;
        }
    }

    fn refresh_repositories(&mut self) -> Result<()> {
        if self.state.is_job_running(&JobKind::RefreshRepos) {
            return Ok(());
        }
        let job_id = self.state.start_job(JobKind::RefreshRepos);
        let sender = self.bg_sender.clone();
        let root = self.root.clone();
        let max_depth = self.max_depth;
        let discovery = self.discovery;
        let git = self.git;
        let github = self.github;
        thread::spawn(move || {
            let result = (|| -> Result<Vec<RepositoryState>> {
                let summaries = discovery.discover(&root, max_depth)?;
                let repos: Vec<RepositoryState> = summaries
                    .into_iter()
                    .map(|summary| {
                        let path = summary.path.clone();
                        let mut repo_state = match git.load_repository(&summary) {
                            Ok(details) => RepositoryState::from_details(details),
                            Err(error) => RepositoryState::from_error(summary, error.to_string()),
                        };
                        repo_state.pull_requests = github.list_prs(&path).unwrap_or_default();
                        repo_state
                    })
                    .collect();
                Ok(repos)
            })();
            let _ = sender.send(BackgroundResult::ReposRefreshed(job_id, result));
        });
        Ok(())
    }

    fn refresh_repositories_sync(&mut self) -> Result<()> {
        let selected_path = self.state.selected_repo_path();
        let summaries = self.discovery.discover(&self.root, self.max_depth)?;
        let repos: Vec<RepositoryState> = summaries
            .into_iter()
            .map(|summary| {
                let path = summary.path.clone();
                let mut repo_state = match self.git.load_repository(&summary) {
                    Ok(details) => RepositoryState::from_details(details),
                    Err(error) => RepositoryState::from_error(summary, error.to_string()),
                };
                repo_state.pull_requests = self.github.list_prs(&path).unwrap_or_default();
                repo_state
            })
            .collect();

        self.state.set_repositories(repos, selected_path);
        if self.state.repos.is_empty() {
            self.state
                .set_info("No Git repositories found in the current directory or descendants");
        } else {
            self.state
                .set_info(format!("Loaded {} repositories", self.state.repos.len()));
        }
        Ok(())
    }

    fn reload_selected_repo(&mut self) -> Result<()> {
        let summary = self
            .state
            .selected_repo_ref()
            .map(|repo| repo.summary.clone())
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let details = self.git.load_repository(&summary)?;
        let mut repo_state = RepositoryState::from_details(details);
        repo_state.pull_requests = self.github.list_prs(&summary.path).unwrap_or_default();
        self.state.replace_selected_repository(repo_state);
        Ok(())
    }

    fn toggle_stage(&mut self) -> Result<()> {
        let section = self
            .state
            .selected_file_section()
            .ok_or_else(|| anyhow!("no file selected"))?;
        if section == FileSection::Staged {
            self.unstage_selected()
        } else {
            self.stage_selected()
        }
    }

    fn stage_selected(&mut self) -> Result<()> {
        let section = self
            .state
            .selected_file_section()
            .ok_or_else(|| anyhow!("no file selected"))?;
        if section == FileSection::Staged {
            return Ok(());
        }
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let file_path = self
            .state
            .selected_file_path()
            .ok_or_else(|| anyhow!("no file selected"))?;
        self.git.stage_file(&repo_path, file_path)?;
        self.reload_selected_repo()?;
        self.state.set_info("Staged selected file");
        Ok(())
    }

    fn unstage_selected(&mut self) -> Result<()> {
        let section = self
            .state
            .selected_file_section()
            .ok_or_else(|| anyhow!("no file selected"))?;
        if section != FileSection::Staged {
            return Ok(());
        }
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let file_path = self
            .state
            .selected_file_path()
            .ok_or_else(|| anyhow!("no file selected"))?;
        self.git.unstage_file(&repo_path, file_path)?;
        self.reload_selected_repo()?;
        self.state.set_info("Unstaged selected file");
        Ok(())
    }

    fn stage_all(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        self.git.stage_all(&repo_path)?;
        self.reload_selected_repo()?;
        self.state.set_info("Staged all changes");
        Ok(())
    }

    fn unstage_all(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        self.git.unstage_all(&repo_path)?;
        self.reload_selected_repo()?;
        self.state.set_info("Unstaged all changes");
        Ok(())
    }

    fn delete_branch(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let branch_name = self
            .state
            .selected_branch_name()
            .ok_or_else(|| anyhow!("no branch selected"))?
            .to_string();

        if let Some(repo) = self.state.selected_repo_ref()
            && repo
                .current_branch
                .as_deref()
                .is_some_and(|current| current == branch_name)
        {
            return Err(anyhow!("cannot delete the current branch"));
        }

        let branch = BranchName::try_from(branch_name)?;
        self.git.delete_branch(&repo_path, &branch)?;
        self.reload_selected_repo()?;
        self.state
            .set_info(format!("Deleted branch {}", branch.as_str()));
        Ok(())
    }

    fn merge_branch(&mut self) -> Result<()> {
        let branch_name = self
            .state
            .selected_branch_name()
            .ok_or_else(|| anyhow!("no branch selected"))?
            .to_string();
        self.state.merge_confirm_branch = Some(branch_name);
        self.state.modal = Modal::MergeConfirm;
        Ok(())
    }

    fn confirm_merge(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let branch_name = self
            .state
            .merge_confirm_branch
            .take()
            .ok_or_else(|| anyhow!("no branch to merge"))?;
        let branch = BranchName::try_from(branch_name)?;
        self.git.merge_branch(&repo_path, &branch)?;
        self.reload_selected_repo()?;
        self.state.modal = Modal::Branches;
        self.state.set_info(format!("Merged {}", branch.as_str()));
        Ok(())
    }

    fn confirm_modal(&mut self) -> Result<()> {
        match self.state.modal {
            Modal::None => Ok(()),
            Modal::Branches => self.confirm_branch_switch(),
            Modal::BranchCreate => self.confirm_branch_create(),
            Modal::MergeConfirm => self.confirm_merge(),
            Modal::CommitLog | Modal::Settings => Ok(()),
            Modal::Commit => self.confirm_commit(),
            Modal::CopilotLogin => Ok(()),
            Modal::CreateRepo(_) => self.confirm_create_repo(),
        }
    }

    fn confirm_branch_switch(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let branch_name = self
            .state
            .selected_branch_name()
            .ok_or_else(|| anyhow!("no branch selected"))?
            .to_string();
        let branch = BranchName::try_from(branch_name)?;
        self.git.switch_branch(&repo_path, &branch)?;
        self.reload_selected_repo()?;
        self.state
            .set_info(format!("Switched to {}", branch.as_str()));
        Ok(())
    }

    fn confirm_branch_create(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let branch = BranchName::try_from(self.state.branch_name_input.clone())?;
        self.git.create_branch(&repo_path, &branch)?;
        self.reload_selected_repo()?;
        self.state.close_modal();
        self.state
            .set_info(format!("Created and switched to {}", branch.as_str()));
        Ok(())
    }

    fn open_commit_amend(&mut self) -> Result<()> {
        let repo_path = self
            .state
            .selected_repo_path()
            .ok_or_else(|| anyhow!("no repository selected"))?;
        let last_message = self.git.last_commit_message(&repo_path)?;
        if last_message.is_empty() {
            return Err(anyhow!("no previous commit to amend"));
        }
        self.state.amend_mode = true;
        self.state.commit_message_input = last_message;
        self.state.modal = Modal::Commit;
        Ok(())
    }

    fn confirm_commit(&mut self) -> Result<()> {
        let repo = self
            .state
            .selected_repo_ref()
            .ok_or_else(|| anyhow!("no repository selected"))?;

        let amend = self.state.amend_mode;

        if !amend && repo.status_files.iter().all(|file| !file.staged) {
            return Err(anyhow!("no staged changes to commit"));
        }

        let message = CommitMessage::try_from(self.state.commit_message_input.clone())?;
        if amend {
            self.git.amend_commit(&repo.summary.path, &message)?;
        } else {
            self.git.commit(&repo.summary.path, &message)?;
        }
        self.reload_selected_repo()?;
        self.state.close_modal();
        let verb = if amend { "Amended" } else { "Committed" };
        self.state.set_info(format!("{verb} {}", message.subject()));
        Ok(())
    }

    fn insert_char(&mut self, ch: char) {
        match &self.state.modal {
            Modal::BranchCreate => {
                let ch = if ch == ' ' { '-' } else { ch };
                self.state.branch_name_input.push(ch);
            }
            Modal::Commit => self.state.commit_message_input.push(ch),
            Modal::CreateRepo(step) => match step {
                crate::app::state::CreateRepoStep::Owner => {
                    if let Some(ref mut rs) = self.state.create_repo_state {
                        rs.owner_input.push(ch);
                    }
                }
                crate::app::state::CreateRepoStep::RepoName => {
                    if let Some(ref mut rs) = self.state.create_repo_state {
                        rs.repo_name_input.push(ch);
                    }
                }
                _ => {}
            },
            Modal::Branches if self.state.branch_filter_active => {
                self.state.branch_filter.push(ch);
                self.state.recompute_branch_filter();
            }
            Modal::None
            | Modal::CopilotLogin
            | Modal::Branches
            | Modal::MergeConfirm
            | Modal::CommitLog
            | Modal::Settings => {}
        }
    }

    fn backspace(&mut self) {
        match &self.state.modal {
            Modal::BranchCreate => {
                self.state.branch_name_input.pop();
            }
            Modal::Commit => {
                self.state.commit_message_input.pop();
            }
            Modal::CreateRepo(step) => match step {
                crate::app::state::CreateRepoStep::Owner => {
                    if let Some(ref mut rs) = self.state.create_repo_state {
                        rs.owner_input.pop();
                    }
                }
                crate::app::state::CreateRepoStep::RepoName => {
                    if let Some(ref mut rs) = self.state.create_repo_state {
                        rs.repo_name_input.pop();
                    }
                }
                _ => {}
            },
            Modal::Branches if self.state.branch_filter_active => {
                self.state.branch_filter.pop();
                self.state.recompute_branch_filter();
            }
            Modal::None
            | Modal::CopilotLogin
            | Modal::Branches
            | Modal::MergeConfirm
            | Modal::CommitLog
            | Modal::Settings => {}
        }
    }

    fn insert_newline(&mut self) {
        if matches!(self.state.modal, Modal::Commit) {
            self.state.commit_message_input.push('\n');
        }
    }

    fn refresh_diff(&mut self) {
        let Some(repo_path) = self.state.selected_repo_path() else {
            self.state.diff_content = None;
            return;
        };
        let Some(entry) = self.state.selected_file_entry().cloned() else {
            self.state.diff_content = None;
            return;
        };
        let Some(file_path) = self
            .state
            .selected_repo_ref()
            .and_then(|repo| repo.status_files.get(entry.file_index))
            .map(|f| f.path.clone())
        else {
            self.state.diff_content = None;
            return;
        };

        match self.git.diff_file(&repo_path, &file_path, entry.section) {
            Ok(diff) if diff.is_empty() => self.state.diff_content = None,
            Ok(diff) => self.state.diff_content = Some(diff),
            Err(_) => self.state.diff_content = None,
        }
    }
}
