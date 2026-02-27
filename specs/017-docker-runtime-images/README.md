---
status: planned
created: 2026-02-26
priority: high
tags:
- docker
- deployment
- runtime
- infra
- container
parent: 009-orchestration-platform
depends_on:
- 010-claw-runtime-interface
created_at: 2026-02-26T02:42:25.266699500Z
updated_at: 2026-02-26T02:42:25.266699500Z
---
# Docker Runtime Images & Deployment

## Overview

ClawDen provides a **single generic Docker container** that can run any supported claw runtime. Users configure it with a minimal YAML file or a single CLI command — no Docker knowledge, no env var pipelines, no image variant selection. ClawDen handles all the container plumbing internally.

## Context

### Problem with Prior Approach

The previous design exposed too much infrastructure to users: TOML config translation, image variant selection, plugin manifest schemas, CRI adapter internals. Normal users shouldn't need to think about any of that. They just want to say "run ZeroClaw on Telegram with git available" and have it work.

### Design Principle

**User config is dead simple. Complexity lives inside ClawDen.**

### Scope: Priority Runtimes First

OpenClaw is the most popular runtime and the top priority, despite being the most complex — Node.js app, JSON5 config, gateway architecture, 10+ channel integrations with different libraries (grammY, discord.js, Baileys, Bolt). NanoClaw has similar Node.js complexity with code-driven config but is also high-demand.

Phase 1 targets the runtimes users need most. Phase 2 picks up lower-adoption runtimes:

| Phase 1 (now)                         | Phase 2 (later)  |
| ------------------------------------- | ---------------- |
| OpenClaw (Node.js) — highest priority | IronClaw (Rust)  |
| ZeroClaw (Rust)                       | NullClaw (Zig)   |
| NanoClaw (Node.js)                    | MicroClaw (Rust) |
| PicoClaw (Go)                         |                  |

Phase 1 requires a Node.js variant image for OpenClaw and NanoClaw, plus a slim image for the binary runtimes.

## Design

### User-Facing Config: `clawden.yaml`

Users write one file. Everything — runtimes, channels, tools — in one place:

```yaml
# clawden.yaml — the whole config

# Channel instances (each maps to exactly one runtime)
channels:
  zc-telegram:
    type: telegram
    token: $ZC_TELEGRAM_TOKEN
  pc-telegram:
    type: telegram
    token: $PC_TELEGRAM_TOKEN
  pc-discord:
    type: discord
    token: $PC_DISCORD_TOKEN

# Runtimes
runtimes:
  - name: zeroclaw
    channels: [zc-telegram]
    tools: [git, http]

  - name: picoclaw
    channels: [pc-telegram, pc-discord]
    tools: [git]
```

Minimal single-runtime example:

```yaml
runtime: zeroclaw
channels:
  telegram:
    token: $TELEGRAM_BOT_TOKEN
```

That's a running AI agent on Telegram. Three lines of real config.

#### Channels with Options

Most channels need just a token. Some have extra options, but they're always optional with sane defaults:

```yaml
channels:
  telegram:
    token: $TELEGRAM_BOT_TOKEN

  discord:
    token: $DISCORD_BOT_TOKEN
    # optional
    guild: "123456789"

  slack:
    bot_token: $SLACK_BOT_TOKEN
    app_token: $SLACK_APP_TOKEN

  whatsapp:
    token: $WHATSAPP_TOKEN
```

Users list field names that make sense for that channel. ClawDen maps them to whatever the runtime expects internally.

#### Environment Variable References

Tokens use `$ENV_VAR` syntax — ClawDen resolves them at startup. Secrets never sit in the YAML file as plaintext.

```yaml
channels:
  telegram:
    token: $TELEGRAM_BOT_TOKEN    # reads from environment
```

Or users can use a `.env` file next to `clawden.yaml` — ClawDen auto-loads it.

### CLI: One Command

```bash
# Run a runtime (reads clawden.yaml if present)
clawden run

# Run a specific runtime directly (no config file needed)
clawden run zeroclaw
clawden run zeroclaw --with git,http
clawden run zeroclaw --channel telegram --with git

# Run multiple runtimes
clawden up                    # starts everything in clawden.yaml
clawden up zeroclaw picoclaw  # starts just these two

# Channel management
clawden channels              # list configured channels + status
clawden channels test         # test all channel connections

# See what's running
clawden ps

# Stop
clawden stop
clawden stop zeroclaw
```

