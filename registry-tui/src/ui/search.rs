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
            Constraint::Length(3),  // Search bar
            Constraint::Percentage(50), // Components table
            Constraint::Percentage(50), // Assets inspector
        ])
        .split(area);

    // 1. Search Bar
    let search_style = if app.is_searching {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let search_title = if app.is_searching {
        " [ Search Keyword (Type to search, press Enter/Esc to lock) ] "
    } else {
        " [ Search (Press '/' to search keyword) ] "
    };

    let search_content = if app.search_input.is_empty() && !app.is_searching {
        Span::styled("<All components. Press '/' to filter by name/version>", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(&app.search_input, search_style)
    };

    let search_bar = Paragraph::new(Line::from(vec![Span::raw("🔍 Query: "), search_content]))
        .block(Block::default().borders(Borders::ALL).title(search_title));
    f.render_widget(search_bar, chunks[0]);

    // 2. Components Table
    let rows: Vec<Row> = app
        .search_results
        .items
        .iter()
        .enumerate()
        .map(|(idx, comp)| {
            let is_selected = idx == app.search_cursor;
            let style = if is_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if is_selected { "▶ " } else { "  " };

            Row::new(vec![
                Span::styled(format!("{}{}", prefix, comp.name), style),
                Span::styled(&comp.group, Style::default().fg(Color::Gray)),
                Span::styled(format!("v{}", comp.version), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(comp.format.to_uppercase(), Style::default().fg(Color::Yellow)),
                Span::styled(&comp.repository, Style::default().fg(Color::Cyan)),
                Span::raw(format!("{} file(s)", comp.assets.len())),
            ])
            .style(style)
        })
        .collect();

    let comp_table = Table::new(
        rows,
        [
            Constraint::Length(28),
            Constraint::Length(24),
            Constraint::Length(14),
            Constraint::Length(10),
            Constraint::Length(18),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec!["COMPONENT NAME", "GROUP / NAMESPACE", "VERSION", "FORMAT", "REPOSITORY", "ASSETS"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title(format!(" [ Components Found ({}) (↑/↓ to inspect) ] ", app.search_results.total)));

    f.render_widget(comp_table, chunks[1]);

    // 3. Asset Inspector
    if let Some(comp) = app.selected_component() {
        let asset_rows: Vec<Row> = comp
            .assets
            .iter()
            .map(|asset| {
                let size_str = format!("{:.1} KB", asset.size as f64 / 1024.0);
                let sha256_str = asset
                    .checksum
                    .as_ref()
                    .and_then(|c| c.sha256.clone())
                    .unwrap_or_else(|| "N/A".to_string());

                Row::new(vec![
                    Span::styled(&asset.path, Style::default().fg(Color::White)),
                    Span::styled(size_str, Style::default().fg(Color::Yellow)),
                    Span::styled(&asset.content_type, Style::default().fg(Color::Gray)),
                    Span::styled(sha256_str, Style::default().fg(Color::Green)),
                ])
            })
            .collect();

        let asset_table = Table::new(
            asset_rows,
            [
                Constraint::Min(35),
                Constraint::Length(12),
                Constraint::Length(24),
                Constraint::Length(66),
            ],
        )
        .header(
            Row::new(vec!["ASSET FILE PATH", "SIZE", "CONTENT TYPE", "SHA-256 CHECKSUM"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        )
        .block(Block::default().borders(Borders::ALL).title(format!(" [ Artifact Assets for {} v{} ] ", comp.name, comp.version)));

        f.render_widget(asset_table, chunks[2]);
    } else {
        let empty_assets = Paragraph::new("No component selected.")
            .block(Block::default().borders(Borders::ALL).title(" [ Artifact Assets ] "));
        f.render_widget(empty_assets, chunks[2]);
    }
}
