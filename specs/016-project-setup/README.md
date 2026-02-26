---
status: planned
created: 2026-02-26
priority: critical
tags:
- infra
- setup
- monorepo
created_at: 2026-02-26T02:08:29.576100007Z
updated_at: 2026-02-26T02:08:29.576100007Z
---

# Project Setup & Monorepo Scaffolding

## Overview

Scaffold the ClawLab monorepo with package structure, dependencies, build tooling, and development workflow for the orchestration platform.

## Design

### Monorepo Structure
```
clawlab/
├── packages/
│   ├── core/           # CRI interfaces, types, shared utilities
│   ├── control-plane/  # Lifecycle, health, recovery services
│   ├── fleet/          # Discovery, routing, swarm coordination
│   ├── config/         # Config schema, translators, secret vault
│   ├── api/            # REST + WebSocket API server (Fastify)
│   ├── dashboard/      # React web dashboard
│   ├── cli/            # clawlab CLI
│   └── sdk/            # @clawlab/sdk for skill developers
├── adapters/
│   ├── openclaw/       # @clawlab/adapter-openclaw
│   ├── zeroclaw/       # @clawlab/adapter-zeroclaw
│   ├── picoclaw/       # @clawlab/adapter-picoclaw
│   └── nanoclaw/       # @clawlab/adapter-nanoclaw
├── specs/              # LeanSpec specs
├── .github/            # CI/CD
└── package.json        # Workspace root (pnpm)
```

### Tooling
- **Package manager**: pnpm workspaces
- **Build**: tsup (fast TypeScript bundler)
- **Test**: Vitest
- **Lint**: ESLint + Prettier (Biome as alternative)
- **CI**: GitHub Actions

## Plan

- [ ] Initialize pnpm workspace with package structure
- [ ] Set up TypeScript config (base + per-package)
- [ ] Configure tsup for builds
- [ ] Configure Vitest for testing
- [ ] Set up ESLint + Prettier
- [ ] Create GitHub Actions CI pipeline
- [ ] Add development scripts (dev, build, test, lint)

## Test

- [ ] `pnpm install` succeeds
- [ ] `pnpm build` compiles all packages
- [ ] `pnpm test` runs all tests
- [ ] CI pipeline passes on clean checkout
