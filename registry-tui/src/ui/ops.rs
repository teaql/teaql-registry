use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(8)])
        .split(area);

    // 1. Quick Ops Action Cards
    let action_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(chunks[0]);

    let gc_card = Paragraph::new(vec![
        Line::from(Span::styled("Press 'g' to trigger", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Scan and safely purge unreferenced orphaned blobs from S3 / RustFS storage."),
    ])
    .block(Block::default().borders(Borders::ALL).title(" [ (g) Garbage Collection ] "));
    f.render_widget(gc_card, action_chunks[0]);

    let cleanup_card = Paragraph::new(vec![
        Line::from(Span::styled("Press 'c' to trigger", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Enforce retention rules: retain max 5 latest versions for each component."),
    ])
    .block(Block::default().borders(Borders::ALL).title(" [ (c) Retention Cleanup ] "));
    f.render_widget(cleanup_card, action_chunks[1]);

    let token_card = Paragraph::new(vec![
        Line::from(Span::styled("Press 't' to trigger", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Generate a temporary 7-day PAT token (tql_pat_*) for local SSH / CI debug."),
    ])
    .block(Block::default().borders(Borders::ALL).title(" [ (t) Generate Temp Token ] "));
    f.render_widget(token_card, action_chunks[2]);

    // 2. Operation Log Window
    let log_lines: Vec<Line> = app
        .op_log
        .iter()
        .rev()
        .map(|msg| {
            let color = if msg.contains("ERROR") || msg.contains("failed") {
                Color::Red
            } else if msg.contains("SUCCESS") || msg.contains("completed") || msg.contains("Freed") {
                Color::Green
            } else if msg.contains("TOKEN:") {
                Color::Yellow
            } else {
                Color::Gray
            };
            Line::from(Span::styled(msg.clone(), Style::default().fg(color)))
        })
        .collect();

    let log_window = Paragraph::new(log_lines)
        .block(Block::default().borders(Borders::ALL).title(" [ Operation History & Live Reports ] "));
    f.render_widget(log_window, chunks[1]);
}
