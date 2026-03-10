---
status: planned
created: 2026-03-09
priority: high
tags:
- core
- fleet
- persistence
- reliability
depends_on:
- 065-agent-message-bus-task-orchestration
parent: 090-fleet-execution-foundation
created_at: 2026-03-09T06:02:09.191794098Z
updated_at: 2026-03-09T06:02:09.191794098Z
---

# Fleet State Persistence & Recovery — SQLite Backend & Crash Resilience

## Overview

Without persistence, a ClawDen restart loses all fleet state: agent registrations, in‑flight tasks, partial results, and audit history. This spec replaces the in-memory `HashMap`s with a SQLite backend so that fleets survive crashes and restarts.

This is what makes the fleet production-grade on a single host. Combined with spec 062's distributed auth, it provides the foundation for durable multi-host orchestration.

## Design

### SQLite Backend

Use `rusqlite` (zero external deps, embedded) for persistent fleet state:

| Table | Purpose |
|---|---|
| `agents` | Registration, state, capabilities, config |
| `teams` | Team definitions and membership |
| `tasks` | Task tree with parent-child relationships, state machine position |
| `task_results` | Worker outputs linked to tasks |
| `messages` | Message log for debugging and replay |
| `audit_events` | Existing audit log, now durable |

Location: `~/.clawden/state.db` (project-scoped via config hash).

### Recovery

On `clawden up`, if a previous state DB exists:
- Restore agent registrations and team memberships.
- Resume in-flight tasks from their last known state.
- Re-spawn agents that were running before the crash.
- Replay queued messages that were not delivered.

Configurable policy: `fleet.recovery: resume | clean-start`.

### CLI Surfaces

- `clawden logs <agent>` — stream or tail per-agent logs from the persistent message store.
- `clawden ps` — show fleet status from durable state (survives server restart).
- `clawden audit` — query auth and lifecycle events.

## Plan

- [ ] Add SQLite schema and migration for agents, teams, tasks, results, messages, audit tables.
- [ ] Migrate process supervisor and message bus to write state changes to SQLite.
- [ ] Implement fleet recovery on restart: agent re-spawn, task resume, message replay.
- [ ] Wire `clawden logs <agent>` to stream from the persistent message store.
- [ ] Wire `clawden audit` to read from the durable audit event table.

## Test

- [ ] Restart ClawDen server; verify fleet state persists and agents are recoverable.
- [ ] In-flight task survives a crash and resumes from its last state.
- [ ] `clawden logs <agent>` shows historical messages after a restart.
- [ ] `clawden audit` returns events from previous sessions.
- [ ] `fleet.recovery: clean-start` discards previous state and starts fresh.

## Notes

SQLite is the right choice at this scale — dozens of agents, thousands of tasks — with zero ops burden. If multi-host distributed state is ever needed, the SQLite backend can be swapped for a networked store behind the same trait interface without changing the supervisor or bus code.