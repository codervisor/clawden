use clawden_core::{AgentMessage, AgentResponse, ChannelSupport, ChannelType, ClawRuntime, RuntimeMetadata};
use serde::Serialize;

/// Channel proxy status for a proxied connection.
#[derive(Debug, Clone, Serialize)]
pub struct ProxyStatus {
    pub channel_type: String,
    pub runtime: String,
    pub is_proxied: bool,
    pub reason: Option<String>,
}

/// Determines whether a channel needs proxying for a given runtime by checking
/// the runtime's native channel support from adapter metadata.
pub fn needs_proxy(metadata: &RuntimeMetadata, channel: &ChannelType) -> bool {
    match metadata.channel_support.get(channel) {
        Some(ChannelSupport::Native) | Some(ChannelSupport::Via(_)) => false,
        Some(ChannelSupport::Unsupported) | None => true,
    }
}

/// Build a proxy status report for a runtime × channel combination.
pub fn proxy_status(metadata: &RuntimeMetadata, channel: &ChannelType) -> ProxyStatus {
    let proxied = needs_proxy(metadata, channel);
    ProxyStatus {
        channel_type: channel.to_string(),
        runtime: format!("{:?}", metadata.runtime),
        is_proxied: proxied,
        reason: if proxied {
            Some(format!(
                "{:?} does not natively support {}; ClawDen will proxy",
                metadata.runtime, channel
            ))
        } else {
            None
        },
    }
}

/// Proxy a message from an unsupported channel to a runtime via CRI send().
/// This is the core bridge function: receive message on channel X, relay to
/// runtime through its CRI adapter, and return the response.
///
/// In a real deployment, this would be called by the channel webhook handler.
/// The channel adapter (e.g., Telegram bot) receives a message, determines the
/// target runtime doesn't natively support this channel, and routes through
/// this proxy.
pub fn create_proxy_message(
    channel_type: &ChannelType,
    sender: &str,
    content: &str,
) -> AgentMessage {
    AgentMessage {
        role: format!("proxy:{}", channel_type),
        content: format!("[{sender}] {content}"),
    }
}

/// Format a proxied response for sending back to the channel.
pub fn format_proxy_response(response: &AgentResponse) -> String {
    response.content.clone()
}
