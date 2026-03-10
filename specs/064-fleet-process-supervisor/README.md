---
status: planned
created: 2026-03-09
priority: critical
tags:
- core
- fleet
- orchestration
- process-management
depends_on:
- 012-fleet-orchestration
parent: 090-fleet-execution-foundation
created_at: 2026-03-09T06:02:09.110143810Z
updated_at: 2026-03-09T06:02:09.110143810Z
---

# Fleet Process Supervisor & Lifecycle — Agent Spawning, Health & Restart

## Overview

The execution substrate for ClawDen fleets. Before agents can collaborate, they need to start and stay alive. This spec wires up real process management for heterogeneous claw runtimes, replacing the current in-memory stubs.

Scope is deliberately single-host: spawn agent processes, attach communication pipes, monitor health, restart on failure, and shut down cleanly. The fleet config (`clawden.yaml`) is parsed here so `clawden up` can bring an entire fleet online from a single file.

## Design

### Process Supervisor

Wraps the existing `ProcessManager` with supervised lifecycle:

- **Direct mode** (priority): spawn runtime binary as a child process via `tokio::process::Command`. Attach stdin/stdout as JSON-Lines pipes. Capture stderr for log streaming.
- **Docker mode**: `docker run` via the existing `DockerAdapter` path, with attach for message passing.
- Supervised restart with exponential backoff (reuse existing `backoff_ms`).
- Graceful shutdown: SIGTERM → configurable wait → SIGKILL.
- Process group management for clean teardown of the entire fleet.

### Fleet Config Parsing

Parse `agents:` and `teams:` sections from `clawden.yaml`. Each agent declares:
- `runtime` — which claw runtime to use
- `model` — LLM provider/model
- `role` — leader, worker, reviewer, specialist
- `capabilities` — what the agent can do (used by the task engine in the next child spec)
- `channels` — external channel bindings

`clawden up` reads this config, spawns all agent processes, and attaches pipes. `clawden ps` shows live fleet status.

### Health Probes

Per-agent health monitoring via adapter probes on a configurable interval. Unhealthy agents trigger restart. Fleet-level health is the aggregate of all agent health states.

## Plan

- [ ] Wire up direct-mode process spawning for at least OpenClaw and ZeroClaw with stdin/stdout pipe attachment.
- [ ] Implement supervised restart with exponential backoff and graceful shutdown.
- [ ] Parse `agents:` and `teams:` fleet config sections from `clawden.yaml`.
- [ ] Wire `clawden up` to spawn all configured agents and attach pipes.
- [ ] Add per-agent health probes and restart-on-failure logic.
- [ ] Implement `clawden ps` for live fleet status.

## Test

- [ ] Spawn 2+ agents in direct mode; verify they start and respond to health checks.
- [ ] Kill an agent process; verify supervisor restarts it with backoff.
- [ ] `clawden up` from a `clawden.yaml` with 3+ agents; verify entire fleet comes online.
- [ ] Graceful shutdown terminates all agents without orphaned processes.
- [ ] Health probe detects unresponsive agent and triggers restart.

## Notes

This spec intentionally does not cover inter-agent messaging or task orchestration — those build on top of the pipes established here. The stdin/stdout JSON-Lines interface is the contract between this spec and the message bus child spec.