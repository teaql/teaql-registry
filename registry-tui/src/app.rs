use std::sync::Arc;

use crossterm::event::KeyCode;

use crate::api::RegistryApi;
use crate::types::{ComponentItem, RepositoryItem, SearchResponse, ServerOverview};

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
    /// Status dashboard
    Status { overview: ServerOverview },
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
    ComponentDetail { component: ComponentItem },
    /// Help text
    Help,
}

impl Block {
    /// Returns true if this block has an interactive cursor.
    pub fn is_interactive(&self) -> bool {
        matches!(self, Block::RepoList { .. } | Block::SearchResults { .. })
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
    pub fn new(client: Arc<dyn RegistryApi>) -> Self {
        Self {
            client,
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
        // Load data
        if let Ok(overview) = self.client.fetch_overview().await {
            self.overview = overview;
        }
        if let Ok(repos) = self.client.fetch_repositories().await {
            self.repositories = repos;
        }

        // Push startup blocks
        self.blocks.push(Block::Info("Welcome to TeaQL Registry TUI. Type a command below, or 'help' for available commands.".to_string()));
        self.blocks.push(Block::Status {
            overview: self.overview.clone(),
        });
        self.blocks.push(Block::RepoList {
            repos: self.repositories.clone(),
            cursor: 0,
        });
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
                self.inspect_active_selection();
                self.scroll_offset = 0;
                Action::None
            }

            // Input editing
            KeyCode::Char(c) => {
                self.input.insert(self.input_cursor, c);
                self.input_cursor += 1;
                self.cmd_history_idx = None;
                Action::None
            }
            KeyCode::Backspace => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                    self.input.remove(self.input_cursor);
                }
                Action::None
            }
            KeyCode::Left => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
                Action::None
            }
            KeyCode::Right => {
                if self.input_cursor < self.input.len() {
                    self.input_cursor += 1;
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

            "search" | "s" | "find" => {
                let keyword = args.join(" ");
                if let Ok(results) = self.client.search_components(&keyword).await {
                    self.search_results = results;
                }
                self.blocks.push(Block::SearchResults {
                    keyword: args.join(" "),
                    results: self.search_results.clone(),
                    cursor: 0,
                });
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
                        self.add_log(&format!("TOKEN: {}", token));
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

    // ── Inspect handler ──────────────────────────────────────────

    fn handle_inspect(&mut self, args: &[&str]) {
        if let Some(name) = args.first() {
            // Try to find a repo by name
            if let Some(repo) = self.repositories.iter().find(|r| r.name == *name) {
                self.blocks.push(Block::RepoDetail {
                    repo: repo.clone(),
                    endpoint: "http://localhost:8081".to_string(),
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
                                endpoint: "http://localhost:8081".to_string(),
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
                            endpoint: "http://localhost:8081".to_string(),
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
                Block::RepoList { cursor, .. } | Block::SearchResults { cursor, .. }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use async_trait::async_trait;

    struct MockApi {
        pub repos: Vec<RepositoryItem>,
        pub search: SearchResponse,
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
        App::new(Arc::new(mock))
    }

    #[tokio::test]
    async fn test_initialize_creates_blocks() {
        let mut app = make_app(MockApi::default_ok());
        app.initialize().await;
        // Should have 3 blocks: Info, Status, RepoList
        assert_eq!(app.blocks.len(), 3);
        assert!(matches!(app.blocks[0], Block::Info(_)));
        assert!(matches!(app.blocks[1], Block::Status { .. }));
        assert!(matches!(app.blocks[2], Block::RepoList { .. }));
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

        // Last interactive block is RepoList with cursor=0
        app.move_active_cursor_down();
        if let Some(Block::RepoList { cursor, .. }) =
            app.blocks.iter().rev().find(|b| b.is_interactive())
        {
            assert_eq!(*cursor, 1);
        } else {
            panic!("Expected RepoList block");
        }

        app.move_active_cursor_up();
        if let Some(Block::RepoList { cursor, .. }) =
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
    }
}
