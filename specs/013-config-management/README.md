---
status: planned
created: 2026-02-26
priority: medium
tags:
- core
- config
- secrets
depends_on:
- 010-claw-runtime-interface
created_at: 2026-02-26T02:08:29.575930222Z
updated_at: 2026-02-26T02:08:40.055694095Z
---

# Unified Configuration Management

## Overview

Each claw runtime has its own config format (JSON, TOML, env vars, markdown). ClawLab provides a unified configuration layer that translates between a canonical schema and runtime-specific formats.

## Design

### Canonical Config Schema
```typescript
interface ClawLabConfig {
  agent: {
    name: string;
    runtime: string;
    model: ModelConfig;        // provider, model name, API key ref
    tools: ToolConfig[];       // enabled tools and permissions
    channels: ChannelConfig[]; // messaging channels
    memory: MemoryConfig;      // memory backend settings
    security: SecurityConfig;  // sandbox, allowlists
    schedule: ScheduleConfig;  // cron jobs, heartbeat
  };
}
```

### Config Translation
Each CRI adapter includes a config translator:
- `toRuntimeConfig(canonical)` → runtime-specific format
- `fromRuntimeConfig(native)` → canonical format
- Validates required fields per runtime
- Handles runtime-specific extensions via `extras` field

### Secret Management
- API keys stored in encrypted vault (age/sops or system keychain)
- Referenced by name in config, injected at deploy time
- Never stored in plain text or committed to git

## Plan

- [ ] Define canonical config schema with Zod validation
- [ ] Implement config translator interface in CRI
- [ ] Build OpenClaw config translator (JSON ↔ canonical)
- [ ] Build ZeroClaw config translator (TOML ↔ canonical)
- [ ] Build PicoClaw config translator (JSON ↔ canonical)
- [ ] Implement encrypted secret vault
- [ ] Add config diff and drift detection

## Test

- [ ] Canonical config round-trips through each translator
- [ ] Invalid configs are rejected with clear error messages
- [ ] Secrets are never exposed in logs or API responses
- [ ] Config drift detection identifies out-of-sync agents
