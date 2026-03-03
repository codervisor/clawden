---
status: planned
created: 2026-03-03
priority: high
tags:
- positioning
- product
- ux
- strategy
created_at: 2026-03-03T08:49:22.936640Z
updated_at: 2026-03-03T08:49:22.936640Z
---
# ClawDen Product Positioning — UX Shell, Runtime Manager, SDK Platform

## Overview

ClawDen has evolved beyond "orchestration platform" into three distinct, complementary product roles. This spec clarifies ClawDen's identity and establishes positioning language to guide architecture decisions, documentation, and marketing.

### Problem

The current positioning — "unified orchestration platform" / "Kubernetes of claw agents" — is technically accurate but creates two issues:

1. **Over-indexes on infra.** It frames ClawDen as ops tooling for fleet management, when most users are solo developers or hobbyists running 1–2 runtimes locally. The CLI-Direct architecture (023) already acknowledged this by eliminating the mandatory server.
2. **Under-sells the UX/DX value.** ClawDen's biggest value isn't orchestration — it's that a user can do `npx clawden run zeroclaw` and everything just works.

### Unique Selling Point

**ClawDen simplifies the UX/DX for xxxclaw deployment and usage.**

Every claw runtime (OpenClaw, ZeroClaw, PicoClaw, etc.) has its own config format, deployment model, dependency chain, and startup ritual. ClawDen collapses all of that into a single command. The USP is not "orchestration" — it's that ClawDen makes claw runtimes **accessible to anyone**, regardless of infra expertise.

### The `uv run` Model — One Command to Rule Them All

The gold-standard UX is **one command from zero to running**:

```bash
npx clawden run zeroclaw
```

This single command should:
1. **Install ClawDen** — `npx` handles this via the npm package (already works)
2. **Install the runtime** — `ensure_installed_runtime()` auto-installs if missing (already works)
3. **Prompt for credentials** — if no API key is configured, interactively ask for it (new)
4. **Start the runtime** — launch and stream logs (already works)

This is the `uv run` / `bunx` philosophy: resolve all dependencies on the fly, ask for what's needed, never make the user run prerequisite commands.

#### What already works today

| Step | Status | Implementation |
|------|--------|----------------|
| `npx` entry point | Done | `npm/clawden/package.json` bin + postinstall |
| Auto-install runtime | Done | `ensure_installed_runtime()` in `util.rs` — installs on first `run` |
| Version pinning from config | Done | Reads `clawden.yaml` for pinned versions |
| Start + log streaming | Done | `ProcessManager::start_direct_with_env_and_project()` |
| Provider key vault | Done | `providers set-key` stores encrypted keys |

#### What's missing: interactive credential flow during `run`

When a user runs `npx clawden run zeroclaw` with no config and no API key, the experience should be:

```
$ npx clawden run zeroclaw

Runtime 'zeroclaw' not installed. Installing latest...
Installed zeroclaw@0.8.1

No LLM provider API key found.
Which provider? [openai/anthropic/custom]: openai
Enter your OpenAI API key: sk-••••••••
✓ Key validated and saved to vault.

Starting zeroclaw...
```

Key design decisions for the credential flow:
- Only prompt when running interactively (detect `stdin.is_terminal()` — already used in `set_provider_key`)
- In non-interactive/CI mode, fail with clear error: "Missing API key. Set OPENAI_API_KEY or run `clawden providers set-key openai`"
- Validate the key before saving (reuse `test_provider_endpoint()`)
- Store in encrypted vault (reuse `store_provider_key_in_vault()`)
- Remember for subsequent runs — prompt only once ever

### The Three Roles

#### 1. UX Shell (primary)

ClawDen is the **unified command-line and dashboard experience** for the xxxclaw ecosystem. Like how `gh` wraps Git+GitHub into a cohesive workflow, ClawDen wraps heterogeneous claw runtimes behind a single, opinionated interface.

**Analogy:** `uv` / `gh` CLI / Docker Desktop

Key UX surfaces:
- CLI commands: `run`, `up`, `ps`, `stop`, `channels`, `config`
- Guided onboarding: `clawden init` → interactive runtime selection
- Dashboard: real-time monitoring, log streaming, channel management
- Config generation: `clawden config gen` → unified TOML regardless of runtime

What this means for decisions:
- CLI ergonomics and error messages are first-class concerns
- Default behaviors should "just work" for the single-runtime case
- Power-user features (fleet, swarm) are discoverable but not required

