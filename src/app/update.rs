use crate::app::state::*;
use crate::app::message::Message;
use crate::db::Database;
use crate::models::*;
use chrono::{Local, NaiveDateTime, NaiveDate, NaiveTime, Datelike, Duration};

pub fn update(app: &mut App, msg: Message, db: &Database) -> Option<Message> {
    match msg {
        // Navigation
        Message::NavigateScreen(screen) => {
            app.prev_screen = app.screen;
            app.screen = screen;
            // Refresh data for the target screen
            match screen {
                Screen::Dashboard => {
                    app.refresh_active_timer(db);
                    app.refresh_today(db);
                    refresh_week_totals(app, db);
                }
                Screen::Entries => app.refresh_entries(db),
                Screen::Projects => app.refresh_projects(db),
                Screen::Clients => app.refresh_clients(db),
                Screen::Reports => refresh_report_data(app, db),
                _ => {}
            }
            None
        }
        Message::NextTab => {
            let next = (app.screen.index() + 1) % 7;
            Some(Message::NavigateScreen(Screen::from_index(next)))
        }
        Message::PrevTab => {
            let prev = if app.screen.index() == 0 { 6 } else { app.screen.index() - 1 };
            Some(Message::NavigateScreen(Screen::from_index(prev)))
        }

        // Timer
        Message::StartTimer(project_id) => {
            if app.active_entry.is_some() {
                app.set_status("Timer already running. Stop it first.");
                return None;
            }
            let new_entry = NewTimeEntry {
                project_id,
                description: String::new(),
                start_time: Local::now().naive_local(),
                end_time: None,
                billable: app.config.default_billable,
                tags: Vec::new(),
            };
            match db.insert_entry(&new_entry) {
                Ok(_) => {
                    app.refresh_active_timer(db);
                    app.set_status("Timer started!");
                }
                Err(e) => app.set_status(&format!("Error: {}", e)),
            }
            None
        }
        Message::StopTimer => {
            match db.stop_active_entry(None) {
                Ok(Some(_)) => {
                    app.active_entry = None;
                    app.active_project_name = None;
                    app.timer_seconds = 0;
                    app.set_status("Timer stopped.");
                    app.refresh_today(db);
                }
                Ok(None) => app.set_status("No timer running."),
                Err(e) => app.set_status(&format!("Error: {}", e)),
            }
            None
        }
        Message::CancelTimer => {
            match db.cancel_active_entry() {
                Ok(true) => {
                    app.active_entry = None;
                    app.active_project_name = None;
                    app.timer_seconds = 0;
                    app.set_status("Timer cancelled.");
                }
                Ok(false) => app.set_status("No timer running."),
                Err(e) => app.set_status(&format!("Error: {}", e)),
            }
            None
        }
        Message::ToggleTimer => {
            if app.active_entry.is_some() {
                Some(Message::StopTimer)
            } else {
                // Open project picker to select project
                Some(Message::OpenProjectPicker)
            }
        }
        Message::Tick => {
            if let Some(ref entry) = app.active_entry {
                app.timer_seconds = Local::now()
                    .naive_local()
                    .signed_duration_since(entry.start_time)
                    .num_seconds() as u64;
            }
            // Clear old status messages (after 5 seconds)
            if let Some((_, time)) = &app.status_message {
                if time.elapsed().as_secs() > 5 {
                    app.status_message = None;
                }
            }
            None
        }

        // Entries
        Message::SelectNextEntry => {
            if !app.entries.is_empty() {
                app.entries_selected = (app.entries_selected + 1).min(app.entries.len() - 1);
            }
            None
        }
        Message::SelectPrevEntry => {
            if app.entries_selected > 0 {
                app.entries_selected -= 1;
            }
            None
        }
        Message::DeleteSelectedEntry => {
            if !app.entries.is_empty() {
                let entry = &app.entries[app.entries_selected];
                app.confirm_message = format!("Delete entry from {}?", entry.start_time.format("%Y-%m-%d %H:%M"));
                app.confirm_action = Some(Message::Refresh);
                let entry_id = entry.id;
                if let Err(e) = db.delete_entry(entry_id) {
                    app.set_status(&format!("Error: {}", e));
                } else {
                    app.set_status("Entry deleted.");
                    app.refresh_entries(db);
                    app.refresh_today(db);
                }
            }
            None
        }
        Message::EditSelectedEntry => {
            if !app.entries.is_empty() {
                let entry = &app.entries[app.entries_selected];
                app.entry_form = EntryFormState::new();
                app.entry_form.editing_id = Some(entry.id);
                app.entry_form.description = TextInput::with_value("Description", &entry.description);
                app.entry_form.date = TextInput::with_value("Date", &entry.start_time.format("%Y-%m-%d").to_string());
                app.entry_form.start_time = TextInput::with_value("Start", &entry.start_time.format("%H:%M").to_string());
                if let Some(end) = entry.end_time {
                    app.entry_form.end_time = TextInput::with_value("End", &end.format("%H:%M").to_string());
                }
                let proj_name = app.entry_project_names.get(&entry.project_id).cloned().unwrap_or_default();
                app.entry_form.project_name = TextInput::with_value("Project", &proj_name);
                app.entry_form.billable = entry.billable;
                app.modal = Modal::EntryForm;
                app.input_mode = InputMode::Editing;
            }
            None
        }
        Message::NewEntry => {
            app.entry_form = EntryFormState::new();
            app.modal = Modal::EntryForm;
            app.input_mode = InputMode::Editing;
            None
        }

        // Projects
        Message::SelectNextProject => {
            if !app.projects.is_empty() {
                app.projects_selected = (app.projects_selected + 1).min(app.projects.len() - 1);
            }
            None
        }
        Message::SelectPrevProject => {
            if app.projects_selected > 0 {
                app.projects_selected -= 1;
            }
            None
        }
        Message::NewProject => {
            // For now, use a simple approach - later can add a project form modal
            app.set_status("Use CLI 'saga projects add <name>' to add projects.");
            None
        }
        Message::ArchiveSelectedProject => {
            if !app.projects.is_empty() {
                let project = &app.projects[app.projects_selected];
                let result = if project.status == ProjectStatus::Active {
                    db.archive_project(project.id)
                } else {
                    db.activate_project(project.id)
                };
                match result {
                    Ok(_) => {
                        app.set_status("Project status updated.");
                        app.refresh_projects(db);
                    }
                    Err(e) => app.set_status(&format!("Error: {}", e)),
                }
            }
            None
        }
        Message::ToggleArchivedProjects => {
            app.show_archived_projects = !app.show_archived_projects;
            app.refresh_projects(db);
            None
        }

        // Clients
        Message::SelectNextClient => {
            if !app.clients.is_empty() {
                app.clients_selected = (app.clients_selected + 1).min(app.clients.len() - 1);
            }
            None
        }
        Message::SelectPrevClient => {
            if app.clients_selected > 0 {
                app.clients_selected -= 1;
            }
            None
        }
        Message::NewClient => {
            app.set_status("Use CLI 'saga clients add <name>' to add clients.");
            None
        }
        Message::DeleteSelectedClient => {
            if !app.clients.is_empty() {
                let client = &app.clients[app.clients_selected];
                match db.delete_client(client.id) {
                    Ok(_) => {
                        app.set_status("Client deleted.");
                        app.refresh_clients(db);
                    }
                    Err(e) => app.set_status(&format!("Error: {}", e)),
                }
            }
            None
        }

        // Forms
        Message::SubmitEntryForm => {
            let form = &app.entry_form;
            // Parse the form fields
            let date_str = &form.date.value;
            let start_str = &form.start_time.value;
            let end_str = &form.end_time.value;
            let project_name = &form.project_name.value;

            if project_name.is_empty() || start_str.is_empty() {
                app.set_status("Project and start time are required.");
                return None;
            }

            // Find or create project
            let project = match db.get_project_by_name(project_name) {
                Ok(Some(p)) => p,
                Ok(None) => {
                    match db.create_project(project_name, None, Some("#5B9BD5"), None, None) {
                        Ok(p) => p,
                        Err(e) => {
                            app.set_status(&format!("Error creating project: {}", e));
                            return None;
                        }
                    }
                }
                Err(e) => {
                    app.set_status(&format!("Error: {}", e));
                    return None;
                }
            };

            // Parse times
            let start_dt = match parse_datetime(date_str, start_str) {
                Some(dt) => dt,
                None => {
                    app.set_status("Invalid start time format.");
                    return None;
                }
            };

            let end_dt = if end_str.is_empty() {
                None
            } else {
                match parse_datetime(date_str, end_str) {
                    Some(dt) => Some(dt),
                    None => {
                        app.set_status("Invalid end time format.");
                        return None;
                    }
                }
            };

            if let Some(editing_id) = form.editing_id {
                // Update existing entry: delete and re-insert
                let description = form.description.value.clone();
                let billable = form.billable;
                if let Err(e) = db.delete_entry(editing_id) {
                    app.set_status(&format!("Error: {}", e));
                    return None;
                }
                let new_entry = NewTimeEntry {
                    project_id: project.id,
                    description,
                    start_time: start_dt,
                    end_time: end_dt,
                    billable,
                    tags: Vec::new(),
                };
                match db.insert_entry(&new_entry) {
                    Ok(_) => app.set_status("Entry updated."),
                    Err(e) => app.set_status(&format!("Error: {}", e)),
                }
            } else {
                // Create new entry
                let new_entry = NewTimeEntry {
                    project_id: project.id,
                    description: form.description.value.clone(),
                    start_time: start_dt,
                    end_time: end_dt,
                    billable: form.billable,
                    tags: Vec::new(),
                };
                match db.insert_entry(&new_entry) {
                    Ok(_) => app.set_status("Entry created."),
                    Err(e) => app.set_status(&format!("Error: {}", e)),
                }
            }

            app.modal = Modal::None;
            app.input_mode = InputMode::Normal;
            app.refresh_entries(db);
            app.refresh_today(db);
            None
        }
        Message::CancelForm => {
            app.modal = Modal::None;
            app.input_mode = InputMode::Normal;
            None
        }
        Message::NextField => {
            let count = app.entry_form.field_count();
            app.entry_form.focused_field = (app.entry_form.focused_field + 1) % count;
            None
        }
        Message::PrevField => {
            let count = app.entry_form.field_count();
            app.entry_form.focused_field = if app.entry_form.focused_field == 0 {
                count - 1
            } else {
                app.entry_form.focused_field - 1
            };
            None
        }
        Message::ToggleBillable => {
            app.entry_form.billable = !app.entry_form.billable;
            None
        }

        // Input
        Message::InputChar(c) => {
            match app.modal {
                Modal::EntryForm => {
                    let field = app.entry_form.focused_field;
                    match field {
                        0 => app.entry_form.project_name.insert(c),
                        1 => app.entry_form.description.insert(c),
                        2 => app.entry_form.date.insert(c),
                        3 => app.entry_form.start_time.insert(c),
                        4 => app.entry_form.end_time.insert(c),
                        _ => {}
                    }
                }
                Modal::ProjectPicker => {
                    app.picker_filter.insert(c);
                    filter_picker_projects(app, db);
                }
                _ => {}
            }
            None
        }
        Message::InputBackspace => {
            match app.modal {
                Modal::EntryForm => {
                    let field = app.entry_form.focused_field;
                    match field {
                        0 => app.entry_form.project_name.delete_back(),
                        1 => app.entry_form.description.delete_back(),
                        2 => app.entry_form.date.delete_back(),
                        3 => app.entry_form.start_time.delete_back(),
                        4 => app.entry_form.end_time.delete_back(),
                        _ => {}
                    }
                }
                Modal::ProjectPicker => {
                    app.picker_filter.delete_back();
                    filter_picker_projects(app, db);
                }
                _ => {}
            }
            None
        }
        Message::InputLeft => {
            if app.modal == Modal::EntryForm {
                let field = app.entry_form.focused_field;
                match field {
                    0 => app.entry_form.project_name.move_left(),
                    1 => app.entry_form.description.move_left(),
                    2 => app.entry_form.date.move_left(),
                    3 => app.entry_form.start_time.move_left(),
                    4 => app.entry_form.end_time.move_left(),
                    _ => {}
                }
            }
            None
        }
        Message::InputRight => {
            if app.modal == Modal::EntryForm {
                let field = app.entry_form.focused_field;
                match field {
                    0 => app.entry_form.project_name.move_right(),
                    1 => app.entry_form.description.move_right(),
                    2 => app.entry_form.date.move_right(),
                    3 => app.entry_form.start_time.move_right(),
                    4 => app.entry_form.end_time.move_right(),
                    _ => {}
                }
            }
            None
        }

        // Project picker
        Message::OpenProjectPicker => {
            app.picker_filter.clear();
            app.picker_selected = 0;
            if let Ok(projects) = db.list_projects(false) {
                app.picker_projects = projects;
            }
            app.modal = Modal::ProjectPicker;
            app.input_mode = InputMode::Editing;
            None
        }
        Message::SelectNextPickerItem => {
            if !app.picker_projects.is_empty() {
                app.picker_selected = (app.picker_selected + 1).min(app.picker_projects.len() - 1);
            }
            None
        }
        Message::SelectPrevPickerItem => {
            if app.picker_selected > 0 {
                app.picker_selected -= 1;
            }
            None
        }
        Message::ConfirmPicker => {
            if !app.picker_projects.is_empty() {
                let project = &app.picker_projects[app.picker_selected];
                let project_id = project.id;
                app.modal = Modal::None;
                app.input_mode = InputMode::Normal;
                return Some(Message::StartTimer(project_id));
            }
            None
        }

        // Confirm dialog
        Message::ConfirmYes => {
            let action = app.confirm_action.take();
            app.modal = Modal::None;
            action
        }
        Message::ConfirmNo => {
            app.confirm_action = None;
            app.modal = Modal::None;
            None
        }

        // Reports
        Message::NextReportPeriod => {
            app.report_period = match app.report_period {
                ReportPeriod::Daily => ReportPeriod::Weekly,
                ReportPeriod::Weekly => ReportPeriod::Monthly,
                ReportPeriod::Monthly => ReportPeriod::Daily,
            };
            refresh_report_data(app, db);
            None
        }
        Message::PrevReportPeriod => {
            app.report_period = match app.report_period {
                ReportPeriod::Daily => ReportPeriod::Monthly,
                ReportPeriod::Weekly => ReportPeriod::Daily,
                ReportPeriod::Monthly => ReportPeriod::Weekly,
            };
            refresh_report_data(app, db);
            None
        }
        Message::ExportCsv => {
            // TODO: implement CSV export once the export module is available
            app.set_status("CSV export not yet implemented.");
            None
        }
        Message::ExportPdf => {
            // TODO: implement PDF export once the export module is available
            app.set_status("PDF export not yet implemented.");
            None
        }

        // Modal
        Message::OpenHelp => {
            app.modal = Modal::Help;
            None
        }
        Message::CloseModal => {
            app.modal = Modal::None;
            app.input_mode = InputMode::Normal;
            None
        }

        // App
        Message::Quit => {
            app.running = false;
            None
        }
        Message::Refresh => {
            app.refresh_all(db);
            None
        }
        Message::SetStatus(msg) => {
            app.set_status(&msg);
            None
        }
    }
}

