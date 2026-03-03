---
status: draft
created: 2026-03-03
priority: high
tags:
- direct-mode
- config
- channels
- bug
- cli
depends_on:
- 013-config-management
- 029-docker-mode-config-injection
created_at: 2026-03-03T03:09:07.295096Z
updated_at: 2026-03-03T03:09:07.295096Z
---
# Direct Mode Config Injection — Config-Dir Translation

## Overview

When `clawden up` runs in direct mode, it passes channel credentials and provider config as env vars to the spawned runtime process. However, runtimes like zeroclaw prioritize their own config file (`~/.zeroclaw/config.toml`) over env vars — so a stale `bot_token` in zeroclaw's config silently overrides the correct value from `clawden.yaml` + `.env`.

The fix: generate a translated runtime config file from `clawden.yaml` into a per-project config directory under `~/.clawden/configs/<project_hash>/<runtime>/`, then pass `--config-dir <path>` to the runtime. This makes clawden the single source of truth when launching via `clawden up`.

## Context

### Root Cause

- `clawden up` correctly resolves `$ENV_VAR` refs and passes `TELEGRAM_BOT_TOKEN` / `ZEROCLAW_TELEGRAM_BOT_TOKEN` as env vars to the spawned process
- zeroclaw loads `~/.zeroclaw/config.toml` at startup, which may contain a stale `[channels_config.telegram].bot_token`
- zeroclaw's config.toml takes precedence over env vars for channel credentials
- **Result**: user sees `401 Unauthorized` on a valid token because zeroclaw reads the wrong value

### Why `--config-dir` Solves This

Three runtimes (zeroclaw, picoclaw, nullclaw) already support `--config-dir` — listed in `runtime_supported_extra_args()`. This flag tells the runtime to read config from a custom directory instead of the default (`~/.zeroclaw/`, etc.). By generating a translated config file there, clawden controls exactly what the runtime sees.

### Runtime Config Format Map

| Runtime | Language | Config Format | Config File | `--config-dir` | Status |
|---------|----------|--------------|-------------|----------------|--------|
| zeroclaw | Rust | TOML | `~/.zeroclaw/config.toml` | ✅ Yes | Phase 1 |
| picoclaw | Go | JSON | config dir-based | ✅ Yes | Phase 1 |
| nullclaw | — | TOML | config dir-based | ✅ Yes | Phase 1 |
| openclaw | TypeScript | JSON5 | env vars only | ❌ No | Env-only |
| nanoclaw | TypeScript | Code/inline | env vars only | ❌ No | Env-only |
| ironclaw | — | WASM caps | — | ❌ No | Phase 2 |
| microclaw | — | YAML-like | — | ❌ No | Phase 2 |
| openfang | Rust | TOML | — | ❌ Disabled | Disabled |

### clawden.yaml → Runtime Config Field Mapping

#### ZeroClaw (TOML)

| clawden.yaml | config.toml field | Notes |
|---|---|---|
| `provider` | `default_provider` | e.g. "openrouter" |
| `model` | `default_model` | e.g. "anthropic/claude-sonnet-4-6" |
| `providers.<name>.api_key` | `reliability.api_keys` | Array of `{provider, key}` |
| `channels.telegram.token` | `[channels_config.telegram].bot_token` | |
| `channels.telegram.allowed_users` | `[channels_config.telegram].allowed_users` | |
| `channels.discord.token` | `[channels_config.discord].bot_token` | |
| `channels.discord.guild` | `[channels_config.discord].guild_id` | |
| `channels.slack.bot_token` | `[channels_config.slack].bot_token` | |
| `channels.slack.app_token` | `[channels_config.slack].app_token` | |
| `channels.signal.phone` | `[channels_config.signal].phone` | |
| `channels.signal.token` | `[channels_config.signal].token` | |
| `config.*` | Merged as-is into TOML root | Catch-all for runtime-specific overrides |

#### PicoClaw (JSON)

| clawden.yaml | JSON field | Notes |
|---|---|---|
| `provider` | `llm.provider` | PicoClaw uses "llm" not "model" |
| `model` | `llm.model` | |
| `providers.<name>.api_key` | `llm.apiKeyRef` | |
| `channels.<name>` | Per-channel JSON objects | Via `picoclaw_channel_config()` |
| `config.*` | Merged into root | |

#### OpenClaw (JSON5) — env-only for now

No `--config-dir` support. Relies entirely on env vars + Docker `-e` flags. The `openclaw_channel_config()` function produces JSON channel objects for Docker mode config store but not for direct-mode file generation.

