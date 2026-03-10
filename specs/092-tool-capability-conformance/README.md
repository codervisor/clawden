---
status: planned
created: 2026-03-10
priority: high
tags:
- coordination
- ai-native
- conformance
- claude-code
- codex-cli
- tool-binding
parent: 089-coordination-model-theory-implementation
depends_on:
- 073-coordination-model-theory
- 074-coordination-model-design
- 075-coordination-model-validation-distribution
created_at: 2026-03-10T00:00:00Z
updated_at: 2026-03-10T00:00:00Z
---

# Tool Capability Conformance Layer — Mapping AI Coding Tools to the Coordination Model

## Overview

The coordination model (specs 072–085) defines an ideal abstract layer: 6 operations, 11 primitives, composability rules, and a formal algebra. This spec addresses a practical question: **how well do mainstream AI coding tools actually implement that model today?**

This is a *conformance layer*, not a critique. The goal is to give ClawDen operators a precise map of which operations are native, which can be emulated, and which require ClawDen fleet orchestration to bridge. Without this layer, operators building on top of Claude Code or Codex CLI will independently rediscover the same gaps and write ad-hoc workarounds.

Two tools are assessed:

- **Claude Code** — Anthropic's agentic coding CLI with native subagent spawning, worktree isolation, MCP integration, and skill system. See also spec 086.
- **OpenAI Codex CLI** — OpenAI's experimental multi-agent coding tool with role-based orchestration and MCP server exposure.

---

## Conformance Tier Definitions

Each operation and primitive is rated on a four-tier scale:

| Tier | Symbol | Meaning |
|------|--------|---------|
| **Full** | ✅ | Operation is natively supported with spec-compliant semantics |
| **Partial** | ⚠️ | Operation is supported but with reduced semantics or manual orchestration overhead |
| **Emulated** | 🔧 | Operation has no native support; a documented emulation pattern exists |
| **Unsupported** | ❌ | No native support and no viable emulation in single-tool context |

---

## Abstract Operations — Conformance Table

### Claude Code

| Operation | Tier | Native Mechanism | Gap Description |
|-----------|------|-----------------|-----------------|
| `spawn` | ✅ Full | `Agent` tool with `subagent_type`, `prompt`, `run_in_background` | Minor: no typed template registry; prompts are free-form |
| `fork` | ⚠️ Partial | `isolation: "worktree"` for file isolation; multiple parallel `Agent` calls for prompt variants | No true state-clone — children always start fresh from system prompt context, not mid-execution state |
| `observe` | ⚠️ Partial | Lead observes teammates via task lists, idle notifications, and returned messages | No mid-execution state inspection API; only final message is returned |
| `convergence` | 🔧 Emulated | Post-hoc: parent agent compares subagent output artifacts for similarity | No semantic convergence threshold during execution; requires explicit artifact comparison after completion |
| `prune` | 🔧 Emulated | `maxTurns` as a turn budget; scoped task descriptions limit runaway agents | No mechanism to terminate a running subagent mid-task; must pre-scope or rely on timeout |
| `merge` | ⚠️ Partial | Parent agent synthesizes results in its context window (NL fragment fusion) | No typed merge strategies (fragment-fusion, winner-take-all, weighted-blend) — always implicit NL synthesis |

### OpenAI Codex CLI

| Operation | Tier | Native Mechanism | Gap Description |
|-----------|------|-----------------|-----------------|
| `spawn` | ⚠️ Partial | Role-based agent spawning via orchestrator; experimental multi-agent mode | Less configurable than Claude Code; fewer subagent_type options |
| `fork` | ❌ Unsupported | No state-clone; agents always spawn fresh | No worktree isolation equivalent documented |
| `observe` | ❌ Unsupported | Thread inspection only; no agent state API | No persistent memory between invocations |
| `convergence` | ⚠️ Partial | Wait-for-all in batch CSV workflows | Convergence is completion-based, not semantic |
| `prune` | ❌ Unsupported | No mid-execution termination | Must use timeout or max-turn limits if available |
| `merge` | ⚠️ Partial | CSV result consolidation in batch mode | Implicit NL merge only; no typed strategies |

---

## Coordination Primitives — Conformance Table

### Category A: Organizational Primitives

