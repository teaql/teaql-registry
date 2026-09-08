use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};

mod api;
mod app;
mod types;
mod ui;

use api::{RegistryApi, RegistryClient};
use app::{Action, App};

#[derive(Parser, Debug)]
#[command(
    name = "registry-tui",
    version = "0.1.0",
    about = "Terminal UI Client for TeaQL Registry"
)]
struct Args {
    /// Registry backend HTTP endpoint URL
    #[arg(
        short,
        long,
        env = "REGISTRY_ENDPOINT",
        default_value = "http://127.0.0.1:8081"
    )]
    endpoint: String,

    /// Username for HTTP Basic Authentication
    #[arg(short, long, env = "REGISTRY_USER")]
    username: Option<String>,

    /// Password for HTTP Basic Authentication
    #[arg(short, long, env = "REGISTRY_PASSWORD")]
    password: Option<String>,

    /// Personal Access Token (PAT) for Bearer authentication
    #[arg(short, long, env = "REGISTRY_TOKEN")]
    token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Resolve credentials: token > username/password > interactive prompt
    let (username, password, token) = resolve_credentials(&args)?;

    let client: Arc<dyn RegistryApi> = Arc::new(RegistryClient::new(
        &args.endpoint,
        username.as_deref(),
        password.as_deref(),
        token.as_deref(),
    ));

    // Verify connection before entering TUI
    eprint!("Connecting to {} ... ", args.endpoint);
    match client.verify_connection().await {
        Ok(true) => eprintln!("ok"),
        Ok(false) => {
            eprintln!("failed (authentication error)");
            eprintln!("Check your username/password or token.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("failed");
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    let mut app = App::new(client);
    app.initialize().await;

    // Run the TUI event loop
    run_tui(&mut app).await?;

    Ok(())
}

/// Resolve credentials from args or interactive prompt.
fn resolve_credentials(args: &Args) -> Result<(Option<String>, Option<String>, Option<String>)> {
    if args.token.is_some() {
        return Ok((
            args.username.clone(),
            args.password.clone(),
            args.token.clone(),
        ));
    }

    if args.username.is_some() && args.password.is_some() {
        return Ok((args.username.clone(), args.password.clone(), None));
    }

    // Interactive login
    eprintln!("TeaQL Registry TUI — {}", args.endpoint);
    eprintln!();

    let user = if let Some(u) = &args.username {
        u.clone()
    } else {
        eprint!("Username: ");
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        buf.trim().to_string()
    };

    let pass = if let Some(p) = &args.password {
        p.clone()
    } else {
        eprint!("Password: ");
        rpassword::read_password()?
    };

    Ok((Some(user), Some(pass), None))
}

/// Terminal setup, event loop, teardown.
/// All business logic is delegated to `app.handle_key()`.
async fn run_tui(app: &mut App) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::render_app(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let action = app.handle_key(key.code).await;
                    if action == Action::Quit {
                        break;
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
