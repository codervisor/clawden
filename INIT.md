# ClawLab — xxxClaw Orchestration Platform

**Project:** ClawLab

**Objective:** The unified orchestration platform for the xxxclaw ecosystem — deploy, manage, monitor, and coordinate heterogeneous AI agent runtimes (OpenClaw, ZeroClaw, PicoClaw, NanoClaw, IronClaw, NullClaw, and more) from a single control plane.

**Core Tech Stack:**

* **Language:** TypeScript / Node.js (pnpm monorepo)
* **API:** Fastify (REST) + WebSocket (real-time)
* **Dashboard:** React 19 + Tailwind CSS + shadcn/ui
* **Database:** SQLite (dev) → PostgreSQL (production)
* **Build:** tsup + Vitest + ESLint
* **Adapters:** Shell-out to native binaries + HTTP API integration

**Three Pillars:**

1. **Control Plane** — Unified lifecycle management (deploy, start, stop, restart, upgrade) with health monitoring, auto-recovery, and audit logging across all claw runtimes.
2. **Fleet Orchestration** — Agent discovery and registration, capability-based task routing, load balancing, cost optimization, and multi-agent swarm coordination.
3. **Developer Platform** — Cross-claw SDK (`@clawlab/sdk`), CLI (`clawlab`), skill testing harness, and skill marketplace for building plugins that work across runtimes.

**Architecture:**

```
┌──────────────────────────────────────────────────────────────┐
│                    ClawLab Dashboard (Web UI)                 │
├──────────────────────────────────────────────────────────────┤
│                     REST + WebSocket API                      │
├──────────────────────────────────────────────────────────────┤
│  Control Plane  │  Fleet Orchestration  │  Developer SDK     │
├──────────────────────────────────────────────────────────────┤
│           Claw Runtime Interface (CRI) — Adapter Layer       │
├──────┬──────┬──────┬──────┬──────┬──────┬──────┬─────────────┤
│  OC  │  ZC  │  PC  │  NC  │  IC  │ NuC  │ MiC  │  + more    │
└──────┴──────┴──────┴──────┴──────┴──────┴──────┴─────────────┘
```

**Monorepo Structure:**

```
clawlab/
├── packages/
│   ├── core/           # CRI interfaces, types, shared utilities
│   ├── control-plane/  # Lifecycle, health, recovery services
│   ├── fleet/          # Discovery, routing, swarm coordination
│   ├── config/         # Config schema, translators, secret vault
│   ├── api/            # REST + WebSocket API server
│   ├── dashboard/      # React web dashboard
│   ├── cli/            # clawlab CLI
│   └── sdk/            # @clawlab/sdk for skill developers
├── adapters/
│   ├── openclaw/       # @clawlab/adapter-openclaw
│   ├── zeroclaw/       # @clawlab/adapter-zeroclaw
│   ├── picoclaw/       # @clawlab/adapter-picoclaw
│   └── nanoclaw/       # @clawlab/adapter-nanoclaw
└── specs/              # LeanSpec specs
```

**Supported Claw Runtimes:**

| Runtime | Language | Stars | RAM | Startup | Adapter |
|---------|----------|-------|-----|---------|---------|
| OpenClaw | TypeScript | Massive | >1GB | >500s | REST API |
| ZeroClaw | Rust | 19.2k | <5MB | <10ms | REST + CLI |
| PicoClaw | Go | 19.9k | <10MB | <1s | REST + CLI |
| NanoClaw | TypeScript | 14.7k | Moderate | Moderate | Filesystem IPC |
| IronClaw | Rust | 3.5k | Low | Fast | REST + CLI |
| NullClaw | Zig | 2.2k | ~1MB | <2ms | REST + CLI |

**Inspiration:** Kubernetes CRI, Docker Compose, Terraform providers, OpenClaw Mission Control.
