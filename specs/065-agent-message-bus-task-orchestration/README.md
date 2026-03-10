---
status: planned
created: 2026-03-09
priority: critical
tags:
- core
- fleet
- orchestration
- message-bus
- master-worker
depends_on:
- 064-fleet-process-supervisor
parent: 090-fleet-execution-foundation
created_at: 2026-03-09T06:02:09.173804710Z
updated_at: 2026-03-09T06:02:09.173804710Z
---

# Agent Message Bus & Task Orchestration — Fleet Collaboration Protocol

## Overview

With agents running and pipes attached (spec 064), this spec adds the collaboration layer: the message bus that routes communication between agents, and the task lifecycle engine that coordinates master-worker workflows.

This is where "multiple isolated agents" becomes "a team that works together on tasks."

## Design

### Message Bus

In-process async message bus using `tokio::sync::broadcast` + `mpsc`:

- Per-agent inbox (`agent_id → mpsc::Sender<AgentEnvelope>`).
- Broadcast channel for fleet-wide events (agent joined, agent failed, task completed).
- `AgentEnvelope` carries: id, from, to, payload, correlation_id (links request ↔ response), timestamp.
- Payload types: `TaskAssignment`, `TaskResult`, `Chat` (peer-to-peer freeform), `System` (health, shutdown, config).
- Delivery: bus writes `AgentEnvelope` as JSON-Lines to each agent's stdin pipe. Agent responses come back via stdout, parsed by the supervisor, and routed through the bus.

### Task Lifecycle Engine

Extends `SwarmCoordinator` with an execution-aware state machine:

```
Created → Delegated (fan-out) → Executing → Results → Aggregated → Done
```

**Master-Worker flow:**
1. Human or leader agent submits a task.
2. Engine decomposes into subtasks based on team config and worker capabilities.
3. Subtasks sent as `TaskAssignment` messages to workers via the bus.
4. Workers process and return `TaskResult` messages.
5. Engine aggregates results using configurable strategy: collect-all, first-wins, majority-vote.
6. Aggregated result returned to requester.

### Team Coordination

Teams group agents with a leader and workers. The leader can delegate tasks, the workers execute. Team config in `clawden.yaml` defines membership and aggregation strategy.

## Plan

- [ ] Implement `MessageBus` with tokio channels and `AgentEnvelope` wire format.
- [ ] Route messages between supervisor stdin/stdout pipes and the bus.
- [ ] Extend `SwarmCoordinator` with execution-aware task states and the decompose → delegate → collect → aggregate flow.
- [ ] Implement result aggregation strategies (collect-all, first-wins, majority-vote).
- [ ] Wire team coordination so leader agents can delegate to their workers.
- [ ] Add `clawden send <agent> <message>` for ad-hoc messaging.

## Test

- [ ] Send a `TaskAssignment` from leader to worker via the bus; verify `TaskResult` comes back.
- [ ] Full fan-out: submit a task to a team; verify all workers receive subtasks and results aggregate correctly.
- [ ] Correlation IDs link request messages to their responses.
- [ ] Broadcast events reach all agents in the fleet.
- [ ] Agent that fails mid-task: verify task engine handles the missing result gracefully.
- [ ] `clawden send` delivers a message and prints the response.

## Notes

The message bus is single-host (in-process tokio channels) in this spec. Cross-host message relay will build on top of spec 062's control channel — the bus interface stays the same, only the transport backend changes. This is why the `AgentEnvelope` format matters: it's the stable protocol that local and remote delivery both use.