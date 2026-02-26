---
status: planned
created: 2026-02-26
priority: critical
tags:
- core
- adapter
- cri
depends_on:
- 009-orchestration-platform
created_at: 2026-02-26T02:08:29.575446436Z
updated_at: 2026-02-26T02:08:40.054769542Z
---

# Claw Runtime Interface (CRI) / Adapter Layer

## Overview

The Claw Runtime Interface (CRI) is the adapter layer that abstracts communication with heterogeneous claw runtimes. Like Kubernetes' Container Runtime Interface, CRI provides a unified TypeScript interface that each claw runtime implements via a driver/adapter.

## Design

### Core Interface

```typescript
interface ClawAdapter {
  readonly runtime: ClawRuntime; // metadata: name, version, lang, capabilities
  
  // Lifecycle
  install(config: InstallConfig): Promise<void>;
  start(config: AgentConfig): Promise<AgentHandle>;
  stop(handle: AgentHandle): Promise<void>;
  restart(handle: AgentHandle): Promise<void>;
  
  // Health
  health(handle: AgentHandle): Promise<HealthStatus>;
  metrics(handle: AgentHandle): Promise<AgentMetrics>;
  
  // Communication
  send(handle: AgentHandle, message: AgentMessage): Promise<AgentResponse>;
  subscribe(handle: AgentHandle, event: string, cb: EventCallback): Unsubscribe;
  
  // Configuration  
  getConfig(handle: AgentHandle): Promise<RuntimeConfig>;
  setConfig(handle: AgentHandle, config: Partial<RuntimeConfig>): Promise<void>;
  
  // Skills
  listSkills(handle: AgentHandle): Promise<Skill[]>;
  installSkill(handle: AgentHandle, skill: SkillManifest): Promise<void>;
}
```

### Supported Runtimes (Initial)

| Runtime | Communication Method | Adapter Strategy |
|---------|---------------------|------------------|
| OpenClaw | HTTP Gateway API | REST client |
| ZeroClaw | HTTP Gateway + CLI | REST + subprocess |
| PicoClaw | HTTP Gateway + CLI | REST + subprocess |
| NanoClaw | Filesystem IPC | File watch + subprocess |
| IronClaw | HTTP Gateway + CLI | REST + subprocess |
| NullClaw | HTTP Gateway + CLI | REST + subprocess |

### Adapter Registration
Adapters are discovered via a plugin directory (`~/.clawlab/adapters/`) or npm packages (`@clawlab/adapter-*`).

## Plan

- [ ] Define `ClawAdapter` TypeScript interface and types
- [ ] Implement `OpenClawAdapter` (most mature ecosystem)
- [ ] Implement `ZeroClawAdapter` (Rust, popular)
- [ ] Implement `PicoClawAdapter` (Go, popular)
- [ ] Implement `NanoClawAdapter` (TypeScript, containerized)
- [ ] Create adapter plugin loader and registry
- [ ] Add adapter discovery and auto-detection

## Test

- [ ] Each adapter can connect to its respective runtime
- [ ] Adapters correctly report health status
- [ ] Plugin loader discovers and registers adapters
- [ ] Adapters handle connection failures gracefully
