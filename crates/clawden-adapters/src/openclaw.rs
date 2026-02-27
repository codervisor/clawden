use anyhow::{bail, Result};
use async_trait::async_trait;
use clawden_core::{
    AgentConfig, AgentHandle, AgentMessage, AgentMetrics, AgentResponse, ChannelSupport,
    ChannelType, ClawAdapter, ClawRuntime, EventStream, HealthStatus, InstallConfig,
    RuntimeConfig, RuntimeMetadata, Skill, SkillManifest,
};
use std::collections::HashMap;

pub struct OpenClawAdapter;

#[async_trait]
impl ClawAdapter for OpenClawAdapter {
    fn metadata(&self) -> RuntimeMetadata {
        let mut channel_support = HashMap::new();
        channel_support.insert(ChannelType::Telegram, ChannelSupport::Native);
        channel_support.insert(ChannelType::Discord, ChannelSupport::Native);
        channel_support.insert(ChannelType::Slack, ChannelSupport::Native);
        channel_support.insert(ChannelType::Whatsapp, ChannelSupport::Via("Baileys".into()));
        channel_support.insert(ChannelType::Signal, ChannelSupport::Via("signal-cli".into()));
        channel_support.insert(ChannelType::Feishu, ChannelSupport::Native);
        channel_support.insert(ChannelType::Mattermost, ChannelSupport::Native);
        channel_support.insert(ChannelType::Irc, ChannelSupport::Native);
        channel_support.insert(ChannelType::Teams, ChannelSupport::Native);
        channel_support.insert(ChannelType::Imessage, ChannelSupport::Native);
        channel_support.insert(ChannelType::GoogleChat, ChannelSupport::Native);
        channel_support.insert(ChannelType::Nostr, ChannelSupport::Native);

        RuntimeMetadata {
            runtime: ClawRuntime::OpenClaw,
            version: "unknown".to_string(),
            language: "typescript".to_string(),
            capabilities: vec!["chat".to_string(), "tools".to_string()],
            default_port: Some(18789),
            config_format: Some("json5".to_string()),
            channel_support,
        }
    }

    async fn install(&self, _config: &InstallConfig) -> Result<()> {
        Ok(())
    }

    async fn start(&self, config: &AgentConfig) -> Result<AgentHandle> {
        Ok(AgentHandle {
            id: format!("openclaw-{}", config.name),
            name: config.name.clone(),
            runtime: ClawRuntime::OpenClaw,
        })
    }

    async fn stop(&self, _handle: &AgentHandle) -> Result<()> {
        Ok(())
    }

    async fn restart(&self, _handle: &AgentHandle) -> Result<()> {
        Ok(())
    }

    async fn health(&self, _handle: &AgentHandle) -> Result<HealthStatus> {
        Ok(HealthStatus::Unknown)
    }

    async fn metrics(&self, _handle: &AgentHandle) -> Result<AgentMetrics> {
        Ok(AgentMetrics {
            cpu_percent: 0.0,
            memory_mb: 0.0,
            queue_depth: 0,
        })
    }

    async fn send(&self, _handle: &AgentHandle, _message: &AgentMessage) -> Result<AgentResponse> {
        bail!("OpenClawAdapter.send not implemented")
    }

    async fn subscribe(&self, _handle: &AgentHandle, _event: &str) -> Result<EventStream> {
        Ok(vec![])
    }

    async fn get_config(&self, _handle: &AgentHandle) -> Result<RuntimeConfig> {
        Ok(RuntimeConfig {
            values: serde_json::json!({}),
        })
    }

    async fn set_config(&self, _handle: &AgentHandle, _config: &RuntimeConfig) -> Result<()> {
        Ok(())
    }

    async fn list_skills(&self, _handle: &AgentHandle) -> Result<Vec<Skill>> {
        Ok(vec![])
    }

    async fn install_skill(&self, _handle: &AgentHandle, _skill: &SkillManifest) -> Result<()> {
        Ok(())
    }
}