| Primitive | Claude Code Tier | Claude Code Mechanism | Codex CLI Tier |
|-----------|-----------------|----------------------|----------------|
| Hierarchical | ✅ Full | Agent Teams with lead + typed subagent roles | ⚠️ Partial — all routing through orchestrator, no direct peer messaging |
| Pipeline | ✅ Full | Sequential `Agent` calls where each receives prior output | ⚠️ Partial — sequential instruction routing; no dependency graph |
| Committee | ✅ Full | Multiple subagents analyze same input; parent synthesizes | ⚠️ Partial — achievable but undocumented pattern |
| Departmental | ✅ Full | Subagents scoped by domain; parent handles cross-domain sync | ⚠️ Partial — role-based dispatch is supported |
| Marketplace | ⚠️ Partial | Parent assesses task and dispatches to named agent; no dynamic agent discovery | ❌ Unsupported |
| Matrix | ⚠️ Partial | Dual-scoped prompts (functional + project); no native matrix routing | ❌ Unsupported |

### Category B: AI-Native Primitives

| Primitive | Claude Code Tier | Gap Summary | Codex CLI Tier |
|-----------|-----------------|-------------|----------------|
| Speculative Swarm | ⚠️ Partial | Serial (not parallel) execution; no fork-from-state; no convergence-based pruning; no fragment-fusion typed strategy | ❌ Unsupported |
| Context Mesh | ⚠️ Partial | Shared filesystem IS a mesh substrate; but no reactive DAG, no gap-filling semantics, no pheromone propagation | ❌ Unsupported |
| Fractal Decomposition | ⚠️ Partial | `runSubagent` models split well; but children start from system-prompt context only (not full parent state), breaking lossless inheritance | ❌ Unsupported |
| Generative-Adversarial | ✅ Full | Generator + critic loop in parent; escalation via enriched prompts; no fatigue; no native escalation ladder | ⚠️ Partial — achievable with orchestrator |
| Stigmergic | 🔧 Emulated | Shared filesystem as pheromone surface; markers via files/comments; but no reactive triggers — parent must poll or dispatch | ❌ Unsupported |

---

## Additional Capability Conformance

| Spec Requirement | Claude Code | Codex CLI |
|-----------------|-------------|-----------|
| JSON Schema playbooks | ❌ None — orchestration is imperative in prompts | ❌ None |
| Budget enforcement (tokens/cost/time) | ⚠️ Partial — per-agent model selection + `maxTurns`; no first-class cost budget | ⚠️ Partial — model + sandbox restrictions only |
| Anti-pattern validation at parse time | ❌ None | ❌ None |
| Artifact versioning + fragment model | ❌ None — git serves as artifact store but no fragment tracking | ❌ None |
| Behavioral evals compatibility | ✅ Full — `evals.json` format works with Claude API | ⚠️ Unknown |
| MCP integration | ✅ Full — OAuth, dynamic tools, project/user scope | ✅ Full — can expose itself as MCP server |
| Persistent memory / skill system | ✅ Full — CLAUDE.md + markdown skills with YAML frontmatter | ❌ Not documented |
| Cost tiering (frontier/mid/student) | ⚠️ Partial — Opus/Sonnet/Haiku maps loosely; no automatic routing | ⚠️ Partial — model selection only |

---

## Canonical Emulation Patterns

For operations with **Emulated** or **Partial** tier, these patterns bridge the gap when using Claude Code directly (without full ClawDen fleet orchestration).

### Emulating `fork` (state-clone)

The spec-073 `fork` operation clones an agent from its current mid-execution state. Claude Code cannot do this — children always start fresh. Canonical emulation:

```
1. Parent agent snapshots its current context as a structured artifact (e.g., context-snapshot.md)
2. Parent spawns N child agents, each receiving:
   - The full context snapshot in their prompt
   - A distinct variant instruction (divergent parameter)
3. Children operate as if they inherited parent state
```

**Limitation:** Children receive a *textual representation* of state, not true state inheritance. This breaks if parent state includes implicit reasoning not captured in the snapshot.

**Claude Code mapping:**
```
Agent tool call with prompt = "[context snapshot content]\n\nYour variant: [strategy N]"
isolation: "worktree"  # for file-level isolation
```

### Emulating `observe` (mid-execution state)

The spec-073 `observe` operation reads an agent's full internal state non-destructively. Claude Code returns only final messages. Canonical emulation:

```
1. Design subagents to emit structured progress artifacts to disk (progress.json, status.md)
2. Parent polls shared filesystem between subagent completions
3. For real-time observation: use background subagents that write incremental artifacts
```

**Claude Code mapping:**
```
Subagent writes progress files → parent reads via Glob/Grep
run_in_background: true enables parent to observe workspace changes while subagent runs
```

### Emulating `convergence` (semantic threshold)

The spec-073 `convergence` operation detects semantic similarity across parallel branches at configurable thresholds. Canonical emulation:

```
1. All branches write outputs to canonical artifact paths (output-branch-N.md)
2. After all branches complete, spawn a convergence-checker subagent
3. Convergence-checker compares outputs semantically and returns similarity scores
4. Parent uses scores to decide which branches to synthesize
```