### What Happens When You Add a Channel

User writes:
```yaml
channels:
  telegram:
    token: $TELEGRAM_BOT_TOKEN
runtimes:
  - name: zeroclaw
    channels: [telegram]
```

ClawDen:
1. Reads the channel config from YAML
2. Resolves `$TELEGRAM_BOT_TOKEN` from env / `.env` file
3. Knows that ZeroClaw expects Telegram config via `ZEROCLAW_TELEGRAM_BOT_TOKEN` env var
4. Passes it to the container — done

User doesn't know or care that ZeroClaw uses TOML config internally or that IronClaw uses WASM capabilities injection. ClawDen translates behind the scenes.

### What "Tools" Are

Tools are pre-packaged environment capabilities. Users just name them.

**Built-in tools:**

| Tool      | What it gives the runtime            | Phase |
| --------- | ------------------------------------ | ----- |
| `git`     | Git + SSH client                     | 1     |
| `http`    | curl, wget                           | 1     |
| `browser` | Headless Chromium (Playwright-ready) | 2     |
| `gui`     | Xvfb + VNC/noVNC desktop             | 2     |

**Custom tools (advanced):** Drop a folder into `~/.clawden/tools/<name>/` with a `setup.sh`. Most users never need this.

### Channel Auto-Detection

If a runtime is assigned a channel it doesn't natively support, ClawDen automatically proxies — receives messages on the channel, relays to the runtime via its API, sends responses back. Users see a "proxied" badge in `clawden ps` but otherwise it just works.

### Container Internals (Developer-Facing)

Phase 1 includes both Node.js runtimes (OpenClaw, NanoClaw) and binary runtimes (ZeroClaw, PicoClaw), so the container ships with Node.js:

```
ghcr.io/codervisor/clawden-runtime:latest
├── /opt/clawden/
│   ├── entrypoint.sh           # Selects runtime, runs tool setup
│   ├── runtimes/               # Runtime executables
│   │   ├── openclaw/           # Node.js app (JSON5 config, gateway)
│   │   ├── nanoclaw/           # Node.js app (code-driven, Agent SDK)
│   │   ├── zeroclaw            # Rust binary
│   │   └── picoclaw            # Go binary
│   └── tools/                  # Built-in tool scripts
│       ├── git/setup.sh
│       ├── http/setup.sh
│       ├── browser/setup.sh
│       └── gui/setup.sh
├── /home/clawden/workspace/    # Volume mount point
└── /etc/clawden/defaults.toml  # Internal defaults
```

Single image tag: `clawden-runtime:latest` (Debian slim + Node.js 22 LTS). Phase 2 adds IronClaw (Rust + PostgreSQL sidecar), NullClaw (Zig), and MicroClaw (Rust).

The entrypoint:
1. Reads `RUNTIME` and `TOOLS` from env (set by `clawden` CLI)
2. For each tool, sources `/opt/clawden/tools/<name>/setup.sh`
3. Execs into the runtime binary

### Runtime Config Passthrough

Each runtime has its own native config format. ClawDen doesn't expose this to users. Instead:

- Runtimes use their own default configs (baked into the image)
- Users can override specific runtime settings in `clawden.yaml`:

```yaml
runtime: zeroclaw
tools: [git]
config:
  port: 9000
  log_level: debug
```

ClawDen passes `config` entries as env vars. Each runtime handles its own env → config mapping.

### Per-Runtime Notes

**Phase 1:**

| Runtime  | Type        | Default Port | Config Format                      |
| -------- | ----------- | ------------ | ---------------------------------- |
| OpenClaw | Node.js app | 18789        | JSON5 config, gateway architecture |
| ZeroClaw | Rust binary | 42617        | TOML config + env vars             |
| NanoClaw | Node.js app | auto         | Code-driven, Claude Agent SDK      |
| PicoClaw | Go binary   | auto         | JSON config                        |

**Phase 2 (deferred):**

