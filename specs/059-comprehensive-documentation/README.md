---
status: planned
created: 2026-03-07
priority: high
tags:
- documentation
- docs
- user-guide
- cli-reference
- onboarding
- channels
- config
- sdk
created_at: 2026-03-07T15:11:55.678101Z
updated_at: 2026-03-07T15:11:55.678101Z
---

# Comprehensive Documentation — User Guides, CLI Reference, Config Schema & Channel Onboarding

## Overview

ClawDen has shipped 50+ specs worth of features — 9 runtimes, 6 validated channels, 6 LLM providers, Docker/direct/npm deployment modes, a React dashboard, a skill SDK, workspace persistence, and a 20+ subcommand CLI — but the entire user-facing documentation is a single README.md. Users cannot discover `clawden.yaml` fields without reading Rust source, have no channel setup guides, no runtime comparison, no deployment depth beyond brief README examples, and no contributor guidance.

This spec establishes a `docs/` site with structured user documentation covering all shipped features.

## Problem

### No discoverability beyond README

The README.md covers quick-start examples for the most common paths (direct install, Docker, Docker Compose) but:
- **`clawden.yaml` is undocumented** — the config file is the primary interface for multi-runtime orchestration, yet users have no reference for available fields, channel config structure, provider definitions, tool lists, or workspace persistence settings
- **CLI flags are only in `--help`** — 20+ subcommands with dozens of flags; no searchable reference
- **Env vars are scattered** — 30+ environment variables across entrypoint.sh, .env.example, and code; no consolidated reference
- **Channel setup is opaque** — Telegram, Discord, Slack, Feishu, Signal, and WhatsApp each have unique onboarding steps, credential formats, and gotchas; Feishu has a dedicated `clawden channels feishu setup` wizard but zero docs
- **Runtime selection is guesswork** — 9 runtimes with different cost tiers (1-3), config formats (TOML/JSON/EnvVars), capabilities, and install methods; no comparison guide

### No contributor onboarding

- No CONTRIBUTING.md — blocks external contributions
- No ARCHITECTURE.md for humans — AGENTS.md is agent-facing
- No CHANGELOG tracking breaking changes

### SDK is type-exports-only

The `@clawden/sdk` package exports `defineSkill()`, `testSkill()`, and marketplace types but has zero usage documentation or tutorials.

## Design

### Documentation structure

```
docs/
├── README.md                    # Docs index / navigation
├── getting-started/
│   ├── installation.md          # npm, direct install, Docker
│   ├── quickstart.md            # First runtime in 5 minutes
│   └── configuration.md         # clawden.yaml deep dive
├── guides/
│   ├── runtimes.md              # Runtime comparison & selection
│   ├── channels/
│   │   ├── telegram.md          # Telegram bot setup end-to-end
│   │   ├── discord.md           # Discord bot setup
│   │   ├── slack.md             # Slack app setup
│   │   ├── feishu.md            # Feishu/Lark app setup
│   │   ├── whatsapp.md          # WhatsApp setup
│   │   └── signal.md            # Signal setup
│   ├── providers.md             # LLM provider configuration
│   ├── deployment/
│   │   ├── docker.md            # Docker & Docker Compose deployment
│   │   ├── direct.md            # Direct install deployment
│   │   └── railway.md           # Railway / cloud container platforms
│   ├── multi-runtime.md         # Running multiple runtimes with clawden up
│   ├── tools.md                 # Built-in tools configuration
│   ├── workspace-persistence.md # Git-backed workspace sync
│   └── troubleshooting.md       # Common issues & doctor command
├── reference/
│   ├── cli.md                   # Full CLI command reference (auto-generated from clap)
│   ├── config.md                # clawden.yaml schema reference
│   ├── env-vars.md              # Environment variable reference
│   └── api.md                   # REST & WebSocket API reference
├── sdk/
│   ├── getting-started.md       # SDK installation & first skill
│   ├── skill-authoring.md       # defineSkill() guide
│   └── marketplace.md           # Publishing skills
├── contributing/
│   ├── CONTRIBUTING.md          # How to contribute (also symlinked to repo root)
│   ├── architecture.md          # Human-readable architecture overview
│   └── development.md           # Dev setup, testing, PR workflow
└── CHANGELOG.md                 # Also symlinked to repo root
```

### Content strategy

1. **Getting Started** — zero-to-running in under 5 minutes, covering the three deployment modes (npm/direct, Docker, Docker Compose). Migrate and expand the current README quick-start into proper docs.

2. **Channel guides** — each channel gets a dedicated page with:
   - Prerequisites (bot token creation, app registration)
   - `clawden.yaml` configuration example
   - `clawden run` one-liner example
   - Docker env var example
   - Verification steps (`clawden channels test`, `clawden channels feishu verify`)
   - Common errors and fixes

3. **Runtime comparison** — table with cost tier, config format, supported channels, install method, Docker image availability, and recommended use cases. Pull data from `RuntimeDescriptor` and `DESCRIPTORS` array.

4. **CLI reference** — auto-generated from clap's `--help` output or parsed from cli.rs definitions. One entry per command/subcommand with flags, defaults, and examples.

5. **Config reference** — document every `clawden.yaml` field from the `ClawDenYaml`, `RuntimeInstanceConfig`, `ChannelConfig`, `ProviderConfig`, and `WorkspaceConfig` structs. Include type, default, and example for each.

