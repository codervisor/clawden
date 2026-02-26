use std::collections::HashMap;
use std::sync::Arc;

use clawlab_core::{ClawAdapter, ClawRuntime};

#[derive(Default)]
pub struct AdapterRegistry {
    adapters: HashMap<ClawRuntime, Arc<dyn ClawAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, runtime: ClawRuntime, adapter: Arc<dyn ClawAdapter>) {
        self.adapters.insert(runtime, adapter);
    }

    pub fn get(&self, runtime: &ClawRuntime) -> Option<Arc<dyn ClawAdapter>> {
        self.adapters.get(runtime).cloned()
    }

    pub fn list(&self) -> Vec<ClawRuntime> {
        self.adapters.keys().cloned().collect()
    }
}
