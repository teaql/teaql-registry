use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

mod api;
mod app;
mod types;
mod ui;

use api::RegistryClient;
use app::{App, Tab};

#[derive(Parser, Debug)]
#[command(name = "registry-tui", version = "0.1.0", about = "Terminal UI Client for TeaQL Registry")]
struct Args {
    /// Registry backend HTTP endpoint URL
    #[arg(short, long, env = "REGISTRY_ENDPOINT", default_value = "http://127.0.0.1:8081")]
    endpoint: String,

    /// Username for HTTP Basic Authentication
    #[arg(short, long, env = "REGISTRY_USER", default_value = "admin")]
    username: Option<String>,

    /// Password for HTTP Basic Authentication
    #[arg(short, long, env = "REGISTRY_PASSWORD", default_value = "admin123")]
    password: Option<String>,

    /// Personal Access Token (PAT) for Bearer authentication
    #[arg(short, long, env = "REGISTRY_TOKEN")]
    token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let client = RegistryClient::new(
        &args.endpoint,
        args.username.as_deref(),
        args.password.as_deref(),
        args.token.as_deref(),
    );

    let mut app = App::new(client);
    app.refresh_all().await;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::render_app(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // If in search input mode
                    if app.is_searching {
                        match key.code {
                            KeyCode::Esc | KeyCode::Enter => {
                                app.is_searching = false;
                                app.refresh_all().await;
                            }
                            KeyCode::Backspace => {
                                app.search_input.pop();
                            }
                            KeyCode::Char(c) => {
                                app.search_input.push(c);
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // Standard global keys
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Tab => {
                            app.next_tab();
                        }
                        KeyCode::BackTab => {
                            app.prev_tab();
                        }
                        KeyCode::Char('1') => app.set_tab(Tab::Overview),
                        KeyCode::Char('2') => app.set_tab(Tab::Repositories),
                        KeyCode::Char('3') => app.set_tab(Tab::Artifacts),
                        KeyCode::Char('4') => app.set_tab(Tab::QuickOps),
                        KeyCode::Char('r') => {
                            app.refresh_all().await;
                        }
                        KeyCode::Char('/') => {
                            app.set_tab(Tab::Artifacts);
                            app.is_searching = true;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.move_cursor_up();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.move_cursor_down();
                        }
                        KeyCode::Char('g') => {
                            // Quick Ops: Run GC
                            app.add_log("Starting BlobStore Garbage Collection...");
                            match app.client.run_gc().await {
                                Ok(rep) => {
                                    app.add_log(&format!(
                                        "SUCCESS: GC purged {} orphaned blobs, freed {:.2} KB.",
                                        rep.orphaned_blobs_deleted,
                                        rep.freed_bytes as f64 / 1024.0
                                    ));
                                }
                                Err(e) => {
                                    app.add_log(&format!("ERROR: GC failed: {}", e));
                                }
                            }
                            app.refresh_all().await;
                        }
                        KeyCode::Char('c') => {
                            // Quick Ops: Retention Cleanup
                            let target_repo = app
                                .selected_repo()
                                .map(|r| r.name.clone())
                                .unwrap_or_else(|| "maven-releases".to_string());

                            app.add_log(&format!("Running retention cleanup on {}...", target_repo));
                            match app.client.run_cleanup(&target_repo, 5).await {
                                Ok(rep) => {
                                    app.add_log(&format!(
                                        "SUCCESS: Cleanup deleted {} old components ({} assets), freed {:.2} KB.",
                                        rep.deleted_components_count,
                                        rep.deleted_assets_count,
                                        rep.freed_bytes as f64 / 1024.0
                                    ));
                                }
                                Err(e) => {
                                    app.add_log(&format!("ERROR: Cleanup failed: {}", e));
                                }
                            }
                            app.refresh_all().await;
                        }
                        KeyCode::Char('t') => {
                            // Quick Ops: Generate Temp Token
                            app.add_log("Generating temporary 7-day PAT token...");
                            match app.client.create_temp_token("tui-temp-cli-token").await {
                                Ok(token) => {
                                    app.add_log(&format!("TOKEN: {}", token));
                                }
                                Err(e) => {
                                    app.add_log(&format!("ERROR: Token generation failed: {}", e));
                                }
                            }
                        }
                        _ => {}
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
