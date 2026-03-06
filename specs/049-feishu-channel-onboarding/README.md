---
status: complete
created: 2026-03-06
priority: high
tags:
- feishu
- lark
- channels
- onboarding
- cli
- ux
depends_on:
- 018-channel-support-matrix
created_at: 2026-03-06T01:45:36.461489394Z
updated_at: 2026-03-06T02:00:52.754713056Z
completed_at: 2026-03-06T02:00:52.754713056Z
transitions:
- status: in-progress
  at: 2026-03-06T01:51:39.395544883Z
- status: complete
  at: 2026-03-06T02:00:52.754713056Z
---

# Feishu Channel Onboarding & Credential Verification

## Overview

Feishu/Lark channel setup has significantly more friction than Telegram. Telegram users create a bot via @BotFather, get a token, and run `clawden run` — done. Feishu requires users to navigate the Feishu Open Platform developer console, create an app, enable bot capability, configure event subscriptions, set permissions, and publish — all before ClawDen can even receive messages.

ClawDen currently only passes `app_id` + `app_secret` through to runtimes and hopes the runtime handles the rest. This spec adds a `clawden channels feishu` subcommand group with guided setup, credential verification, and diagnostics to bridge the gap.

## Motivation

- **Telegram vs Feishu setup parity**: Telegram has `clawden channels telegram resolve-id` for identity resolution. Feishu has nothing — not even credential validation.
- **Silent failures**: If `app_id`/`app_secret` are wrong, or event subscriptions aren't configured, the runtime starts but never receives messages. Users get no feedback.
- **Multi-step console setup**: Feishu's developer console requires 5+ manual steps. Users forget permissions, skip event subscriptions, or don't publish the app version. ClawDen should guide and verify.
- **Long connection vs webhook confusion**: Runtimes use long connection mode (WebSocket) which only needs `app_id`/`app_secret`, but users familiar with webhook mode may configure unnecessary fields or get confused when there's no webhook URL to set.

## Design

### CLI Structure

All Feishu utilities live under `clawden channels feishu`, following the existing `clawden channels telegram` pattern:

```
clawden channels feishu
├── verify    — Verify app_id/app_secret credentials and check app configuration
└── setup     — Interactive guided setup walkthrough
```

### `clawden channels feishu verify`

Validates Feishu credentials and checks app readiness by calling Feishu Open APIs.

**Steps:**

1. **Resolve credentials**: Read `app_id`/`app_secret` from `clawden.yaml` channel config (or `--app-id`/`--app-secret` flags). Follow the same env-var resolution as `clawden channels test`.

2. **Obtain tenant access token**: Call `POST https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal` with `app_id` + `app_secret`. If this fails, credentials are wrong — stop with clear error.

3. **Check bot capability**: Call `GET https://open.feishu.cn/open-apis/bot/v3/info` with the tenant token. If the bot info endpoint returns an error or indicates bot is not enabled, warn that bot capability must be enabled in the developer console.

4. **Report results**:
   ```
   Feishu app verification:
     App ID:           cli_a5xxxxx
     Credentials:      ✓ valid (tenant token obtained)
     Bot capability:   ✓ enabled
     
   ⚠ Reminder: Ensure these event subscriptions are enabled in your Feishu Developer Console:
     • im.message.receive_v1 (receive messages)
     
   ⚠ Reminder: Ensure these permissions are granted:
     • im:message (read messages)
     • im:message:send_as_bot (send messages as bot)
     
   Note: ClawDen runtimes use long connection mode (WebSocket).
   No webhook URL or verification token is needed.
   ```

**Flags:**
- `--app-id <APP_ID>` — Override app_id (otherwise read from clawden.yaml)
- `--app-secret <APP_SECRET>` — Override app_secret (otherwise read from clawden.yaml)
- `--channel <name>` — Specify which channel instance in clawden.yaml to verify (when multiple feishu channels exist)

### `clawden channels feishu setup`

Interactive guided walkthrough that helps users create and configure a Feishu bot from scratch.

**Flow:**