#### 2. Runtime Manager (secondary)

ClawDen manages claw runtime **installations, versions, and updates** — exactly like `nvm` manages Node.js versions or `rustup` manages Rust toolchains.

**Analogy:** nvm / rustup / pyenv

Key capabilities:
- `clawden install zeroclaw` — download/install a runtime
- `clawden install zeroclaw@0.5.2` — pin a specific version (planned)
- `clawden install --upgrade` — update installed runtimes (spec 028)
- `clawden install --list` — show installed runtimes
- `clawden install --outdated` — check for available updates
- `clawden uninstall zeroclaw` — remove a runtime
- Auto-install on `run` — like `uv run`, resolves automatically

**Note on naming:** `install` is preferred over `pull` because:
- Matches user mental model — you "install" software, you "pull" images
- Consistent with `npm install`, `brew install`, `apt install`
- `pull` implies Docker/Git semantics that don't apply to direct-install mode

#### 3. SDK Platform (tertiary)

ClawDen provides the **cross-runtime development kit** for building skills/plugins that work across claw variants.

**Analogy:** Terraform Provider SDK / VS Code Extension API

Key capabilities:
- `@clawden/sdk` — TypeScript SDK with `defineSkill()` API
- `clawden skill create` / `clawden skill test` — scaffolding and cross-runtime testing
- Adapter abstraction — skills don't know which runtime they're running on
- (Future) Skill marketplace

### Positioning Statement

> **ClawDen** simplifies xxxclaw deployment and usage. One command to install, configure, and run any claw runtime — plus a cross-runtime SDK for building skills that work everywhere.

### Elevator Pitches by Role

| Role | One-liner |
|------|-----------|
| UX Shell | "`npx clawden run zeroclaw` — zero to running in one command" |
| Runtime Manager | "nvm for claw runtimes — install, switch, and update automatically" |
| SDK Platform | "Build once, run on any claw — cross-runtime skills with TypeScript" |

## Design

### Persona Alignment

| Persona | Primary role used | Entry point |
|---------|-------------------|-------------|
| Hobbyist/student | UX Shell | `npx clawden run zeroclaw` |
| Solo developer | UX Shell + Runtime Manager | `npx clawden init && clawden up` |
| Skill author | SDK Platform | `clawden skill create my-skill` |
| Team/enterprise | All three + fleet features | `clawden dashboard` + fleet orchestration |

### Impact on Architecture

This positioning reinforces several existing architectural decisions:
- **CLI-Direct (023)**: Correct — UX Shell should work without server overhead
- **Guided onboarding (026)**: Correct — first-run experience is critical for UX Shell role
- **Runtime pull/update (028)**: Correct — this is core Runtime Manager functionality
- **SDK package (015, 019)**: Correct — SDK is a distinct distribution concern
- **npm distribution (019)**: Critical — `npx clawden` as zero-install entry point is the USP

Gaps this positioning reveals:
- **Interactive credential prompt during `run`**: The biggest missing piece — auto-prompt for API keys on first run
- **Runtime version pinning**: `clawden install zeroclaw@0.5.2` syntax (planned)
- **Persona-aware docs**: README should lead with `npx clawden run zeroclaw`, not architecture
- **`clawden doctor`**: Already exists — validates local setup before startup

### Documentation & Messaging Guidance

- README should lead with: `npx clawden run zeroclaw` — one command, nothing else needed
- Hero section: "Get a claw agent running in 10 seconds"
- Error messages should suggest next steps, not expose internal state
- `--help` text should use plain language ("Run a claw agent" not "Invoke lifecycle management")

## Plan

- [ ] Add interactive credential prompt to `run` command when API key is missing
- [ ] Update README.md hero section: `npx clawden run zeroclaw`
- [ ] Refine README positioning: "simplifies UX/DX for xxxclaw deployment and usage"
- [ ] Audit CLI `--help` text for plain-language clarity
- [ ] Implement runtime version pinning (`@version` syntax)
- [ ] Review AGENTS.md and npm package description to align with new positioning

## Test

- [ ] `npx clawden run zeroclaw` works from zero with interactive credential prompt
- [ ] Non-interactive mode fails with clear error when API key is missing
- [ ] README communicates value proposition in first 3 lines
- [ ] `clawden --help` output is understandable by someone who has never seen ClawDen
- [ ] Each persona can complete their entry-point workflow in under 60 seconds