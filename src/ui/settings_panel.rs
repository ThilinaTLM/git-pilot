use ratatui::prelude::*;
use ratatui::widgets::{Clear, List, ListItem, ListState, Paragraph};

use crate::app::state::AppState;
use crate::ui::layout::centered_rect;
use crate::ui::theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsItem {
    FetchEnabled,
    FetchInterval,
    StatusEnabled,
    StatusInterval,
    BranchesEnabled,
    BranchesInterval,
    PrsEnabled,
    PrsInterval,
    AiBranchInstructions,
    AiCommitInstructions,
    AiPrTitleInstructions,
    AiPrDescriptionInstructions,
    DraftByDefault,
    SubjectMaxLength,
    DefaultView,
}

impl SettingsItem {
    const ALL: &[SettingsItem] = &[
        Self::FetchEnabled,
        Self::FetchInterval,
        Self::StatusEnabled,
        Self::StatusInterval,
        Self::BranchesEnabled,
        Self::BranchesInterval,
        Self::PrsEnabled,
        Self::PrsInterval,
        Self::AiBranchInstructions,
        Self::AiCommitInstructions,
        Self::AiPrTitleInstructions,
        Self::AiPrDescriptionInstructions,
        Self::DraftByDefault,
        Self::SubjectMaxLength,
        Self::DefaultView,
    ];

    pub fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or(Self::FetchEnabled)
    }

    fn label(&self, state: &AppState) -> String {
        let ar = &state.settings.auto_refresh;
        match self {
            Self::FetchEnabled => format!("Fetch: {}", if ar.fetch_enabled { "ON" } else { "OFF" }),
            Self::FetchInterval => format!("Fetch interval: {}s", ar.fetch_interval_secs),
            Self::StatusEnabled => {
                format!("Status: {}", if ar.status_enabled { "ON" } else { "OFF" })
            }
            Self::StatusInterval => format!("Status interval: {}s", ar.status_interval_secs),
            Self::BranchesEnabled => format!(
                "Branches: {}",
                if ar.branches_enabled { "ON" } else { "OFF" }
            ),
            Self::BranchesInterval => {
                format!("Branches interval: {}s", ar.branches_interval_secs)
            }
            Self::PrsEnabled => {
                format!("PRs: {}", if ar.prs_enabled { "ON" } else { "OFF" })
            }
            Self::PrsInterval => format!("PRs interval: {}s", ar.prs_interval_secs),
            Self::AiBranchInstructions => "Branch name instructions".to_string(),
            Self::AiCommitInstructions => "Commit message instructions".to_string(),
            Self::AiPrTitleInstructions => "PR title instructions".to_string(),
            Self::AiPrDescriptionInstructions => "PR description instructions".to_string(),
            Self::DraftByDefault => format!(
                "Draft by default: {}",
                if state.settings.pull_requests.draft_by_default {
                    "ON"
                } else {
                    "OFF"
                }
            ),
            Self::SubjectMaxLength => {
                format!(
                    "Subject max length: {}",
                    state.settings.commit.subject_max_length
                )
            }
            Self::DefaultView => format!("Default view: {}", state.settings.ui.default_view),
        }
    }

    fn section(&self) -> &'static str {
        match self {
            Self::FetchEnabled
            | Self::FetchInterval
            | Self::StatusEnabled
            | Self::StatusInterval
            | Self::BranchesEnabled
            | Self::BranchesInterval
            | Self::PrsEnabled
            | Self::PrsInterval => "Auto-refresh",
            Self::AiBranchInstructions
            | Self::AiCommitInstructions
            | Self::AiPrTitleInstructions
            | Self::AiPrDescriptionInstructions => "AI Instructions",
            Self::DraftByDefault => "Pull Requests",
            Self::SubjectMaxLength => "Commit",
            Self::DefaultView => "UI",
        }
    }
}

pub const SETTINGS_ITEM_COUNT: usize = SettingsItem::ALL.len();

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    theme::render_backdrop(frame, area);
    let modal = centered_rect(65, 70, area);
    frame.render_widget(Clear, modal);
    let block = theme::modal_elevated_block("Settings");
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(inner);

    render_settings_list(frame, halves[0], state);
    render_settings_detail(frame, halves[1], state);
}