```
──────────────────────────────────────────────
  Feishu Bot Setup Guide
──────────────────────────────────────────────

Step 1: Create a Feishu App
  → Open: https://open.feishu.cn/app
  → Click "Create Custom App"
  → Choose "Enterprise Custom App"
  → Give it a name and description

Step 2: Get Credentials
  → Go to "Credentials & Basic Info" in your app settings
  → Copy the App ID and App Secret

  App ID: █
  App Secret: █

Step 3: Enable Bot Capability
  → Go to "Add Features" → "Bot"
  → Enable the bot feature
  → Set a bot name

Step 4: Configure Event Subscriptions
  → Go to "Event Subscriptions"
  → Select "Long Connection" mode (recommended — no public URL needed)
  → Add event: im.message.receive_v1

Step 5: Set Permissions
  → Go to "Permissions & Scopes"
  → Add: im:message, im:message:send_as_bot

Step 6: Publish
  → Go to "Version Management & Release"
  → Create a new version and submit for review
  → For testing, use "Create Test Version" for immediate access

Verifying credentials...
  ✓ Credentials valid
  ✓ Bot capability enabled

Your clawden.yaml channel config:

  channels:
    feishu:
      app_id: $FEISHU_APP_ID
      app_secret: $FEISHU_APP_SECRET

Add these to your .env file:
  FEISHU_APP_ID=cli_a5xxxxx
  FEISHU_APP_SECRET=xxxxx
```

After collecting credentials, automatically runs `verify` to confirm everything works.

### Error Scenarios

| Scenario | Detection | Message |
|----------|-----------|---------|
| Wrong `app_id`/`app_secret` | Tenant token API returns error | "Invalid credentials. Check App ID and App Secret in your Feishu Developer Console." |
| Bot not enabled | Bot info API returns error | "Bot capability is not enabled. Go to your app settings → Add Features → Bot." |
| App not published | Bot works in test but not production | "If the bot doesn't respond, ensure the app version is published or a test version is active." |
| Events not subscribed | Runtime starts but no messages | Print reminder checklist during verify (can't detect programmatically without admin scopes) |
| Wrong connection mode | User configures webhook instead of long connection | "ClawDen runtimes use long connection mode. No webhook URL is needed—select 'Long Connection' in Event Subscriptions." |

### Code Organization

Following the existing `telegram.rs` pattern:

- `crates/clawden-cli/src/commands/feishu.rs` — `exec_feishu()`, `FeishuVerifier` struct (API calls), setup flow
- `crates/clawden-cli/src/cli.rs` — `FeishuCommand` enum with `Verify`/`Setup` variants, nested under `ChannelCommand::Feishu`
- `crates/clawden-cli/src/commands/channels.rs` — Route `ChannelCommand::Feishu` to `feishu::exec_feishu()`

### Feishu API Calls

Only two API calls, both using the Internal App authentication flow (tenant access token):

1. **Get tenant access token** — `POST https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal`
   - Body: `{ "app_id": "...", "app_secret": "..." }`
   - Success: returns `{ "tenant_access_token": "...", "expire": 7200 }`

2. **Get bot info** — `GET https://open.feishu.cn/open-apis/bot/v3/info`
   - Header: `Authorization: Bearer {tenant_access_token}`
   - Success: returns bot name, open_id, etc.

Both are safe read-only calls with no side effects.

## Plan

- [x] Add `FeishuCommand` enum to `cli.rs` with `Verify` and `Setup` variants
- [x] Add `ChannelCommand::Feishu` arm mirroring `ChannelCommand::Telegram`
- [x] Create `feishu.rs` with `exec_feishu()` dispatcher
- [x] Implement `FeishuVerifier` — tenant token + bot info API calls
- [x] Implement `clawden channels feishu verify` — credential check + checklist output
- [x] Implement `clawden channels feishu setup` — interactive guided walkthrough
- [x] Route `ChannelCommand::Feishu` in `channels.rs`
- [x] Add unit tests for credential resolution and API response parsing
- [x] Add integration test: valid credentials → successful verification output

## Test

- [x] `clawden channels feishu verify` with valid credentials → "✓ valid" output
- [x] `clawden channels feishu verify` with invalid credentials → clear error message
- [x] `clawden channels feishu verify` with missing clawden.yaml → helpful error
- [x] `clawden channels feishu verify --app-id X --app-secret Y` → uses flag values over yaml
- [x] `clawden channels feishu verify --channel my-feishu-bot` → picks correct channel instance
- [x] `clawden channels feishu setup` → prints all 6 steps, prompts for credentials, runs verify
- [x] Bot not enabled → specific error message about enabling bot capability
- [x] Multiple feishu channels in yaml without `--channel` → prompts user to select

## Notes

- This follows the same pattern as `clawden channels telegram resolve-id` — channel-specific subcommands under `clawden channels <type>`.
- The `verify` command is safe and read-only — it only obtains a short-lived tenant token to validate credentials and check bot status.
- Event subscription and permission checks are best-effort reminders, not programmatic verification (checking those would require admin-level scopes that a bot app typically doesn't have).
- Long connection mode is the recommended approach for ClawDen because it doesn't require public network ingress — the runtime connects outbound to Feishu's servers via WebSocket.
- Future enhancement: `clawden channels feishu send-test` could send a test message to verify end-to-end connectivity after setup.