# saga

A local-first time tracking application for the terminal, built in Rust.

Saga combines a full-featured terminal UI (TUI) with a scriptable CLI so you can track time, manage projects and clients, generate reports, and produce invoices — all without leaving the command line.

## Features

- **Timer** — start, stop, resume, and cancel timers from the CLI or TUI
- **Interactive TUI** — dashboard with live timer, today's entries, and a weekly bar chart
- **Projects & clients** — organize work by project (with optional budgets) and client
- **Tags** — categorize entries with colored tags
- **Billing rates** — set per-project or per-client hourly rates
- **Reports** — daily, weekly, or monthly summaries as a table, CSV, or PDF
- **Invoices** — generate PDF invoices for a client and date range
- **Local-first** — all data lives in a local SQLite database; no account required
- **Config** — customize tick rate, default currency, and more via `saga config`

## Installation

### From crates.io

```sh
cargo install saga-time
```

### From source

```sh
cargo install --path .
```

### Build manually

```sh
git clone https://github.com/squirrelsoft-dev/saga.git
cd saga
cargo build --release
# binary is at target/release/saga
```

## Quick start

```sh
# Start a timer
saga start myproject -d "working on feature X"

# Check status
saga status

# Stop the timer
saga stop

# View today's entries
saga log --today

# Launch the interactive TUI (also the default with no arguments)
saga
```

## CLI reference

| Command | Description |
|---------|-------------|
| `saga` / `saga tui` | Launch the interactive TUI |
| `saga start <project>` | Start a timer (`-d` description, `-t` tags, `--no-billable`) |
| `saga stop` | Stop the active timer |
| `saga status` | Show the current timer |
| `saga cancel` | Discard the active timer |
| `saga resume` | Restart the last stopped timer |
| `saga add` | Add a completed entry (`-p`, `-s`, `-e`, `-d`, `-t`) |
| `saga log` | List entries (`--today`, `--week`, `--month`, `--project`, `--client`) |
| `saga report` | Generate a report (`--period`, `--format`, `-o`) |
| `saga projects <add\|list\|edit\|archive\|activate>` | Manage projects |
| `saga clients <add\|list\|edit\|delete>` | Manage clients |
| `saga tags <add\|list\|delete>` | Manage tags |
| `saga rates <set\|list>` | Manage billing rates |
| `saga invoice <generate\|list>` | Generate or list invoices |
| `saga config <show\|set\|path>` | View or update configuration |

## Data storage

Saga stores its SQLite database and configuration in the standard platform directories (via the `directories` crate):

| Platform | Data directory |
|----------|---------------|
| Linux | `~/.local/share/saga/` |
| macOS | `~/Library/Application Support/saga/` |
| Windows | `{FOLDERID_LocalAppData}\saga\` |

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