fn render_settings_list(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut items = Vec::new();
    // Track which visual row maps to which selectable item
    let mut selectable_row_map: Vec<Option<usize>> = Vec::new();
    let mut current_section = "";

    for (i, item) in SettingsItem::ALL.iter().enumerate() {
        let section = item.section();
        if section != current_section {
            current_section = section;
            // Add section header (non-selectable)
            if i > 0 {
                items.push(ListItem::new(Line::from("")));
                selectable_row_map.push(None);
            }
            items.push(ListItem::new(Line::from(Span::styled(
                section,
                theme::modal_accent_style(),
            ))));
            selectable_row_map.push(None);
        }

        items.push(ListItem::new(Line::from(Span::styled(
            format!("  {}", item.label(state)),
            theme::modal_text_style(),
        ))));
        selectable_row_map.push(Some(i));
    }

    // Find the visual row for the current selection
    let visual_row = selectable_row_map
        .iter()
        .position(|r| *r == Some(state.selected_settings_item))
        .unwrap_or(0);

    let list = List::new(items)
        .style(theme::modal_text_style())
        .highlight_style(theme::selected_list_style())
        .highlight_symbol("▸ ");

    let mut list_state = ListState::default();
    list_state.select(Some(visual_row));
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_settings_detail(frame: &mut Frame, area: Rect, state: &AppState) {
    let item = SettingsItem::from_index(state.selected_settings_item);
    let mut lines = Vec::new();

    match item {
        SettingsItem::FetchEnabled => {
            detail_header(&mut lines, "Auto-fetch");
            detail_description(
                &mut lines,
                "Automatically fetch from remote at a regular interval.",
            );
            detail_bool_value(&mut lines, state.settings.auto_refresh.fetch_enabled);
            detail_toggle_hint(&mut lines);
        }
        SettingsItem::FetchInterval => {
            detail_header(&mut lines, "Fetch Interval");
            detail_description(&mut lines, "How often to automatically fetch (in seconds).");
            detail_numeric_value(
                &mut lines,
                state.settings.auto_refresh.fetch_interval_secs,
                "s",
                "30s - 600s",
                "30s",
            );
        }
        SettingsItem::StatusEnabled => {
            detail_header(&mut lines, "Auto-refresh Status");
            detail_description(&mut lines, "Automatically refresh staged/unstaged changes.");
            detail_bool_value(&mut lines, state.settings.auto_refresh.status_enabled);
            detail_toggle_hint(&mut lines);
        }
        SettingsItem::StatusInterval => {
            detail_header(&mut lines, "Status Refresh Interval");
            detail_description(&mut lines, "How often to refresh file status (in seconds).");
            detail_numeric_value(
                &mut lines,
                state.settings.auto_refresh.status_interval_secs,
                "s",
                "10s - 300s",
                "10s",
            );
        }
        SettingsItem::BranchesEnabled => {
            detail_header(&mut lines, "Auto-refresh Branches");
            detail_description(
                &mut lines,
                "Automatically refresh branch list and remote statuses.",
            );
            detail_bool_value(&mut lines, state.settings.auto_refresh.branches_enabled);
            detail_toggle_hint(&mut lines);
        }
        SettingsItem::BranchesInterval => {
            detail_header(&mut lines, "Branches Refresh Interval");
            detail_description(&mut lines, "How often to refresh branches (in seconds).");
            detail_numeric_value(
                &mut lines,
                state.settings.auto_refresh.branches_interval_secs,
                "s",
                "30s - 600s",
                "30s",
            );
        }
        SettingsItem::PrsEnabled => {
            detail_header(&mut lines, "Auto-refresh PRs");
            detail_description(&mut lines, "Automatically refresh the pull request list.");
            detail_bool_value(&mut lines, state.settings.auto_refresh.prs_enabled);
            detail_toggle_hint(&mut lines);
        }
        SettingsItem::PrsInterval => {
            detail_header(&mut lines, "PRs Refresh Interval");
            detail_description(
                &mut lines,
                "How often to refresh pull requests (in seconds).",
            );
            detail_numeric_value(
                &mut lines,
                state.settings.auto_refresh.prs_interval_secs,
                "s",
                "30s - 600s",
                "30s",
            );
        }
        SettingsItem::AiBranchInstructions => {
            detail_header(&mut lines, "Branch Name Instructions");
            detail_description(
                &mut lines,
                "Extra instructions for AI branch name generation.",
            );
            detail_ai_value(
                &mut lines,
                state.settings.ai.branch_name_instructions.as_deref(),
            );
        }
        SettingsItem::AiCommitInstructions => {
            detail_header(&mut lines, "Commit Message Instructions");
            detail_description(
                &mut lines,
                "Extra instructions for AI commit message generation.",
            );
            detail_ai_value(
                &mut lines,
                state.settings.ai.commit_message_instructions.as_deref(),
            );
        }
        SettingsItem::AiPrTitleInstructions => {
            detail_header(&mut lines, "PR Title Instructions");
            detail_description(&mut lines, "Extra instructions for AI PR title generation.");
            detail_ai_value(
                &mut lines,
                state.settings.ai.pr_title_instructions.as_deref(),
            );
        }
        SettingsItem::AiPrDescriptionInstructions => {
            detail_header(&mut lines, "PR Description Instructions");
            detail_description(
                &mut lines,
                "Extra instructions for AI PR description generation.",
            );
            detail_ai_value(
                &mut lines,
                state.settings.ai.pr_description_instructions.as_deref(),
            );
        }
        SettingsItem::DraftByDefault => {
            detail_header(&mut lines, "Draft by Default");
            detail_description(&mut lines, "Create pull requests as drafts by default.");
            detail_bool_value(&mut lines, state.settings.pull_requests.draft_by_default);
            detail_toggle_hint(&mut lines);
        }
        SettingsItem::SubjectMaxLength => {
            detail_header(&mut lines, "Subject Max Length");
            detail_description(
                &mut lines,
                "Maximum characters for commit message subject line.",
            );
            detail_numeric_value(
                &mut lines,
                state.settings.commit.subject_max_length as u64,
                "",
                "50 - 120",
                "1",
            );
        }
        SettingsItem::DefaultView => {
            detail_header(&mut lines, "Default View");
            detail_description(&mut lines, "Which view to show on startup (changes or pr).");
            lines.push(Line::default());
            lines.push(Line::from(vec![
                Span::styled("Current: ", theme::modal_muted_style()),
                Span::styled(&state.settings.ui.default_view, theme::modal_accent_style()),
            ]));
            lines.push(Line::default());
            detail_edit_file_hint(&mut lines);
        }
    }

    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("esc ", theme::modal_accent_style()),
        Span::styled("close", theme::modal_muted_style()),
    ]));

    let paragraph = Paragraph::new(Text::from(lines)).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn detail_header(lines: &mut Vec<Line<'_>>, title: &'static str) {
    lines.push(Line::from(Span::styled(title, theme::modal_accent_style())));
}

