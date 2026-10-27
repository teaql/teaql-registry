use std::sync::Arc;

use crossterm::event::KeyCode;

use crate::api::RegistryApi;
use crate::types::{
    ComponentItem, ComponentSearch, RepositoryItem, SearchResponse, ServerOverview, ServiceLogItem,
};

#[derive(Debug, Clone)]
pub struct ReadinessResult {
    pub coordinate: String,
    pub repository: Option<String>,
}

// ── Transcript Block ─────────────────────────────────────────────

/// A single rendered block in the transcript stream.
#[derive(Debug, Clone)]
pub enum Block {
    /// User command echo, e.g. "> repos"
    Command(String),
    /// System informational message
    Info(String),
    /// Success message
    Success(String),
    /// Error message
    Error(String),
    Home {
        recent_failures: Vec<ServiceLogItem>,
        repository_count: usize,
        logs_error: bool,
        cursor: usize,
    },
    FailureList {
        query: String,
        logs: Vec<ServiceLogItem>,
        cursor: usize,
    },
    FailureDetail {
        log: ServiceLogItem,
    },
    ReadinessGuide,
    ReadinessResults {
        results: Vec<ReadinessResult>,
    },
    Operations,
    /// Status dashboard
    Status {
        overview: ServerOverview,
    },
    /// Repository list (interactive if it's the latest table)
    RepoList {
        repos: Vec<RepositoryItem>,
        cursor: usize,
    },
    /// Repository detail inspector
    RepoDetail {
        repo: RepositoryItem,
        endpoint: String,
    },
    /// Search results (interactive if it's the latest table)
    SearchResults {
        keyword: String,
        results: SearchResponse,
        cursor: usize,
    },
    /// Component detail with assets
    ComponentDetail {
        component: ComponentItem,
    },
    /// Help text
    Help,
}

impl Block {
    /// Returns true if this block has an interactive cursor.
    pub fn is_interactive(&self) -> bool {
        matches!(
            self,
            Block::Home { .. }
                | Block::FailureList { .. }
                | Block::RepoList { .. }
                | Block::SearchResults { .. }
        )
    }
}

// ── Action ───────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
}

// ── App ──────────────────────────────────────────────────────────

pub struct App {
    pub client: Arc<dyn RegistryApi>,
    pub endpoint: String,

    /// The transcript — an ordered list of blocks.
    pub blocks: Vec<Block>,

    /// Scroll offset (in lines from bottom). 0 = pinned to bottom.
    pub scroll_offset: usize,

    /// Bottom input field content.
    pub input: String,

    /// Input cursor position within the input string.
    pub input_cursor: usize,

    /// Command history for up-arrow recall.
    pub cmd_history: Vec<String>,
    pub cmd_history_idx: Option<usize>,

    /// Cached data for operations.
    pub repositories: Vec<RepositoryItem>,
    pub search_results: SearchResponse,
    pub overview: ServerOverview,

    pub op_log: Vec<String>,
}

impl App {
    pub fn new(client: Arc<dyn RegistryApi>, endpoint: String) -> Self {
        Self {
            client,
            endpoint,
            blocks: Vec::new(),
            scroll_offset: 0,
            input: String::new(),
            input_cursor: 0,
            cmd_history: Vec::new(),
            cmd_history_idx: None,
            repositories: Vec::new(),
            search_results: SearchResponse::default(),
            overview: ServerOverview::default(),
            op_log: Vec::new(),
        }
    }

    // ── Initial data loading ─────────────────────────────────────

    /// Called once at startup: load data and push initial blocks.
    pub async fn initialize(&mut self) {
        if let Ok(repos) = self.client.fetch_repositories().await {
            self.repositories = repos;
        }
        self.show_home().await;
    }

    async fn show_home(&mut self) {
        let (recent_failures, logs_error) = match self.client.fetch_service_logs(20).await {
            Ok(logs) => (logs.into_iter().filter(is_failure).take(3).collect(), false),
            Err(_) => (Vec::new(), true),
        };
        self.blocks.push(Block::Home {
            recent_failures,
            repository_count: self.repositories.len(),
            logs_error,
            cursor: 0,
        });
    }

    pub async fn open_startup_package(&mut self, package: &str) {
        self.execute_command(&format!("search {package}")).await;
    }

    pub async fn open_startup_failure(&mut self, query: &str) {
        self.show_failures(query).await;
    }

    // ── Key handling ─────────────────────────────────────────────

