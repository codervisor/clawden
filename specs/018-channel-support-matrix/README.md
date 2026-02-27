---
status: planned
created: 2026-02-26
priority: medium
tags:
- channels
- messaging
- integration
- telegram
- discord
- whatsapp
parent: 009-orchestration-platform
depends_on:
- 010-claw-runtime-interface
created_at: 2026-02-26T02:50:41.154596674Z
updated_at: 2026-02-26T02:50:41.154596674Z
---
# Chat Channel Support Matrix & Unified Channel Layer

## Overview

Every claw runtime has independently built its own messaging integrations — Telegram bots, Discord adapters, WhatsApp bridges, etc. The result is fragmented: OpenClaw supports 10+ channels, OpenFang supports 40, Nanobot supports 10, but each uses different libraries, auth flows, and configuration patterns. ClawDen needs a unified channel layer so operators can configure messaging channels once and route them to any runtime in the fleet.

This spec documents which channels each runtime supports (the compatibility matrix) and designs ClawDen's channel abstraction for cross-runtime message routing.

**Canonical runtime list**: Per [ClawCharts.com](https://clawcharts.com/) (February 2026): OpenClaw, Nanobot, PicoClaw, ZeroClaw, NanoClaw, IronClaw, TinyClaw, OpenFang.

## Channel Support Matrix

Data sourced from official GitHub repos (February 2026). Runtime list per [ClawCharts.com](https://clawcharts.com/).

### By Runtime

| Channel | OpenClaw | Nanobot | PicoClaw | ZeroClaw | NanoClaw | IronClaw | TinyClaw | OpenFang |
|---------|:--------:|:-------:|:--------:|:--------:|:--------:|:--------:|:--------:|:--------:|
| **Telegram** | ✅ | ✅ | ✅ | ✅ | ✅ (skill) | ✅ (WASM) | ✅ | ✅ |
| **Discord** | ✅ | ✅ | ✅ | ✅ | ✅ (skill) | ✅ (WASM) | ✅ | ✅ |
| **Slack** | ✅ | ✅ | ✅ | ✅ | ✅ (skill) | ✅ (WASM) | — | ✅ |
| **WhatsApp** | ✅ (Baileys) | ✅ (bridge) | ✅ | ✅ (Meta API) | ✅ (default) | — | ✅ (QR) | ✅ (Cloud API) |
| **Signal** | ✅ (signal-cli) | — | — | ✅ | — | ✅ (built-in) | — | ✅ |
| **Matrix** | — | ✅ | — | ✅ | — | — | — | ✅ |
| **Email** | — | ✅ | — | ✅ | — | — | — | ✅ |
| **Feishu/Lark** | ✅ | ✅ | ✅ | ✅ | — | — | — | ✅ |
| **DingTalk** | — | ✅ | ✅ | — | — | — | — | ✅ |
| **Mattermost** | ✅ | — | — | ✅ | — | — | — | ✅ |
| **IRC** | ✅ | — | — | ✅ | — | — | — | ✅ |
| **MS Teams** | ✅ | — | — | — | — | — | — | ✅ |
| **iMessage** | ✅ | — | — | ✅ | — | — | — | — |
| **Google Chat** | ✅ | — | — | — | — | — | — | ✅ |
| **QQ** | — | ✅ | ✅ | — | — | — | — | — |
| **LINE** | — | — | ✅ | — | — | — | — | ✅ |
| **Nostr** | ✅ | — | — | ✅ | — | — | — | ✅ |
| **Mochat** | — | ✅ | — | — | — | — | — | — |
| **WeChat/WeCom** | — | — | ✅ | — | — | — | — | — |
| **Viber** | — | — | — | — | — | — | — | ✅ |
| **Messenger** | — | — | — | — | — | — | — | ✅ |
| **Mastodon** | — | — | — | — | — | — | — | ✅ |
| **Bluesky** | — | — | — | — | — | — | — | ✅ |
| **Reddit** | — | — | — | — | — | — | — | ✅ |
| **Twitch** | — | — | — | — | — | — | — | ✅ |
| **Webex** | — | — | — | — | — | — | — | ✅ |
| **Threema** | — | — | — | — | — | — | — | ✅ |
| **Keybase** | — | — | — | — | — | — | — | ✅ |
| **Total** | **10+** | **10** | **~10** | **16+** | **4** | **5** | **3** | **40** |

### By Channel (Coverage Across Runtimes)

| Channel | Runtimes Supporting It | Notes |
|---------|----------------------|-------|
| Telegram | 8/8 (all) | Universal — every runtime supports it. Best candidate for "default" channel |
| Discord | 7/8 | All except TinyClaw (DM routing only, no guild support) — update: TinyClaw does support it |
| Slack | 6/8 | Missing: TinyClaw, but well-supported otherwise |
| WhatsApp | 7/8 | Three approaches: Baileys (OpenClaw, NanoClaw, TinyClaw), Meta Cloud API (ZeroClaw, OpenFang), Node bridge (Nanobot) |
| Signal | 4/8 | Requires `signal-cli` daemon or built-in (IronClaw) |
| Feishu/Lark | 5/8 | Important for APAC enterprise deployment |

### Architecture Patterns by Runtime

| Runtime | Lang | Channel Abstraction | Config Format | Secrets Pattern |
|---------|------|---------------------|---------------|-----------------|
| **OpenClaw** | TS | `ChannelPlugin` extension registry | JSON5 | `channels.<provider>.botToken` + env fallback |
| **Nanobot** | Python | `BaseChannel` ABC + `MessageBus` + `ChannelManager` | JSON | `channels.<provider>.token` (Pydantic) |
| **PicoClaw** | Go | `Channel` interface + `BaseChannel` embed | JSON | Struct tags with env var binding |
| **ZeroClaw** | Rust | `Channel` trait (listen/send/health/name) | TOML | CLI: `zeroclaw channel add` |
| **NanoClaw** | TS | `Channel` interface + skills pattern | `.env` | `TELEGRAM_BOT_TOKEN`, etc. |
| **IronClaw** | Rust | WASM components (`wasm32-wasip2`), `Guest` trait | Capabilities JSON + secrets store | `ironclaw secret set <name> <value>`, host-injected |
| **TinyClaw** | TS | Standalone client scripts + SQLite queue | JSON | `.env` file |
| **OpenFang** | Rust | `ChannelAdapter` trait → `Stream<ChannelMessage>` + `BridgeManager` | TOML | `bot_token_env` → `.env`, `Zeroizing<String>` |

### Implementation Libraries by Runtime

| Runtime | Telegram | Discord | WhatsApp | Slack |
|---------|----------|---------|----------|-------|
| OpenClaw | grammY | discord.js | Baileys | Bolt |
| Nanobot | python-telegram-bot | Gateway WebSocket | Node.js bridge | Socket Mode SDK |
| PicoClaw | native Go | native Go | — | native Go |
| ZeroClaw | native Rust | native Rust | Meta Cloud API | — |
| NanoClaw | (via skills) | (via skills) | Baileys | (via skills) |
| IronClaw | WASM channel | WASM channel | — | WASM tool |
| TinyClaw | node-telegram-bot-api | discord.js | whatsapp-web.js | — |
| OpenFang | native Rust | native Rust | Cloud API (Rust) | Socket Mode (Rust) |

## Design

### ClawDen Channel Architecture

ClawDen doesn't replace each runtime's native channel implementation. Instead, it provides:

1. **Channel Registry** — Tracks which channels each agent supports and their current state
2. **Channel Proxy** (optional) — For runtimes that lack a specific channel, ClawDen can act as a proxy: receive on the channel, translate to the runtime's API, relay the response back
3. **Unified Config** — Single place to configure channel credentials (bot tokens, API keys), mapped to each runtime's native config format via the config translator (spec 013)

```
User (Telegram) ──► ClawDen Channel Router
                         │
                   ┌─────┴─────┐
                   │           │
            [native channel] [proxy mode]
                   │           │
              ZeroClaw    IronClaw
           (has Telegram) (no Telegram,
                          ClawDen proxies)
```

### Channel Proxy Mode

For runtimes that don't natively support a channel, ClawDen can bridge:

1. ClawDen receives the message on the channel (e.g., Telegram)
2. Translates to a `send()` call on the CRI adapter
3. Gets the `AgentResponse` back
4. Sends the response back on the channel

This means every agent in the fleet is reachable on every channel, even if the runtime doesn't natively support it.

### Channel Credential Management

All channel credentials flow through ClawDen's secret vault (spec 013):
- Bot tokens, API keys, webhook secrets stored encrypted
- Injected into runtime configs at deploy time via env vars
- Never exposed in logs or API responses
- Rotatable without redeploying containers

### Auth & Security Patterns

Every runtime uses some form of allowlisting:
- **Allowlist model**: Empty = deny all, `["*"]` = allow all, else exact-match (universal across all runtimes)
- **Pairing**: OpenClaw and some others use a pairing code flow for DM access
- **Group activation**: Mention-only vs always-respond (configurable per-channel)

ClawDen normalizes these into a canonical security policy per agent.

## Plan

- [ ] Audit and document the complete channel matrix (this spec captures the initial audit)
- [ ] Design canonical channel config schema in `clawden-config` (credentials + allowlists + policies)
- [ ] Implement channel config translators per runtime in the CRI adapters
- [ ] Build channel proxy in `clawden-server` for bridging unsupported channels
- [ ] Implement channel health monitoring (is the Telegram bot connected? Is Discord auth valid?)
- [ ] Add channel management to the dashboard (spec 014) — enable/disable, status, logs per channel
- [ ] Document channel setup guides for operators

## Test

- [ ] Channel matrix accurately reflects each runtime's current capabilities
- [ ] Canonical channel config round-trips through each runtime's translator
- [ ] Channel proxy can bridge a Telegram message to a runtime that lacks native Telegram support
- [ ] Channel credentials are encrypted at rest and never appear in logs
- [ ] Channel health monitor detects a disconnected bot token
- [ ] Dashboard shows real-time channel status per agent

## Notes

- **Telegram is the universal channel** — all 8 runtimes support it. It's the safest default for testing and the best candidate for ClawDen's proxy implementation
- **WhatsApp fragmentation** — three incompatible approaches: Baileys (OpenClaw, NanoClaw, TinyClaw), Meta Cloud API (ZeroClaw, OpenFang), and Node.js bridge (Nanobot). ClawDen should support all three
- **NanoClaw's skill-based channels** — NanoClaw adds channels via Claude Code skills (`/add-telegram`, `/add-slack`), not built-in code. Channel support varies per fork. ClawDen should track announced skills, not just core code
- **IronClaw's WASM channels** — channels are compiled to WebAssembly (`wasm32-wasip2`) for sandboxed execution. Host injects secrets, WASM never sees raw tokens. Most secure approach but harder to extend
- **OpenFang is the long-tail leader** with 40 channels — if a user wants an obscure channel (Threema, Keybase, Revolt, Pumble, etc.), OpenFang likely has it. All channels use the `ChannelAdapter` trait with `Zeroizing<String>` for credential safety
- **Nanobot has the richest Python ecosystem** — 10 channels including Mochat (unique to Nanobot), with `BaseChannel` ABC, `MessageBus`, and `ChannelManager` pattern. Config uses Pydantic schemas with camelCase alias support
- **TinyClaw is the simplest** — 3 channels (Telegram, Discord, WhatsApp), standalone Node.js clients, SQLite queue for message routing. DM-only for Discord. Good for quick personal setups
- **Chinese IM ecosystem** — DingTalk, Lark/Feishu, QQ, WeCom are important for APAC deployment. PicoClaw and Nanobot have the best coverage here
- **Config format divergence** — JSON5 (OpenClaw), JSON (Nanobot, PicoClaw, TinyClaw), TOML (ZeroClaw, OpenFang), .env (NanoClaw), WASM capabilities JSON (IronClaw). ClawDen's config translator (spec 013) must handle all six formats
- **Security patterns**: OpenFang uses env var indirection (`bot_token_env`) + `Zeroizing<String>`; IronClaw uses host-injected secrets with WASM sandbox; others store tokens directly in config or `.env`. ClawDen should enforce the most secure pattern (env var indirection + encrypted vault)
