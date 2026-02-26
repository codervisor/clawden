mod audit;
mod lifecycle;

use crate::audit::{AuditEvent, AuditLog};
use crate::lifecycle::AgentState;
use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "clawlab-server",
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .compact()
        .init();

    let app = Router::new().route("/health", get(health));
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let audit_log = AuditLog::default();

    let startup_event = AuditEvent {
        actor: "system".to_string(),
        action: "server.start".to_string(),
        target: "clawlab-server".to_string(),
        timestamp_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_millis() as u64,
    };
    audit_log.append(startup_event);
    if let Some(last) = audit_log.list().last() {
        info!(
            actor = %last.actor,
            action = %last.action,
            target = %last.target,
            timestamp_unix_ms = last.timestamp_unix_ms,
            "audit event recorded"
        );
    }

    let lifecycle_path_valid = AgentState::Registered.can_transition_to(AgentState::Installed)
        && AgentState::Installed.can_transition_to(AgentState::Running);
    let known_states = [
        AgentState::Registered,
        AgentState::Installed,
        AgentState::Running,
        AgentState::Stopped,
        AgentState::Degraded,
    ];
    info!(
        lifecycle_path_valid,
        known_state_count = known_states.len(),
        "lifecycle transition check"
    );

    info!(%addr, "starting clawlab server");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    axum::serve(listener, app)
        .await
        .expect("server failed unexpectedly");
}