fn parse_datetime(date_str: &str, time_str: &str) -> Option<NaiveDateTime> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
    let time = NaiveTime::parse_from_str(time_str, "%H:%M").ok()?;
    Some(NaiveDateTime::new(date, time))
}

fn filter_picker_projects(app: &mut App, db: &Database) {
    let filter = app.picker_filter.value.to_lowercase();
    if let Ok(projects) = db.list_projects(false) {
        if filter.is_empty() {
            app.picker_projects = projects;
        } else {
            app.picker_projects = projects
                .into_iter()
                .filter(|p| p.name.to_lowercase().contains(&filter))
                .collect();
        }
        app.picker_selected = 0;
    }
}

fn refresh_week_totals(app: &mut App, db: &Database) {
    let today = Local::now().date_naive();
    let weekday = today.weekday().num_days_from_monday();
    let monday = today - Duration::days(weekday as i64);

    let mut totals = Vec::new();
    for i in 0..7 {
        let day = monday + Duration::days(i);
        let day_str = day.format("%Y-%m-%d").to_string();
        let label = day.format("%a").to_string();
        if let Ok(summary) = db.daily_summary(&day_str) {
            totals.push((label, summary.total_seconds));
        } else {
            totals.push((label, 0));
        }
    }
    app.week_daily_totals = totals;
}

