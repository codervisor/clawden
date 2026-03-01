use std::collections::HashMap;
use std::sync::Arc;

use clawden_core::{ClawAdapter, ClawRuntime, RuntimeMetadata};

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

    /// Dynamically register an adapter at runtime (e.g. from a plugin directory).
    /// Returns `true` if this replaced an existing adapter for the same runtime.
    pub fn register_dynamic(
        &mut self,
        runtime: ClawRuntime,
        adapter: Arc<dyn ClawAdapter>,
    ) -> bool {
        self.adapters.insert(runtime, adapter).is_some()
    }

    pub fn unregister(&mut self, runtime: &ClawRuntime) -> bool {
        self.adapters.remove(runtime).is_some()
    }

    pub fn get(&self, runtime: &ClawRuntime) -> Option<Arc<dyn ClawAdapter>> {
        self.adapters.get(runtime).cloned()
    }

    pub fn adapters_map(&self) -> HashMap<ClawRuntime, Arc<dyn ClawAdapter>> {
        self.adapters.clone()
    }

    pub fn list(&self) -> Vec<ClawRuntime> {
        let mut runtimes: Vec<_> = self.adapters.keys().cloned().collect();
        runtimes.sort_by_key(|runtime| format!("{runtime:?}"));
        runtimes
    }

    pub fn list_metadata(&self) -> Vec<RuntimeMetadata> {
        let mut entries: Vec<_> = self
            .adapters
            .values()
            .map(|adapter| adapter.metadata())
            .collect();
        entries.sort_by(|a, b| format!("{:?}", a.runtime).cmp(&format!("{:?}", b.runtime)));
        entries
    }

    pub fn has(&self, runtime: &ClawRuntime) -> bool {
        self.adapters.contains_key(runtime)
    }

    pub fn detect_runtime_for_capability(&self, capability: &str) -> Option<ClawRuntime> {
        self.adapters.iter().find_map(|(runtime, adapter)| {
            let supports = adapter
                .metadata()
                .capabilities
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(capability));
            if supports {
                Some(runtime.clone())
            } else {
                None
            }
        })
    }

    /// Auto-detect available runtimes by checking which adapters
    /// report themselves as available on this system.
    pub fn detect_available(&self) -> Vec<RuntimeMetadata> {
        self.adapters
            .values()
            .map(|adapter| adapter.metadata())
            .collect()
    }
}
