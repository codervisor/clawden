---
name: runtime-sync
description: >
  Enforce consistency across ClawDen runtime adapter implementations and guide adding
  new runtimes to the claw ecosystem. Use when: (1) Adding a new runtime adapter
  (e.g., OpenFang, IronClaw, NullClaw, MimiClaw), (2) Modifying any existing adapter
  in crates/clawden-adapters/src/, (3) Auditing adapters for consistency issues,
  (4) Fixing inconsistencies across runtime integrations, (5) Working with any file
  matching *claw*.rs in the adapters crate, (6) Updating docker/Dockerfile or
  docker/entrypoint.sh for runtime support, (7) Touching dashboard RuntimeCatalog
  component for runtime display.
---

# Runtime Sync

Enforce consistency across ClawDen's `ClawAdapter` implementations. All runtime adapters
follow an identical structure — only metadata values (language, channels, ports) differ.

## Decision Tree

- **Adding a new runtime?** → First run the [upstream research workflow](references/upstream-sources.md#research-workflow)
  to gather accurate metadata, then follow [full-stack-checklist.md](references/full-stack-checklist.md).
  Most metadata goes in a single `RuntimeDescriptor` entry — only add a full adapter if the
  runtime needs lifecycle ops beyond what the descriptor provides.
- **Updating an existing runtime's metadata?** → Edit its entry in
  `crates/clawden-core/src/runtime_descriptor.rs`. No other Rust files need changes for
  metadata-only updates (install source, health port, config format, cost tier, etc.).
- **Updating an existing adapter?** → Check [upstream-sources.md](references/upstream-sources.md)
  for the runtime's repo/npm, review recent releases for changes, then update metadata
- **Modifying adapter behavior?** → Read [consistency-rules.md](references/consistency-rules.md)
  first, fix any known violations you encounter
- **Auditing all adapters?** → Follow the audit procedure in [consistency-rules.md](references/consistency-rules.md)

## Upstream Research (Do This First)

Before writing any adapter code, research the runtime's upstream repo/registry to get
accurate metadata. The [upstream-sources.md](references/upstream-sources.md) reference documents:

- **Source registry** — GitHub repo URLs, npm packages, and git repos for all 9 runtimes
- **5-step research workflow** — README → CHANGELOG → channel details → install method → document
- **Quick commands** — curl/npm/git one-liners to check latest versions and READMEs
- **Install logic cross-reference** — how each runtime maps to `install.rs` functions

Key repos:
| Runtime | Upstream |
|---------|----------|
| ZeroClaw | `zeroclaw-labs/zeroclaw` (GitHub releases) |
| PicoClaw | `picoclaw-labs/picoclaw` (GitHub releases) |
| OpenClaw | `openclaw` npm package |
| NanoClaw | `qwibitai/nanoclaw` (git clone) |

## Architecture Quick Reference

All per-runtime metadata is consolidated in `RuntimeDescriptor` structs in
`crates/clawden-core/src/runtime_descriptor.rs`. This is the **single source of truth**
for installation, config, health checks, CLI args, and cost tiers. Subsystems like
`install.rs`, `process.rs`, `config_gen.rs`, and `manager.rs` all consume descriptors
instead of hardcoding match statements per runtime.

Adapters live in `crates/clawden-adapters/src/` and implement the `ClawAdapter` trait
for lifecycle operations (start, stop, health, send, etc.). They are separate from
descriptors — a runtime can have a descriptor without a full adapter (e.g., stub runtimes).

**Currently implemented (adapter + descriptor):** ZeroClaw (Rust), OpenClaw (TypeScript), PicoClaw (Go), NanoClaw (TypeScript), OpenFang (Rust)

**Descriptor-only (stub, no adapter):** IronClaw, NullClaw, MicroClaw, MimiClaw

**Files that reference runtimes:**

| Layer | File | What to update |
|-------|------|---------------|
| Core enum | `crates/clawden-core/src/lib.rs` | `ClawRuntime` enum variant |
| Descriptor | `crates/clawden-core/src/runtime_descriptor.rs` | Add entry to `DESCRIPTORS` array |
| Adapter | `crates/clawden-adapters/src/{slug}.rs` | New module implementing `ClawAdapter` |
| Features | `crates/clawden-adapters/Cargo.toml` | Feature flag + default list |
| Registry | `crates/clawden-adapters/src/lib.rs` | mod, pub use, builtin_registry() |
| Docker | `docker/Dockerfile` | Version ARG + install command |
| Entrypoint | `docker/entrypoint.sh` | Runtime case statement |
| Dashboard | `dashboard/src/components/runtimes/RuntimeCatalog.tsx` | Language colors (only if new language) |

**Files that are descriptor-driven (no per-runtime edits needed):**

| File | Descriptor fields consumed |
|------|---------------------------|
| `crates/clawden-core/src/install.rs` | `install_source`, `version_source`, `default_start_args`, `subcommand_hints`, `supports_config_dir`, `direct_install_supported` |
| `crates/clawden-core/src/process.rs` | `health_port` (via `health_url()`) |
| `crates/clawden-cli/src/commands/config_gen.rs` | `config_format`, `config_dir_flag`, `has_onboard_command` |
| `crates/clawden-core/src/manager.rs` | `cost_tier` |

## Critical Consistency Rules

1. **`send()` uses echo pattern** — `Ok(AgentResponse { content: format!("{Name} echo: ...") })`, never `bail!()`
2. **`get_config()` fallback includes runtime key** — `json!({ "runtime": "{slug}" })`, never empty `{}`
3. **Every adapter has tests** — `start_persists_forwarded_runtime_config` test
4. **No `bail!` import** — only `anyhow::Result`
5. **Identical method bodies** — lifecycle, health, metrics, subscribe, skills are copy-paste with variant substitution

See [consistency-rules.md](references/consistency-rules.md) for the complete rule set and known violations.

## References

- **[upstream-sources.md](references/upstream-sources.md)** — Upstream repo URLs for all runtimes, 5-step research workflow, quick CLI commands, and install logic cross-reference. **Read this first** before any adapter work.
- **[adapter-template.md](references/adapter-template.md)** — Canonical Rust adapter with every method annotated. Use as copy-paste source for new adapters.
- **[full-stack-checklist.md](references/full-stack-checklist.md)** — Step-by-step checklist covering core enum → adapter → Cargo features → registry → Docker → dashboard → verification.
- **[consistency-rules.md](references/consistency-rules.md)** — Hard rules, known violations in existing adapters, and audit procedure.