**Limitation:** This is post-hoc, not real-time. Branches cannot be pruned mid-execution based on early convergence.

### Emulating `prune` (mid-execution termination)

The spec-073 `prune` operation terminates agents that are no longer adding value. Claude Code runs subagents to completion. Canonical emulation:

```
1. Use maxTurns to set an upper bound on each subagent's execution
2. Scope the subagent task tightly so natural completion approximates pruning
3. Use background execution + artifact polling: if branch N hasn't produced progress artifacts
   within T seconds, do not wait for it or incorporate its output
```

**Limitation:** This is a blunt proxy. Branches run to completion (consuming tokens) even if their output will be discarded.

### Emulating `merge` (typed strategies)

The spec-073 `merge` operation supports typed strategies: fragment-fusion, winner-take-all, weighted-blend. Claude Code's parent performs NL synthesis implicitly. Canonical emulation:

```
Fragment-fusion:
  Parent prompt: "You have outputs from 3 agents. For each section [A, B, C], select the
  best-quality version and cite which agent produced it. Combine into a unified document."

Winner-take-all:
  Parent prompt: "Evaluate outputs from 3 agents on criteria [X, Y, Z]. Select the single
  best output and explain why it wins on the most important criteria."

Weighted-blend:
  Parent prompt: "Combine outputs from 3 agents with emphasis: Agent1 (architecture) = 50%,
  Agent2 (tests) = 30%, Agent3 (docs) = 20%. Apply weights to section selection."
```

---

## Where ClawDen Fleet Fills the Gaps

When Claude Code agents run inside a ClawDen fleet (spec 068), the orchestration gaps are resolved at the fleet layer:

| Claude Code Gap | ClawDen Fleet Bridge |
|----------------|---------------------|
| No parallel execution | ClawDen spawns multiple Claude Code processes concurrently |
| No mid-execution observation | ClawDen monitors workspace artifacts as proxy state |
| No semantic convergence | ClawDen compares outputs across parallel Claude Code agents |
| No prune operation | ClawDen terminates underperforming Claude Code processes |
| No reactive artifact triggers | ClawDen file-watcher spawns new agents on artifact changes |
| No persistent agent identity | ClawDen maintains agent registry with session continuity |
| No declarative playbooks | ClawDen interprets `clawden.yaml` playbook format |
| No anti-pattern validation | ClawDen validates playbook composition at parse time |

---

## Alignment Summary

| Tool | Overall Alignment | Strong Areas | Weak Areas |
|------|------------------|-------------|------------|
| Claude Code | ~45% | spawn, hierarchical, pipeline, MCP, skills, evals | fork semantics, prune, typed merge, declarative playbooks, anti-pattern validation |
| Codex CLI | ~20% | MCP server exposure, basic spawn | fork, observe, prune, all AI-native primitives, persistent memory |

Neither tool implements the formal coordination algebra natively. Both require ClawDen fleet orchestration to achieve full conformance with specs 072–085.

---

## Plan

- [x] Define four-tier conformance scale (Full / Partial / Emulated / Unsupported)
- [x] Map 6 abstract operations for Claude Code and Codex CLI
- [x] Map 11 primitives (6 organizational + 5 AI-native) per tool
- [x] Map additional capabilities (playbooks, budgets, MCP, skills, evals)
- [x] Document canonical emulation patterns for Emulated/Partial operations
- [x] Document ClawDen fleet bridge table
- [ ] Validate conformance tier assignments against one live eval run (speculative-swarm-ratelimiter.md)
- [ ] Update `conformance.schema.json` with `tool_bindings` field (see spec 074 update)

## Test

- [ ] Run `speculative-swarm-ratelimiter.md` eval using Claude Code Agent tool system; record which operations succeed natively vs. require emulation vs. fail
- [ ] Verify tier assignments match observed eval behavior
- [ ] Confirm emulation patterns are implementable without ClawDen fleet layer
- [ ] Codex CLI: re-assess once multi-agent mode exits experimental status

## Notes

This spec should be updated whenever Claude Code or Codex CLI release new multi-agent capabilities. The most likely near-term changes:

- Claude Code gaining native parallel subagent execution → would upgrade `speculative-swarm` from Partial to Full
- Claude Code gaining mid-execution streaming → would upgrade `observe` from Partial to Full
- Either tool adopting declarative playbook format → would add Partial tier for `declarative_playbooks`

The boundary with spec 086 (Claude Code Coordination Implementation): spec 086 defines *how* Claude Code implements each primitive in detail. This spec provides the *conformance tier rating* and *emulation patterns* for operators who need to understand gaps without reading the full implementation spec.