| Runtime   | Type        | Default Port | Why Deferred                                               |
| --------- | ----------- | ------------ | ---------------------------------------------------------- |
| IronClaw  | Rust binary | auto         | WASM channels, requires PostgreSQL sidecar, lower adoption |
| NullClaw  | Zig binary  | 3000         | Lower adoption                                             |
| MicroClaw | Rust binary | auto         | Lower adoption                                             |

**IronClaw** needs PostgreSQL — `clawden up` starts a Postgres sidecar automatically (Phase 2).

**MimiClaw** (ESP32 firmware) — not containerizable, supported via serial/MQTT bridge.

### Conventions

1. **Non-root**: Containers run as `clawden` user (UID 10000)
2. **Health**: `GET /health` — ClawDen polls this automatically
3. **Workspace**: `./workspace:/home/clawden/workspace` volume mount
4. **Secrets**: Tokens via `$ENV_VAR` references in YAML + `.env` file auto-load. Never plaintext in config.
5. **Logs**: `clawden logs <runtime>` streams container logs

## Plan

### Phase 1: Priority Runtimes
- [ ] Define `clawden.yaml` schema (runtimes, channels, tools, config overrides)
- [ ] Implement OpenClaw credential mapping (JSON5 config, grammY, discord.js, Baileys, Bolt) — highest priority
- [ ] Implement ZeroClaw credential mapping (env vars, TOML config)
- [ ] Implement NanoClaw credential mapping (code-driven, skill-based channels)
- [ ] Implement PicoClaw credential mapping (JSON config, native Go)
- [ ] Implement `clawden run <runtime> --channel <name> --with <tools>` CLI command
- [ ] Build `clawden-runtime` Dockerfile (Debian slim + Node.js 22 LTS)
- [ ] Implement `entrypoint.sh` — runtime selection + tool setup loop
- [ ] Create `git` and `http` tool setup scripts
- [ ] Package Phase 1 runtimes into image
- [ ] Implement `clawden up` / `clawden ps` / `clawden stop` / `clawden logs`
- [ ] Implement `clawden channels` — list + test
- [ ] Implement channel proxy for unsupported runtime+channel combos
- [ ] CI: build + push image to GHCR

### Phase 2: Remaining Runtimes & Advanced Tools
- [ ] Add IronClaw to image (WASM capabilities, PostgreSQL sidecar)
- [ ] Add NullClaw to image (Zig binary)
- [ ] Add MicroClaw to image (Rust binary)
- [ ] Create `browser` and `gui` tool setup scripts
- [ ] Support custom tools via `~/.clawden/tools/`

## Test

- [ ] `clawden run zeroclaw --channel telegram` connects to Telegram and responds
- [ ] `clawden.yaml` with channels section correctly configures all assigned runtimes
- [ ] `$ENV_VAR` references resolve from environment and `.env` file
- [ ] Channel proxy works for runtime+channel combos the runtime doesn't natively support
- [ ] `clawden channels test` validates credentials without starting runtimes
- [ ] Token uniqueness enforced — same bot token can't be used by two runtime instances
- [ ] Container runs as non-root
- [ ] `clawden ps` shows runtime status + connected channels

## Notes

- **OpenClaw is Phase 1 top priority**: Most popular runtime. Complex (Node.js, JSON5 config, gateway on port 18789, 10+ channel integrations each using different libraries) but critical to support first
- **NanoClaw in Phase 1**: Also Node.js, code-driven config (not file-based), depends on Claude Agent SDK. Bundled with OpenClaw since both need Node.js in the image
- **IronClaw deferred to Phase 2**: Requires PostgreSQL sidecar, WASM-based channels — lower adoption, added infrastructure complexity
- **NullClaw, MicroClaw deferred to Phase 2**: Lower priority runtimes, straightforward to add when needed
- Phase 1 image includes Node.js 22 LTS for OpenClaw/NanoClaw — larger than binary-only but necessary
- Channels use named instances in YAML — `type` + `token` plus optional extras (see spec 018)
- `.env` file auto-load keeps secrets separate from config
- Channel proxy makes every runtime reachable on every channel
- Multi-arch (amd64 + arm64) is a stretch goal