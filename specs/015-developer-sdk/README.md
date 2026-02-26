---
status: planned
created: 2026-02-26
priority: medium
tags:
- sdk
- cli
- developer
- skills
depends_on:
- 010-claw-runtime-interface
created_at: 2026-02-26T02:08:29.576054643Z
updated_at: 2026-02-26T02:08:40.056135402Z
---

# Cross-Claw Developer SDK & CLI

## Overview

A unified SDK and CLI that enables developers to build skills/plugins that work across multiple claw runtimes. Includes a testing harness, skill packaging, and distribution via a marketplace.

## Design

### CLI (`clawlab`)
```bash
clawlab init                    # Initialize ClawLab project
clawlab agent list              # List registered agents
clawlab agent start <name>      # Start an agent
clawlab agent stop <name>       # Stop an agent
clawlab agent health            # Fleet health summary
clawlab fleet status            # Fleet overview
clawlab task send <agent> <msg> # Send task to agent
clawlab skill create <name>     # Scaffold a new skill
clawlab skill test <name>       # Test skill across runtimes
clawlab skill publish <name>    # Publish to marketplace
clawlab config set <key> <val>  # Set config value
clawlab config diff             # Show config drift
```

### Skill SDK
```typescript
import { defineSkill } from '@clawlab/sdk';

export default defineSkill({
  name: 'web-scraper',
  version: '1.0.0',
  runtimes: ['openclaw', 'zeroclaw', 'picoclaw'], // compatible runtimes
  tools: ['browser_open', 'http_request'],         // required tools
  
  async execute(context: SkillContext) {
    // Runtime-agnostic skill logic
  },
  
  // Per-runtime adaptations
  adapters: {
    openclaw: { /* OpenClaw-specific config */ },
    zeroclaw: { /* ZeroClaw-specific config */ },
  }
});
```

### Skill Marketplace
- Package registry (npm-style) for cross-claw skills
- Compatibility matrix showing which runtimes are supported
- Version management and dependency resolution

## Plan

- [ ] Build CLI with commander.js / yargs
- [ ] Implement agent management commands
- [ ] Implement fleet status commands
- [ ] Define Skill SDK with `defineSkill` API
- [ ] Build skill scaffolding (`clawlab skill create`)
- [ ] Create cross-runtime skill test harness
- [ ] Design marketplace registry protocol

## Test

- [ ] CLI commands execute correctly against running ClawLab
- [ ] Skill SDK produces valid skill packages
- [ ] Test harness runs skills against multiple runtimes
- [ ] Published skills can be installed and executed
