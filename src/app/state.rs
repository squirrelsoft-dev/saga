use crate::db::Database;
use crate::models::*;
use crate::config::SagaConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Timer,
    Entries,
    Projects,
    Clients,
    Reports,
    Settings,
}

impl Screen {
    pub fn index(&self) -> usize {
        match self {
            Screen::Dashboard => 0,
            Screen::Timer => 1,
            Screen::Entries => 2,
            Screen::Projects => 3,
            Screen::Clients => 4,
            Screen::Reports => 5,
            Screen::Settings => 6,
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Screen::Dashboard,
            1 => Screen::Timer,
            2 => Screen::Entries,
            3 => Screen::Projects,
            4 => Screen::Clients,
            5 => Screen::Reports,
            6 => Screen::Settings,
            _ => Screen::Dashboard,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Timer => "Timer",
            Screen::Entries => "Entries",
            Screen::Projects => "Projects",
            Screen::Clients => "Clients",
            Screen::Reports => "Reports",
            Screen::Settings => "Settings",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modal {
    None,
    EntryForm,
    ProjectPicker,
    ConfirmDelete,
    Help,
}

#[derive(Debug, Clone)]
pub struct TextInput {
    pub value: String,
    pub cursor: usize,
    pub label: String,
}

impl TextInput {
    pub fn new(label: &str) -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            label: label.to_string(),
        }
    }

    pub fn with_value(label: &str, value: &str) -> Self {
        let len = value.len();
        Self {
            value: value.to_string(),
            cursor: len,
            label: label.to_string(),
        }
    }

    pub fn insert(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn delete_back(&mut self) {
        if self.cursor > 0 {
            let prev = self.value[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.value.remove(prev);
            self.cursor = prev;
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.value[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.value.len() {
            self.cursor += self.value[self.cursor..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
        }
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }
}

#[derive(Debug, Clone)]
pub struct EntryFormState {
    pub editing_id: Option<i64>,
    pub project_name: TextInput,
    pub description: TextInput,
    pub date: TextInput,
    pub start_time: TextInput,
    pub end_time: TextInput,
    pub billable: bool,
    pub focused_field: usize,
}

impl EntryFormState {
    pub fn new() -> Self {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        Self {
            editing_id: None,
            project_name: TextInput::new("Project"),
            description: TextInput::new("Description"),
            date: TextInput::with_value("Date", &today),
            start_time: TextInput::new("Start (HH:MM)"),
            end_time: TextInput::new("End (HH:MM)"),
            billable: true,
            focused_field: 0,
        }
    }

    pub fn field_count(&self) -> usize {
        6 // project, description, date, start, end, billable
    }
}

pub struct App {
    pub running: bool,
    pub screen: Screen,
    pub prev_screen: Screen,
    pub input_mode: InputMode,
    pub modal: Modal,
    pub config: SagaConfig,

    // Active timer
    pub active_entry: Option<TimeEntry>,
    pub active_project_name: Option<String>,
    pub timer_seconds: u64,

    // Dashboard
    pub today_entries: Vec<TimeEntry>,
    pub today_total_secs: i64,
    pub week_daily_totals: Vec<(String, i64)>, // (day_label, seconds)

    // Entries screen
    pub entries: Vec<TimeEntry>,
    pub entries_selected: usize,
    pub entries_scroll: usize,
    pub entry_project_names: std::collections::HashMap<i64, String>,

    // Projects screen
    pub projects: Vec<Project>,
    pub projects_selected: usize,
    pub show_archived_projects: bool,

    // Clients screen
    pub clients: Vec<Client>,
    pub clients_selected: usize,

    // Reports screen
    pub report_period: ReportPeriod,
    pub report_project_breakdown: Vec<ProjectBreakdown>,
    pub report_daily_summaries: Vec<DailySummary>,
    pub report_total_secs: i64,
    pub report_billable_secs: i64,

    // Project picker
    pub picker_projects: Vec<Project>,
    pub picker_filter: TextInput,
    pub picker_selected: usize,

    // Entry form
    pub entry_form: EntryFormState,

    // Confirm dialog
    pub confirm_message: String,
    pub confirm_action: Option<Message>,

    // Status message
    pub status_message: Option<(String, std::time::Instant)>,

    // Tags
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportPeriod {
    Daily,
    Weekly,
    Monthly,
}

impl ReportPeriod {
    pub fn label(&self) -> &'static str {
        match self {
            ReportPeriod::Daily => "Daily",
            ReportPeriod::Weekly => "Weekly",
            ReportPeriod::Monthly => "Monthly",
        }
    }
}

// Need to import Message here
use crate::app::message::Message;

impl App {
    pub fn new(config: SagaConfig) -> Self {
        Self {
            running: true,
            screen: Screen::Dashboard,
            prev_screen: Screen::Dashboard,
            input_mode: InputMode::Normal,
            modal: Modal::None,
            config,

            active_entry: None,
            active_project_name: None,
            timer_seconds: 0,

            today_entries: Vec::new(),
            today_total_secs: 0,
            week_daily_totals: Vec::new(),

            entries: Vec::new(),
            entries_selected: 0,
            entries_scroll: 0,
            entry_project_names: std::collections::HashMap::new(),

            projects: Vec::new(),
            projects_selected: 0,
            show_archived_projects: false,

            clients: Vec::new(),
            clients_selected: 0,

            report_period: ReportPeriod::Weekly,
            report_project_breakdown: Vec::new(),
            report_daily_summaries: Vec::new(),
            report_total_secs: 0,
            report_billable_secs: 0,

            picker_projects: Vec::new(),
            picker_filter: TextInput::new("Search projects"),
            picker_selected: 0,

            entry_form: EntryFormState::new(),

            confirm_message: String::new(),
            confirm_action: None,

            status_message: None,

            tags: Vec::new(),
        }
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some((msg.to_string(), std::time::Instant::now()));
    }

    pub fn refresh_active_timer(&mut self, db: &Database) {
        match db.get_active_entry() {
            Ok(Some(entry)) => {
                let elapsed = chrono::Local::now()
                    .naive_local()
                    .signed_duration_since(entry.start_time)
                    .num_seconds() as u64;
                self.timer_seconds = elapsed;
                // Look up project name
                if let Ok(project) = db.get_project(entry.project_id) {
                    self.active_project_name = Some(project.name);
                }
                self.active_entry = Some(entry);
            }
            Ok(None) => {
                self.active_entry = None;
                self.active_project_name = None;
                self.timer_seconds = 0;
            }
            Err(_) => {}
        }
    }

    pub fn refresh_today(&mut self, db: &Database) {
        if let Ok(entries) = db.get_today_entries() {
            self.today_total_secs = entries.iter().map(|e| {
                if let Some(d) = e.duration_secs {
                    d
                } else {
                    chrono::Local::now()
                        .naive_local()
                        .signed_duration_since(e.start_time)
                        .num_seconds()
                }
            }).sum();
            self.today_entries = entries;
        }
    }

    pub fn refresh_entries(&mut self, db: &Database) {
        if let Ok(entries) = db.list_entries(None, None, None, Some(100)) {
            // Build project name map
            self.entry_project_names.clear();
            for entry in &entries {
                if !self.entry_project_names.contains_key(&entry.project_id) {
                    if let Ok(p) = db.get_project(entry.project_id) {
                        self.entry_project_names.insert(entry.project_id, p.name);
                    }
                }
            }
            self.entries = entries;
            if self.entries_selected >= self.entries.len() && !self.entries.is_empty() {
                self.entries_selected = self.entries.len() - 1;
            }
        }
    }

    pub fn refresh_projects(&mut self, db: &Database) {
        if let Ok(projects) = db.list_projects(self.show_archived_projects) {
            self.projects = projects;
            if self.projects_selected >= self.projects.len() && !self.projects.is_empty() {
                self.projects_selected = self.projects.len() - 1;
            }
        }
    }

    pub fn refresh_clients(&mut self, db: &Database) {
        if let Ok(clients) = db.list_clients() {
            self.clients = clients;
            if self.clients_selected >= self.clients.len() && !self.clients.is_empty() {
                self.clients_selected = self.clients.len() - 1;
            }
        }
    }

    pub fn refresh_all(&mut self, db: &Database) {
        self.refresh_active_timer(db);
        self.refresh_today(db);
        self.refresh_entries(db);
        self.refresh_projects(db);
        self.refresh_clients(db);
    }
}
