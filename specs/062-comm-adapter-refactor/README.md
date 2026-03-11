---
status: planned
created: 2026-03-11
priority: high
tags:
- core
- adapter
- comm
- refactor
- ai-infra
parent: 060-ai-infra-pivot
created_at: 2026-03-11T06:12:38.304830Z
updated_at: 2026-03-11T06:12:38.304830Z
---

# Comm Adapter Refactor — Claw Runtimes as Communication Layer

## Overview

Refactor the existing `ClawAdapter` trait into a focused `CommAdapter` trait. Claw runtimes (OpenClaw, ZeroClaw, etc.) are repositioned from "the product" to "the communication layer" — they're how ClawDen talks to humans.

### What Changes

The current `ClawAdapter` has ~15 methods covering everything (install, start, stop, health, metrics, messaging, config, skills). The new `CommAdapter` trims this to only what's needed for the comm role:

- **Keep**: start, stop, health, config injection, send_notification
- **Remove from trait**: skill management methods, metrics collection
- **Simplify**: messaging becomes `send_notification()` — one-way push to channel

### What Stays the Same

- Per-runtime adapter files (openclaw.rs, zeroclaw.rs, etc.) stay
- Config translation pipeline stays (clawden.yaml → native format)
- Runtime descriptor registry stays
- Docker + direct-install execution modes stay
- The `clawden run <runtime>` command stays and works exactly as before

## Design

### CommAdapter Trait

```rust
#[async_trait]
pub trait CommAdapter: Send + Sync {
    fn metadata(&self) -> &RuntimeDescriptor;

    async fn start(&self, config: &RuntimeConfig) -> Result<ProcessHandle>;
    async fn stop(&self, handle: &ProcessHandle) -> Result<()>;
    async fn health(&self, handle: &ProcessHandle) -> Result<HealthStatus>;

    /// Push a notification to the runtime's connected channels
    async fn send_notification(
        &self,
        handle: &ProcessHandle,
        notification: &Notification,
    ) -> Result<()>;
}
```

### Notification Type

```rust
pub struct Notification {
    pub channel: ChannelTarget,     // e.g., "telegram/pm-bot"
    pub message: String,
    pub level: NotificationLevel,   // Info, Warning, Error, Success
    pub context: Option<TaskRef>,   // link back to the task that triggered it
}
```

### How send_notification Works

Claw runtimes expose HTTP APIs or stdin-based message injection. The notification flows:
1. ClawDen formats the notification message
2. Calls the runtime's API endpoint (e.g., ZeroClaw's HTTP API, OpenClaw's REST endpoint)
3. Runtime delivers to the configured channel (Telegram, Slack, etc.)

This is the inverse of the current flow — instead of the claw runtime receiving messages and doing work, ClawDen pushes outbound messages through the runtime.

### Migration Path

1. Keep `ClawAdapter` temporarily as an alias/wrapper
2. Implement `CommAdapter` as a subset of existing adapter methods
3. Existing `clawden run` continues to call the same start/stop/health methods
4. New notification path added alongside existing messaging

## Plan

- [ ] Define `CommAdapter` trait in `clawden-core`
- [ ] Define `Notification`, `ChannelTarget`, `NotificationLevel` types
- [ ] Refactor existing runtime adapters to implement `CommAdapter`
- [ ] Remove skill-related methods from adapter trait
- [ ] Implement `send_notification` for OpenClaw adapter (HTTP API)
- [ ] Implement `send_notification` for ZeroClaw adapter (HTTP API)
- [ ] Ensure `clawden run` works unchanged through the refactor

## Test

- [ ] Existing `clawden run zeroclaw` works identically after refactor
- [ ] `send_notification` delivers message to running claw runtime
- [ ] Adapter trait compiles without skill-related methods
- [ ] Config translation pipeline unchanged
