---
status: planned
created: 2026-03-09
priority: critical
tags:
- core
- fleet
- auth
- security
- identity
- secrets
depends_on:
- 054-agent-fleet-execution-layer
- 025-llm-provider-api-key-management
- 053-agent-workspace-persistence
created_at: 2026-03-09T03:11:29.260307188Z
updated_at: 2026-03-09T03:11:29.260307188Z
---

# Agent Fleet Identity & Authorization — Secure Multi-Agent Auth Architecture

## Overview

ClawDen's fleet layer (spec 054) handles execution — spawning agents, routing messages, collecting results. But it treats all agents as equally trusted and conflates human credentials with agent credentials. When a human deploys a fleet of AI agents, critical questions are unanswered:

- **Who owns which secrets?** If the coordinator agent has an Anthropic API key and a GitHub PAT, should the worker agents also have access to those?
- **How do agents authenticate to external services?** Each agent might need different scopes — one agent codes (needs repo write), another researches (needs read-only web access).
- **How do humans authenticate to ClawDen itself?** Multiple humans may manage the same fleet with different permission levels.
- **What stops a compromised agent from escalating?** If a worker agent is jailbroken, it shouldn't be able to read the coordinator's secrets or impersonate a human.

This spec defines the identity, authentication, and authorization architecture for ClawDen fleets — treating agents as managed employees with scoped credentials, not as extensions of the human operator.

### The Employee Metaphor

Think of a ClawDen fleet as a small company:

| Concept | Company Analogy | ClawDen Equivalent |
|---|---|---|
| **LLM** | Employee's brain & intelligence | Model provider (OpenAI, Anthropic, etc.) |
| **Memory system** | Employee's notes, knowledge, experience | Workspace persistence (spec 053) — git-backed memory |
| **Claw runtime** | Employee's workstation (laptop, desk, tools) | OpenClaw/ZeroClaw/PicoClaw process + config |
| **Identity** | Employee badge / corporate ID | Agent identity token issued by ClawDen |
| **Credentials** | Keycards, passwords, access badges per system | Scoped secrets vaulted per agent |
| **Role** | Job title & responsibilities | Agent role (leader, worker, reviewer) |
| **Clearance level** | Security clearance tier | Permission scope (what secrets/tools/channels the agent can access) |
| **HR / IT department** | Issues badges, manages access, revokes on termination | ClawDen control plane — the fleet auth manager |

Just as a company doesn't give every employee the CEO's email password, ClawDen shouldn't give every agent the human operator's full credential set.

### Why Now

- Spec 054 is about to wire up real multi-agent execution — agents will actually run side-by-side
- Spec 025 added provider API keys but they're fleet-global — every agent sees every key
- Spec 053 gives agents persistent memory — if an agent is compromised, its memory repo contains sensitive context
- Without auth boundaries, a single compromised agent = full fleet compromise
- This is the difference between "running multiple chatbots" and "operating aligned AI employees"

## Design

### 1. Identity Model

Every entity in ClawDen gets a typed identity:

```rust
pub enum Principal {
    /// Human operator — owns the fleet, manages config
    Human {
        id: String,
        name: String,
        auth_method: HumanAuthMethod,
    },
    /// AI agent — a managed employee in the fleet
    Agent {
        id: String,          // stable across restarts: "coordinator", "coder-1"
        runtime: String,     // "openclaw", "zeroclaw", etc.
        role: AgentRole,
        team: Option<String>,
    },
    /// ClawDen system — internal operations (sync, health checks)
    System,
}

pub enum HumanAuthMethod {
    /// Local CLI — authenticated by OS user (single-user default)
    LocalSession,
    /// Token-based — for remote dashboard / API access
    BearerToken { token_hash: String, scopes: Vec<Scope> },
    /// OAuth — GitHub, Google SSO for team deployments
    OAuth { provider: String, subject: String },
}

pub enum AgentRole {
    Leader,      // can delegate tasks, see team-wide context
    Worker,      // executes assigned tasks within scope
    Reviewer,    // read-only + approval authority
    Specialist,  // worker with elevated access to specific tools/services
}
```

### 2. Scoped Secret Vault

Replace the current flat secret store with a per-principal vault:

```
~/.clawden/vault/
├── fleet.key              # Master key (encrypted with human's passphrase or OS keychain)
├── human/
│   └── default/           # Human operator's secrets
│       ├── github-pat      # Full-scope PAT (human only)
│       └── anthropic-key   # Personal API key
├── agents/
│   ├── coordinator/
│   │   ├── anthropic-key   # Coordinator's own API key (or delegated subset)
│   │   ├── github-pat      # Scoped: repo read + issue write only
│   │   └── telegram-token  # Coordinator owns the Telegram channel
│   ├── coder-1/
│   │   ├── openai-key      # Worker's LLM key
│   │   └── github-pat      # Scoped: repo read + PR create only
│   └── researcher/
│       ├── anthropic-key
│       └── web-search-key  # Only researcher has web search API access
└── shared/                 # Secrets explicitly shared across the fleet
    └── sentry-dsn          # Error reporting — all agents can read
```

