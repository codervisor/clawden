---
status: planned
created: 2026-03-06
priority: critical
tags:
- core
- fleet
- orchestration
- message-bus
- master-worker
depends_on:
- 012-fleet-orchestration
created_at: 2026-03-06T06:56:22.809363808Z
updated_at: 2026-03-06T06:56:22.809456972Z
---
# Agent Fleet Execution Layer — Master-Worker Orchestration, Message Bus & Task Lifecycle

## Overview

This spec is now the umbrella for the fleet execution layer — actually running agents, passing messages between them, collecting results, and persisting state.

The original spec packed six concerns into one. The work splits naturally into three layers that build on each other:

1. **Execution substrate** — agents running and staying alive.
2. **Collaboration protocol** — agents communicating and working together on tasks.
3. **Reliability layer** — fleet state surviving crashes and restarts.

All three are single-host by design. Distributed execution (cross-host message relay, remote supervisor) builds on top of spec 062's control channel, reusing the same `AgentEnvelope` protocol and supervisor interface.

## Design

This umbrella coordinates three child specs:

| Child | Purpose |
| --- | --- |
| `064-fleet-process-supervisor` | Spawn agents, attach pipes, health probes, supervised restart, graceful shutdown, fleet config parsing, `clawden up` |
| `065-agent-message-bus-task-orchestration` | In-process message bus, `AgentEnvelope` protocol, team coordination, task lifecycle engine, result aggregation |
| `066-fleet-state-persistence-recovery` | SQLite backend for agents/teams/tasks/results/messages/audit, crash recovery, `clawden logs`/`clawden audit` |

Shared architectural rules:

- JSON-Lines over stdin/stdout is the agent communication wire format.
- `AgentEnvelope` is the stable message protocol used by both local and (future) remote delivery.
- The process supervisor owns agent lifecycle; the message bus owns routing; persistence is the durability layer underneath both.
- Master-worker is the core collaboration pattern; aggregation strategies are pluggable.

## Plan

- [ ] Complete spec 064 to establish agent process management and fleet startup.
- [ ] Complete spec 065 to add inter-agent messaging and task orchestration on top of the running fleet.
- [ ] Complete spec 066 to make fleet state persistent and recoverable.

## Test

- [ ] A fleet of 3+ heterogeneous agents starts, stays healthy, and shuts down cleanly.
- [ ] A master-worker task flow produces aggregated results from multiple workers.
- [ ] Fleet state survives a crash and resumes on restart.

## Notes

Implementation order is strictly sequential: 064 → 065 → 066. Each layer depends on the previous one.

The distributed story connects here:
- Spec 062 (remote enrollment + control channel) provides the transport for cross-host message relay.
- Spec 065's `AgentEnvelope` format is the protocol that travels over that transport.
- A future spec can add a `RemoteMessageBus` backend that routes envelopes through 062's control channel, swapping the tokio channel backend without changing the bus API.