---
status: planned
created: 2026-03-06
priority: high
tags:
- core
- memory
- persistence
- workspace
- git
depends_on:
- 050-agent-fleet-execution-layer
created_at: 2026-03-06T07:10:43.312124552Z
updated_at: 2026-03-06T07:10:43.312226612Z
---

# Agent Workspace Persistence — Git-Backed Memory Sync & Recovery

> **Status**: planned · **Priority**: high · **Created**: 2026-03-06

## Overview

AI agents running on ephemeral infrastructure (GitHub Codespaces, cloud VMs, containers) lose their entire workspace — memory files, identity, user context, tools config — when the host is destroyed. This makes agents amnesiac across infrastructure cycles.

ClawDen should treat agent memory as a first-class managed resource: automatically persisting each agent's workspace to a durable backend and restoring it on fresh deployments. The agent shouldn't need to solve its own persistence — ClawDen handles it.

### Why Now

- Agents accumulate valuable context over time: user preferences, project knowledge, decision history, relationship nuance
- Ephemeral compute (Codespaces, spot instances, autoscaling containers) is the default deployment model
- Manual backup is fragile — one forgotten push and weeks of context are lost
- This is table stakes for any serious agent fleet: agents must survive infrastructure churn

## Design

### Git as the Persistence Backend

Git is the natural choice for agent workspaces:
- **Versioned history** — full audit trail of how memory evolved
- **Conflict resolution** — built-in merge for multi-device scenarios
- **Free hosting** — GitHub/GitLab private repos at zero cost
- **Auth already solved** — tokens, SSH keys, GitHub Apps
- **Diffable** — memory is markdown/JSON, perfect for git

### Architecture

```
┌───────────────────────────────────────────────────┐
│                   ClawDen CLI/Server               │
│                                                    │
│  ┌──────────────────────────────────────────────┐  │
│  │         Workspace Persistence Manager         │  │
│  │                                               │  │
│  │  Per-agent config (in clawden.yaml):          │  │
│  │    workspace:                                 │  │
│  │      repo: codervisor/agent-memory            │  │
│  │      path: agents/{agent-name}/               │  │
│  │      sync_interval: 30m                       │  │
│  │      auto_restore: true                       │  │
│  │                                               │  │
│  │  Operations:                                  │  │
│  │    clawden workspace sync [agent]             │  │
│  │    clawden workspace restore [agent]          │  │
│  │    clawden workspace status [agent]           │  │
│  │    clawden workspace history [agent]          │  │
│  └──────┬──────────────────────────┬─────────────┘  │
│         │                          │                │
│    ┌────▼────┐              ┌──────▼──────┐         │
│    │  Sync   │              │  Restore    │         │
│    │  Engine │              │  Engine     │         │
│    │         │              │             │         │
│    │ Watch → │              │ Clone/pull  │         │
│    │ Commit →│              │ → workspace │         │
│    │ Push    │              │   init      │         │
│    └─────────┘              └─────────────┘         │
└───────────────────────────────────────────────────┘
```

### Sync Engine

Runs as a background task within `clawden up` or triggered by the agent's heartbeat:

1. **Change detection**: `git status` on agent workspace directory
2. **Smart commit**: Only commit if meaningful changes exist (skip if only timestamps changed)
3. **Push**: Push to configured remote with retry + exponential backoff
4. **Conflict handling**: If remote has diverged (e.g., agent ran on two hosts), pull with rebase. Memory files are append-friendly markdown, so conflicts are rare and resolvable.

Sync interval is configurable per-agent. Default: 30 minutes. Critical agents (leader/coordinator) can sync more frequently.

### Restore Engine

Triggered on `clawden up` when an agent's workspace directory is empty or missing:

1. Check if `workspace.repo` is configured for the agent
2. Clone (or pull if partial) into the agent's workspace path
3. Verify workspace integrity (MEMORY.md, IDENTITY.md exist)
4. Signal agent ready — the runtime reads restored files on startup

### Multi-Agent Layout

A single repo can host multiple agents using path prefixes:

```
codervisor/agent-memory/
├── agents/
│   ├── coordinator/
│   │   ├── MEMORY.md
│   │   ├── IDENTITY.md
│   │   ├── USER.md
│   │   └── memory/
│   │       └── 2026-03-06.md
│   ├── coder-1/
│   │   ├── MEMORY.md
│   │   └── memory/
│   └── researcher/
│       ├── MEMORY.md
│       └── memory/
└── shared/              # Optional: shared context across agents
    └── PROJECT.md
```

### Fleet Config Extension

```yaml
# clawden.yaml
agents:
  coordinator:
    runtime: openclaw
    workspace:
      repo: codervisor/agent-memory
      path: agents/coordinator
      sync_interval: 15m
      auto_restore: true

  coder-1:
    runtime: zeroclaw
    workspace:
      repo: codervisor/agent-memory
      path: agents/coder-1
      sync_interval: 1h
```

### Security

- Workspace repos MUST be private — they contain personal context, preferences, and potentially sensitive project knowledge
- ClawDen validates repo visibility before first sync and warns if public
- Git auth reuses existing `GITHUB_TOKEN` or SSH key config from ClawDen's credential store
- `.gitignore` excludes runtime internals (`.openclaw/`, credentials, temp files)

## Plan

- [ ] **Phase 1: Sync Engine** — Implement git-based workspace sync as a Rust module in `clawden-core`. Support manual `clawden workspace sync` command.
- [ ] **Phase 2: Restore Engine** — Implement workspace restore on `clawden up` when workspace is empty. Clone from configured repo, verify integrity.
- [ ] **Phase 3: Auto-Sync** — Background sync task that runs on a configurable interval during `clawden up`. Integrate with process supervisor from spec 050.
- [ ] **Phase 4: Config & CLI** — Add `workspace:` section to `clawden.yaml` schema. Add `clawden workspace status/history/sync/restore` subcommands.
- [ ] **Phase 5: Multi-Agent Layout** — Support path-prefixed multi-agent repos. Shared context directory for cross-agent knowledge.

## Test

- [ ] Sync engine commits and pushes workspace changes to a test repo
- [ ] Restore engine clones workspace into empty directory and verifies file integrity
- [ ] Kill agent workspace dir, run `clawden up`, verify workspace is auto-restored from repo
- [ ] Two agents in same repo with different paths don't interfere with each other
- [ ] Conflict scenario: modify workspace on two hosts, verify rebase resolves cleanly
- [ ] Public repo detection: `clawden up` warns if workspace repo is not private

## Notes

### Real-World Validation

This spec was born from a live problem: an OpenClaw agent running on a GitHub Codespace had no persistence story. The manual workaround (agent self-syncing via heartbeat to a private repo) works but is fragile and agent-dependent. ClawDen should own this.

### Alternatives Considered

- **S3/GCS blob storage**: Loses versioning, diffability, and free hosting. Git is better for text-heavy workspaces.
- **SQLite in spec 050**: That's for fleet orchestration state (agents, tasks, routing). Workspace memory is conceptually different — it's the agent's own cognitive state, not ClawDen's operational state.
- **Runtime-native solutions**: Some runtimes may have their own persistence (e.g., OpenClaw's memory system). ClawDen's approach is runtime-agnostic and works as a safety net regardless.

### Open Questions

- Should ClawDen support non-Git backends (S3, local rsync) as plugins? Start with Git only, add later if needed.
- Should there be a `clawden workspace diff` that shows what changed since last sync? Useful for debugging agent memory drift.
- Memory pruning: should ClawDen help agents trim old daily logs? Or leave that to the agent's own housekeeping?
