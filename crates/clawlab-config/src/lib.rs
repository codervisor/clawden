use clawlab_core::ClawRuntime;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClawLabConfig {
    pub agent: AgentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub runtime: ClawRuntime,
    pub model: ModelConfig,
    #[serde(default)]
    pub tools: Vec<ToolConfig>,
    #[serde(default)]
    pub channels: Vec<ChannelConfig>,
    pub security: SecurityConfig,
    #[serde(default)]
    pub extras: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub provider: String,
    pub name: String,
    pub api_key_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub name: String,
    #[serde(default)]
    pub allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub channel: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub allowlist: Vec<String>,
    #[serde(default)]
    pub sandboxed: bool,
}

impl ClawLabConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.agent.name.trim().is_empty() {
            return Err("agent.name must not be empty".to_string());
        }

        if self.agent.model.provider.trim().is_empty() || self.agent.model.name.trim().is_empty() {
            return Err("agent.model provider and name must not be empty".to_string());
        }

        Ok(())
    }

    pub fn to_safe_json(&self) -> Value {
        let mut value = serde_json::to_value(self).unwrap_or(Value::Null);
        if let Some(api_ref) = value
            .get_mut("agent")
            .and_then(|a| a.get_mut("model"))
            .and_then(|m| m.get_mut("api_key_ref"))
        {
            *api_ref = Value::String("<redacted>".to_string());
        }
        value
    }
}
