/// SQL CREATE TABLE statements for the saga time tracking database.

pub const CREATE_CLIENTS: &str = "
CREATE TABLE clients (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    contact TEXT,
    email TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S','now'))
);";

pub const CREATE_PROJECTS: &str = "
CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    client_id INTEGER REFERENCES clients(id) ON DELETE SET NULL,
    color TEXT NOT NULL DEFAULT '#5B9BD5',
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','archived')),
    budget_hours REAL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S','now')),
    UNIQUE(name, client_id)
);";

pub const CREATE_TAGS: &str = "
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL DEFAULT '#808080'
);";

pub const CREATE_TIME_ENTRIES: &str = "
CREATE TABLE time_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL UNIQUE,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
    description TEXT NOT NULL DEFAULT '',
    start_time TEXT NOT NULL,
    end_time TEXT,
    duration_secs INTEGER,
    billable INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S','now'))
);";

pub const CREATE_TIME_ENTRIES_INDEXES: &str = "
CREATE INDEX idx_entries_project ON time_entries(project_id);
CREATE INDEX idx_entries_start ON time_entries(start_time);
CREATE INDEX idx_entries_active ON time_entries(end_time) WHERE end_time IS NULL;
";

pub const CREATE_TIME_ENTRY_TAGS: &str = "
CREATE TABLE time_entry_tags (
    entry_id INTEGER NOT NULL REFERENCES time_entries(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (entry_id, tag_id)
);";

pub const CREATE_RATES: &str = "
CREATE TABLE rates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rate_type TEXT NOT NULL CHECK (rate_type IN ('project','client','default')),
    project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
    client_id INTEGER REFERENCES clients(id) ON DELETE CASCADE,
    hourly_rate REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    effective_from TEXT NOT NULL DEFAULT '1970-01-01',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S','now')),
    CHECK (
        (rate_type='default' AND project_id IS NULL AND client_id IS NULL) OR
        (rate_type='project' AND project_id IS NOT NULL AND client_id IS NULL) OR
        (rate_type='client'  AND project_id IS NULL AND client_id IS NOT NULL)
    )
);";

pub const CREATE_INVOICES: &str = "
CREATE TABLE invoices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_number TEXT NOT NULL UNIQUE,
    client_id INTEGER NOT NULL REFERENCES clients(id) ON DELETE RESTRICT,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    total_hours REAL NOT NULL,
    total_amount REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','sent','paid')),
    notes TEXT,
    generated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%S','now'))
);";

pub const CREATE_SCHEMA_VERSION: &str = "
CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);
";
