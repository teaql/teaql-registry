use std::sync::Arc;

use crate::api::RegistryClient;
use crate::types::{ComponentItem, RepositoryItem, SearchResponse, ServerOverview};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview = 0,
    Repositories = 1,
    Artifacts = 2,
    QuickOps = 3,
}

impl Tab {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Tab::Overview,
            1 => Tab::Repositories,
            2 => Tab::Artifacts,
            3 => Tab::QuickOps,
            _ => Tab::Overview,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Tab::Overview => "[1] Overview & Metrics",
            Tab::Repositories => "[2] Repositories",
            Tab::Artifacts => "[3] Artifact Search",
            Tab::QuickOps => "[4] Quick Ops",
        }
    }
}

pub struct App {
    pub client: Arc<RegistryClient>,
    pub current_tab: Tab,
    pub overview: ServerOverview,
    pub repositories: Vec<RepositoryItem>,
    pub search_results: SearchResponse,
    
    pub repo_cursor: usize,
    pub search_cursor: usize,
    pub search_input: String,
    pub is_searching: bool,

    pub status_message: String,
    pub is_loading: bool,
    pub op_log: Vec<String>,
}

impl App {
    pub fn new(client: RegistryClient) -> Self {
        Self {
            client: Arc::new(client),
            current_tab: Tab::Overview,
            overview: ServerOverview::default(),
            repositories: Vec::new(),
            search_results: SearchResponse::default(),
            repo_cursor: 0,
            search_cursor: 0,
            search_input: String::new(),
            is_searching: false,
            status_message: "Ready. Press 1-4 or Tab to switch views, 'r' to refresh, 'q' to quit.".to_string(),
            is_loading: false,
            op_log: vec!["Registry TUI started.".to_string()],
        }
    }

    pub async fn refresh_all(&mut self) {
        self.is_loading = true;
        self.status_message = "Refreshing data from registry...".to_string();

        if let Ok(overview) = self.client.fetch_overview().await {
            self.overview = overview;
        }

        if let Ok(repos) = self.client.fetch_repositories().await {
            self.repositories = repos;
            if self.repo_cursor >= self.repositories.len() && !self.repositories.is_empty() {
                self.repo_cursor = self.repositories.len() - 1;
            }
        }

        if let Ok(search) = self.client.search_components(&self.search_input).await {
            self.search_results = search;
            if self.search_cursor >= self.search_results.items.len() && !self.search_results.items.is_empty() {
                self.search_cursor = self.search_results.items.len() - 1;
            }
        }

        self.is_loading = false;
        self.status_message = format!(
            "Data refreshed at {}. {} Repositories, {} Components.",
            chrono::Local::now().format("%H:%M:%S"),
            self.repositories.len(),
            self.search_results.total
        );
    }

    pub fn next_tab(&mut self) {
        let next_idx = (self.current_tab as usize + 1) % 4;
        self.current_tab = Tab::from_index(next_idx);
    }

    pub fn prev_tab(&mut self) {
        let prev_idx = if self.current_tab as usize == 0 { 3 } else { self.current_tab as usize - 1 };
        self.current_tab = Tab::from_index(prev_idx);
    }

    pub fn set_tab(&mut self, tab: Tab) {
        self.current_tab = tab;
    }

    pub fn move_cursor_up(&mut self) {
        match self.current_tab {
            Tab::Repositories => {
                if self.repo_cursor > 0 {
                    self.repo_cursor -= 1;
                }
            }
            Tab::Artifacts => {
                if self.search_cursor > 0 {
                    self.search_cursor -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn move_cursor_down(&mut self) {
        match self.current_tab {
            Tab::Repositories => {
                if !self.repositories.is_empty() && self.repo_cursor + 1 < self.repositories.len() {
                    self.repo_cursor += 1;
                }
            }
            Tab::Artifacts => {
                if !self.search_results.items.is_empty() && self.search_cursor + 1 < self.search_results.items.len() {
                    self.search_cursor += 1;
                }
            }
            _ => {}
        }
    }

    pub fn selected_repo(&self) -> Option<&RepositoryItem> {
        self.repositories.get(self.repo_cursor)
    }

    pub fn selected_component(&self) -> Option<&ComponentItem> {
        self.search_results.items.get(self.search_cursor)
    }

    pub fn add_log(&mut self, msg: &str) {
        let entry = format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg);
        self.op_log.push(entry);
        if self.op_log.len() > 100 {
            self.op_log.remove(0);
        }
    }
}
