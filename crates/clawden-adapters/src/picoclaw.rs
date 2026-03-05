use crate::{DockerAdapter, RuntimeMeta};
use clawden_core::{ChannelSupport, ChannelType, ClawRuntime, RuntimeMetadata};
use std::collections::HashMap;

pub struct PicoClawMeta;

impl RuntimeMeta for PicoClawMeta {
    const RUNTIME: ClawRuntime = ClawRuntime::PicoClaw;
    const NAME: &'static str = "PicoClaw";

    fn metadata() -> RuntimeMetadata {
        let mut channel_support = HashMap::new();
        channel_support.insert(ChannelType::Telegram, ChannelSupport::Native);
        channel_support.insert(ChannelType::Discord, ChannelSupport::Native);
        channel_support.insert(ChannelType::Slack, ChannelSupport::Native);
        channel_support.insert(ChannelType::Whatsapp, ChannelSupport::Native);
        channel_support.insert(ChannelType::Feishu, ChannelSupport::Native);
        channel_support.insert(ChannelType::Dingtalk, ChannelSupport::Native);
        channel_support.insert(ChannelType::Qq, ChannelSupport::Native);
        channel_support.insert(ChannelType::Line, ChannelSupport::Native);

        RuntimeMetadata {
            runtime: ClawRuntime::PicoClaw,
            version: "unknown".to_string(),
            language: "go".to_string(),
            capabilities: vec!["chat".to_string(), "embedded".to_string()],
            default_port: None,
            config_format: Some("json".to_string()),
            channel_support,
        }
    }
}

pub type PicoClawAdapter = DockerAdapter<PicoClawMeta>;