6. **Env var reference** — consolidate all `CLAWDEN_*`, provider, and channel env vars with descriptions, defaults, and which deployment mode they apply to.

7. **SDK guide** — tutorial-style docs for `defineSkill()` and `testSkill()`, plus marketplace publishing flow.

8. **Contributing** — dev setup (Rust + pnpm), testing commands, PR checklist, architecture overview.

### Approach

- **Docs live in `docs/`** — plain Markdown, no static site generator initially (can add mdBook/Docusaurus later)
- **README.md gets trimmed** — keep a concise overview + link to docs/ for details
- **Auto-generation where possible** — CLI reference from clap, config reference from struct definitions, env var reference from grep
- **CONTRIBUTING.md and CHANGELOG.md** symlinked from repo root to docs/contributing/

### Priority order

Phase 1 (Critical — unblocks users):
- `docs/getting-started/` (installation, quickstart, configuration)
- `docs/reference/cli.md` (full CLI reference)
- `docs/reference/config.md` (clawden.yaml schema)
- `docs/reference/env-vars.md` (env var reference)
- `docs/guides/runtimes.md` (runtime comparison)

Phase 2 (High — channel onboarding):
- All 6 channel guides (`docs/guides/channels/`)
- `docs/guides/providers.md`
- `docs/guides/deployment/` (Docker, direct, Railway)

Phase 3 (Medium — contributor & SDK):
- `docs/contributing/` (CONTRIBUTING.md, architecture, development)
- `docs/sdk/` (skill authoring guide)
- `docs/CHANGELOG.md`
- `docs/guides/troubleshooting.md`

Phase 4 (Low — completeness):
- `docs/guides/multi-runtime.md`
- `docs/guides/tools.md`
- `docs/guides/workspace-persistence.md`
- `docs/reference/api.md` (REST/WS API)

## Plan

- [ ] Create `docs/` directory structure and index README
- [ ] Write `docs/getting-started/installation.md` — npm, direct, Docker install paths
- [ ] Write `docs/getting-started/quickstart.md` — first runtime in 5 minutes
- [ ] Write `docs/getting-started/configuration.md` — clawden.yaml walkthrough
- [ ] Write `docs/reference/cli.md` — full CLI command reference
- [ ] Write `docs/reference/config.md` — clawden.yaml schema reference
- [ ] Write `docs/reference/env-vars.md` — environment variable reference
- [ ] Write `docs/guides/runtimes.md` — runtime comparison and selection
- [ ] Write `docs/guides/channels/telegram.md` — Telegram setup guide
- [ ] Write `docs/guides/channels/discord.md` — Discord setup guide
- [ ] Write `docs/guides/channels/slack.md` — Slack setup guide
- [ ] Write `docs/guides/channels/feishu.md` — Feishu/Lark setup guide
- [ ] Write `docs/guides/channels/whatsapp.md` and `signal.md`
- [ ] Write `docs/guides/providers.md` — LLM provider configuration
- [ ] Write `docs/guides/deployment/docker.md` — Docker deep dive
- [ ] Write `docs/guides/deployment/direct.md` — direct install guide
- [ ] Write `docs/guides/deployment/railway.md` — cloud container deployment
- [ ] Write `docs/contributing/CONTRIBUTING.md` and symlink to repo root
- [ ] Write `docs/contributing/architecture.md` — human-readable architecture
- [ ] Write `docs/contributing/development.md` — dev setup and testing
- [ ] Write `docs/sdk/getting-started.md` and `skill-authoring.md`
- [ ] Write `docs/guides/troubleshooting.md`
- [ ] Write `docs/guides/multi-runtime.md`, `tools.md`, `workspace-persistence.md`
- [ ] Trim README.md to concise overview with links to docs/
- [ ] Create `docs/CHANGELOG.md` with version history

## Test

- [ ] All docs pages render valid Markdown (no broken links, headings, code blocks)
- [ ] CLI reference covers all 20+ subcommands with correct flags
- [ ] Config reference covers all `ClawDenYaml` fields
- [ ] Env var reference includes all `CLAWDEN_*` variables from codebase
- [ ] Each channel guide includes yaml config + CLI one-liner + Docker env examples
- [ ] Runtime comparison table matches current `DESCRIPTORS` array data
- [ ] Getting started quickstart works end-to-end on a fresh machine
- [ ] CONTRIBUTING.md includes verified build/test commands
- [ ] No stale references to removed features or old command syntax

## Notes

This is the first dedicated documentation effort. Previously all user-facing docs lived in README.md.

Data sources for auto-generation:
- CLI reference: `crates/clawden-cli/src/cli.rs` (clap definitions)
- Config schema: `crates/clawden-config/src/lib.rs` (ClawDenYaml struct)
- Runtime descriptors: `crates/clawden-core/src/runtime_descriptor.rs` (DESCRIPTORS array)
- Channel descriptors: `crates/clawden-core/src/channel.rs` (CHANNEL_DESCRIPTORS)
- Provider descriptors: `crates/clawden-core/src/provider.rs` (PROVIDERS)
- Env vars: grep for `CLAWDEN_` across codebase

Related specs:
- 026 (guided-onboarding) — first-run UX that docs should reference
- 033 (product-positioning) — framing for docs tone and user personas
- 049 (feishu-channel-onboarding) — Feishu setup wizard to document
- 058 (container-safe-runtime-execution) — new `--exec` flag to document
