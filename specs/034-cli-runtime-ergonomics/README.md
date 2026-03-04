---
status: planned
created: 2026-03-04
priority: high
tags:
- cli
- ux
- ergonomics
- run
- config
- developer-experience
depends_on:
- 033-product-positioning
- 031-direct-mode-config-injection
- 025-llm-provider-api-key-management
created_at: 2026-03-04T01:48:05.129647004Z
updated_at: 2026-03-04T01:50:05.038594321Z
---

# CLI Run-Time Ergonomics — Inline Credentials, Model Override & Config Show

## Overview

`clawden run` and `clawden up` work well when `clawden.yaml` + `.env` are fully configured, but the CLI lacks escape hatches for the most common ad-hoc workflows. Users cannot:

1. **Pass credentials inline** — no way to specify a Telegram bot token, API key, or any credential from the command line without editing `.env` or `clawden.yaml` first
2. **Override model/provider** — no way to quickly test a different model or provider without editing the YAML
3. **Pass arbitrary env vars** — no `-e KEY=VAL` flag like `docker run` offers
4. **See resolved config** — no way to inspect what config/env vars ClawDen will actually pass to a runtime
5. **Set a system prompt inline** — common need for quick testing
6. **Expose/map ports** — no way to forward runtime ports to the host

These gaps force users into a "edit YAML → run → repeat" loop for tasks that should be one-liners, undermining the `uv run`-style transparent execution model (spec 033).

## Context

### What Works Today

| Capability | How | CLI flag |
|---|---|---|
| Channel selection | `--channel telegram` | ✅ |
| Tool selection | `--with git,http` | ✅ |
| Docker bypass | `--no-docker` | ✅ |
| Detach | `-d` / `--detach` | ✅ |
| Cleanup | `--rm` | ✅ |
| Restart policy | `--restart` | ✅ |
| Runtime args passthrough | Trailing args after runtime name | ✅ |

### What's Missing

| Capability | Expected flag | Status |
|---|---|---|
| Inline env var | `-e KEY=VAL` | ❌ Missing |
| Bot/channel token | `--token` | ❌ Missing |
| LLM API key | `--api-key` | ❌ Missing |
| Provider override | `--provider openai` | ❌ Missing |
| Model override | `--model gpt-4o` | ❌ Missing |
| System prompt | `--system-prompt "..."` | ❌ Missing |
| Port mapping | `-p 8080:8080` | ❌ Missing |
| Show resolved config | `clawden config show [runtime]` | ❌ Missing (no `config` command) |
| Verbose/debug output | `--verbose` / `--log-level` | ❌ Missing (global) |
| .env file override | `--env-file path` | ❌ Missing |

### User Stories

**Quick Telegram bot test** (today requires editing .env + YAML):
```sh
# Desired one-liner:
clawden run --token 123:abc --channel telegram zeroclaw
```

**Full zero-config quickstart** (no YAML, no .env, nothing):
```sh
clawden run --api-key sk-... --token 123:abc --channel telegram zeroclaw
```

**Test a different model without touching config**:
```sh
clawden run --model claude-sonnet-4-20250514 --provider anthropic zeroclaw
```

**Debug config translation** (what does zeroclaw actually receive?):
```sh
clawden config show zeroclaw
# Shows resolved TOML, env vars, channels, tools
```

**Quick bot with system prompt**:
```sh
clawden run --channel telegram --system-prompt "You are a helpful coding assistant" zeroclaw
```

### Why This Matters

Spec 033 positions `clawden run` as the "`uv run` for claw agents." But `uv run` lets you override everything inline (`uv run --python 3.12 --with requests script.py`). ClawDen's `run` only controls channels and tools — the most important parameters (credentials, model, provider) require file edits.

## Design

### 1. Inline Environment Variables (`-e`)

Add `-e KEY=VAL` flag to `run` and `up`, matching Docker's convention:

```sh
clawden run -e TELEGRAM_BOT_TOKEN=123:abc -e OPENAI_API_KEY=sk-... zeroclaw
clawden up -e OPENAI_API_KEY=sk-...
```

**Behavior**:
- Parsed as `KEY=VALUE` pairs, injected into the runtime's env alongside `build_runtime_env_vars()` output
- `-e` values take precedence over `.env` and `clawden.yaml` resolved values
- Multiple `-e` flags allowed
- Value-only `KEY` (no `=`) reads from host environment (like `docker run -e KEY`)