    pub async fn handle_key(&mut self, key: KeyCode) -> Action {
        match key {
            // Submit command, or inspect selected item if input is empty
            KeyCode::Enter => {
                let cmd = self.input.trim().to_string();
                if !cmd.is_empty() {
                    self.cmd_history.push(cmd.clone());
                    self.cmd_history_idx = None;
                    self.input.clear();
                    self.input_cursor = 0;
                    self.scroll_offset = 0; // snap to bottom
                    return self.execute_command(&cmd).await;
                }
                // Input empty — inspect currently selected item
                self.activate_selection().await;
                self.scroll_offset = 0;
                Action::None
            }

            // Input editing
            KeyCode::Char(c) => {
                self.input.insert(self.input_cursor, c);
                self.input_cursor += c.len_utf8();
                self.cmd_history_idx = None;
                Action::None
            }
            KeyCode::Backspace => {
                if self.input_cursor > 0 {
                    self.input_cursor = self.input[..self.input_cursor]
                        .char_indices()
                        .last()
                        .map(|(index, _)| index)
                        .unwrap_or(0);
                    self.input.remove(self.input_cursor);
                }
                Action::None
            }
            KeyCode::Left => {
                if self.input_cursor > 0 {
                    self.input_cursor = self.input[..self.input_cursor]
                        .char_indices()
                        .last()
                        .map(|(index, _)| index)
                        .unwrap_or(0);
                }
                Action::None
            }
            KeyCode::Right => {
                if self.input_cursor < self.input.len() {
                    self.input_cursor += self.input[self.input_cursor..]
                        .chars()
                        .next()
                        .map(char::len_utf8)
                        .unwrap_or(0);
                }
                Action::None
            }
            KeyCode::Home => {
                self.input_cursor = 0;
                Action::None
            }
            KeyCode::End => {
                self.input_cursor = self.input.len();
                Action::None
            }

            // Cursor navigation on latest interactive block / command history
            KeyCode::Up => {
                if self.input.is_empty() {
                    // Try to move cursor in latest interactive block
                    if !self.move_active_cursor_up() {
                        // Otherwise, recall command history
                        self.recall_history_prev();
                    }
                } else {
                    self.recall_history_prev();
                }
                Action::None
            }
            KeyCode::Down => {
                if self.input.is_empty() {
                    self.move_active_cursor_down();
                } else {
                    self.recall_history_next();
                }
                Action::None
            }

            // Scroll transcript
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_add(10);
                Action::None
            }
            KeyCode::PageDown => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                Action::None
            }

            // Ctrl+C / Esc to quit
            KeyCode::Esc => Action::Quit,

