use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub actor: String,
    pub action: String,
    pub target: String,
    pub timestamp_unix_ms: u64,
}

#[derive(Clone, Default)]
pub struct AuditLog {
    inner: Arc<Mutex<Vec<AuditEvent>>>,
}

impl AuditLog {
    pub fn append(&self, event: AuditEvent) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.push(event);
        }
    }

    pub fn list(&self) -> Vec<AuditEvent> {
        self.inner
            .lock()
            .map_or_else(|_| Vec::new(), |guard| guard.clone())
    }
}

pub fn append_audit(audit: &Arc<AuditLog>, actor: &str, action: &str, target: &str) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX_EPOCH")
        .as_millis() as u64;

    audit.append(AuditEvent {
        actor: actor.to_string(),
        action: action.to_string(),
        target: target.to_string(),
        timestamp_unix_ms: now,
    });
}
