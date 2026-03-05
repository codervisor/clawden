use crate::{DockerAdapter, RuntimeMeta};
use clawden_core::{ChannelSupport, ChannelType, ClawRuntime, RuntimeMetadata};
use std::collections::HashMap;

pub struct NanoClawMeta;

impl RuntimeMeta for NanoClawMeta {
    const RUNTIME: ClawRuntime = ClawRuntime::NanoClaw;
    const NAME: &'static str = "NanoClaw";

    fn metadata() -> RuntimeMetadata {
        let mut channel_support = HashMap::new();
        channel_support.insert(ChannelType::Telegram, ChannelSupport::Via("skill".into()));
        channel_support.insert(ChannelType::Discord, ChannelSupport::Via("skill".into()));
        channel_support.insert(ChannelType::Slack, ChannelSupport::Via("skill".into()));
        channel_support.insert(ChannelType::Whatsapp, ChannelSupport::Native);

        RuntimeMetadata {
            runtime: ClawRuntime::NanoClaw,
            version: "unknown".to_string(),
            language: "typescript".to_string(),
            capabilities: vec!["chat".to_string(), "skills".to_string()],
            default_port: None,
            config_format: Some("code".to_string()),
            channel_support,
        }
    }
}

pub type NanoClawAdapter = DockerAdapter<NanoClawMeta>;
