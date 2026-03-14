pub mod layout;
pub mod theme;
pub mod tui;
pub mod screens;
pub mod components;

use ratatui::Frame;
use crate::app::App;
use crate::app::state::Screen;

pub fn view(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    // Main layout: tab bar at top, content in middle, status bar at bottom
    let chunks = layout::main_layout(area);

    // Draw tab bar
    components::tab_bar::render(app, frame, chunks[0]);

    // Draw screen content
    match app.screen {
        Screen::Dashboard => screens::dashboard::render(app, frame, chunks[1]),
        Screen::Timer => screens::timer::render(app, frame, chunks[1]),
        Screen::Entries => screens::entries::render(app, frame, chunks[1]),
        Screen::Projects => screens::projects::render(app, frame, chunks[1]),
        Screen::Clients => screens::clients::render(app, frame, chunks[1]),
        Screen::Reports => screens::reports::render(app, frame, chunks[1]),
        Screen::Settings => screens::settings::render(app, frame, chunks[1]),
    }

    // Draw status bar
    components::status_bar::render(app, frame, chunks[2]);

    // Draw modals on top
    match app.modal {
        crate::app::state::Modal::EntryForm => components::entry_form::render(app, frame, area),
        crate::app::state::Modal::ProjectPicker => components::project_picker::render(app, frame, area),
        crate::app::state::Modal::ConfirmDelete => components::confirm_dialog::render(app, frame, area),
        crate::app::state::Modal::Help => components::help_overlay::render(app, frame, area),
        crate::app::state::Modal::None => {}
    }
}
