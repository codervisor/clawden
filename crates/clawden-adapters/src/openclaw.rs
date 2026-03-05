use crate::{DockerAdapter, RuntimeMeta};
use clawden_core::{ChannelSupport, ChannelType, ClawRuntime, RuntimeMetadata};
use std::collections::HashMap;

pub struct OpenClawMeta;

impl RuntimeMeta for OpenClawMeta {
    const RUNTIME: ClawRuntime = ClawRuntime::OpenClaw;
    const NAME: &'static str = "OpenClaw";

    fn metadata() -> RuntimeMetadata {
        let mut channel_support = HashMap::new();
        channel_support.insert(ChannelType::Telegram, ChannelSupport::Native);
        channel_support.insert(ChannelType::Discord, ChannelSupport::Native);
        channel_support.insert(ChannelType::Slack, ChannelSupport::Native);
        channel_support.insert(ChannelType::Whatsapp, ChannelSupport::Via("Baileys".into()));
        channel_support.insert(
            ChannelType::Signal,
            ChannelSupport::Via("signal-cli".into()),
        );
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
}

pub type OpenClawAdapter = DockerAdapter<OpenClawMeta>;
