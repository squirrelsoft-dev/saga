use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crate::app::state::{App, InputMode, Modal};
use crate::app::message::Message;

pub fn handle_event(app: &App, event: Event) -> Option<Message> {
    match event {
        Event::Key(key) => handle_key(app, key),
        _ => None,
    }
}

fn handle_key(app: &App, key: KeyEvent) -> Option<Message> {
    // Handle modal-specific keys first
    match app.modal {
        Modal::Help => {
            return match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => Some(Message::CloseModal),
                _ => None,
            };
        }
        Modal::ConfirmDelete => {
            return match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => Some(Message::ConfirmYes),
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(Message::ConfirmNo),
                _ => None,
            };
        }
        Modal::EntryForm => {
            return handle_entry_form_key(app, key);
        }
        Modal::ProjectPicker => {
            return handle_picker_key(app, key);
        }
        Modal::None => {}
    }

    // Global shortcuts (Normal mode)
    if app.input_mode == InputMode::Normal {
        match key.code {
            KeyCode::Char('q') => return Some(Message::Quit),
            KeyCode::Char('?') => return Some(Message::OpenHelp),
            KeyCode::Char('s') => return Some(Message::ToggleTimer),
            KeyCode::Char('1') => return Some(Message::NavigateScreen(crate::app::state::Screen::Dashboard)),
            KeyCode::Char('2') => return Some(Message::NavigateScreen(crate::app::state::Screen::Timer)),
            KeyCode::Char('3') => return Some(Message::NavigateScreen(crate::app::state::Screen::Entries)),
            KeyCode::Char('4') => return Some(Message::NavigateScreen(crate::app::state::Screen::Projects)),
            KeyCode::Char('5') => return Some(Message::NavigateScreen(crate::app::state::Screen::Clients)),
            KeyCode::Char('6') => return Some(Message::NavigateScreen(crate::app::state::Screen::Reports)),
            KeyCode::Char('7') => return Some(Message::NavigateScreen(crate::app::state::Screen::Settings)),
            KeyCode::Tab => return Some(Message::NextTab),
            KeyCode::BackTab => return Some(Message::PrevTab),
            _ => {}
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('p') => return Some(Message::OpenProjectPicker),
                KeyCode::Char('c') => return Some(Message::Quit),
                _ => {}
            }
        }

        // Screen-specific normal mode keys
        return handle_screen_key(app, key);
    }

    None
}

fn handle_screen_key(app: &App, key: KeyEvent) -> Option<Message> {
    match app.screen {
        crate::app::state::Screen::Dashboard => match key.code {
            _ => None,
        },
        crate::app::state::Screen::Timer => match key.code {
            KeyCode::Enter => Some(Message::ToggleTimer),
            KeyCode::Char('c') => Some(Message::CancelTimer),
            KeyCode::Char('p') => Some(Message::OpenProjectPicker),
            _ => None,
        },
        crate::app::state::Screen::Entries => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Message::SelectNextEntry),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::SelectPrevEntry),
            KeyCode::Char('d') | KeyCode::Delete => Some(Message::DeleteSelectedEntry),
            KeyCode::Char('e') | KeyCode::Enter => Some(Message::EditSelectedEntry),
            KeyCode::Char('n') | KeyCode::Char('a') => Some(Message::NewEntry),
            _ => None,
        },
        crate::app::state::Screen::Projects => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Message::SelectNextProject),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::SelectPrevProject),
            KeyCode::Char('n') | KeyCode::Char('a') => Some(Message::NewProject),
            KeyCode::Char('x') => Some(Message::ArchiveSelectedProject),
            KeyCode::Char('h') => Some(Message::ToggleArchivedProjects),
            _ => None,
        },
        crate::app::state::Screen::Clients => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Message::SelectNextClient),
            KeyCode::Char('k') | KeyCode::Up => Some(Message::SelectPrevClient),
            KeyCode::Char('n') | KeyCode::Char('a') => Some(Message::NewClient),
            KeyCode::Char('d') | KeyCode::Delete => Some(Message::DeleteSelectedClient),
            _ => None,
        },
        crate::app::state::Screen::Reports => match key.code {
            KeyCode::Left | KeyCode::Char('h') => Some(Message::PrevReportPeriod),
            KeyCode::Right | KeyCode::Char('l') => Some(Message::NextReportPeriod),
            KeyCode::Char('c') => Some(Message::ExportCsv),
            KeyCode::Char('p') => Some(Message::ExportPdf),
            _ => None,
        },
        crate::app::state::Screen::Settings => None,
    }
}

fn handle_entry_form_key(app: &App, key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Esc => Some(Message::CancelForm),
        KeyCode::Enter => Some(Message::SubmitEntryForm),
        KeyCode::Tab => Some(Message::NextField),
        KeyCode::BackTab => Some(Message::PrevField),
        KeyCode::Char(c) => {
            if app.entry_form.focused_field == 5 {
                // Billable toggle field
                if c == ' ' {
                    return Some(Message::ToggleBillable);
                }
                None
            } else {
                Some(Message::InputChar(c))
            }
        }
        KeyCode::Backspace => Some(Message::InputBackspace),
        KeyCode::Left => Some(Message::InputLeft),
        KeyCode::Right => Some(Message::InputRight),
        _ => None,
    }
}

fn handle_picker_key(_app: &App, key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Esc => Some(Message::CloseModal),
        KeyCode::Enter => Some(Message::ConfirmPicker),
        KeyCode::Up => Some(Message::SelectPrevPickerItem),
        KeyCode::Down => Some(Message::SelectNextPickerItem),
        KeyCode::Char(c) => Some(Message::InputChar(c)),
        KeyCode::Backspace => Some(Message::InputBackspace),
        _ => None,
    }
}
