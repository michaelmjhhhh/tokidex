use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::{Parser, ValueEnum};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokidex::app::{App, DateRange, PrivacyMode};
use tokidex::codex_store::{load_records, resolve_codex_home};

#[derive(Debug, Parser)]
#[command(version, about = "Local Codex token usage TUI")]
struct Cli {
    #[arg(long)]
    codex_home: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = RangeArg::Today)]
    range: RangeArg,
    #[arg(long, default_value_t = 30)]
    refresh: u64,
    #[arg(long)]
    privacy: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RangeArg {
    Today,
    Week,
    All,
}

impl From<RangeArg> for DateRange {
    fn from(value: RangeArg) -> Self {
        match value {
            RangeArg::Today => DateRange::Today,
            RangeArg::Week => DateRange::Week,
            RangeArg::All => DateRange::All,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let codex_home = resolve_codex_home(cli.codex_home)?;
    let records = load_records(&codex_home).unwrap_or_else(|err| {
        eprintln!("{err:#}");
        Vec::new()
    });
    let privacy = if cli.privacy {
        PrivacyMode::On
    } else {
        PrivacyMode::Off
    };
    let app = App::new(records, cli.range.into(), privacy);
    run_terminal(app, codex_home, Duration::from_secs(cli.refresh.max(1)))
}

fn run_terminal(mut app: App, codex_home: PathBuf, refresh: Duration) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_app_loop(&mut terminal, &mut app, &codex_home, refresh);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run_app_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    codex_home: &Path,
    refresh: Duration,
) -> Result<()> {
    let mut last_refresh = Instant::now();
    loop {
        terminal.draw(|frame| tokidex::ui::draw(frame, app))?;

        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('q') if !app.search_mode => break,
                KeyCode::Char('/') if !app.search_mode => app.search_mode = true,
                KeyCode::Esc => {
                    app.search_mode = false;
                    app.search.clear();
                    app.recompute();
                }
                KeyCode::Enter => app.search_mode = false,
                KeyCode::Backspace if app.search_mode => {
                    app.search.pop();
                    app.recompute();
                }
                KeyCode::Char(ch) if app.search_mode => {
                    app.search.push(ch);
                    app.recompute();
                }
                KeyCode::Down | KeyCode::Char('j') => app.move_next(),
                KeyCode::Up | KeyCode::Char('k') => app.move_previous(),
                KeyCode::Char('d') => {
                    app.range = DateRange::Today;
                    app.recompute();
                }
                KeyCode::Char('w') => {
                    app.range = DateRange::Week;
                    app.recompute();
                }
                KeyCode::Char('a') => {
                    app.range = DateRange::All;
                    app.recompute();
                }
                KeyCode::Char('r') => refresh_records(app, codex_home),
                _ => {}
            }
        }

        if last_refresh.elapsed() >= refresh {
            refresh_records(app, codex_home);
            last_refresh = Instant::now();
        }
    }
    Ok(())
}

fn refresh_records(app: &mut App, codex_home: &Path) {
    match load_records(codex_home) {
        Ok(records) => {
            app.replace_records(records);
            app.status = "refreshed".to_string();
        }
        Err(err) => {
            app.status = format!("{err:#}");
        }
    }
}
