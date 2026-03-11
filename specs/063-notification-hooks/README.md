---
status: planned
created: 2026-03-11
priority: high
tags:
- core
- hooks
- notification
- ai-infra
depends_on:
- 061-worker-adapter-interface
- 062-comm-adapter-refactor
parent: 060-ai-infra-pivot
created_at: 2026-03-11T06:12:38.314245Z
updated_at: 2026-03-11T06:12:38.314245Z
---

# Notification Hooks — Task Events to Communication Channels

## Overview

The bridge between AI workers and human teams. When a worker completes a task (or fails, or needs approval), ClawDen pushes a notification through a connected claw runtime to the human's preferred channel.

This is ClawDen's **killer feature** — the thing no AI coding tool provides today.

### Examples

- Claude Code finishes a PR → Slack message to #eng-leads with summary + PR link
- Codex task fails with error → Telegram alert to on-call engineer
- Worker needs to delete a production database table → Slack message asking for approval, blocks until human responds

## Design

### Hook Configuration in clawden.yaml

```yaml
hooks:
  on_task_complete:
    notify:
      - channel: slack/eng-leads
        template: "✅ {{worker}} completed: {{task.summary}}\n{{task.output_summary}}"
      - channel: telegram/pm-bot

  on_task_failed:
    notify:
      - channel: telegram/on-call
        template: "❌ {{worker}} failed: {{task.summary}}\nError: {{task.error}}"

  on_approval_needed:
    notify:
      - channel: slack/eng-leads
        template: "🔒 {{worker}} needs approval: {{task.description}}"
    wait: true  # block worker until human responds via channel
```

### Hook Engine

```rust
pub struct HookEngine {
    comm_registry: CommAdapterRegistry,
}

impl HookEngine {
    /// Called by the worker dispatch loop after task completes
    pub async fn fire(&self, event: HookEvent) -> Result<()>;
}

pub enum HookEvent {
    TaskComplete { worker: String, result: TaskResult },
    TaskFailed { worker: String, error: String },
    ApprovalNeeded { worker: String, description: String, response_tx: oneshot::Sender<bool> },
}
```

### Template Rendering

Simple `{{variable}}` substitution — no need for a full template engine. Variables:
- `{{worker}}` — worker display name
- `{{task.summary}}` — first line of task prompt
- `{{task.output_summary}}` — truncated output (first 500 chars)
- `{{task.error}}` — error message (for failures)
- `{{task.duration}}` — human-readable duration

### Approval Flow (wait: true)

1. Worker requests approval via HookEngine
2. HookEngine sends notification to channel with approve/reject prompt
3. HookEngine blocks on `response_rx`
4. Human responds in channel (e.g., replies "approve" in Telegram)
5. Claw runtime forwards response back to ClawDen
6. HookEngine unblocks worker with the decision

This is the hardest part — requires bidirectional communication with the claw runtime. MVP can start with one-way (notify only), add approval flow in a follow-up.

## Plan

- [ ] Define `HookEvent` enum and `HookConfig` in clawden-config
- [ ] Parse `hooks:` section from clawden.yaml
- [ ] Implement `HookEngine::fire()` for one-way notifications
- [ ] Implement `{{variable}}` template rendering
- [ ] Wire hook engine into worker dispatch loop (fire on complete/fail)
- [ ] Add default notification templates (used when no custom template specified)
- [ ] (Follow-up) Implement approval flow with bidirectional channel communication

## Test

- [ ] Hook fires on task completion and sends notification via comm adapter
- [ ] Template variables are correctly substituted
- [ ] Missing comm adapter (channel not running) produces clear error
- [ ] Hook config with no `notify` targets is a no-op (no crash)
