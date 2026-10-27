use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, Block};

pub fn render_app(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top bar
            Constraint::Min(6),    // Transcript area
            Constraint::Length(3), // Input area (prompt + hints)
        ])
        .split(f.area());

    render_top_bar(f, app, chunks[0]);
    render_transcript(f, app, chunks[1]);
    render_input(f, app, chunks[2]);
}

fn render_top_bar(f: &mut Frame, app: &App, area: Rect) {
    let endpoint = app.endpoint.trim_end_matches('/');
    let dashes = "─".repeat(area.width.saturating_sub((endpoint.len() + 21) as u16) as usize);
    let line = Line::from(vec![
        Span::styled(
            " ❄ TeaQL Registry ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(dashes, Style::default().fg(Color::DarkGray)),
        Span::styled(format!(" {endpoint}"), Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let input_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    // Separator
    let sep = Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(Color::DarkGray),
    ));
    f.render_widget(Paragraph::new(sep), input_chunks[0]);

    // Input line with cursor
    let before = &app.input[..app.input_cursor];
    let after = &app.input[app.input_cursor..];
    let available = area.width.saturating_sub(6) as usize;
    let after_chars: Vec<char> = after.chars().collect();
    let after_visible = after_chars.len().min(available.saturating_sub(1) / 2);
    let before_budget = available.saturating_sub(after_visible + 1);
    let mut before_chars: Vec<char> = before.chars().rev().take(before_budget).collect();
    before_chars.reverse();
    let before_visible: String = before_chars.into_iter().collect();
    let after_visible: String = after_chars.into_iter().take(after_visible).collect();
    let input_line = Line::from(vec![
        Span::styled(
            "  > ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(before_visible, Style::default().fg(Color::White)),
        Span::styled("▎", Style::default().fg(Color::Yellow)),
        Span::styled(after_visible, Style::default().fg(Color::White)),
    ]);
    f.render_widget(Paragraph::new(input_line), input_chunks[1]);

    // Hint line
    let hints = Line::from(vec![
        Span::styled("    ", Style::default()),
        Span::styled(
            "home · failures [name] · ready <format> <coordinate> · connect [repo] · search <name> · ops · q",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    f.render_widget(Paragraph::new(hints), input_chunks[2]);
}

fn render_transcript(f: &mut Frame, app: &App, area: Rect) {
    let padded = area.inner(Margin {
        vertical: 0,
        horizontal: 2,
    });

    // Flatten all blocks into owned lines
    let mut all_lines: Vec<Line<'static>> = Vec::new();

    let last_interactive_idx = app
        .blocks
        .iter()
        .enumerate()
        .rev()
        .find(|(_, b)| b.is_interactive())
        .map(|(i, _)| i);

    for (block_idx, block) in app.blocks.iter().enumerate() {
        let is_active = last_interactive_idx == Some(block_idx);
        render_block_to_lines(block, is_active, padded.width, &mut all_lines);
        all_lines.push(Line::from("")); // spacer between blocks
    }

    // Apply scroll offset (from bottom)
    let visible_height = padded.height as usize;
    let total = all_lines.len();
    let end = total.saturating_sub(app.scroll_offset);
    let start = end.saturating_sub(visible_height);

    let visible_lines: Vec<Line> = all_lines
        .into_iter()
        .skip(start)
        .take(end - start)
        .collect();
    f.render_widget(Paragraph::new(visible_lines), padded);
}

fn section_line(title: &str, width: u16) -> Line<'static> {
    let dashes_len = (width as usize).saturating_sub(title.len() + 4);
    let dashes = "─".repeat(dashes_len);
    Line::from(vec![
        Span::styled("── ".to_string(), Style::default().fg(Color::DarkGray)),
        Span::styled(
            title.to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {}", dashes), Style::default().fg(Color::DarkGray)),
    ])
}

/// Convert a Block into a sequence of owned Line<'static> values.
fn render_block_to_lines(
    block: &Block,
    is_active: bool,
    width: u16,
    lines: &mut Vec<Line<'static>>,
) {
    match block {
        Block::Command(cmd) => {
            lines.push(Line::from(vec![
                Span::styled("  > ".to_string(), Style::default().fg(Color::DarkGray)),
                Span::styled(
                    cmd.clone(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        Block::Info(msg) => {
            lines.push(Line::from(vec![
                Span::styled("  ℹ ".to_string(), Style::default().fg(Color::Cyan)),
                Span::styled(msg.clone(), Style::default().fg(Color::Cyan)),
            ]));
        }

        Block::Success(msg) => {
            lines.push(Line::from(vec![
                Span::styled("  ✓ ".to_string(), Style::default().fg(Color::Green)),
                Span::styled(msg.clone(), Style::default().fg(Color::Green)),
            ]));
        }

        Block::Error(msg) => {
            lines.push(Line::from(vec![
                Span::styled("  ✗ ".to_string(), Style::default().fg(Color::Red)),
                Span::styled(msg.clone(), Style::default().fg(Color::Red)),
            ]));
        }

        Block::Home {
            recent_failures,
            repository_count,
            logs_error,
            cursor,
        } => {
            lines.push(section_line("What do you need to do?", width));
            lines.push(Line::from(Span::styled(
                format!("    {repository_count} repositories available · ↑↓ choose, Enter open · type a command at any time"),
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
            let tasks = [
                ("1", "Diagnose a failed request", "failures [package]"),
                (
                    "2",
                    "Check offline inventory",
                    "ready <format> <coordinates>",
                ),
                ("3", "Connect a native client", "connect [repository]"),
                ("4", "Verify a publication", "search <package>"),
                ("5", "Maintain the registry", "ops"),
            ];
            for (index, (number, label, command)) in tasks.iter().enumerate() {
                let selected = is_active && index == *cursor;
                let style = if selected {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                lines.push(Line::from(vec![
                    Span::styled(if selected { "  ▸ " } else { "    " }, style),
                    Span::styled(format!("{number}. {label:<30}"), style),
                    Span::styled((*command).to_string(), Style::default().fg(Color::Cyan)),
                ]));
            }
            lines.push(Line::from(""));
            lines.push(section_line("Recent failed requests", width));
            if *logs_error {
                lines.push(Line::from(
                    "    Request history unavailable. Type 'failures' to retry.",
                ));
            } else if recent_failures.is_empty() {
                lines.push(Line::from("    No failures in the latest 20 requests."));
            } else {
                for log in recent_failures {
                    lines.push(Line::from(format!(
                        "    {}  {}  {}",
                        log.status, log.repository_name, log.artifact_path
                    )));
                }
                lines.push(Line::from(Span::styled(
                    "    Type 'failures' to inspect the latest 100 requests.",
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        Block::FailureList {
            query,
            logs,
            cursor,
        } => {
            let (start, end) = visible_window(*cursor, logs.len(), 8);
            let title = if query.is_empty() {
                format!("Recent failed requests ({})", logs.len())
            } else {
                format!("Failures matching '{query}' ({})", logs.len())
            };
            lines.push(section_line(&title, width));
            lines.push(Line::from(
                "    Most recent 100 requests · ↑↓ select · Enter inspect",
            ));
            if logs.is_empty() {
                lines.push(Line::from("    No matching failures in this window."));
            }
            for (index, log) in logs.iter().enumerate().take(end).skip(start) {
                let selected = is_active && index == *cursor;
                let style = if selected {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                lines.push(Line::from(vec![
                    Span::styled(if selected { "  ▸ " } else { "    " }, style),
                    Span::styled(
                        format!("{:<8} {:<20} ", log.status, log.repository_name),
                        Style::default().fg(Color::Red),
                    ),
                    Span::styled(log.artifact_path.clone(), style),
                ]));
            }
        }

        Block::FailureDetail { log } => {
            lines.push(section_line("Request failure", width));
            for (label, value) in [
                ("Time", log.event_time.as_str()),
                ("Status", log.status.as_str()),
                ("Action", log.action.as_str()),
                ("Repository", log.repository_name.as_str()),
                ("Artifact", log.artifact_path.as_str()),
                ("Error", log.error_message.as_str()),
            ] {
                lines.push(Line::from(format!("    {label:<12} {value}")));
            }
            lines.push(Line::from("    Next: search <package> to inspect inventory; connect <repo> to verify client URL."));
        }

        Block::ReadinessGuide => {
            lines.push(section_line("Check offline inventory", width));
            lines.push(Line::from("    Enter 1–20 exact versions in one command:"));
            lines.push(Line::from(
                "    ready maven2 org.example:library:1.2.3 [more...] [--repo maven-releases]",
            ));
            lines.push(Line::from(
                "    ready npm @scope/package@1.2.3 [more...] [--repo npm-hosted]",
            ));
            lines.push(Line::from(
                "    ready cargo serde@1.0.0 [more...] [--repo cargo-hosted]",
            ));
            lines.push(Line::from("    Recorded means catalog presence only; it does not prove blobs or transitive dependencies."));
        }

        Block::ReadinessResults { results } => {
            let found = results
                .iter()
                .filter(|result| result.repository.is_some())
                .count();
            lines.push(section_line(
                &format!("Offline inventory: {found}/{} recorded", results.len()),
                width,
            ));
            for result in results {
                let label = result.repository.as_deref().unwrap_or("Missing");
                lines.push(Line::from(format!(
                    "    {}  {}  {label}",
                    if result.repository.is_some() {
                        "✓"
                    } else {
                        "✗"
                    },
                    result.coordinate
                )));
            }
            lines.push(Line::from("    Final check: empty the native client cache, disconnect upstream, then run the project build."));
        }

        Block::Operations => {
            lines.push(section_line("Registry maintenance", width));
            lines.push(Line::from(
                "    repos                 Review repository routes and formats",
            ));
            lines.push(Line::from(
                "    status                Check service and inventory totals",
            ));
            lines.push(Line::from("    gc                    Purge orphaned blobs"));
            lines.push(Line::from(
                "    cleanup <repo>        Apply retention cleanup to a repository",
            ));
            lines.push(Line::from(
                "    token                 Create a seven-day client token",
            ));
        }

        Block::Status { overview } => {
            lines.push(section_line("System Status", width));
            lines.push(Line::from(""));
            let health_style = if overview.is_online {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            };
            lines.push(Line::from(vec![
                Span::styled(
                    "    Engine      ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    if overview.is_online {
                        "● Online".to_string()
                    } else {
                        "● Offline".to_string()
                    },
                    health_style,
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "    Repositories  ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{}", overview.total_repositories),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(
                        "  (Hosted {} / Proxy {} / Group {})",
                        overview.hosted_count, overview.proxy_count, overview.group_count
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "    Components    ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{}", overview.total_components),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        Block::RepoList { repos, cursor } => {
            let total = repos.len();
            let page_size = 10usize;
            // Compute visible window around cursor
            let (win_start, win_end) = visible_window(*cursor, total, page_size);
            let range_hint = if total > page_size {
                format!("  {}-{} of {}  ↑↓ scroll", win_start + 1, win_end, total)
            } else {
                String::new()
            };
            lines.push(section_line(
                &format!("Repositories ({}){}", total, range_hint),
                width,
            ));
            lines.push(Line::from(""));

            lines.push(Line::from(vec![
                Span::styled(
                    "    NAME                        ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    "FORMAT     ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    "TYPE       ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));

            for (idx, r) in repos.iter().enumerate().take(win_end).skip(win_start) {
                let is_selected = is_active && idx == *cursor;
                let prefix = if is_selected { " ▸  " } else { "    " };
                let style = if is_selected {
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let type_color = match r.repo_type.to_lowercase().as_str() {
                    "hosted" => Color::Cyan,
                    "proxy" => Color::Magenta,
                    "group" => Color::Yellow,
                    _ => Color::White,
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("{}{:<28}", prefix, r.name), style),
                    Span::styled(
                        format!("{:<11}", r.format.to_uppercase()),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(
                        format!("{:<11}", r.repo_type.to_uppercase()),
                        Style::default().fg(type_color),
                    ),
                    Span::styled(
                        if r.online {
                            "●".to_string()
                        } else {
                            "○".to_string()
                        },
                        Style::default().fg(if r.online { Color::Green } else { Color::Red }),
                    ),
                ]));
            }
        }

        Block::RepoDetail { repo, endpoint } => {
            lines.push(section_line(&format!("Inspector  {}", repo.name), width));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    "    Name      ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    repo.name.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "    Format    ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    repo.format.to_uppercase(),
                    Style::default().fg(Color::Yellow),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "    Type      ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    repo.repo_type.to_uppercase(),
                    Style::default().fg(Color::Magenta),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "    Endpoint  ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{}{}", endpoint, repo.url),
                    Style::default().fg(Color::White),
                ),
            ]));
            lines.push(Line::from(""));

            let fmt = repo.format.to_lowercase();
            let snippet = match fmt.as_str() {
                "maven2" => format!(
                    "<repository>\n      <id>{}</id>\n      <url>{}{}</url>\n    </repository>",
                    repo.name, endpoint, repo.url
                ),
                "npm" => format!("npm config set registry {}{}/npm/", endpoint, repo.url),
                "docker" => format!(
                    "docker pull {}{}/IMAGE:TAG",
                    endpoint
                        .trim_start_matches("http://")
                        .trim_start_matches("https://"),
                    repo.url
                ),
                "pypi" => format!(
                    "pip install --index-url {}{}/simple/ PKG",
                    endpoint, repo.url
                ),
                "cargo" => format!(
                    "[registries.{}]\n    index = \"sparse+{}{}cargo/index/\"",
                    repo.name, endpoint, repo.url
                ),
                "gomod" => format!("GOPROXY={}{}/gomod,direct", endpoint, repo.url),
                "nuget" => format!(
                    "dotnet nuget add source {}{}/v3/index.json -n {}",
                    endpoint, repo.url, repo.name
                ),
                "swift" => format!("swift package-registry set {}{}/swift", endpoint, repo.url),
                "dart" => format!("PUB_HOSTED_URL={}{}/dart dart pub get", endpoint, repo.url),
                "rubygems" => format!("gem sources --add {}{}/rubygems", endpoint, repo.url),
                "composer" => format!(
                    "composer config repositories.{} composer {}{}/composer",
                    repo.name, endpoint, repo.url
                ),
                "conan" => format!(
                    "conan remote add {} {}{}/conan",
                    repo.name, endpoint, repo.url
                ),
                "hex" => format!(
                    "mix hex.repo add {} {}{}/hex/repo --public-key public_key.pem",
                    repo.name, endpoint, repo.url
                ),
                _ => format!("curl -O {}{}/PATH", endpoint, repo.url),
            };
            lines.push(Line::from(Span::styled(
                "    Client Setup".to_string(),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            for snippet_line in snippet.lines() {
                lines.push(Line::from(Span::styled(
                    format!("    {}", snippet_line),
                    Style::default().fg(Color::LightCyan),
                )));
            }
        }

        Block::SearchResults {
            keyword,
            results,
            cursor,
        } => {
            let total = results.items.len();
            let page_size = 10usize;
            let (win_start, win_end) = visible_window(*cursor, total, page_size);
            let range_hint = if total > page_size {
                format!("  {}-{} of {}  ↑↓ scroll", win_start + 1, win_end, total)
            } else {
                String::new()
            };
            let title = if keyword.is_empty() {
                format!("Search Results ({}){}", results.total, range_hint)
            } else {
                format!("Search \"{}\" ({}){}", keyword, results.total, range_hint)
            };
            lines.push(section_line(&title, width));
            lines.push(Line::from(""));

            if results.items.is_empty() {
                lines.push(Line::from(Span::styled(
                    "    No components found.".to_string(),
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                lines.push(Line::from(vec![
                    Span::styled(
                        "    NAME                      ".to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        "GROUP                  ".to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        "VERSION      ".to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        "FORMAT     ".to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        "REPOSITORY".to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));

                for idx in win_start..win_end {
                    let comp = &results.items[idx];
                    let is_selected = is_active && idx == *cursor;
                    let prefix = if is_selected { " ▸  " } else { "    " };
                    let style = if is_selected {
                        Style::default()
                            .bg(Color::DarkGray)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    lines.push(Line::from(vec![
                        Span::styled(format!("{}{:<26}", prefix, comp.name), style),
                        Span::styled(
                            format!("{:<23}", comp.group),
                            Style::default().fg(Color::Gray),
                        ),
                        Span::styled(
                            format!("v{:<12}", comp.version),
                            Style::default().fg(Color::Green),
                        ),
                        Span::styled(
                            format!("{:<11}", comp.format.to_uppercase()),
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::styled(comp.repository.clone(), Style::default().fg(Color::Cyan)),
                    ]));
                }
            }
        }

        Block::ComponentDetail { component } => {
            lines.push(section_line(
                &format!("{} v{}", component.name, component.version),
                width,
            ));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    "    Group       ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(component.group.clone(), Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "    Format      ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    component.format.to_uppercase(),
                    Style::default().fg(Color::Yellow),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "    Repository  ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    component.repository.clone(),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "    Assets      ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{} file(s)", component.assets.len()),
                    Style::default().fg(Color::White),
                ),
            ]));
            lines.push(Line::from(""));

            if !component.assets.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled(
                        "    PATH                                   ".to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        "SIZE         ".to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled("TYPE".to_string(), Style::default().fg(Color::DarkGray)),
                ]));
                for asset in &component.assets {
                    let size_str = format!("{:.1} KB", asset.size as f64 / 1024.0);
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("    {:<39}", asset.path),
                            Style::default().fg(Color::White),
                        ),
                        Span::styled(
                            format!("{:<13}", size_str),
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::styled(asset.content_type.clone(), Style::default().fg(Color::Gray)),
                    ]));
                }
                lines.push(Line::from(""));
            }

            // Usage instructions based on format
            let snippet = component_usage_snippet(component);
            if !snippet.is_empty() {
                lines.push(Line::from(Span::styled(
                    "    How to Use".to_string(),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                for s in snippet.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("    {}", s),
                        Style::default().fg(Color::LightCyan),
                    )));
                }
            }
        }

        Block::Help => {
            lines.push(section_line("Task commands", width));
            lines.push(Line::from(""));
            let commands: &[(&str, &str)] = &[
                ("home", "Return to the task menu and recent failures"),
                ("failures [package]", "Inspect recent failed requests"),
                ("ready <format> ...", "Check exact versions in the catalog"),
                ("connect [repo]", "Get native-client setup for a repository"),
                ("search <package>", "Verify versions and files"),
                ("ops", "Show maintenance commands"),
                ("repos / comps", "Browse repositories / components"),
                ("inspect [name]", "Inspect selected or named item"),
                ("status / gc / cleanup", "Status and storage maintenance"),
                ("token", "Generate a temporary client token"),
                ("clear, cls", "Clear the transcript"),
                ("help, h, ?", "Show this help"),
                ("quit, q", "Exit"),
            ];
            for (cmd, desc) in commands {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("    {:<22}", cmd),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(desc.to_string(), Style::default().fg(Color::Gray)),
                ]));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    "    Navigation  ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("↑↓ ".to_string(), Style::default().fg(Color::Yellow)),
                Span::styled("select   ".to_string(), Style::default().fg(Color::Gray)),
                Span::styled("Enter ".to_string(), Style::default().fg(Color::Yellow)),
                Span::styled("inspect   ".to_string(), Style::default().fg(Color::Gray)),
                Span::styled("PgUp/PgDn ".to_string(), Style::default().fg(Color::Yellow)),
                Span::styled("scroll".to_string(), Style::default().fg(Color::Gray)),
            ]));
        }
    }
}

/// Compute a visible window of `page_size` items centered around `cursor`.
/// Returns (start, end) where end is exclusive.
fn visible_window(cursor: usize, total: usize, page_size: usize) -> (usize, usize) {
    if total <= page_size {
        return (0, total);
    }
    // Try to center cursor in the window
    let half = page_size / 2;
    let start = if cursor < half {
        0
    } else if cursor + half >= total {
        total.saturating_sub(page_size)
    } else {
        cursor - half
    };
    let end = (start + page_size).min(total);
    (start, end)
}

/// Generate a format-specific usage snippet for a component.
fn component_usage_snippet(component: &crate::types::ComponentItem) -> String {
    let fmt = component.format.to_lowercase();
    match fmt.as_str() {
        "maven2" => format!(
            "<dependency>\n  <groupId>{}</groupId>\n  <artifactId>{}</artifactId>\n  <version>{}</version>\n</dependency>",
            component.group, component.name, component.version
        ),
        "npm" => {
            if component.group.is_empty() || component.group == "/" {
                format!("npm install {}@{}", component.name, component.version)
            } else {
                format!("npm install @{}/{}@{}", component.group, component.name, component.version)
            }
        }
        "pypi" => format!("pip install {}=={}", component.name, component.version),
        "cargo" => format!(
            "# Cargo.toml\n[dependencies]\n{} = \"{}\"",
            component.name, component.version
        ),
        "nuget" => format!(
            "dotnet add package {} --version {}",
            component.name, component.version
        ),
        "swift" => format!(
            ".package(id: \"{}.{}\", exact: \"{}\")",
            component.group, component.name, component.version
        ),
        "dart" => format!("dart pub add {} --hosted REGISTRY_URL", component.name),
        "rubygems" => format!(
            "gem install {} -v {} --source REGISTRY_URL",
            component.name, component.version
        ),
        "composer" => {
            let package = if component.group.is_empty() {
                component.name.clone()
            } else {
                format!("{}/{}", component.group, component.name)
            };
            format!("composer require {}:{}", package, component.version)
        }
        "conan" => format!(
            "conan install --requires={}/{} -r={} --build=missing",
            component.name,
            component.version.split('#').next().unwrap_or(&component.version),
            component.repository
        ),
        "hex" => format!(
            "mix hex.package fetch {} {} --repo={}",
            component.name, component.version, component.repository
        ),
        "docker" => format!(
            "docker pull {}:{}",
            component.name, component.version
        ),
        "gomod" => format!(
            "go get {}/{}@v{}",
            component.group, component.name, component.version
        ),
        "raw" => {
            if let Some(asset) = component.assets.first() {
                format!("curl -O http://localhost:8081/repository/{}/{}", component.repository, asset.path)
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::RegistryClient;
    use ratatui::{backend::TestBackend, Terminal};
    use std::sync::Arc;

    #[test]
    fn home_tasks_fit_a_standard_terminal() {
        let endpoint = "https://registry.example.test".to_string();
        let mut app = App::new(
            Arc::new(RegistryClient::new(&endpoint, None, None, None)),
            endpoint,
        );
        app.blocks.push(Block::Home {
            recent_failures: Vec::new(),
            repository_count: 18,
            logs_error: false,
            cursor: 0,
        });
        let mut terminal = Terminal::new(TestBackend::new(90, 24)).unwrap();
        terminal.draw(|frame| render_app(frame, &app)).unwrap();
        let buffer = terminal.backend().buffer();
        let screen: String = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        for label in [
            "Diagnose a failed request",
            "Check offline inventory",
            "Connect a native client",
            "Verify a publication",
            "Maintain the registry",
            "Recent failed requests",
        ] {
            assert!(screen.contains(label), "missing {label} on home screen");
        }
    }

    #[test]
    fn long_readiness_command_keeps_cursor_and_last_coordinate_visible() {
        let endpoint = "https://registry.example.test".to_string();
        let mut app = App::new(
            Arc::new(RegistryClient::new(&endpoint, None, None, None)),
            endpoint,
        );
        app.input = format!(
            "ready maven2 {} org.example:last:1.0.0",
            "org.example:library:1.0.0 ".repeat(10)
        );
        app.input_cursor = app.input.len();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| render_app(frame, &app)).unwrap();
        let buffer = terminal.backend().buffer();
        let screen: String = (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(screen.contains("org.example:last:1.0.0▎"));
    }
}