**Key principles:**
- **Default deny**: an agent can only access secrets in its own vault directory + `shared/`
- **Human escalation**: agents cannot access `human/` secrets — ever
- **Delegation, not sharing**: when a human wants an agent to have a GitHub PAT, they issue a *scoped* token and store it in the agent's vault — not copy their own
- **Rotation-friendly**: each secret has metadata (created_at, expires_at, source) for auditing

### 3. Permission Scopes

Fine-grained capabilities that bound what an agent can do:

```rust
pub enum Scope {
    // Secret access
    SecretRead(SecretPattern),     // "anthropic-key", "github-*"
    SecretWrite(SecretPattern),    // can store new secrets (e.g., OAuth tokens obtained during work)
    SharedSecretRead,              // access shared/ vault

    // Fleet interaction
    MessageSend(AgentPattern),     // who this agent can message
    MessageReceive(AgentPattern),  // who can message this agent
    TaskDelegate(AgentPattern),    // who this agent can assign tasks to
    TaskView,                      // can see fleet-wide task board

    // External access
    ToolUse(ToolPattern),          // which tools the agent can invoke
    ChannelAccess(ChannelPattern), // which channels the agent can interact with
    NetworkAccess(NetworkPolicy),  // outbound network restrictions

    // System
    ConfigRead,                    // can read clawden.yaml (sanitized)
    ConfigWrite,                   // can modify fleet config (leader only)
    AuditRead,                     // can read audit logs
}
```

### 4. Agent Credential Lifecycle

```
Human deploys fleet
        │
        ▼
┌───────────────────┐
│ clawden up         │
│                    │
│  For each agent:   │
│  1. Generate agent │◄── Agent gets a unique identity token
│     identity token │    (JWT signed by fleet.key)
│  2. Resolve scopes │◄── From role + clawden.yaml overrides
│  3. Mount secrets  │◄── Only agent's vault dir + shared/
│  4. Inject env     │◄── Secrets as env vars into runtime process
│  5. Start process  │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│ Agent runtime      │
│                    │
│ Sees only:         │
│ - Own API keys     │
│ - Own channel tkns │
│ - Shared secrets   │
│ - Identity token   │
│                    │
│ Cannot see:        │
│ - Human's secrets  │
│ - Other agents' keys│
│ - Fleet master key │
└───────────────────┘
```

**Identity tokens** are short-lived JWTs signed by the fleet master key:
- Contains: agent_id, role, scopes, team, issued_at, expires_at
- Refreshed automatically by the process supervisor
- Used for message bus authentication (agents prove identity when sending messages)
- Revoked immediately on agent stop/decommission

### 5. Fleet Configuration Extension

```yaml
# clawden.yaml
fleet:
  auth:
    # How secrets are encrypted at rest
    vault_backend: os-keychain  # or: passphrase, age-encryption, vault-server
    # Default scopes for each role
    role_defaults:
      leader:
        scopes: [secret-read:*, message-send:*, task-delegate:*, config-read, audit-read]
      worker:
        scopes: [secret-read:own, message-send:leader, tool-use:*]
      reviewer:
        scopes: [secret-read:own, message-receive:*, task-view, audit-read]

agents:
  coordinator:
    runtime: openclaw
    role: leader
    # Per-agent scope overrides
    scopes:
      - channel-access:telegram
      - secret-read:shared/*
    secrets:
      anthropic-key: $COORDINATOR_ANTHROPIC_KEY
      github-pat: $COORDINATOR_GITHUB_TOKEN
      telegram-token: $TELEGRAM_BOT_TOKEN

  coder-1:
    runtime: zeroclaw
    role: worker
    scopes:
      - tool-use:git,code,test
      - network-access:github.com,npmjs.com
    secrets:
      openai-key: $CODER_OPENAI_KEY
      github-pat: $CODER_GITHUB_TOKEN  # scoped: repo read + PR create

  researcher:
    runtime: picoclaw
    role: specialist
    scopes:
      - tool-use:web-search,summarize
      - network-access:*              # researcher needs broad web access
    secrets:
      anthropic-key: $RESEARCHER_ANTHROPIC_KEY
      web-search-key: $TAVILY_API_KEY
```

### 6. Human Authentication

For single-user local deployments (the common case), auth is implicit — the OS user running `clawden` CLI is the human principal. For multi-user or remote access:

**Dashboard / API access:**
- `clawden auth login` — generates a session token stored in OS keychain
- `clawden auth token create --scopes admin` — creates a long-lived API token
- Dashboard uses bearer token auth against the ClawDen server
- Tokens are revocable via `clawden auth token revoke <id>`

**Team deployments (future):**
- GitHub OAuth — `clawden auth login --provider github`
- Map GitHub org membership to fleet permissions
- Org admin = fleet admin, org member = read-only dashboard

### 7. Threat Model

