---
status: planned
created: 2026-03-11
priority: high
tags:
- refactor
- core
- tech-debt
- ai-infra
parent: 060-ai-infra-pivot
created_at: 2026-03-11T06:12:38.319856Z
updated_at: 2026-03-11T06:12:38.319856Z
---

# Core Slimdown — Remove Swarm, Simplify Manager & Channels

## Overview

Trim `clawden-core` to match the lean AI infra positioning. Remove modules that reimplmement capabilities the AI workers already have (swarm coordination, fleet discovery), and simplify modules that were over-engineered for the claw-orchestration model.

### Guiding Principle

ClawDen is a **thin dispatch layer**, not a scheduler or orchestrator. AI coding tools (Claude Code, Codex) have their own multi-agent, sub-agent, and tool-use systems. ClawDen doesn't need to replicate that — it just needs to start/stop workers, send them tasks, and report results.

## Design

### Remove: Swarm Coordinator (`swarm.rs`)

**What it has**: SwarmTeam, SwarmRole (Leader/Worker/Reviewer), SwarmTask, SwarmCoordinator, fan-out task distribution, task status tracking.

**Why remove**: AI coding tools manage their own sub-agents. Claude Code spawns agents, Codex has internal orchestration. ClawDen re-implementing this is redundant complexity.

**Migration**: Delete `swarm.rs`. Any code referencing swarm types gets removed.

### Remove: Discovery Service (`discovery.rs`)

**What it has**: DiscoveryService, DiscoveredEndpoint, DiscoveryMethod, mDNS-style service mesh.

**Why remove**: Overkill for the new model. ClawDen tracks running processes via ProcessManager — it doesn't need network-level discovery.

**Migration**: Delete `discovery.rs`. Process registry in manager.rs is sufficient.

### Simplify: LifecycleManager (`manager.rs`)

**Current**: Round-robin task assignment, consecutive failure tracking with backoff, recovery attempts, multi-agent registration with capability matching.

**Target**: Simple process registry. Register a worker/comm adapter, start it, check health, stop it. No scheduling logic.

**Keep**: `start()`, `stop()`, `health()`, `list()`, agent state enum.
**Remove**: Round-robin routing, `assign_task()`, recovery backoff, capability-based routing.

### Simplify: Channels (`channels.rs`)

**Current**: ChannelStore with Active/Draining/Released binding states, ChannelBinding, ChannelConnectionStatus, ChannelHealthEntry, credential health checks, MatrixRow.

**Target**: Keep `ChannelType` enum and `ChannelInstanceConfig` (needed for config translation). Remove the binding/draining state machine — claw runtimes manage their own channel connections.

**Keep**: `ChannelType`, `ChannelInstanceConfig`, channel descriptor metadata.
**Remove**: `ChannelStore`, `ChannelBinding`, `ChannelBindingStatus`, `ChannelConnectionStatus`, `ChannelHealthEntry`.

### Simplify: Skill System

**Current**: `Skill`, `SkillManifest`, `list_skills()`, `install_skill()`, skill-related `ClawAdapter` methods.

**Target**: Remove entirely. Workers use their own native tool/plugin systems (MCP servers for Claude Code, etc.).

### Keep Untouched

- `runtime_descriptor.rs` — still needed for comm adapters
- `process.rs` — process management is core to both workers and comms
- `provider_registry.rs` — API key management still needed
- Config translation pipeline — still needed for comm adapters
- `audit.rs` — small, harmless, can stay dormant

## Plan

- [ ] Delete `swarm.rs` and remove from `lib.rs` exports
- [ ] Delete `discovery.rs` and remove from `lib.rs` exports
- [ ] Simplify `manager.rs` — strip routing/recovery, keep process registry
- [ ] Simplify `channels.rs` — keep types, remove state machine
- [ ] Remove skill types and skill-related adapter methods
- [ ] Fix all compilation errors from removals
- [ ] Run `cargo clippy` and `cargo test` to verify clean build

## Test

- [ ] `cargo build` succeeds after removals
- [ ] `cargo test` passes (existing tests may need removal/update)
- [ ] `clawden run zeroclaw` works unchanged
- [ ] `clawden ps` / `clawden stop` work unchanged
- [ ] No dead code warnings for removed types
