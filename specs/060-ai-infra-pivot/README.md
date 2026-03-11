---
status: in-progress
created: 2026-03-11
priority: critical
tags:
- umbrella
- pivot
- strategy
- ai-infra
created_at: 2026-03-11T06:11:16.401477Z
updated_at: 2026-03-11T06:11:16.401477Z
---

# ClawDen: AI Workforce Infrastructure

## Overview

ClawDen is repositioned from "UX shell for xxxclaw runtimes" to **AI workforce infrastructure** — the orchestration layer between AI coding workers and human teams.

### The Problem

Modern AI coding tools (Claude Code, Codex CLI, Copilot CLI, Gemini CLI, OpenCode) are powerful autonomous workers but have no standard way to:
- Report status to human managers via IM/email
- Get approval before taking high-impact actions
- Coordinate across different AI tools on the same project
- Provide a unified interface for human oversight

Meanwhile, claw runtimes (OpenClaw, ZeroClaw, etc.) excel at multi-channel communication (17+ channels) but don't control coding tools.

### The Solution

ClawDen bridges these two worlds with a thin, powerful core:

```
Human Stakeholders (Telegram, Slack, Email, etc.)
        │
  Comm Layer — Claw runtimes as communication adapters
        │
  ClawDen Core — Process registry, config, dispatch, hooks
        │
  Worker Layer — Coding tools as worker adapters
        │
  Codebases / Projects / Infra
```

### Positioning Statement

> **ClawDen** — AI workforce infrastructure. Manage coding AI tools (Claude Code, Codex, Copilot) and connect them to your team's communication channels. One config, any worker, any channel.

### What Changes from Previous Positioning (033)

| Before | After |
|--------|-------|
| UX Shell for claw runtimes | Infra layer for AI workers + comms |
| Claw runtimes = the product | Claw runtimes = comm adapters |
| SDK Platform for cross-runtime skills | Workers use their own native tools (MCP, etc.) |
| Fleet orchestration / swarm coordination | Thin dispatch — workers handle their own sub-agents |
| Dashboard as core pillar | CLI-first, dashboard deferred |

### What Stays

- CLI-Direct architecture (no mandatory server)
- Config translation pipeline (`clawden.yaml` → native format)
- Process management (start/stop/logs/health)
- Provider/key vault
- Runtime install/update management
- Guided onboarding (`clawden init`)

## Design

### Two CRI Interfaces

The existing `ClawAdapter` trait splits into two focused traits:

**CommAdapter** — wraps claw runtimes for human communication
- Start/stop/health for the runtime process
- `send_notification()` — push messages to human channels
- Reuses existing adapter code (openclaw.rs, zeroclaw.rs, etc.)

**WorkerAdapter** — wraps AI coding tools
- Start/stop/health for the coding tool process
- `dispatch(task)` → `TaskResult` — send work, get results
- `stream_output()` — real-time log streaming
- First adapter: Claude Code (`claude -p "task" --output-format json`)

### Notification Hooks

The bridge between workers and comms:
```yaml
# clawden.yaml
hooks:
  on_task_complete:
    notify: [slack/eng-leads, telegram/pm-bot]
  on_task_failed:
    notify: [telegram/on-call]
  on_approval_needed:
    notify: [slack/eng-leads]
    wait: true  # block until human responds
```

### CLI Surface

```sh
# Worker management
clawden worker add claude-code --project ./my-app
clawden worker run claude-code "fix the login bug"
clawden worker status
clawden worker stop claude-code

# Comm management (existing, reframed)
clawden run zeroclaw          # start a comm adapter
clawden channels list         # show connected channels

# Unified
clawden status                # workers + comms at a glance
```

## Plan

- [ ] Archive superseded specs (fleet, sdk, swarm, dashboard-heavy) (064)
- [ ] Define WorkerAdapter trait and Claude Code adapter (061)
- [ ] Refactor CommAdapter from existing ClawAdapter (062)
- [ ] Implement notification hooks (063)
- [ ] Slim down clawden-core (remove swarm, simplify manager/channels) (064)
- [ ] Add `clawden worker` CLI subcommand
- [ ] Update README and positioning language

## Test

- [ ] `clawden worker run claude-code "describe this project"` launches Claude Code and captures output
- [ ] Task completion triggers notification via connected claw runtime channel
- [ ] `clawden status` shows both workers and comm adapters
- [ ] Existing `clawden run zeroclaw` continues to work unchanged

## Notes

- This is the second major pivot (001-008 → 009 orchestration → 060 AI infra)
- Existing runtime adapter code is reusable — it's a reframing, not a rewrite
- MVP: Claude Code worker + Telegram notification via OpenClaw/ZeroClaw
- Workers are process-level — ClawDen doesn't inject into the AI tool's internals (start shallow, go deeper later)