**CLI definition** (added to both `Run` and `Up` commands):
```rust
/// Set environment variables (KEY=VAL). Overrides .env and clawden.yaml values.
#[arg(short = 'e', long = "env")]
env_vars: Vec<String>,
```

**Security**: Values passed via `-e` are visible in process arguments on the host. This matches Docker's behavior and is acceptable for local development. The CLI must NOT log `-e` values to audit files — only the key names.

### 2. Bot Token and API Key Shortcuts (`--token`, `--api-key`)

The two most common credentials — the channel bot token and the LLM API key — deserve dedicated flags instead of requiring the verbose `-e KEY=VAL` syntax:

```sh
# Zero-config Telegram bot:
clawden run --token 123:abc --channel telegram zeroclaw

# With explicit API key too:
clawden run --api-key sk-... --token 123:abc --channel telegram --provider openai zeroclaw

# Multiple channels use the same --token for all (or -e for per-channel control):
clawden run --token 123:abc --channel telegram --channel discord zeroclaw
```

**`--token` behavior**:
- Sets the token for the channel(s) specified by `--channel`
- Mapped to the correct env var per channel type: `TELEGRAM_BOT_TOKEN`, `DISCORD_BOT_TOKEN`, etc.
- If multiple `--channel` flags are given with different token requirements, `--token` applies to all of them; use `-e` for per-channel tokens
- If `--channel` is not specified, error: `"--token requires --channel to know which channel the token belongs to"`

**`--api-key` behavior**:
- Sets the LLM provider API key for the current run
- Auto-detects which env var to use based on `--provider` or the configured provider: `--provider openai` → `OPENAI_API_KEY`, `--provider anthropic` → `ANTHROPIC_API_KEY`, etc.
- If no provider can be determined, sets a generic `CLAWDEN_API_KEY` (runtimes fall back to this)
- If `--provider` is given alongside `--api-key`, both resolve together

**CLI definition**:
```rust
/// Bot/channel token (used with --channel)
#[arg(long)]
token: Option<String>,

/// LLM provider API key (auto-maps to provider env var)
#[arg(long)]
api_key: Option<String>,
```

**Security**: Same as `-e` — values visible in process args on the host, acceptable for local dev. Never logged to audit files.

### 3. Model and Provider Override (`--model`, `--provider`)

Add `--model` and `--provider` flags to `run`:

```sh
clawden run --provider anthropic --model claude-sonnet-4-20250514 zeroclaw
clawden run --model gpt-4o zeroclaw  # uses provider from YAML or infers from model
```

**Behavior**:
- `--provider` overrides `provider` in `clawden.yaml` for this run
- `--model` overrides `model` in `clawden.yaml` for this run
- Both are translated to the runtime's expected env vars/config using the existing config translation pipeline
- If `--provider` is used without an API key configured for that provider, error with: `"Provider 'anthropic' has no API key. Set ANTHROPIC_API_KEY in environment or .env, or pass -e ANTHROPIC_API_KEY=..."`

**CLI definition**:
```rust
/// LLM provider override (e.g. openai, anthropic, openrouter)
#[arg(long)]
provider: Option<String>,

/// LLM model override (e.g. gpt-4o, claude-sonnet-4-20250514)
#[arg(long)]
model: Option<String>,
```

### 4. System Prompt (`--system-prompt`)

```sh
clawden run --system-prompt "You are a Python tutor" zeroclaw
```

**Behavior**:
- Injected as `CLAWDEN_SYSTEM_PROMPT` env var
- Runtimes that support system prompts read from this env var
- Also written to the generated config file for config-dir runtimes (zeroclaw: `system_prompt` in TOML, picoclaw: `systemPrompt` in JSON)
- If the value starts with `@`, read from file: `--system-prompt @prompt.txt`

**CLI definition**:
```rust
/// System prompt for the agent (or @file to read from file)
#[arg(long)]
system_prompt: Option<String>,
```

### 5. Env File Override (`--env-file`)

```sh
clawden run --env-file ./staging.env zeroclaw
```

**Behavior**:
- Loads the specified `.env` file instead of (not in addition to) the auto-detected one
- Allows switching between credential sets (dev, staging, production) without renaming files

**CLI definition**:
```rust
/// Path to .env file (overrides auto-detected .env)
#[arg(long)]
env_file: Option<String>,
```

