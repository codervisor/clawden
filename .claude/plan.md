# ClawDen Pivot: AI Infra Layer — Lean & Focused

## The Thesis

ClawDen becomes the **orchestration layer between AI coding workers and human teams**. Claw runtimes (OpenClaw, ZeroClaw, etc.) serve as the communication bridge — not as the primary product, but as pluggable comm adapters. The primary value is: **one place to manage AI workers + report to humans**.

## What's IN (Keep + Evolve)

### Core (already built, stays as-is)
- **CLI-Direct architecture** — no mandatory server, `clawden run` just works
- **Runtime Descriptor / CRI adapter pattern** — extend this to coding tools
- **Config translation pipeline** — `clawden.yaml` → native format (proven, works well)
- **Process management** — start/stop/logs/health (ProcessManager, LogStream)
- **Provider/key management** — multi-provider API key vault
- **Channel registry** — channel types + credential descriptors (metadata only)
- **Guided onboarding** — `clawden init` interactive flow

### New (the pivot additions)
- **Worker Interface (Tool CRI)** — new adapter trait for coding tools:
  - `claude-code` adapter (launch `claude -p`, capture output)
  - `codex` adapter (launch `codex`, capture output)
  - More adapters over time (copilot, gemini, opencode)
- **Comm routing** — claw runtime receives human message → dispatches to worker → reports back
  - This is the killer feature. Human says "fix the login bug" in Telegram → ClawDen routes to Claude Code → reports result back to Telegram
- **Status/reporting hooks** — "when task completes, notify Slack channel"

## What's OUT (Trim)

### 1. Swarm Coordinator (`swarm.rs`) — **ARCHIVE**
- Leader/Worker/Reviewer roles, fan-out task distribution
- Over-engineered for the new positioning. AI coding tools already have their own sub-agent/multi-agent capabilities (Claude Code spawns agents, Codex has its own orchestration)
- ClawDen should be a thin dispatch layer, not reimplment multi-agent coordination
- **Keep**: simple task routing (send task to best available worker). **Remove**: SwarmTeam, SwarmRole, SwarmTask, SwarmCoordinator

### 2. Fleet Orchestration (`discovery.rs`, spec 012) — **SIMPLIFY**
- Full fleet discovery with mDNS/consul-style service mesh is overkill
- For the infra layer: just track which workers are running locally and their status
- **Keep**: basic process registry. **Remove**: DiscoveryService, DiscoveredEndpoint, DiscoveryMethod

### 3. LifecycleManager multi-agent complexity (`manager.rs`) — **SIMPLIFY**
- Round-robin task assignment, consecutive failure tracking, recovery attempts — this is Kubernetes-level orchestration
- The new model: each worker is a named process. Start it, check health, stop it. That's it.
- **Keep**: start/stop/health per worker. **Remove**: round-robin routing, multi-agent registration complexity

### 4. Channel binding/draining system (`channels.rs`) — **SIMPLIFY**
- ChannelStore with Active/Draining/Released binding states, MatrixRow, credential health checks
- This assumes ClawDen directly manages channel connections. In the new model, the claw runtime handles that — ClawDen just starts the claw runtime with the right config.
- **Keep**: ChannelType enum, ChannelInstanceConfig. **Remove**: ChannelStore, ChannelBinding, ChannelBindingStatus, ChannelConnectionStatus, ChannelHealthEntry

### 5. Skill system (`Skill`, `SkillManifest`, `list_skills`, `install_skill`) — **ARCHIVE**
- Cross-runtime skill SDK was the "SDK Platform" pillar
- In the new model, "skills" are just MCP servers or tool configurations injected into the AI coding worker — the worker ecosystem handles this natively
- The `@clawden/sdk` TypeScript package and `clawden skill create` workflow become dead weight
- **Keep**: nothing. **Remove**: Skill, SkillManifest, SDK package, skill-related ClawAdapter methods

