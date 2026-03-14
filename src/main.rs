use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, poll};
use std::time::Duration;

mod app;
mod cli;
mod config;
mod db;
mod export;
mod models;
mod ui;

use app::{App, Message};
use cli::{Cli, Commands};
use config::SagaConfig;
use db::Database;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Tui) => run_tui()?,
        Some(cmd) => run_command(cmd)?,
    }

    Ok(())
}

fn run_tui() -> Result<()> {
    let config = SagaConfig::load().unwrap_or_default();
    let db_path = SagaConfig::db_path()?;
    let db = Database::open(&db_path)?;

    let tick_rate = Duration::from_millis(config.tick_rate_ms);
    let mut app = App::new(config);

    // Initial data load
    app.refresh_all(&db);

    let mut terminal = ui::tui::init()?;

    while app.running {
        terminal.draw(|frame| ui::view(&mut app, frame))?;

        if poll(tick_rate)? {
            let evt = event::read()?;
            if let Some(msg) = app::handler::handle_event(&app, evt) {
                let mut current = Some(msg);
                while let Some(m) = current {
                    current = app::update::update(&mut app, m, &db);
                }
            }
        } else {
            // Tick for timer display update
            app::update::update(&mut app, Message::Tick, &db);
        }
    }

    ui::tui::restore()?;
    Ok(())
}

fn run_command(cmd: Commands) -> Result<()> {
    let db_path = SagaConfig::db_path()?;
    let db = Database::open(&db_path)?;

    match cmd {
        Commands::Tui => unreachable!(),
        Commands::Start { project, description, tag, no_billable } => {
            cli::commands::handle_start(&db, &project, description.as_deref(), &tag, no_billable)?;
        }
        Commands::Stop { description } => {
            cli::commands::handle_stop(&db, description.as_deref())?;
        }
        Commands::Status => {
            cli::commands::handle_status(&db)?;
        }
        Commands::Cancel => {
            cli::commands::handle_cancel(&db)?;
        }
        Commands::Resume => {
            cli::commands::handle_resume(&db)?;
        }
        Commands::Add { project, start, end, description, tag } => {
            cli::commands::handle_add(&db, &project, &start, &end, description.as_deref(), &tag)?;
        }
        Commands::Log { today, week, month, project, client } => {
            cli::commands::handle_log(&db, today, week, month, project.as_deref(), client.as_deref())?;
        }
        Commands::Report { period, format, output } => {
            cli::commands::handle_report(&db, &period, &format, output.as_deref())?;
        }
        Commands::Projects { action } => {
            cli::commands::handle_projects(&db, action)?;
        }
        Commands::Clients { action } => {
            cli::commands::handle_clients(&db, action)?;
        }
        Commands::Tags { action } => {
            cli::commands::handle_tags(&db, action)?;
        }
        Commands::Rates { action } => {
            cli::commands::handle_rates(&db, action)?;
        }
        Commands::Invoice { action } => {
            cli::commands::handle_invoice(&db, action)?;
        }
        Commands::Config { action } => {
            cli::commands::handle_config(action)?;
        }
    }

    Ok(())
}
