---
status: planned
created: 2026-02-27
priority: high
tags:
- dashboard
- ui
- deployment
- channels
- runtime-management
depends_on:
- 011-control-plane
- 014-dashboard
- 017-docker-runtime-images
- 018-channel-support-matrix
- 020-dashboard-ui-ux-enhancement
parent: 009-orchestration-platform
created_at: 2026-02-27T03:34:06.627574Z
updated_at: 2026-02-27T03:34:06.627574Z
---
# Runtime Instance Manager & Channel Management Dashboard

## Overview

The ClawDen dashboard currently shows fleet status, agent health, task routing, config editing, and audit logs — but it lacks two critical operator workflows:

1. **Runtime Instance Management** — No way to deploy, install, or visually track Claw runtime instances (OpenClaw, Nanobot, PicoClaw, ZeroClaw, NanoClaw, IronClaw, TinyClaw, OpenFang — per [ClawCharts.com](https://clawcharts.com/)) from the dashboard. Operators can't see deployment state, trigger installs, or watch bootstrap progress.
2. **Channel / Bot Management** — No UI to configure messaging channels (Telegram, Slack, Discord, WhatsApp, etc.) and auto-bind them to deployed instances. Operators must hand-edit config files. Each runtime uses a different config format (JSON5, JSON, TOML, .env, WASM capabilities) and credential pattern.

This spec adds two new dashboard pages and their supporting backend APIs to close these gaps. Spec 017 (Docker Runtime Images) and spec 018 (Channel Support Matrix) define the backend foundations — this spec builds the frontend and API glue so operators can actually use them.

## Design

### Part 1: Runtime Instance Manager

#### New Dashboard Page — "Runtimes"

A new top-level nav item **Runtimes** in the sidebar (between Fleet and Tasks).

**A. Runtime Catalog** — Grid of available runtimes (8 per ClawCharts: OpenClaw, Nanobot, PicoClaw, ZeroClaw, NanoClaw, IronClaw, TinyClaw, OpenFang) with metadata from `AdapterRegistry::list_metadata()`:
- Name, language, version, star count, capabilities (chat, tools, vision, etc.)
- Status: Not installed / Installed / Has running instances
- Channel support cross-ref with spec 018 matrix (total channel count badge)
- **Deploy** button opens the deployment flow

**B. Instance List** — Table of deployed instances grouped by runtime:
- Instance name, runtime, lifecycle state badge, health indicator, uptime, host, connected channel badges
- Actions: Start / Stop / Restart / Configure / Logs

#### Deployment Flow

Multi-step wizard triggered by **Deploy** button:

1. **Configure** — Form: instance name, deployment target (Local / Docker / Remote), model provider + model name, channel selection, advanced env overrides
2. **Deploy** — Real-time progress: pulling image → installing runtime (`ClawAdapter::install()`) → applying config → starting instance (`ClawAdapter::start()`) → health check. Each step shows pending / spinner / checkmark / error
3. **Complete** — Instance appears in list with Running state, toast confirms success

#### Instance Detail Panel

Clicking an instance opens a side panel enhanced with:
- Deployment info (runtime version, container ID, deploy timestamp, target)
- Live log streaming (via SSE)
- Resource metrics sparklines (CPU / Memory from `ClawAdapter::metrics()`)
- Connected channels with per-channel status
- Restart / Stop / Redeploy with confirmation dialogs

#### New Backend Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|          
| `/runtimes` | GET | List available runtimes with adapter metadata |
| `/runtimes/{runtime}/deploy` | POST | Deploy new instance (install + configure + start) |
| `/agents/{id}/deploy-status` | GET | Deployment progress tracking |
| `/agents/{id}/logs` | GET (SSE) | Stream agent logs |
| `/agents/{id}/metrics/history` | GET | Historical metrics for charting |

### Part 2: Channel / Bot Management

#### New Dashboard Page — "Channels"

A new top-level nav item **Channels** in the sidebar (after Runtimes).

**A. Channel Overview Grid** — Card per channel type (Telegram, Slack, Discord, WhatsApp, etc.):
- Configured instance count, connection status (Connected / Disconnected / Partial)
- Configure / View instances actions

**B. Channel Configuration Form** — Per-channel credential + policy form. Config format varies by runtime (spec 018):
- Telegram: bot token, allowed user IDs, group activation mode
- Slack: bot token + app token (Socket Mode), signing secret, allowed channels
- Discord: bot token, guild ID, allowed roles, intents bitmask
- WhatsApp: implementation type (Baileys / Meta Cloud API / Node bridge), phone/API key
- Signal, Feishu, DingTalk, generic webhook, etc.
- Assignment: multi-select which deployed instances use this channel
- Policy: allowlist mode, pairing code toggle, group mention-only toggle
- Credentials stored encrypted via secret vault (spec 013)

**C. Channel Status Matrix** — Real-time instance × channel status grid:
- Per-cell: Connected ✅ / Disconnected ❌ / Rate limited ⚠️ / Proxied 🔄
- Status updates stream via WebSocket

#### Auto-Configuration Flow

1. Operator configures channel + assigns instances in UI
2. ClawDen translates to each runtime's native config format (`RuntimeConfigTranslator`) — handles JSON5, JSON, TOML, .env, WASM capabilities
3. Pushes config to each instance (`ClawAdapter::set_config()`)
4. Monitors channel health, reports status back to dashboard

#### Channel Proxy Indicator

For runtimes lacking native support (per spec 018 matrix), UI shows a "Proxy" badge. ClawDen bridges via channel proxy. Operator sees native vs proxied, proxy latency, and can disable per-channel.

#### New Backend Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|          
| `/channels` | GET | List configured channel types with status |
| `/channels/{type}` | GET/PUT/DELETE | CRUD for channel config (credentials encrypted) |
| `/channels/{type}/instances` | GET/PUT | Manage instance assignments |
| `/channels/{type}/test` | POST | Test channel credentials |
| `/agents/{id}/channels` | GET | Per-agent channel status |
| `/channels/matrix` | GET | Full channel × runtime support matrix |

### Component Structure

```
dashboard/src/components/
├── runtimes/
│   ├── RuntimeCatalog.tsx      # Runtime grid (8 ClawCharts runtimes)
│   ├── RuntimeCard.tsx         # Runtime card with Deploy
│   ├── InstanceList.tsx        # Deployed instances table
│   ├── DeployDialog.tsx        # Multi-step deploy wizard
│   └── DeployProgress.tsx      # Real-time progress panel
├── channels/
│   ├── ChannelOverview.tsx     # Channel type cards
│   ├── ChannelConfigForm.tsx   # Credential + policy form
│   ├── ChannelStatusMatrix.tsx # Instance × channel grid
│   └── ChannelAssignment.tsx   # Instance multi-select
```

### Sidebar Navigation Update

Fleet → **Runtimes** (NEW) → **Channels** (NEW) → Tasks → Config → Audit

## Plan

### Phase 1: Runtime Instance Manager
- [ ] Add `/runtimes` and `/runtimes/{runtime}/deploy` API endpoints
- [ ] Add `/agents/{id}/deploy-status` and `/agents/{id}/logs` endpoints
- [ ] Build RuntimeCatalog + RuntimeCard components
- [ ] Build InstanceList with state/health badges and actions
- [ ] Build DeployDialog multi-step wizard with DeployProgress panel
- [ ] Add "Runtimes" nav item and wire end-to-end

### Phase 2: Channel Management
- [ ] Add `/channels` CRUD endpoints with encrypted credential storage
- [ ] Add `/channels/{type}/test` and `/channels/matrix` endpoints
- [ ] Build ChannelOverview grid with status indicators
- [ ] Build ChannelConfigForm with per-channel credential fields
- [ ] Build ChannelAssignment + ChannelStatusMatrix components
- [ ] Implement auto-config push (channel → translator → set_config)
- [ ] Add "Channels" nav item and wire end-to-end

### Phase 3: Integration & Polish
- [ ] Link runtime cards to channel support badges (native vs proxied)
- [ ] Add deployment + channel events to audit log
- [ ] Toast notifications, loading skeletons, empty states for new pages
- [ ] Dark/light theme support, keyboard shortcuts (R → Runtimes, C → Channels)

## Test

- [ ] `/runtimes` returns metadata for all registered adapters
- [ ] Deploy flow transitions through install → configure → start → running
- [ ] Deploy progress updates in real-time via WebSocket/SSE
- [ ] RuntimeCatalog and InstanceList render correctly with state badges
- [ ] DeployDialog validates required fields before deploying
- [ ] Channel CRUD stores/retrieves configs; credentials encrypted, never in logs
- [ ] ChannelConfigForm validates credential format per channel type
- [ ] Auto-config push translates and applies config to assigned instances
- [ ] ChannelStatusMatrix reflects real-time connection state
- [ ] Proxy badge appears for runtimes lacking native channel support
- [ ] All new views render in both light and dark themes
- [ ] Existing dashboard tests continue to pass