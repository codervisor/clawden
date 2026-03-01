use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Registered,
    Installed,
    Running,
    Stopped,
    Degraded,
}

impl AgentState {
    pub fn can_transition_to(self, next: AgentState) -> bool {
        use AgentState::{Degraded, Installed, Registered, Running, Stopped};

        match (self, next) {
            (Registered, Installed) => true,
            (Installed, Running) => true,
            (Running, Stopped) => true,
            (Running, Degraded) => true,
            (Degraded, Running) => true,
            (Stopped, Running) => true,
            _ if self == next => true,
            _ => false,
        }
    }
}
