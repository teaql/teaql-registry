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
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Status Cards
            Constraint::Length(8),  // Format Distribution
            Constraint::Min(6),     // Metrics raw / counters
        ])
        .split(area);

    // 1. Top Status Cards
    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(chunks[0]);

    let health_style = if app.overview.is_online {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    };

    let health_text = vec![
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(if app.overview.is_online { "● HEALTHY / ONLINE" } else { "● OFFLINE" }, health_style),
        ]),
        Line::from(vec![
            Span::raw("Uptime Engine: "),
            Span::styled("Rust + Axum (TeaQL)", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Multi-Tenancy: "),
            Span::styled("Tenant isolation active", Style::default().fg(Color::Gray)),
        ]),
    ];
    let health_card = Paragraph::new(health_text)
        .block(Block::default().borders(Borders::ALL).title(" [ Engine Health ] "));
    f.render_widget(health_card, status_chunks[0]);

    let repo_stats_text = vec![
        Line::from(vec![
            Span::raw("Total Repositories: "),
            Span::styled(format!("{}", app.overview.total_repositories), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("  • Hosted: "),
            Span::styled(format!("{}", app.overview.hosted_count), Style::default().fg(Color::Cyan)),
            Span::raw("  • Proxy: "),
            Span::styled(format!("{}", app.overview.proxy_count), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::raw("  • Group: "),
            Span::styled(format!("{}", app.overview.group_count), Style::default().fg(Color::Green)),
        ]),
    ];
    let repo_card = Paragraph::new(repo_stats_text)
        .block(Block::default().borders(Borders::ALL).title(" [ Repositories Summary ] "));
    f.render_widget(repo_card, status_chunks[1]);

    let storage_text = vec![
        Line::from(vec![
            Span::raw("Total Components: "),
            Span::styled(format!("{}", app.overview.total_components), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("Blob Storage: "),
            Span::styled("S3 / RustFS (Deduplicated)", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Retention Policy: "),
            Span::styled("Auto-cleanup ready", Style::default().fg(Color::Gray)),
        ]),
    ];
    let storage_card = Paragraph::new(storage_text)
        .block(Block::default().borders(Borders::ALL).title(" [ Artifacts & Storage ] "));
    f.render_widget(storage_card, status_chunks[2]);

    // 2. Format Distribution Table
    let rows: Vec<Row> = app
        .overview
        .format_counts
        .iter()
        .map(|(fmt, count)| {
            Row::new(vec![
                Span::styled(fmt.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} repo(s)", count)),
                Span::styled("Supported (Push/Pull/Proxy)", Style::default().fg(Color::Green)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(18),
            Constraint::Length(20),
            Constraint::Min(30),
        ],
    )
    .header(
        Row::new(vec!["PACKAGE FORMAT", "COUNT", "STATUS"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title(" [ Supported Package Ecosystems (8 Formats) ] "));
    f.render_widget(table, chunks[1]);

    // 3. Metrics Sample Table
    let mut metric_lines = Vec::new();
    for line in app.overview.metrics_raw.lines().take(15) {
        if !line.starts_with('#') && !line.trim().is_empty() {
            metric_lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(Color::Gray),
            )));
        }
    }

    if metric_lines.is_empty() {
        metric_lines.push(Line::from("No prometheus metrics available."));
    }

    let metrics_p = Paragraph::new(metric_lines)
        .block(Block::default().borders(Borders::ALL).title(" [ Prometheus Live Metrics Sample (/metrics) ] "));
    f.render_widget(metrics_p, chunks[2]);
}