### 6. Dashboard (`dashboard/`) — **DEFER**
- React 19 + Vite dashboard is nice-to-have but not core to the infra layer
- Focus on CLI-first, add dashboard later when there's something worth visualizing
- **Keep**: the code (don't delete). **Defer**: no new dashboard work until CLI is solid

### 7. Docker-heavy runtime management — **SIMPLIFY**
- Docker compose orchestration, image hardening, Dockerfile layering
- Direct-install already won as the primary mode. Docker is a deployment detail, not a core feature.
- **Keep**: docker as one execution mode. **Remove**: docker as a first-class UX concern (no more docker-specific specs)

### 8. Audit log (`audit.rs`) — **DEFER**
- Append-only audit trail is an enterprise feature
- Not needed for the lean infra layer MVP
- **Keep**: the code (small, harmless). **Defer**: no new audit work

## Specs Impact

### Archive these in-progress/planned specs:
- **024-built-in-tool-layer** — Container env & tool management. Replaced by worker adapters.
- **042-openclaw-telegram-config-parity** — Too deep into claw-specific config. Claw runtimes handle their own config.
- **048-telegram-username-id-resolution** — Claw runtime concern, not ClawDen infra concern.
- **039-zeroclaw-security-defaults-compat** — Runtime-specific. Let the runtimes handle their own security.
- **047-cli-security-profile-flag** — Only makes sense if ClawDen deeply manages runtime security.
- **059-comprehensive-documentation** — Premature. Write docs after the pivot lands.

### Keep these in-progress specs (still relevant):
- **018-channel-support-matrix** — Channels are the comm fabric. Still needed.
- **028-runtime-pull-update** — Still need to install/update claw runtimes.
- **030-log-lifecycle-hygiene** — Log management stays relevant for workers too.
- **038-proxy-auto-detection** — Network infra, always useful.
- **040-runtime-smoke-test-matrix** — Testing still matters.
- **058-container-safe-runtime-execution** — Keep if simplified.

## New Spec Needed: `060-ai-infra-pivot`

### Repositioned Architecture
```
┌─────────────────────────────────────┐
│          Human Stakeholders          │
│   (Telegram, Slack, Email, etc.)     │
└──────────────┬──────────────────────┘
               │
┌──────────────┴──────────────────────┐
│  Comm Layer (Claw Runtimes as CRI)   │
│  OpenClaw · ZeroClaw · PicoClaw      │
│  → receive human messages            │
│  → send status reports               │
└──────────────┬──────────────────────┘
               │
┌──────────────┴──────────────────────┐
│        ClawDen Core (thin)           │
│  • Process registry (who's running)  │
│  • Config translation (yaml→native)  │
│  • Task dispatch (msg → worker)      │
│  • Status hooks (done → notify)      │
└──────────────┬──────────────────────┘
               │
┌──────────────┴──────────────────────┐
│  Worker Layer (Coding Tools as CRI)  │
│  Claude Code · Codex · Copilot       │
│  Gemini CLI · OpenCode               │
│  → do the actual coding work         │
└─────────────────────────────────────┘
```

### ClawAdapter trait → split into two traits
```rust
// Communication adapters (claw runtimes)
trait CommAdapter: Send + Sync {
    fn metadata(&self) -> CommMetadata;
    async fn start(&self, config: &CommConfig) -> Result<ProcessHandle>;
    async fn stop(&self, handle: &ProcessHandle) -> Result<()>;
    async fn health(&self, handle: &ProcessHandle) -> Result<HealthStatus>;
    async fn send_notification(&self, handle: &ProcessHandle, msg: &Notification) -> Result<()>;
}

// Worker adapters (coding tools)
trait WorkerAdapter: Send + Sync {
    fn metadata(&self) -> WorkerMetadata;
    async fn start(&self, config: &WorkerConfig) -> Result<ProcessHandle>;
    async fn stop(&self, handle: &ProcessHandle) -> Result<()>;
    async fn health(&self, handle: &ProcessHandle) -> Result<HealthStatus>;
    async fn dispatch(&self, handle: &ProcessHandle, task: &Task) -> Result<TaskResult>;
    async fn stream_output(&self, handle: &ProcessHandle) -> Result<LogStream>;
}
```

### MVP scope (what to build first)
1. `WorkerAdapter` trait + Claude Code adapter (launch `claude -p "task"`, capture JSON output)
2. Simple task dispatch: receive task description → pick worker → run → return result
3. Notification hook: on task complete → send message via claw runtime channel
4. `clawden worker add claude-code` / `clawden worker run claude-code "fix the bug"`

### Repositioned value prop
> **ClawDen** — AI workforce infrastructure. Manage coding AI tools (Claude Code, Codex, Copilot) and connect them to your team's communication channels. One config, any worker, any channel.

## Implementation Steps

1. **Create spec 060-ai-infra-pivot** — captures the new positioning and architecture
2. **Archive 6 specs** listed above
3. **Refactor ClawAdapter** → split into CommAdapter + WorkerAdapter
4. **Slim down clawden-core** — remove swarm, simplify manager, simplify channels
5. **Add clawden-workers crate** — WorkerAdapter trait + Claude Code adapter
6. **Add `clawden worker` CLI subcommand** — add/run/stop/status for coding tools
7. **Wire notification hooks** — task complete → comm adapter → channel message