#### NanoClaw — env-only

No `--config-dir` support. Uses `NANOCLAW_*` prefixed env vars via `nanoclaw_env_vars()`.

### Related Specs

- **029-docker-mode-config-injection**: Fixed the same gap for Docker mode (via `-e` flags)
- **013-config-management**: Defines `RuntimeConfigTranslator` traits and canonical schema

## Design

### 1. Add `toml` Crate

Add `toml = "0.8"` to workspace dependencies and `clawden-cli`'s Cargo.toml.

### 2. Config Directory Layout

```
~/.clawden/configs/<project_hash>/
  └── <runtime>/
      └── config.toml     # (or config.json for picoclaw)
```

The `project_hash` isolates configs per-project directory (already used for pid-file scoping). The runtime subdirectory is what gets passed to `--config-dir`.

### 3. Config Generation Module

New file `crates/clawden-cli/src/commands/config_gen.rs` with per-runtime generators:

```rust
/// Generate a runtime config directory from ClawDenYaml.
/// Returns the path to pass as --config-dir, or None if the runtime
/// doesn't support config-dir injection.
pub fn generate_config_dir(
    config: &ClawDenYaml,
    runtime: &str,
    project_hash: &str,
) -> Result<Option<PathBuf>>
```

Dispatches to:
- `generate_zeroclaw_config()` → writes `config.toml`
- `generate_picoclaw_config()` → writes `config.json`
- `generate_nullclaw_config()` → writes `config.toml`
- Returns `None` for openclaw/nanoclaw (no `--config-dir` support)

### 4. ZeroClaw TOML Generation

Builds a `toml::Value::Table` with:

```toml
default_provider = "openrouter"
default_model = "anthropic/claude-sonnet-4-6"

[channels_config]
cli = true

[channels_config.telegram]
bot_token = "<resolved-token>"
allowed_users = ["@user1"]

# ... any config overrides from clawden.yaml config field
```

Uses `toml::to_string_pretty()` to serialize.

### 5. Inject `--config-dir` into start args

In the `ExecutionMode::Direct` branch of `exec_up()`:

```rust
let config_dir = if let Some(cfg) = config.as_ref() {
    generate_config_dir(cfg, &runtime, &project_hash()?)?
} else {
    None
};

let mut args = installed.start_args.clone();
if let Some(dir) = &config_dir {
    args.push("--config-dir".to_string());
    args.push(dir.to_string_lossy().to_string());
}
```

### 6. Keep Env Vars as Supplementary

Continue passing env vars (via `build_runtime_env_vars`) since:
- They provide `CLAWDEN_CHANNELS`, `CLAWDEN_TOOLS` (runtime-agnostic)
- They serve as fallback for fields not in config files
- Docker mode still depends on them exclusively
- Runtimes without `--config-dir` (openclaw, nanoclaw) depend on them exclusively

### 7. Cleanup on `clawden down`

Remove `~/.clawden/configs/<project_hash>/` when `clawden down` is run.

## Plan

- [ ] Add `toml = "0.8"` to workspace deps and `clawden-cli` Cargo.toml
- [ ] Create `config_gen.rs` module with `generate_config_dir()` dispatcher
- [ ] Implement `generate_zeroclaw_config()` — TOML generation
- [ ] Implement `generate_picoclaw_config()` — JSON generation (stretch)
- [ ] Create config dir at `~/.clawden/configs/<project_hash>/<runtime>/`
- [ ] Inject `--config-dir` into start args in `exec_up()` direct-mode branch
- [ ] Apply the same pattern for `exec_run()` direct-mode branch
- [ ] Clean up config dirs on `clawden down`
- [ ] Add test: generated TOML matches expected zeroclaw config.toml format
- [ ] Add test: `--config-dir` arg is injected for supported runtimes only

## Test

- [ ] `clawden up` with telegram channel in clawden.yaml → generated config.toml contains correct `[channels_config.telegram].bot_token`
- [ ] `clawden up` with openrouter provider → generated config.toml has `default_provider = "openrouter"` and API key
- [ ] Stale `~/.zeroclaw/config.toml` does NOT interfere when `--config-dir` is used
- [ ] `clawden down` removes the generated config directory
- [ ] Runtimes without `--config-dir` (openclaw, nanoclaw) still work via env vars only
- [ ] `config` overrides from clawden.yaml are merged into the generated config file
- [ ] Works alongside env var passthrough (no regression for Docker mode)