### 6. Port Mapping (`-p`)

```sh
clawden run -p 3000:42617 zeroclaw  # map host:3000 → runtime:42617
```

**Behavior**:
- In Direct mode: sets `CLAWDEN_PORT_MAP` env var (runtime-specific interpretation) and adds port-related args if the runtime supports them
- In Docker mode: passes `-p host:container` to `docker run`
- Common use case: exposing the runtime's HTTP gateway on a known port

**CLI definition**:
```rust
/// Port mapping HOST:CONTAINER (e.g. 3000:42617)
#[arg(short = 'p', long = "port")]
ports: Vec<String>,
```

### 7. Config Show Command (`clawden config show`)

New subcommand to inspect resolved configuration:

```sh
clawden config show              # show all runtimes
clawden config show zeroclaw     # show zeroclaw only
clawden config show --format env # show as env vars instead of native format
```

**Output (default — native format)**:
```
─── zeroclaw (TOML) ───
default_provider = "openrouter"
default_model = "anthropic/claude-sonnet-4-6"

[channels_config.telegram]
bot_token = "***redacted***"

─── Environment Variables ───
CLAWDEN_CHANNELS=telegram
CLAWDEN_TOOLS=git,http
OPENROUTER_API_KEY=***redacted***
```

**Behavior**:
- Loads `clawden.yaml`, resolves env vars, runs config translation, displays the result
- Redacts secrets by default; `--reveal` flag to show actual values (for debugging)
- Shows both the native config file content AND the env vars that would be passed

**CLI definition**:
```rust
#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Show resolved configuration for runtimes
    Show {
        /// Runtime to show (shows all if omitted)
        runtime: Option<String>,
        /// Output format: native, env, yaml, json
        #[arg(long, default_value = "native")]
        format: String,
        /// Show actual secret values instead of redacting
        #[arg(long, default_value_t = false)]
        reveal: bool,
    },
}
```

### 8. Verbose / Log Level (Global)

Add a global `--verbose` flag and `--log-level` option:

```sh
clawden --verbose run zeroclaw           # show debug output
clawden --log-level trace up             # maximum verbosity
```

**Behavior**:
- `--verbose` / `-v`: enables debug-level logging for ClawDen itself (not runtime logs)
- `--log-level`: sets specific log level (error, warn, info, debug, trace)
- Shows config resolution steps, env var injection, health check results, command construction
- Uses `tracing` crate (already a common Rust pattern with Axum)

**CLI definition** (global args in `Cli` struct):
```rust
/// Enable verbose output
#[arg(short = 'v', long, global = true, default_value_t = false)]
pub verbose: bool,

/// Set log level (error, warn, info, debug, trace)
#[arg(long, global = true)]
pub log_level: Option<String>,
```

### Summary of Flag Changes

**`clawden run` — new flags:**

| Flag | Short | Type | Description |
|---|---|---|---|
| `--env` | `-e` | `Vec<String>` | Inline env vars (KEY=VAL) |
| `--token` | | `Option<String>` | Bot/channel token (with --channel) |
| `--api-key` | | `Option<String>` | LLM provider API key |
| `--provider` | | `Option<String>` | LLM provider override |
| `--model` | | `Option<String>` | LLM model override |
| `--system-prompt` | | `Option<String>` | System prompt (or @file) |
| `--env-file` | | `Option<String>` | Alternate .env file path |
| `--port` | `-p` | `Vec<String>` | Port mapping HOST:CONTAINER |

**`clawden up` — new flags:**

| Flag | Short | Type | Description |
|---|---|---|---|
| `--env` | `-e` | `Vec<String>` | Inline env vars |
| `--env-file` | | `Option<String>` | Alternate .env file path |

**Global — new flags:**

| Flag | Short | Type | Description |
|---|---|---|---|
| `--verbose` | `-v` | `bool` | Debug output |
| `--log-level` | | `Option<String>` | Log level control |

**New command:**

| Command | Description |
|---|---|
| `clawden config show [runtime]` | Show resolved config per runtime |

### Updated Help Output

