---
status: planned
created: 2026-03-10
priority: critical
tags:
- fleet
- execution
- foundation
- group
- umbrella
parent: 054-agent-fleet-execution-layer
created_at: 2026-03-10T08:42:22.313133145Z
updated_at: 2026-03-10T08:42:22.313133145Z
---

# Fleet Execution Foundation — Process Supervisor, Message Bus & Persistence

## Overview

Group spec for the execution foundation — Layers 1–3 of the fleet execution layer. These three specs form the strictly sequential critical path: process supervisor → message bus → persistence.

All three are single-host by design. Distributed execution builds on top via spec 062's control channel.

## Design

| Child | Layer | Purpose |
|-------|-------|---------|
| `064-fleet-process-supervisor` | 1 — Execution substrate | Spawn agents, health probes, supervised restart, graceful shutdown, `clawden up` |
| `065-agent-message-bus-task-orchestration` | 2 — Collaboration protocol | In-process message bus, AgentEnvelope, team coordination, task lifecycle |
| `066-fleet-state-persistence-recovery` | 3 — Reliability | SQLite backend for agents/teams/tasks/results/messages/audit, crash recovery |

Implementation order is strictly sequential: 064 → 065 → 066.

## Plan

- [ ] Complete 064 (process supervisor) for agent lifecycle management
- [ ] Complete 065 (message bus) for inter-agent communication
- [ ] Complete 066 (persistence) for crash resilience

## Test

- [ ] A fleet of 3+ agents starts, stays healthy, and shuts down cleanly
- [ ] Master-worker task flow produces aggregated results
- [ ] Fleet state survives a crash and resumes on restart