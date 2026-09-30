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

    render_top_bar(f, chunks[0]);
    render_transcript(f, app, chunks[1]);
    render_input(f, app, chunks[2]);
}

fn render_top_bar(f: &mut Frame, area: Rect) {
    let dashes = "─".repeat(area.width.saturating_sub(20) as usize);
    let line = Line::from(vec![
        Span::styled(
            " ❄ TeaQL Registry ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(dashes, Style::default().fg(Color::DarkGray)),
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
    let before: String = app.input[..app.input_cursor].to_string();
    let after: String = app.input[app.input_cursor..].to_string();
    let input_line = Line::from(vec![
        Span::styled(
            "  > ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(before, Style::default().fg(Color::White)),
        Span::styled("▎", Style::default().fg(Color::Yellow)),
        Span::styled(after, Style::default().fg(Color::White)),
    ]);
    f.render_widget(Paragraph::new(input_line), input_chunks[1]);

    // Hint line
    let hints = Line::from(vec![
        Span::styled("    ", Style::default()),
        Span::styled(
            "repos · comps · search <kw> · inspect [name] · gc · cleanup · token · help · q",
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
            lines.push(section_line("Available Commands", width));
            lines.push(Line::from(""));
            let commands: &[(&str, &str)] = &[
                ("repos, r", "List all repositories"),
                ("comps [repo], c", "List components (filter by repo)"),
                ("search <keyword>, s", "Search components by name"),
                ("inspect [name], i", "Inspect selected or named item"),
                ("status, st", "Show system status"),
                ("gc", "Run garbage collection"),
                ("cleanup [repo]", "Run retention cleanup"),
                ("token", "Generate temporary PAT token"),
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