fn detail_description(lines: &mut Vec<Line<'_>>, desc: &'static str) {
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(desc, theme::modal_text_style())));
}

fn detail_bool_value(lines: &mut Vec<Line<'_>>, enabled: bool) {
    lines.push(Line::default());
    let (label, style) = if enabled {
        ("Enabled", theme::modal_accent_style())
    } else {
        ("Disabled", theme::modal_muted_style())
    };
    lines.push(Line::from(vec![
        Span::styled("Status: ", theme::modal_muted_style()),
        Span::styled(label, style),
    ]));
}

fn detail_toggle_hint(lines: &mut Vec<Line<'_>>) {
    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("space/enter ", theme::modal_accent_style()),
        Span::styled("toggle", theme::modal_muted_style()),
    ]));
}

fn detail_numeric_value(
    lines: &mut Vec<Line<'_>>,
    value: u64,
    suffix: &str,
    range: &str,
    step: &str,
) {
    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("Current: ", theme::modal_muted_style()),
        Span::styled(format!("{value}{suffix}"), theme::modal_accent_style()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Range:   ", theme::modal_muted_style()),
        Span::styled(range.to_string(), theme::modal_text_style()),
    ]));
    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled("+/- ", theme::modal_accent_style()),
        Span::styled(format!("adjust by {step}"), theme::modal_muted_style()),
    ]));
}

fn detail_ai_value(lines: &mut Vec<Line<'_>>, value: Option<&str>) {
    lines.push(Line::default());
    match value {
        Some(v) if !v.is_empty() => {
            lines.push(Line::from(vec![
                Span::styled("Current: ", theme::modal_muted_style()),
                Span::styled(v.to_string(), theme::modal_text_style()),
            ]));
        }
        _ => {
            lines.push(Line::from(Span::styled(
                "Not configured",
                theme::modal_muted_style(),
            )));
        }
    }
    lines.push(Line::default());
    detail_edit_file_hint(lines);
}

fn detail_edit_file_hint(lines: &mut Vec<Line<'_>>) {
    lines.push(Line::from(Span::styled(
        "Edit config file to change:",
        theme::modal_muted_style(),
    )));
    lines.push(Line::from(Span::styled(
        "  ~/.config/git-pilot/settings.toml",
        theme::modal_text_style(),
    )));
    lines.push(Line::from(Span::styled(
        "  .git-pilot.toml (team)",
        theme::modal_text_style(),
    )));
    lines.push(Line::from(Span::styled(
        "  .git/git-pilot.toml (personal)",
        theme::modal_text_style(),
    )));
}
