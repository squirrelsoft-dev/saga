pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "saga", version, about = "Time tracking from the terminal")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch interactive TUI
    Tui,

    /// Start a timer
    Start {
        /// Project name
        project: String,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
        /// Tags
        #[arg(short, long)]
        tag: Vec<String>,
        /// Mark as non-billable
        #[arg(long)]
        no_billable: bool,
    },

    /// Stop the active timer
    Stop {
        /// Update description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Show current timer status
    Status,

    /// Cancel the active timer (discard)
    Cancel,

    /// Resume the last stopped timer
    Resume,

    /// Add a completed time entry
    Add {
        /// Project name
        #[arg(short, long)]
        project: String,
        /// Start time (HH:MM or YYYY-MM-DD HH:MM)
        #[arg(short, long)]
        start: String,
        /// End time (HH:MM or YYYY-MM-DD HH:MM)
        #[arg(short, long)]
        end: String,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
        /// Tags
        #[arg(short, long)]
        tag: Vec<String>,
    },

    /// List recent time entries
    Log {
        /// Show today's entries
        #[arg(long)]
        today: bool,
        /// Show this week's entries
        #[arg(long)]
        week: bool,
        /// Show this month's entries
        #[arg(long)]
        month: bool,
        /// Filter by project
        #[arg(long)]
        project: Option<String>,
        /// Filter by client
        #[arg(long)]
        client: Option<String>,
    },

    /// Generate reports
    Report {
        /// Period: daily, weekly, monthly
        #[arg(long, default_value = "weekly")]
        period: String,
        /// Format: table, csv, pdf
        #[arg(long, default_value = "table")]
        format: String,
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Manage projects
    Projects {
        #[command(subcommand)]
        action: ProjectAction,
    },

    /// Manage clients
    Clients {
        #[command(subcommand)]
        action: ClientAction,
    },

    /// Manage tags
    Tags {
        #[command(subcommand)]
        action: TagAction,
    },

    /// Manage billing rates
    Rates {
        #[command(subcommand)]
        action: RateAction,
    },

    /// Generate or list invoices
    Invoice {
        #[command(subcommand)]
        action: InvoiceAction,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectAction {
    /// List projects
    List {
        /// Include archived projects
        #[arg(long)]
        all: bool,
    },
    /// Add a new project
    Add {
        /// Project name
        name: String,
        /// Client name
        #[arg(short, long)]
        client: Option<String>,
        /// Color (hex)
        #[arg(long)]
        color: Option<String>,
        /// Budget hours
        #[arg(short, long)]
        budget: Option<f64>,
    },
    /// Edit a project
    Edit {
        /// Project name
        name: String,
        /// New name
        #[arg(long)]
        new_name: Option<String>,
        /// New color
        #[arg(long)]
        color: Option<String>,
        /// New budget
        #[arg(long)]
        budget: Option<f64>,
    },
    /// Archive a project
    Archive {
        /// Project name
        name: String,
    },
    /// Activate an archived project
    Activate {
        /// Project name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ClientAction {
    /// List clients
    List,
    /// Add a new client
    Add {
        /// Client name
        name: String,
        /// Contact info
        #[arg(long)]
        contact: Option<String>,
        /// Email
        #[arg(long)]
        email: Option<String>,
    },
    /// Edit a client
    Edit {
        /// Client name
        name: String,
        /// New name
        #[arg(long)]
        new_name: Option<String>,
        /// New contact
        #[arg(long)]
        contact: Option<String>,
        /// New email
        #[arg(long)]
        email: Option<String>,
    },
    /// Delete a client
    Delete {
        /// Client name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TagAction {
    /// List tags
    List,
    /// Add a tag
    Add {
        /// Tag name
        name: String,
        /// Color (hex)
        #[arg(long)]
        color: Option<String>,
    },
    /// Delete a tag
    Delete {
        /// Tag name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum RateAction {
    /// Set a rate
    Set {
        /// Hourly rate
        rate: f64,
        /// For project
        #[arg(long)]
        project: Option<String>,
        /// For client
        #[arg(long)]
        client: Option<String>,
        /// Currency
        #[arg(long, default_value = "USD")]
        currency: String,
    },
    /// List rates
    List,
}

#[derive(Subcommand, Debug)]
pub enum InvoiceAction {
    /// Generate an invoice
    Generate {
        /// Client name
        #[arg(short, long)]
        client: String,
        /// Period start (YYYY-MM-DD)
        #[arg(short, long)]
        from: String,
        /// Period end (YYYY-MM-DD)
        #[arg(short, long)]
        to: String,
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },
    /// List invoices
    List,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Show current config
    Show,
    /// Set a config value
    Set {
        /// Key
        key: String,
        /// Value
        value: String,
    },
    /// Show config file path
    Path,
}
