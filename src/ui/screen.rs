use ratatui::Frame;
use ratatui::widgets::Block;

use crate::app::state::{AppState, Modal, View};
use crate::ui::{
    branch_manage_panel, branch_panel, commit_log_panel, commit_panel, copilot_login_panel,
    create_repo_panel, diff_panel, help, layout, pr_view, settings_panel, status_list, tabs, theme,
    view_tabs,
};

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    frame.render_widget(Block::default().style(theme::screen_style()), area);
    let screen = layout::build_layout(area);

    tabs::render(frame, screen.header_row1, state);
    view_tabs::render(frame, screen.header_row2, state);
    theme::render_header_rule(frame, screen.header_rule);

    match state.active_view {
        View::Changes => {
            let (file_area, diff_area) = layout::split_changes_view(screen.view_area);
            status_list::render(frame, file_area, state);
            diff_panel::render(frame, diff_area, state);
        }
        View::Pr => {
            pr_view::render(frame, screen.view_area, state);
        }
    }

    help::render_footer(frame, screen.footer, state);

    if state.show_help {
        help::render_overlay(frame, area, state);
    }

    match state.modal {
        Modal::None => {}
        Modal::Branches => branch_manage_panel::render(frame, area, state),
        Modal::BranchCreate => branch_panel::render_create_modal(frame, area, state),
        Modal::MergeConfirm => branch_manage_panel::render_merge_confirm(frame, area, state),
        Modal::CommitLog => commit_log_panel::render(frame, area, state),
        Modal::Settings => settings_panel::render(frame, area, state),
        Modal::Commit => commit_panel::render(frame, area, state),
        Modal::CopilotLogin => copilot_login_panel::render(frame, area, state),
        Modal::CreateRepo(_) => create_repo_panel::render(frame, area, state),
    }
}
