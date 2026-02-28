use anyhow::Result;
use async_trait::async_trait;
use clawden_core::{
    AgentConfig, AgentHandle, AgentMessage, AgentMetrics, AgentResponse, ChannelSupport,
    ChannelType, ClawAdapter, ClawRuntime, EventStream, HealthStatus, InstallConfig, RuntimeConfig,
    RuntimeMetadata, Skill, SkillManifest,
};
use std::collections::HashMap;

pub struct ZeroClawAdapter;

#[async_trait]
impl ClawAdapter for ZeroClawAdapter {
    fn metadata(&self) -> RuntimeMetadata {
        let mut channel_support = HashMap::new();
        channel_support.insert(ChannelType::Telegram, ChannelSupport::Native);
        channel_support.insert(ChannelType::Discord, ChannelSupport::Native);
        channel_support.insert(ChannelType::Slack, ChannelSupport::Native);
        channel_support.insert(
            ChannelType::Whatsapp,
            ChannelSupport::Via("Meta Cloud API".into()),
        );
        channel_support.insert(ChannelType::Signal, ChannelSupport::Native);
        channel_support.insert(ChannelType::Feishu, ChannelSupport::Native);
        channel_support.insert(ChannelType::Matrix, ChannelSupport::Native);
        channel_support.insert(ChannelType::Email, ChannelSupport::Native);
        channel_support.insert(ChannelType::Mattermost, ChannelSupport::Native);
        channel_support.insert(ChannelType::Irc, ChannelSupport::Native);
        channel_support.insert(ChannelType::Imessage, ChannelSupport::Native);
        channel_support.insert(ChannelType::Nostr, ChannelSupport::Native);

        RuntimeMetadata {
            runtime: ClawRuntime::ZeroClaw,
            version: "unknown".to_string(),
            language: "rust".to_string(),
            capabilities: vec!["chat".to_string(), "reasoning".to_string()],
            default_port: Some(42617),
            config_format: Some("toml".to_string()),
            channel_support,
        }
    }

    async fn install(&self, _config: &InstallConfig) -> Result<()> {
        Ok(())
    }

    async fn start(&self, config: &AgentConfig) -> Result<AgentHandle> {
        Ok(AgentHandle {
            id: format!("zeroclaw-{}", config.name),
            name: config.name.clone(),
            runtime: ClawRuntime::ZeroClaw,
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

    async fn send(&self, _handle: &AgentHandle, message: &AgentMessage) -> Result<AgentResponse> {
        Ok(AgentResponse {
            content: format!("ZeroClaw echo: {}", message.content),
        })
    }

    async fn subscribe(&self, _handle: &AgentHandle, _event: &str) -> Result<EventStream> {
        Ok(vec![])
    }

    async fn get_config(&self, _handle: &AgentHandle) -> Result<RuntimeConfig> {
        Ok(RuntimeConfig {
            values: serde_json::json!({ "runtime": "zeroclaw" }),
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
