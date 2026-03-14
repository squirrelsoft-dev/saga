use crate::app::state::Screen;

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    NavigateScreen(Screen),
    NextTab,
    PrevTab,

    // Timer
    StartTimer(i64), // project_id
    StopTimer,
    CancelTimer,
    ToggleTimer,
    Tick,

    // Entries
    SelectNextEntry,
    SelectPrevEntry,
    DeleteSelectedEntry,
    EditSelectedEntry,
    NewEntry,

    // Projects
    SelectNextProject,
    SelectPrevProject,
    NewProject,
    ArchiveSelectedProject,
    ToggleArchivedProjects,

    // Clients
    SelectNextClient,
    SelectPrevClient,
    NewClient,
    DeleteSelectedClient,

    // Forms
    SubmitEntryForm,
    CancelForm,
    NextField,
    PrevField,
    ToggleBillable,

    // Input
    InputChar(char),
    InputBackspace,
    InputLeft,
    InputRight,

    // Project picker
    OpenProjectPicker,
    SelectNextPickerItem,
    SelectPrevPickerItem,
    ConfirmPicker,

    // Confirm dialog
    ConfirmYes,
    ConfirmNo,

    // Reports
    NextReportPeriod,
    PrevReportPeriod,
    ExportCsv,
    ExportPdf,

    // Modal
    OpenHelp,
    CloseModal,

    // App
    Quit,
    Refresh,
    SetStatus(String),
}
