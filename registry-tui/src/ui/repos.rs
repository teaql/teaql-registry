use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // Left Pane: Repositories Table
    let rows: Vec<Row> = app
        .repositories
        .iter()
        .enumerate()
        .map(|(idx, r)| {
            let is_selected = idx == app.repo_cursor;
            let style = if is_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if is_selected { "▶ " } else { "  " };

            let type_color = match r.repo_type.to_lowercase().as_str() {
                "hosted" => Color::Cyan,
                "proxy" => Color::Magenta,
                "group" => Color::Yellow,
                _ => Color::White,
            };

            Row::new(vec![
                Span::styled(format!("{}{}", prefix, r.name), style),
                Span::styled(r.format.to_uppercase(), Style::default().fg(Color::Yellow)),
                Span::styled(r.repo_type.to_uppercase(), Style::default().fg(type_color)),
                Span::styled(if r.online { "ONLINE" } else { "OFFLINE" }, Style::default().fg(Color::Green)),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(24),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["REPOSITORY NAME", "FORMAT", "TYPE", "STATUS"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title(" [ Repositories (↑/↓ to select) ] "));

    f.render_widget(table, chunks[0]);

    // Right Pane: Selected Repo Inspector
    if let Some(r) = app.selected_repo() {
        let fmt = r.format.to_lowercase();
        let client_guide = match fmt.as_str() {
            "maven2" => format!(
                "<repository>\n  <id>{}</id>\n  <url>{}{}</url>\n</repository>",
                r.name, app.client_endpoint(), r.url
            ),
            "npm" => format!(
                "npm config set registry {}{}/npm/",
                app.client_endpoint(), r.url
            ),
            "docker" => format!(
                "docker pull {}{}/<image>:<tag>",
                app.client_endpoint().trim_start_matches("http://").trim_start_matches("https://"),
                r.url
            ),
            "pypi" => format!(
                "pip install --index-url {}{}/simple/ <pkg>",
                app.client_endpoint(), r.url
            ),
            "cargo" => format!(
                "[registries.{}]\nindex = \"sparse+{}{}cargo/index/\"",
                r.name, app.client_endpoint(), r.url
            ),
            "gomod" => format!(
                "export GOPROXY={}{}/gomod,direct",
                app.client_endpoint(), r.url
            ),
            "nuget" => format!(
                "dotnet nuget add source {}{}/v3/index.json -n {}",
                app.client_endpoint(), r.url, r.name
            ),
            _ => format!("curl -O {}{}/<path>", app.client_endpoint(), r.url),
        };

        let details_text = vec![
            Line::from(vec![
                Span::styled("Repository: ", Style::default().fg(Color::Gray)),
                Span::styled(&r.name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Format:     ", Style::default().fg(Color::Gray)),
                Span::styled(r.format.to_uppercase(), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Type:       ", Style::default().fg(Color::Gray)),
                Span::styled(r.repo_type.to_uppercase(), Style::default().fg(Color::Magenta)),
            ]),
            Line::from(vec![
                Span::styled("Endpoint:   ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{}{}", app.client_endpoint(), r.url), Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(Span::styled("─── Client Setup Snippet ───", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled(client_guide, Style::default().fg(Color::LightCyan))),
        ];

        let details_p = Paragraph::new(details_text)
            .block(Block::default().borders(Borders::ALL).title(" [ Repository Inspector ] "));
        f.render_widget(details_p, chunks[1]);
    } else {
        let empty_p = Paragraph::new("No repository selected.")
            .block(Block::default().borders(Borders::ALL).title(" [ Repository Inspector ] "));
        f.render_widget(empty_p, chunks[1]);
    }
}

impl App {
    pub fn client_endpoint(&self) -> &str {
        // Just a helper to format URLs nicely
        "http://localhost:8081"
    }
}