fn refresh_report_data(app: &mut App, db: &Database) {
    let today = Local::now().date_naive();
    let (from, to) = match app.report_period {
        ReportPeriod::Daily => {
            let d = today.format("%Y-%m-%d").to_string();
            (d.clone(), d)
        }
        ReportPeriod::Weekly => {
            let weekday = today.weekday().num_days_from_monday();
            let monday = today - Duration::days(weekday as i64);
            let sunday = monday + Duration::days(6);
            (monday.format("%Y-%m-%d").to_string(), sunday.format("%Y-%m-%d").to_string())
        }
        ReportPeriod::Monthly => {
            let first = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
            let last = if today.month() == 12 {
                NaiveDate::from_ymd_opt(today.year() + 1, 1, 1).unwrap() - Duration::days(1)
            } else {
                NaiveDate::from_ymd_opt(today.year(), today.month() + 1, 1).unwrap() - Duration::days(1)
            };
            (first.format("%Y-%m-%d").to_string(), last.format("%Y-%m-%d").to_string())
        }
    };

    if let Ok(breakdowns) = db.project_breakdown(&from, &to) {
        app.report_total_secs = breakdowns.iter().map(|b| b.total_seconds).sum();
        app.report_billable_secs = breakdowns.iter().map(|b| b.billable_seconds).sum();
        app.report_project_breakdown = breakdowns;
    }

    if let Ok(summaries) = db.date_range_summary(&from, &to) {
        app.report_daily_summaries = summaries;
    }
}
