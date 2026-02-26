# clawlab

## Overview

ClawLab is the **unified orchestration platform for the xxxclaw ecosystem**. It provides a single control plane to deploy, manage, monitor, and coordinate heterogeneous AI agent runtimes — OpenClaw, ZeroClaw, PicoClaw, NanoClaw, IronClaw, NullClaw, and community variants.

## Skills

This project uses the Agent Skills framework for domain-specific guidance.

### leanspec-sdd - Spec-Driven Development

- **Location**: See your skills directory (installed via `lean-spec skill install`, e.g., `.github/skills/leanspec-sdd/SKILL.md`)
- **Use when**: Working with specs, planning features, multi-step changes
- **Key principle**: Run `board` or `search` before creating specs

Read the skill file for complete SDD workflow guidance.

## Architecture

ClawLab is organized as a pnpm monorepo with three pillars:

1. **Control Plane** (`packages/control-plane`) — Agent lifecycle, health monitoring, auto-recovery
2. **Fleet Orchestration** (`packages/fleet`) — Discovery, task routing, swarm coordination
3. **Developer Platform** (`packages/sdk`, `packages/cli`) — Cross-claw skill SDK and CLI

All communication with claw runtimes goes through the **Claw Runtime Interface** (`packages/core`) — an adapter pattern where each runtime has a pluggable driver.

## Project-Specific Rules

- TypeScript throughout, strict mode
- pnpm workspaces for monorepo management
- Adapters live in `adapters/` directory (one per claw runtime)
- Specs follow LeanSpec SDD workflow
- All lifecycle events must be audit-logged
- Secrets are never stored in plain text
