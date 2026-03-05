use crate::{DockerAdapter, RuntimeMeta};
use clawden_core::{ChannelSupport, ChannelType, ClawRuntime, RuntimeMetadata};
use std::collections::HashMap;

pub struct ZeroClawMeta;

impl RuntimeMeta for ZeroClawMeta {
    const RUNTIME: ClawRuntime = ClawRuntime::ZeroClaw;
    const NAME: &'static str = "ZeroClaw";

    fn metadata() -> RuntimeMetadata {
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
}

pub type ZeroClawAdapter = DockerAdapter<ZeroClawMeta>;

#[cfg(test)]
mod tests {
    use super::ZeroClawAdapter;
    use clawden_core::{AgentConfig, ClawAdapter, ClawRuntime};

    #[test]
    fn start_persists_forwarded_runtime_config() {
        let _guard = crate::adapter_test_env_lock();
        std::env::set_var("CLAWDEN_ADAPTER_DRY_RUN", "1");
        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime should initialize");
        runtime.block_on(async {
            let adapter = ZeroClawAdapter::default();
            let handle = adapter
                .start(&AgentConfig {
                    name: "test-agent".to_string(),
                    runtime: ClawRuntime::ZeroClaw,
                    model: None,
                    env_vars: vec![("OPENAI_API_KEY".to_string(), "sk-test".to_string())],
                    channels: vec!["telegram".to_string()],
                    tools: vec!["git".to_string(), "http".to_string()],
                })
                .await
                .expect("adapter start should succeed");

            let cfg = adapter
                .get_config(&handle)
                .await
                .expect("adapter config should be readable");
            assert_eq!(
                cfg.values["channels"][0].as_str(),
                Some("telegram"),
                "channel passthrough should be retained"
            );
            assert_eq!(
                cfg.values["tools"][0].as_str(),
                Some("git"),
                "tools passthrough should be retained"
            );
            assert_eq!(
                cfg.values["env_vars"][0][0].as_str(),
                Some("OPENAI_API_KEY"),
                "env var passthrough should be retained"
            );
        });
        std::env::remove_var("CLAWDEN_ADAPTER_DRY_RUN");
    }
}