After implementation, `clawden run -h` should show:
```
Run a claw runtime directly

Usage: clawden-cli run [OPTIONS] [RUNTIME_AND_ARGS]...

Arguments:
  [RUNTIME_AND_ARGS]...  Runtime name followed by runtime args

Options:
  -e, --env <ENV>                  Set environment variables (KEY=VAL)
      --env-file <PATH>            Path to .env file (overrides auto-detected .env)
      --channel <CHANNEL>          Channels to connect (must appear before runtime name)
      --token <TOKEN>              Bot/channel token (requires --channel)
      --api-key <KEY>              LLM provider API key
      --with <TOOLS>               Tools to enable (must appear before runtime name)
      --provider <PROVIDER>        LLM provider override
      --model <MODEL>              LLM model override
      --system-prompt <PROMPT>     System prompt for the agent (or @file)
  -p, --port <HOST:CONTAINER>      Port mapping
      --no-docker                  Force direct mode
      --rm                         Remove one-off state after exit
  -d, --detach                     Run in background and return immediately
      --restart <RESTART>          Restart on failure policy
  -h, --help                       Print help
```

## Plan

- [ ] Add `-e` / `--env` flag to `Run` command (parse KEY=VAL, inject into env)
- [ ] Add `-e` / `--env` flag to `Up` command
- [ ] Add `--token` flag to `Run` — map to channel-specific env var based on `--channel`
- [ ] Add `--api-key` flag to `Run` — map to provider-specific env var based on `--provider` or config
- [ ] Add `--provider` flag to `Run` command
- [ ] Add `--model` flag to `Run` command
- [ ] Apply `--model` and `--provider` overrides in config translation pipeline
- [ ] Add `--system-prompt` flag to `Run` command (with @file support)
- [ ] Map `--system-prompt` to env var and config-dir output
- [ ] Add `--env-file` flag to `Run` and `Up` commands
- [ ] Add `-p` / `--port` flag to `Run` command
- [ ] Add `clawden config show` command with runtime resolution and secret redaction
- [ ] Add `--verbose` / `-v` global flag
- [ ] Add `--log-level` global flag
- [ ] Wire verbose/log-level to `tracing` subscriber initialization
- [ ] Update audit logging to redact `-e` values (log keys only)
- [ ] Add tests for `-e` parsing (KEY=VAL, KEY-only, multiple)
- [ ] Add tests for `--model` / `--provider` override in config translation
- [ ] Add tests for `config show` output and redaction

## Test

- [ ] `clawden run -e FOO=bar zeroclaw` → zeroclaw process receives `FOO=bar` in its environment
- [ ] `clawden run -e TELEGRAM_BOT_TOKEN=tok --channel telegram zeroclaw` → works without .env
- [ ] `clawden run --token tok --channel telegram zeroclaw` → sets `TELEGRAM_BOT_TOKEN=tok` in runtime env
- [ ] `clawden run --api-key sk-... --provider openai zeroclaw` → sets `OPENAI_API_KEY=sk-...` in runtime env
- [ ] `clawden run --token tok --channel telegram --api-key sk-... zeroclaw` → full zero-config run works
- [ ] `--token` without `--channel` → clear error message
- [ ] `-e` values override `.env` file values for the same key
- [ ] `clawden run --model gpt-4o --provider openai zeroclaw` → config translation uses overridden values
- [ ] `clawden run --system-prompt "test" zeroclaw` → `CLAWDEN_SYSTEM_PROMPT=test` in runtime env
- [ ] `clawden run --system-prompt @prompt.txt zeroclaw` → reads prompt from file
- [ ] `clawden config show zeroclaw` → displays resolved TOML with redacted secrets
- [ ] `clawden config show --reveal zeroclaw` → displays actual secret values
- [ ] `--verbose` produces debug output showing config resolution steps
- [ ] Audit log for `-e` usage contains key names but not values

## Notes

### Dependencies

- 033-product-positioning (complete) — `uv run` execution model that this spec extends
- 031-direct-mode-config-injection (in-progress) — config-dir translation pipeline that `--model`/`--provider`/`--system-prompt` must integrate with
- 025-llm-provider-api-key-management (complete) — provider config schema and env var resolution

### Non-Goals

- **Interactive credential prompts at run time** — `clawden init` already handles interactive setup. `clawden run` should be non-interactive; use `-e` or `--env-file` for ad-hoc credentials.
- **Remote secret store integration** — HashiCorp Vault, AWS Secrets Manager, etc. are out of scope. Local `.env` + env vars + `-e` cover the local development use case.
- **Runtime-specific flag translation** — `--model` and `--provider` use the existing config translation pipeline. We don't add runtime-specific flags like `--zeroclaw-verbose`; those go as trailing runtime args.