            _ => Action::None,
        }
    }

    // ── Command execution ────────────────────────────────────────

    async fn execute_command(&mut self, cmd: &str) -> Action {
        // Echo the command
        self.blocks.push(Block::Command(cmd.to_string()));

        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let (verb, args) = match parts.split_first() {
            Some((v, a)) => (v.to_lowercase(), a.to_vec()),
            None => return Action::None,
        };

        match verb.as_str() {
            "q" | "quit" | "exit" => return Action::Quit,

            "home" | "menu" => self.show_home().await,

            "1" | "failures" | "failed" => self.show_failures(&args.join(" ")).await,

            "2" | "ready" | "readiness" => self.check_readiness(&args).await,

            "3" | "connect" => self.show_connection(&args),

            "5" | "ops" | "operations" => self.blocks.push(Block::Operations),

            "help" | "h" | "?" => {
                self.blocks.push(Block::Help);
            }

            "clear" | "cls" => {
                self.blocks.clear();
                self.blocks
                    .push(Block::Info("Transcript cleared.".to_string()));
            }

            "status" | "st" => {
                if let Ok(overview) = self.client.fetch_overview().await {
                    self.overview = overview;
                }
                self.blocks.push(Block::Status {
                    overview: self.overview.clone(),
                });
            }

            "repos" | "repo" | "r" => {
                if let Ok(repos) = self.client.fetch_repositories().await {
                    self.repositories = repos;
                }
                self.blocks.push(Block::RepoList {
                    repos: self.repositories.clone(),
                    cursor: 0,
                });
            }

            "inspect" | "i" => {
                self.handle_inspect(&args);
            }

            "4" | "verify" | "search" | "s" | "find" => {
                let keyword = args.join(" ");
                if keyword.is_empty() {
                    self.blocks.push(Block::Info(
                        "Enter a package name or coordinate: search <package>.".to_string(),
                    ));
                } else {
                    match self.client.search_components(&keyword).await {
                        Ok(results) => {
                            self.search_results = results;
                            self.blocks.push(Block::SearchResults {
                                keyword,
                                results: self.search_results.clone(),
                                cursor: 0,
                            });
                        }
                        Err(error) => self
                            .blocks
                            .push(Block::Error(format!("Artifact search failed: {error}"))),
                    }
                }
            }

            "components" | "comps" | "comp" => {
                let repo_filter = args.first().map(|s| s.to_string());
                if let Ok(results) = self.client.search_components("").await {
                    self.search_results = results;
                }
                let filtered = if let Some(ref repo) = repo_filter {
                    let items: Vec<_> = self
                        .search_results
                        .items
                        .iter()
                        .filter(|c| c.repository.eq_ignore_ascii_case(repo))
                        .cloned()
                        .collect();
                    let total = items.len();
                    SearchResponse {
                        items,
                        total,
                        ..self.search_results.clone()
                    }
                } else {
                    self.search_results.clone()
                };
                let keyword = repo_filter.unwrap_or_default();
                self.blocks.push(Block::SearchResults {
                    keyword,
                    results: filtered,
                    cursor: 0,
                });
            }

            "gc" => {
                self.blocks.push(Block::Info(
                    "Running BlobStore Garbage Collection...".to_string(),
                ));
                match self.client.run_gc().await {
                    Ok(rep) => {
                        self.blocks.push(Block::Success(format!(
                            "GC completed: purged {} orphaned blobs, freed {:.2} KB.",
                            rep.orphaned_blobs_deleted,
                            rep.freed_bytes as f64 / 1024.0
                        )));
                        self.add_log(&format!(
                            "SUCCESS: GC purged {} orphaned blobs, freed {:.2} KB.",
                            rep.orphaned_blobs_deleted,
                            rep.freed_bytes as f64 / 1024.0
                        ));
                    }
                    Err(e) => {
                        self.blocks.push(Block::Error(format!("GC failed: {}", e)));
                        self.add_log(&format!("ERROR: GC failed: {}", e));
                    }
                }
            }

            "cleanup" => {
                let repo_name = args.first().map(|s| s.to_string()).unwrap_or_else(|| {
                    self.get_active_repo_name()
                        .unwrap_or_else(|| "maven-releases".to_string())
                });
                self.blocks.push(Block::Info(format!(
                    "Running retention cleanup on {}...",
                    repo_name
                )));
                match self.client.run_cleanup(&repo_name, 5).await {
                    Ok(rep) => {
                        self.blocks.push(Block::Success(format!(
                            "Cleanup completed: deleted {} old components ({} assets), freed {:.2} KB.",
                            rep.deleted_components_count,
                            rep.deleted_assets_count,
                            rep.freed_bytes as f64 / 1024.0
                        )));
                        self.add_log(&format!(
                            "SUCCESS: Cleanup deleted {} old components ({} assets), freed {:.2} KB.",
                            rep.deleted_components_count,
                            rep.deleted_assets_count,
                            rep.freed_bytes as f64 / 1024.0
                        ));
                    }
                    Err(e) => {
                        self.blocks
                            .push(Block::Error(format!("Cleanup failed: {}", e)));
                        self.add_log(&format!("ERROR: Cleanup failed: {}", e));
                    }
                }
            }

            "token" => {
                self.blocks.push(Block::Info(
                    "Generating temporary 7-day PAT token...".to_string(),
                ));
                match self.client.create_temp_token("tui-temp-cli-token").await {
                    Ok(token) => {
                        self.blocks
                            .push(Block::Success(format!("Token: {}", token)));
                        self.add_log("SUCCESS: Temporary token generated.");
                    }
                    Err(e) => {
                        self.blocks
                            .push(Block::Error(format!("Token generation failed: {}", e)));
                        self.add_log(&format!("ERROR: Token generation failed: {}", e));
                    }
                }
            }

            _ => {
                self.blocks.push(Block::Error(format!(
                    "Unknown command: '{}'. Type 'help' for available commands.",
                    verb
                )));
            }
        }

        Action::None
    }

    async fn show_failures(&mut self, query: &str) {
        match self.client.fetch_service_logs(100).await {
            Ok(logs) => {
                let needle = query.to_lowercase();
                let logs = logs
                    .into_iter()
                    .filter(is_failure)
                    .filter(|log| {
                        needle.is_empty()
                            || format!(
                                "{} {} {} {}",
                                log.artifact_path,
                                log.repository_name,
                                log.error_message,
                                log.action
                            )
                            .to_lowercase()
                            .contains(&needle)
                    })
                    .collect();
                self.blocks.push(Block::FailureList {
                    query: query.to_string(),
                    logs,
                    cursor: 0,
                });
            }
            Err(error) => self.blocks.push(Block::Error(format!(
                "Could not load recent requests: {error}"
            ))),
        }
    }

    fn show_connection(&mut self, args: &[&str]) {
        let filter = args.first().copied().unwrap_or("");
        if let Some(repo) = self.repositories.iter().find(|repo| repo.name == filter) {
            self.blocks.push(Block::RepoDetail {
                repo: repo.clone(),
                endpoint: self.endpoint.clone(),
            });
            return;
        }
        let repos: Vec<_> = self
            .repositories
            .iter()
            .filter(|repo| filter.is_empty() || repo.format.eq_ignore_ascii_case(filter))
            .cloned()
            .collect();
        if repos.is_empty() {
            self.blocks.push(Block::Error(format!(
                "No repository or format matches '{filter}'. Type 'repos' to browse all."
            )));
            return;
        }
        self.blocks.push(Block::Info(
            "Choose a repository with ↑↓ and Enter for its native-client setup. Use 'token' if the client needs a new credential."
                .to_string(),
        ));
        self.blocks.push(Block::RepoList { repos, cursor: 0 });
    }

    async fn check_readiness(&mut self, args: &[&str]) {
        let Some(format) = args
            .first()
            .and_then(|value| match value.to_lowercase().as_str() {
                "maven" | "maven2" => Some("maven2"),
                "npm" => Some("npm"),
                "cargo" => Some("cargo"),
                _ => None,
            })
        else {
            self.blocks.push(Block::ReadinessGuide);
            return;
        };

        let mut coordinates = Vec::new();
        let mut repository = None;
        let mut index = 1;
        while index < args.len() {
            if args[index] == "--repo" {
                if repository.is_some() || index + 1 >= args.len() {
                    self.blocks.push(Block::Error(
                        "Use --repo followed by one repository name.".to_string(),
                    ));
                    return;
                }
                repository = Some(args[index + 1].to_string());
                index += 2;
            } else {
                coordinates.push(args[index]);
                index += 1;
            }
        }
        let mut seen = std::collections::HashSet::new();
        coordinates.retain(|coordinate| seen.insert(*coordinate));
        if coordinates.is_empty() || coordinates.len() > 20 {
            self.blocks.push(Block::Error(
                "Enter 1–20 exact coordinates after the format.".to_string(),
            ));
            return;
        }
        if let Some(ref name) = repository {
            let Some(repo) = self.repositories.iter().find(|repo| repo.name == *name) else {
                self.blocks.push(Block::Error(format!(
                    "Repository '{name}' is not available."
                )));
                return;
            };
            if !repo.format.eq_ignore_ascii_case(format)
                || repo.repo_type.eq_ignore_ascii_case("group")
            {
                self.blocks.push(Block::Error(format!(
                    "Repository '{name}' is not a {format} hosted or proxy repository."
                )));
                return;
            }
        }

        let mut results = Vec::new();
        for coordinate in coordinates {
            let Some((group, name, version)) = parse_coordinate(format, coordinate) else {
                self.blocks.push(Block::Error(format!(
                    "Invalid {format} coordinate '{coordinate}'. Type 'ready' for examples."
                )));
                return;
            };
            let query = ComponentSearch {
                name,
                group,
                version,
                format: format.to_string(),
                repository: repository.clone(),
                page: 1,
                page_size: 20,
            };
            match self.find_exact_component(query).await {
                Ok(found) => results.push(ReadinessResult {
                    coordinate: coordinate.to_string(),
                    repository: found,
                }),
                Err(error) => {
                    self.blocks.push(Block::Error(format!(
                        "Inventory check stopped at '{coordinate}': {error}"
                    )));
                    return;
                }
            }
        }
        self.blocks.push(Block::ReadinessResults { results });
    }

    async fn find_exact_component(
        &self,
        mut query: ComponentSearch,
    ) -> anyhow::Result<Option<String>> {
        loop {
            let response = self.client.search_exact_page(&query).await?;
            if let Some(item) = response.items.iter().find(|item| {
                item.name == query.name
                    && item.version == query.version
                    && query
                        .group
                        .as_ref()
                        .is_none_or(|group| item.group == *group)
                    && item.format.eq_ignore_ascii_case(&query.format)
                    && query
                        .repository
                        .as_ref()
                        .is_none_or(|repo| item.repository == *repo)
            }) {
                return Ok(Some(item.repository.clone()));
            }
            if response.items.is_empty()
                || query.page.saturating_mul(query.page_size) >= response.total
            {
                return Ok(None);
            }
            query.page += 1;
        }
    }

    async fn activate_selection(&mut self) {
        match self
            .blocks
            .iter()
            .rev()
            .find(|block| block.is_interactive())
            .cloned()
        {
            Some(Block::Home { cursor, .. }) => match cursor {
                0 => self.show_failures("").await,
                1 => {
                    self.blocks.push(Block::ReadinessGuide);
                    self.input = "ready ".to_string();
                    self.input_cursor = self.input.len();
                }
                2 => self.show_connection(&[]),
                3 => {
                    self.blocks.push(Block::Info(
                        "Enter a package name to inspect published versions and files.".to_string(),
                    ));
                    self.input = "search ".to_string();
                    self.input_cursor = self.input.len();
                }
                _ => self.blocks.push(Block::Operations),
            },
            Some(Block::FailureList { logs, cursor, .. }) => {
                if let Some(log) = logs.get(cursor) {
                    self.blocks.push(Block::FailureDetail { log: log.clone() });
                }
            }
            _ => self.inspect_active_selection(),
        }
    }

    // ── Inspect handler ──────────────────────────────────────────

    fn handle_inspect(&mut self, args: &[&str]) {
        if let Some(name) = args.first() {
            // Try to find a repo by name
            if let Some(repo) = self.repositories.iter().find(|r| r.name == *name) {
                self.blocks.push(Block::RepoDetail {
                    repo: repo.clone(),
                    endpoint: self.endpoint.clone(),
                });
                return;
            }
            // Try to find a component by name
            if let Some(comp) = self.search_results.items.iter().find(|c| c.name == *name) {
                self.blocks.push(Block::ComponentDetail {
                    component: comp.clone(),
                });
                return;
            }
            self.blocks.push(Block::Error(format!(
                "'{}' not found in repositories or search results.",
                name
            )));
        } else {
            // Inspect currently selected item in latest interactive block
            if let Some(block) = self.blocks.iter().rev().find(|b| b.is_interactive()) {
                match block.clone() {
                    Block::RepoList { repos, cursor } => {
                        if let Some(repo) = repos.get(cursor) {
                            self.blocks.push(Block::RepoDetail {
                                repo: repo.clone(),
                                endpoint: self.endpoint.clone(),
                            });
                        }
                    }
                    Block::SearchResults {
                        results, cursor, ..
                    } => {
                        if let Some(comp) = results.items.get(cursor) {
                            self.blocks.push(Block::ComponentDetail {
                                component: comp.clone(),
                            });
                        }
                    }
                    _ => {}
                }
            } else {
                self.blocks.push(Block::Error(
                    "Nothing to inspect. Use 'repos' or 'search' first, or specify a name."
                        .to_string(),
                ));
            }
        }
    }

    /// Called when Enter is pressed with empty input — expand the selected item.
    fn inspect_active_selection(&mut self) {
        if let Some(block) = self.blocks.iter().rev().find(|b| b.is_interactive()) {
            match block.clone() {
                Block::RepoList { repos, cursor } => {
                    if let Some(repo) = repos.get(cursor) {
                        // Show repo detail + its components
                        self.blocks.push(Block::RepoDetail {
                            repo: repo.clone(),
                            endpoint: self.endpoint.clone(),
                        });
                    }
                }
                Block::SearchResults {
                    results, cursor, ..
                } => {
                    if let Some(comp) = results.items.get(cursor) {
                        self.blocks.push(Block::ComponentDetail {
                            component: comp.clone(),
                        });
                    }
                }
                _ => {}
            }
        }
    }

    // ── Active cursor management ─────────────────────────────────

    fn find_last_interactive_mut(&mut self) -> Option<&mut Block> {
        self.blocks.iter_mut().rev().find(|b| b.is_interactive())
    }

    fn move_active_cursor_up(&mut self) -> bool {
        if let Some(block) = self.find_last_interactive_mut() {
            match block {
                Block::Home { cursor, .. }
                | Block::FailureList { cursor, .. }
                | Block::RepoList { cursor, .. }
                | Block::SearchResults { cursor, .. }
                    if *cursor > 0 =>
                {
                    *cursor -= 1;
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    fn move_active_cursor_down(&mut self) -> bool {
        if let Some(block) = self.find_last_interactive_mut() {
            match block {
                Block::Home { cursor, .. } if *cursor < 4 => {
                    *cursor += 1;
                    return true;
                }
                Block::FailureList { logs, cursor, .. } if *cursor + 1 < logs.len() => {
                    *cursor += 1;
                    return true;
                }
                Block::RepoList { repos, cursor } if *cursor + 1 < repos.len() => {
                    *cursor += 1;
                    return true;
                }
                Block::SearchResults {
                    results, cursor, ..
                } if *cursor + 1 < results.items.len() => {
                    *cursor += 1;
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    fn get_active_repo_name(&self) -> Option<String> {
        self.blocks.iter().rev().find_map(|b| {
            if let Block::RepoList { repos, cursor } = b {
                repos.get(*cursor).map(|r| r.name.clone())
            } else {
                None
            }
        })
    }

    // ── Command history ──────────────────────────────────────────

    fn recall_history_prev(&mut self) {
        if self.cmd_history.is_empty() {
            return;
        }
        let idx = match self.cmd_history_idx {
            None => self.cmd_history.len() - 1,
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
        };
        self.cmd_history_idx = Some(idx);
        self.input = self.cmd_history[idx].clone();
        self.input_cursor = self.input.len();
    }

    fn recall_history_next(&mut self) {
        if let Some(idx) = self.cmd_history_idx {
            if idx + 1 < self.cmd_history.len() {
                let next = idx + 1;
                self.cmd_history_idx = Some(next);
                self.input = self.cmd_history[next].clone();
                self.input_cursor = self.input.len();
            } else {
                self.cmd_history_idx = None;
                self.input.clear();
                self.input_cursor = 0;
            }
        }
    }

    // ── Helpers ──────────────────────────────────────────────────

    pub fn add_log(&mut self, msg: &str) {
        let entry = format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg);
        self.op_log.push(entry);
        if self.op_log.len() > 100 {
            self.op_log.remove(0);
        }
    }
}

fn is_failure(log: &ServiceLogItem) -> bool {
    if !log.error_message.is_empty() {
        return true;
    }
    if log.status.eq_ignore_ascii_case("success") || log.status.eq_ignore_ascii_case("ok") {
        return false;
    }
    match log.status.parse::<u16>() {
        Ok(code) => !(200..300).contains(&code),
        Err(_) => !log.status.is_empty(),
    }
}

fn parse_coordinate(format: &str, coordinate: &str) -> Option<(Option<String>, String, String)> {
    if format == "maven2" {
        let mut parts = coordinate.split(':');
        let (Some(group), Some(name), Some(version), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return None;
        };
        if group.is_empty() || name.is_empty() || version.is_empty() {
            return None;
        }
        return Some((
            Some(group.to_string()),
            name.to_string(),
            version.to_string(),
        ));
    }
    let (name, version) = coordinate.rsplit_once('@')?;
    if name.is_empty() || version.is_empty() {
        return None;
    }
    Some((None, name.to_string(), version.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use async_trait::async_trait;

    struct MockApi {
        pub repos: Vec<RepositoryItem>,
        pub search: SearchResponse,
        pub logs: Vec<ServiceLogItem>,
        pub gc_result: Result<GcReport, String>,
    }

    impl MockApi {
        fn default_ok() -> Self {
            Self {
                repos: vec![
                    RepositoryItem {
                        name: "maven-releases".to_string(),
                        format: "maven2".to_string(),
                        repo_type: "hosted".to_string(),
                        url: "/repository/maven-releases".to_string(),
                        online: true,
                    },
                    RepositoryItem {
                        name: "npm-hosted".to_string(),
                        format: "npm".to_string(),
                        repo_type: "hosted".to_string(),
                        url: "/repository/npm-hosted".to_string(),
                        online: true,
                    },
                ],
                search: SearchResponse {
                    items: vec![ComponentItem {
                        id: 1,
                        name: "my-lib".to_string(),
                        group: "com.example".to_string(),
                        version: "1.0.0".to_string(),
                        format: "maven2".to_string(),
                        repository: "maven-releases".to_string(),
                        assets: vec![],
                    }],
                    total: 1,
                    page: 1,
                    page_size: 50,
                },
                logs: vec![
                    ServiceLogItem {
                        status: "404".to_string(),
                        repository_name: "maven-releases".to_string(),
                        artifact_path: "com/example/my-lib/1.0.0".to_string(),
                        error_message: "Not found upstream".to_string(),
                        ..Default::default()
                    },
                    ServiceLogItem {
                        status: "200".to_string(),
                        repository_name: "maven-releases".to_string(),
                        artifact_path: "com/example/other".to_string(),
                        ..Default::default()
                    },
                ],
                gc_result: Ok(GcReport {
                    scanned_blobs_count: 10,
                    orphaned_blobs_deleted: 2,
                    freed_bytes: 4096,
                }),
            }
        }
    }

    #[async_trait]
    impl RegistryApi for MockApi {
        async fn fetch_overview(&self) -> anyhow::Result<ServerOverview> {
            Ok(ServerOverview {
                status: "HEALTHY / ONLINE".to_string(),
                is_online: true,
                total_repositories: self.repos.len(),
                hosted_count: self.repos.len(),
                proxy_count: 0,
                group_count: 0,
                format_counts: vec![],
                total_components: self.search.total,
                metrics_raw: String::new(),
            })
        }

        async fn fetch_repositories(&self) -> anyhow::Result<Vec<RepositoryItem>> {
            Ok(self.repos.clone())
        }

        async fn search_components(&self, _keyword: &str) -> anyhow::Result<SearchResponse> {
            Ok(self.search.clone())
        }

        async fn search_exact_page(
            &self,
            _query: &ComponentSearch,
        ) -> anyhow::Result<SearchResponse> {
            Ok(self.search.clone())
        }

        async fn fetch_service_logs(&self, _size: usize) -> anyhow::Result<Vec<ServiceLogItem>> {
            Ok(self.logs.clone())
        }

        async fn fetch_metrics(&self) -> anyhow::Result<String> {
            Ok("# mock metrics".to_string())
        }

        async fn run_gc(&self) -> anyhow::Result<GcReport> {
            self.gc_result.clone().map_err(|e| anyhow::anyhow!(e))
        }

        async fn run_cleanup(&self, _repo: &str, _max: usize) -> anyhow::Result<CleanupReport> {
            Ok(CleanupReport {
                deleted_components_count: 3,
                deleted_assets_count: 5,
                freed_bytes: 2048,
            })
        }

        async fn create_temp_token(&self, _desc: &str) -> anyhow::Result<String> {
            Ok("tql_pat_mock_token_abc123".to_string())
        }

        async fn verify_connection(&self) -> anyhow::Result<bool> {
            Ok(true)
        }
    }

    fn make_app(mock: MockApi) -> App {
        App::new(Arc::new(mock), "https://registry.example.test".to_string())
    }

    #[tokio::test]
    async fn test_initialize_creates_blocks() {
        let mut app = make_app(MockApi::default_ok());
        app.initialize().await;
        assert_eq!(app.blocks.len(), 1);
        assert!(
            matches!(&app.blocks[0], Block::Home { recent_failures, repository_count: 2, .. } if recent_failures.len() == 1)
        );
    }

    #[tokio::test]
    async fn test_quit_commands() {
        let mut app = make_app(MockApi::default_ok());
        assert_eq!(app.handle_key(KeyCode::Esc).await, Action::Quit);

        let mut app2 = make_app(MockApi::default_ok());
        app2.input = "quit".to_string();
        app2.input_cursor = 4;
        assert_eq!(app2.handle_key(KeyCode::Enter).await, Action::Quit);
    }

    #[tokio::test]
    async fn test_command_repos() {
        let mut app = make_app(MockApi::default_ok());
        app.input = "repos".to_string();
        app.input_cursor = 5;
        app.handle_key(KeyCode::Enter).await;

        // Should have Command echo + RepoList
        let last_two: Vec<_> = app.blocks.iter().rev().take(2).collect();
        assert!(matches!(last_two[0], Block::RepoList { .. }));
        assert!(matches!(last_two[1], Block::Command(_)));
    }

    #[tokio::test]
    async fn test_command_search() {
        let mut app = make_app(MockApi::default_ok());
        app.input = "search teaql".to_string();
        app.input_cursor = 12;
        app.handle_key(KeyCode::Enter).await;

        let last = app.blocks.last().unwrap();
        assert!(matches!(last, Block::SearchResults { .. }));
    }

    #[tokio::test]
    async fn test_gc_success() {
        let mut app = make_app(MockApi::default_ok());
        app.input = "gc".to_string();
        app.input_cursor = 2;
        app.handle_key(KeyCode::Enter).await;

        let success_block = app.blocks.iter().find(|b| matches!(b, Block::Success(_)));
        assert!(success_block.is_some());
    }

    #[tokio::test]
    async fn test_gc_failure() {
        let mock = MockApi {
            gc_result: Err("connection refused".to_string()),
            ..MockApi::default_ok()
        };
        let mut app = make_app(mock);
        app.input = "gc".to_string();
        app.input_cursor = 2;
        app.handle_key(KeyCode::Enter).await;

        let err_block = app.blocks.iter().find(|b| matches!(b, Block::Error(_)));
        assert!(err_block.is_some());
    }

    #[tokio::test]
    async fn test_active_cursor_navigation() {
        let mut app = make_app(MockApi::default_ok());
        app.initialize().await;

        // The first task is selected on the home screen.
        app.move_active_cursor_down();
        if let Some(Block::Home { cursor, .. }) =
            app.blocks.iter().rev().find(|b| b.is_interactive())
        {
            assert_eq!(*cursor, 1);
        } else {
            panic!("Expected Home block");
        }

        app.move_active_cursor_up();
        if let Some(Block::Home { cursor, .. }) =
            app.blocks.iter().rev().find(|b| b.is_interactive())
        {
            assert_eq!(*cursor, 0);
        }
    }

    #[tokio::test]
    async fn test_command_history() {
        let mut app = make_app(MockApi::default_ok());
        app.input = "repos".to_string();
        app.input_cursor = 5;
        app.handle_key(KeyCode::Enter).await;

        app.input = "status".to_string();
        app.input_cursor = 6;
        app.handle_key(KeyCode::Enter).await;

        assert_eq!(app.cmd_history, vec!["repos", "status"]);

        // Recall last command
        app.recall_history_prev();
        assert_eq!(app.input, "status");
        app.recall_history_prev();
        assert_eq!(app.input, "repos");
    }

    #[tokio::test]
    async fn test_unknown_command() {
        let mut app = make_app(MockApi::default_ok());
        app.input = "foobar".to_string();
        app.input_cursor = 6;
        app.handle_key(KeyCode::Enter).await;

        let last = app.blocks.last().unwrap();
        assert!(matches!(last, Block::Error(_)));
    }

    #[tokio::test]
    async fn test_input_editing() {
        let mut app = make_app(MockApi::default_ok());

        app.handle_key(KeyCode::Char('h')).await;
        app.handle_key(KeyCode::Char('i')).await;
        assert_eq!(app.input, "hi");
        assert_eq!(app.input_cursor, 2);

        app.handle_key(KeyCode::Backspace).await;
        assert_eq!(app.input, "h");
        assert_eq!(app.input_cursor, 1);

        app.handle_key(KeyCode::Left).await;
        assert_eq!(app.input_cursor, 0);

        app.handle_key(KeyCode::Right).await;
        assert_eq!(app.input_cursor, 1);
    }

    #[tokio::test]
    async fn test_create_token() {
        let mut app = make_app(MockApi::default_ok());
        app.input = "token".to_string();
        app.input_cursor = 5;
        app.handle_key(KeyCode::Enter).await;

        let success_block = app
            .blocks
            .iter()
            .find(|b| matches!(b, Block::Success(s) if s.contains("tql_pat_mock_token_abc123")));
        assert!(success_block.is_some());
        assert!(!app
            .op_log
            .iter()
            .any(|entry| entry.contains("tql_pat_mock_token")));
    }

    #[tokio::test]
    async fn test_home_opens_failure_details() {
        let mut app = make_app(MockApi::default_ok());
        app.initialize().await;
        app.handle_key(KeyCode::Enter).await;
        assert!(
            matches!(&app.blocks.last().unwrap(), Block::FailureList { logs, .. } if logs.len() == 1)
        );
        app.handle_key(KeyCode::Enter).await;
        assert!(
            matches!(&app.blocks.last().unwrap(), Block::FailureDetail { log } if log.status == "404")
        );
    }

    #[tokio::test]
    async fn test_readiness_checks_exact_coordinates() {
        let mut app = make_app(MockApi::default_ok());
        app.initialize().await;
        app.execute_command(
            "ready maven2 com.example:my-lib:1.0.0 com.example:missing:1.0.0 --repo maven-releases",
        )
        .await;
        assert!(
            matches!(&app.blocks.last().unwrap(), Block::ReadinessResults { results }
            if results.len() == 2 && results[0].repository.as_deref() == Some("maven-releases") && results[1].repository.is_none())
        );
        app.execute_command("ready npm invalid-coordinate").await;
        assert!(matches!(app.blocks.last().unwrap(), Block::Error(_)));
        app.execute_command("ready npm my-lib@1.0.0 --repo maven-releases")
            .await;
        assert!(
            matches!(app.blocks.last().unwrap(), Block::Error(message) if message.contains("not a npm"))
        );
    }

    #[tokio::test]
    async fn test_connect_uses_configured_endpoint() {
        let mut app = make_app(MockApi::default_ok());
        app.initialize().await;
        app.execute_command("connect maven-releases").await;
        assert!(
            matches!(&app.blocks.last().unwrap(), Block::RepoDetail { endpoint, .. }
            if endpoint == "https://registry.example.test")
        );
    }

    #[tokio::test]
    async fn test_unicode_input_keeps_byte_boundaries() {
        let mut app = make_app(MockApi::default_ok());
        app.handle_key(KeyCode::Char('包')).await;
        app.handle_key(KeyCode::Char('a')).await;
        app.handle_key(KeyCode::Left).await;
        app.handle_key(KeyCode::Backspace).await;
        assert_eq!(app.input, "a");
        assert_eq!(app.input_cursor, 0);
    }
}
