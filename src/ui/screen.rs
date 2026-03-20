use ratatui::Frame;

use crate::app::state::{ActivePanel, AppState};
use crate::ui::{branch_panel, commit_panel, help, layout, status_list, tabs};

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    let screen = layout::build_layout(area);

    tabs::render(frame, screen.tabs, state);
    branch_panel::render_summary(frame, screen.content_left, state);
    status_list::render(frame, screen.content_right, state);
    help::render_footer(frame, screen.footer, state);

    if state.show_help {
        help::render_overlay(frame, area);
    }

    match state.active_panel {
        ActivePanel::None => {}
        ActivePanel::BranchSwitch => branch_panel::render_switch_modal(frame, area, state),
        ActivePanel::BranchCreate => branch_panel::render_create_modal(frame, area, state),
        ActivePanel::Commit => commit_panel::render(frame, area, state),
    }
}
