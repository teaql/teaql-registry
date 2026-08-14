use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::handler::RepositoryHandler;

#[derive(Clone, Default)]
pub struct RepositoryRegistry {
    handlers: Arc<RwLock<HashMap<String, Arc<dyn RepositoryHandler>>>>,
}

impl RepositoryRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&self, handler: Arc<dyn RepositoryHandler>) {
        let mut map = self.handlers.write().unwrap();
        map.insert(handler.format_name().to_string(), handler);
    }

    pub fn get(&self, format: &str) -> Option<Arc<dyn RepositoryHandler>> {
        let map = self.handlers.read().unwrap();
        map.get(format).cloned()
    }
}
