use crate::docker_runtime::{
    container_running, restart_container, runtime_config_values, start_container, stop_container,
};
use anyhow::Result;
use async_trait::async_trait;
use clawden_core::{
    AgentConfig, AgentHandle, AgentMessage, AgentMetrics, AgentResponse, ClawAdapter, ClawRuntime,
    EventStream, HealthStatus, InstallConfig, RuntimeConfig, RuntimeMetadata, Skill, SkillManifest,
};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

pub trait RuntimeMeta: Send + Sync + 'static {
    const RUNTIME: ClawRuntime;
    const NAME: &'static str;

    fn metadata() -> RuntimeMetadata;
}

pub trait ConfigStore: Send + Sync + 'static {
    fn set(&self, handle_id: &str, config: RuntimeConfig);
    fn get(&self, handle_id: &str) -> Option<RuntimeConfig>;
    fn remove(&self, handle_id: &str);
}

#[derive(Default)]
pub struct InMemoryConfigStore {
    values: Mutex<HashMap<String, RuntimeConfig>>,
}

impl ConfigStore for InMemoryConfigStore {
    fn set(&self, handle_id: &str, config: RuntimeConfig) {
        if let Ok(mut guard) = self.values.lock() {
            guard.insert(handle_id.to_string(), config);
        }
    }

    fn get(&self, handle_id: &str) -> Option<RuntimeConfig> {
        self.values
            .lock()
            .ok()
            .and_then(|guard| guard.get(handle_id).cloned())
    }

    fn remove(&self, handle_id: &str) {
        if let Ok(mut guard) = self.values.lock() {
            guard.remove(handle_id);
        }
    }
}

pub struct DockerAdapter<R: RuntimeMeta> {
    store: Arc<dyn ConfigStore>,
    _marker: PhantomData<R>,
}

impl<R: RuntimeMeta> DockerAdapter<R> {
    pub fn with_store(store: Arc<dyn ConfigStore>) -> Self {
        Self {
            store,
            _marker: PhantomData,
        }
    }
}

impl<R: RuntimeMeta> Default for DockerAdapter<R> {
    fn default() -> Self {
        Self::with_store(Arc::new(InMemoryConfigStore::default()))
    }
}

#[async_trait]
impl<R: RuntimeMeta> ClawAdapter for DockerAdapter<R> {
    fn metadata(&self) -> RuntimeMetadata {
        R::metadata()
    }

    async fn install(&self, _config: &InstallConfig) -> Result<()> {
        Ok(())
    }

    async fn start(&self, config: &AgentConfig) -> Result<AgentHandle> {
        let container_id = start_container(R::RUNTIME.clone(), config)?;
        let handle = AgentHandle {
            id: container_id,
            name: config.name.clone(),
            runtime: R::RUNTIME.clone(),
        };

        self.store.set(
            &handle.id,
            runtime_config_values(R::RUNTIME.as_slug(), config),
        );
        Ok(handle)
    }

    async fn stop(&self, handle: &AgentHandle) -> Result<()> {
        stop_container(&handle.id)?;
        self.store.remove(&handle.id);
        Ok(())
    }

    async fn restart(&self, handle: &AgentHandle) -> Result<()> {
        restart_container(&handle.id)?;
        Ok(())
    }

    async fn health(&self, handle: &AgentHandle) -> Result<HealthStatus> {
        if container_running(&handle.id)? {
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Unhealthy)
        }
    }

    async fn metrics(&self, _handle: &AgentHandle) -> Result<AgentMetrics> {
        Ok(AgentMetrics {
            cpu_percent: 0.0,
            memory_mb: 0.0,
            queue_depth: 0,
        })
    }

    async fn send(&self, _handle: &AgentHandle, message: &AgentMessage) -> Result<AgentResponse> {
        Ok(AgentResponse {
            content: format!("{} echo: {}", R::NAME, message.content),
        })
    }

    async fn subscribe(&self, _handle: &AgentHandle, _event: &str) -> Result<EventStream> {
        Ok(vec![])
    }

    async fn get_config(&self, handle: &AgentHandle) -> Result<RuntimeConfig> {
        Ok(self.store.get(&handle.id).unwrap_or_else(|| RuntimeConfig {
            values: serde_json::json!({ "runtime": R::RUNTIME.as_slug() }),
        }))
    }

    async fn set_config(&self, handle: &AgentHandle, config: &RuntimeConfig) -> Result<()> {
        self.store.set(&handle.id, config.clone());
        Ok(())
    }

    async fn list_skills(&self, _handle: &AgentHandle) -> Result<Vec<Skill>> {
        Ok(vec![])
    }

    async fn install_skill(&self, _handle: &AgentHandle, _skill: &SkillManifest) -> Result<()> {
        Ok(())
    }
}
