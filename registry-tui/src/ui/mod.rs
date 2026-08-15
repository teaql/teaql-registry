pub mod ops;
pub mod overview;
pub mod repos;
pub mod search;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, Tab};

pub fn render_app(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header & Tab bar
            Constraint::Min(10),   // Main Content View
            Constraint::Length(3), // Footer & Status Bar
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);

    match app.current_tab {
        Tab::Overview => overview::render(f, app, chunks[1]),
        Tab::Repositories => repos::render(f, app, chunks[1]),
        Tab::Artifacts => search::render(f, app, chunks[1]),
        Tab::QuickOps => ops::render(f, app, chunks[1]),
    }

    render_footer(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(26), Constraint::Min(40)])
        .split(area);

    let title_text = Line::from(vec![
        Span::styled(" ❄  TeaQL Registry TUI ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]);
    let title_block = Paragraph::new(title_text)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title_block, header_chunks[0]);

    let tab_titles = vec![
        Tab::Overview.title(),
        Tab::Repositories.title(),
        Tab::Artifacts.title(),
        Tab::QuickOps.title(),
    ];

    let tabs = Tabs::new(tab_titles)
        .select(app.current_tab as usize)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(tabs, header_chunks[1]);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let status_text = Line::from(vec![
        Span::raw("Status: "),
        Span::styled(&app.status_message, Style::default().fg(Color::Cyan)),
    ]);
    let status_bar = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status_bar, footer_chunks[0]);

    let help_text = Line::from(vec![
        Span::styled("Tab/1-4", Style::default().fg(Color::Yellow)),
        Span::raw(":View  "),
        Span::styled("↑↓/jk", Style::default().fg(Color::Yellow)),
        Span::raw(":Nav  "),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::raw(":Refresh  "),
        Span::styled("/", Style::default().fg(Color::Yellow)),
        Span::raw(":Search  "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(":Quit"),
    ]);
    let help_bar = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" [ Controls ] "));
    f.render_widget(help_bar, footer_chunks[1]);
}