| Threat | Mitigation |
|---|---|
| Compromised worker agent reads other agents' API keys | Per-agent vault isolation — process-level env var scoping |
| Jailbroken agent sends messages as another agent | Message bus requires JWT identity verification |
| Agent exfiltrates secrets via LLM conversation | Network access scoping; audit log on secret reads |
| Stolen fleet.key decrypts all secrets | OS keychain or hardware key storage; passphrase-protected fallback |
| Dashboard session hijacking | Short-lived tokens, CSRF protection, SameSite cookies |
| Replay attack on agent identity token | JWT expiry + nonce; supervisor rotates tokens |
| Human's personal PAT leaked via agent | Human vault is never mounted into agent processes |

### 8. Audit Trail

All auth-relevant events are logged to the persistent audit store (spec 054's SQLite):

```rust
pub enum AuthAuditEvent {
    HumanLogin { method: HumanAuthMethod, ip: Option<String> },
    TokenCreated { principal: Principal, scopes: Vec<Scope>, expires_at: u64 },
    TokenRevoked { token_id: String, revoked_by: Principal },
    SecretAccessed { principal: Principal, secret_path: String },
    SecretRotated { secret_path: String, rotated_by: Principal },
    ScopeViolation { principal: Principal, attempted: Scope, denied_reason: String },
    AgentIdentityIssued { agent_id: String, scopes: Vec<Scope> },
    AgentDecommissioned { agent_id: String, secrets_purged: bool },
}
```

Every `ScopeViolation` is logged and optionally alerts the human operator — this is how you know an agent tried to do something outside its role.

## Plan

- [ ] **Phase 1: Identity model** — Define `Principal`, `AgentRole`, `Scope` types in `clawden-core`. Add agent identity to fleet registration.
- [ ] **Phase 2: Scoped vault** — Implement per-agent secret directories. Migrate from flat `SecretVault` to per-principal vault with `fleet.key` encryption.
- [ ] **Phase 3: Process isolation** — Update process supervisor (spec 054) to inject only agent-scoped env vars. Verify no cross-agent secret leakage.
- [ ] **Phase 4: Identity tokens** — JWT issuance and verification for agents. Message bus authenticates senders. Token rotation on configurable interval.
- [ ] **Phase 5: Config schema** — Add `fleet.auth`, `agents.*.scopes`, `agents.*.secrets` to `clawden.yaml` schema. Role-based default scopes.
- [ ] **Phase 6: Human auth** — `clawden auth login/logout/token` commands. Dashboard bearer auth. Session management.
- [ ] **Phase 7: Audit trail** — Auth events in persistent store. `clawden audit` CLI for viewing. Scope violation alerts.
- [ ] **Phase 8: Dashboard integration** — Auth-gated dashboard access. Per-agent secret management UI (create/rotate/revoke). Scope violation feed.

## Test

- [ ] Agent process receives only its own secrets as env vars — not other agents' or human's
- [ ] Agent with `tool-use:git` scope cannot invoke `web-search` tool
- [ ] Message from agent A to agent B is rejected if A lacks `message-send:B` scope
- [ ] Identity token expires and is auto-rotated by supervisor without agent downtime
- [ ] `clawden auth token create` produces a working bearer token for API access
- [ ] Scope violation is logged and visible in `clawden audit`
- [ ] Decommissioned agent's vault is purged and identity token revoked
- [ ] `fleet.key` rotation re-encrypts all vault entries without data loss
- [ ] Multi-human deployment: user A cannot revoke user B's tokens without admin scope

## Notes

### Alternatives Considered

- **HashiCorp Vault integration**: Too heavy for personal/small-team use. Our vault is file-based with OS keychain encryption. Can add Vault backend later via `vault_backend: hashicorp` config.
- **mTLS between agents**: Adds certificate management complexity. JWT over the in-process message bus is simpler for single-host. mTLS makes sense when the bus goes cross-host.
- **RBAC with custom policy engine (OPA/Rego)**: Over-engineered for v1. Static role → scope mapping covers 90% of use cases. Can add policy engine as a Scope variant later.
- **Per-agent OS users**: Strong isolation but impractical for Docker containers and adds operational burden. Process-level env var scoping is sufficient for v1.

### Design Decisions

- **JWT over API keys for agent identity**: API keys are static; JWTs expire and carry embedded scopes. This prevents token reuse after decommission.
- **File-based vault over database**: Secrets are rarely accessed (only at process start), so file I/O is fine. Files are easier to backup, inspect, and version than database rows.
- **Scopes are additive**: An agent starts with its role defaults and can have additional scopes granted. There is no "deny" — you simply don't grant the scope.

### Open Questions

- Should agents be able to request scope escalation at runtime? (e.g., worker needs web access for a specific task → asks leader → leader asks human → temporary scope grant)
- How does this interact with tool-level auth? Some MCP tools carry their own credentials — should ClawDen manage those too or leave them to the tool?
- Should there be a "sandbox" role with no network access and no persistent memory — for running untrusted code?
- Cross-fleet auth: if two fleets need to collaborate, how do agents from fleet A authenticate to fleet B?