---
status: planned
created: 2026-03-09
priority: medium
tags:
- fleet
- orchestration
- coordination
- ai-native
- innovation
parent: 091-fleet-coordination-optimization
depends_on:
  - 067-advanced-coordination-patterns
  - 074-coordination-model-design
created_at: 2026-03-09T06:25:01.610088324Z
updated_at: 2026-03-09T06:25:01.610088324Z
---

# AI-Native Coordination Primitives — ClawDen Implementation

## Overview

Spec 072 defines five AI-native coordination primitives (speculative swarm, context mesh, fractal decomposition, generative-adversarial, stigmergic) as implementation-agnostic algorithms, grounded in agent properties that have no human analogue. See spec 072 for the abstract model, the industrial-revolution analysis motivating the second-wave thesis, and the formal operation semantics (spawn/fork/merge/observe/convergence/prune).

This spec is **ClawDen's implementation** of those primitives — the `AINativeCoordination` Rust trait, `AgentEnvelope`-based message bus integration, `clawden.yaml` config schema, budget enforcement, and composability validation.

## Design

### ClawDen Trait Surface

The spec 067 `CoordinationPattern` trait assumes fixed agent rosters and message-based coordination. ClawDen extends it with `AINativeCoordination` to implement the six abstract operations from spec 072:

```rust
trait AINativeCoordination: CoordinationPattern {
    /// Dynamically spawn new agents mid-task (zero fork cost)
    fn spawn(&mut self, template: &AgentTemplate, context: ContextSnapshot) -> AgentId;
    /// Merge multiple agents' states into one (elastic identity)
    fn merge(&mut self, agents: &[AgentId], strategy: MergeStrategy) -> AgentId;
    /// Observe another agent's full internal state (perfect observability)
    fn observe(&self, agent: AgentId) -> Option<&AgentState>;
    /// Fork an agent into N copies with divergent parameters (speculative execution)
    fn fork(&mut self, agent: AgentId, variants: Vec<VariantConfig>) -> Vec<AgentId>;
    /// Detect convergence across parallel execution paths
    fn convergence(&self, agents: &[AgentId], threshold: f64) -> ConvergenceResult;
    /// Prune agents that are no longer contributing novel progress
    fn prune(&mut self, agents: &[AgentId], criterion: PruneCriterion) -> Vec<AgentId>;
}

enum MergeStrategy {
    /// Take the best parts from each agent's output (fragment fusion)
    FragmentFusion { scorer: Box<dyn Fn(&Fragment) -> f64> },
    /// One agent's state wins entirely (competitive selection)
    WinnerTakeAll { metric: QualityMetric },
    /// Weighted blend of all agents' states
    WeightedBlend { weights: Vec<(AgentId, f64)> },
}
```

### Primitive 1: Speculative Swarm

**What it is:** Fork N agents to explore different strategies for the same task *simultaneously*, with midpoint cross-pollination and convergence-based pruning. The final output is assembled from the best fragments across all surviving branches.

**Why it's impossible with humans:** Humans can brainstorm ideas, but they can't *execute* 5 strategies in parallel and then fuse the best parts of each execution. An agent can be forked, run divergently, observed, and merged — a human cannot.

**Mechanism:**
1. **Seed phase** — the coordinator forks the originating agent N times, each with a different strategy prompt (e.g., "solve via recursion", "solve via iteration", "solve via reduction").
2. **Exploration phase** — all forks execute independently. At configurable checkpoints, each fork's intermediate state is broadcast to all others (cross-pollination). Forks may incorporate useful fragments from siblings.
3. **Convergence detection** — the coordinator continuously measures output similarity. When two branches produce >threshold overlap, the lower-quality branch is pruned (freed).
4. **Fragment fusion** — surviving branches' outputs are decomposed into scored fragments. A merge agent assembles the final output by selecting the highest-scoring fragment for each sub-problem.

**Not a committee. Not an ensemble.** Committees discuss and vote on one solution. Ensembles average independent predictions. Speculative swarm *executes divergently and fuses selectively* — it produces outputs that no single agent could have produced alone.

```yaml
fleet:
  swarms:
    problem-solver:
      base_agent: solver
      strategies:
        - prompt_suffix: "approach via divide-and-conquer"
        - prompt_suffix: "approach via constraint propagation"
        - prompt_suffix: "approach via analogy from similar domains"
        - prompt_suffix: "approach via first principles decomposition"
      checkpoint_interval: 30s
      convergence_threshold: 0.85
      merge: fragment-fusion
      max_forks: 8
      budget: { max_tokens: 500000, max_cost_usd: 2.00 }
```

### Primitive 2: Context Mesh

**What it is:** A shared, reactive knowledge graph where agents observe knowledge gaps and fill them autonomously — no routing, no handoffs, no manager deciding who knows what.

**Why it's impossible with humans:** Human knowledge is opaque and lossy. You can't observe what another person knows, detect gaps in their understanding, or reactively push precisely the knowledge they're missing. Agents can.

**Mechanism:**
1. **Shared context graph** — a DAG where nodes are knowledge claims (facts, code artifacts, decisions) and edges are dependencies. Every agent reads and writes to the same graph.
2. **Gap detection** — agents continuously scan the graph for missing dependencies: "Node X depends on Y, but Y doesn't exist." Any agent with relevant capability can claim the gap.
3. **Reactive propagation** — when a node is filled or updated, all agents who depend on it are notified with the delta. No polling, no status meetings.
4. **Conflict resolution** — if two agents fill the same gap, a brief compete-and-compare selects the higher-confidence version. Unlike human turf wars, this takes milliseconds.

**Not departmental routing.** Departments gate knowledge through managers and gateway agents. Context mesh makes *all knowledge visible to all agents simultaneously* — coordination emerges from information availability, not organizational structure.

```yaml
fleet:
  mesh:
    context_graph:
      storage: shared-kv
      propagation: reactive
      conflict: compete-and-compare
    agents:
      - id: researcher
        watches: ["requirements.*", "constraints.*"]
        publishes: ["findings.*", "evidence.*"]
      - id: architect
        watches: ["findings.*", "constraints.*"]
        publishes: ["design.*", "interfaces.*"]
      - id: implementer
        watches: ["design.*", "interfaces.*"]
        publishes: ["code.*", "tests.*"]
```

### Primitive 3: Fractal Decomposition

**What it is:** An agent facing a complex task *splits itself* into scoped sub-agents, each inheriting the parent's full context but narrowing to a specific sub-problem. Sub-agents may recursively split further. On completion, they reunify into the original.

**Why it's impossible with humans:** Hierarchical delegation requires a manager to understand the problem well enough to decompose it and brief separate workers *who don't share the manager's full context*. In fractal decomposition, the agent IS the workers — it splits with full context preservation and reunifies losslessly. There's no brief, no handoff, no information loss at each hierarchical level.

**Mechanism:**
1. **Split** — agent analyzes its task and identifies N orthogonal sub-problems. It forks itself N times, each fork receiving the full parent context plus a scoping constraint ("you are responsible only for sub-problem K").
2. **Recursive depth** — each child may further split if its sub-problem is still complex. Depth is bounded by config.
3. **Reunification** — when all children complete, their outputs are merged back into the parent agent. Because children were forks of the parent (not strangers), reunification is lossless — the parent can integrate sub-results with full understanding of *why* each child made its choices.
4. **Scope isolation** — during split, children can only modify artifacts within their scoped sub-problem. This prevents conflicting writes without locks.

**Not hierarchical delegation.** Hierarchy has information loss at every level (manager briefs worker, worker briefs sub-worker). Fractal decomposition has *zero information loss* because the children ARE the parent.

```yaml
fleet:
  fractal:
    solver:
      base_agent: architect
      split_strategy: orthogonal-subproblems
      max_depth: 4
      max_children_per_level: 5
      reunification: lossless-merge
      scope_isolation: true
      budget: { max_total_agents: 20 }
```

### Primitive 4: Generative-Adversarial Coordination

**What it is:** Two agent roles — generator and critic — locked in an escalating quality loop. The critic doesn't just review; it *actively tries to break* the generator's output. The generator doesn't just fix; it *anticipates and preempts* the critic's attack patterns. Quality emerges from adversarial pressure, not checklist compliance.

**Why it's impossible with humans:** Human code review has social dynamics: reviewers don't want to seem hostile, authors get defensive, review depth is limited by time and cognitive load. Agents have no ego — the adversarial pressure can be maximally intense without social cost. Additionally, the critic can *execute* the generator's code and construct adversarial inputs automatically, not just read and comment.

**Mechanism:**
1. **Generate** — generator agent produces initial artifact (code, plan, document).
2. **Attack** — critic agent actively attempts to break it: generate adversarial inputs, find logical flaws, construct edge cases, attempt to violate stated invariants.
3. **Escalation** — each round, the critic's attack sophistication increases (simple edge cases → combinatorial inputs → adversarial optimization). The generator sees the full history of attacks and adapts.
4. **Termination** — the loop ends when: (a) the critic fails to find new issues for K consecutive rounds, (b) a quality score exceeds the threshold, or (c) max rounds reached.
5. **Progressive difficulty** — unlike human review where depth is roughly constant, the adversarial agent can increase its "effort budget" each round, going from surface-level to deep semantic analysis.

```yaml
fleet:
  adversarial:
    code-hardening:
      generator: coding-agent
      critic: adversarial-tester
      max_rounds: 10
      escalation: progressive
      termination:
        consecutive_clean_rounds: 2
        quality_threshold: 0.95
      critic_modes:
        - syntax-and-types
        - edge-cases
        - concurrency-safety
        - adversarial-inputs
```

### Primitive 5: Stigmergic Coordination

**What it is:** Agents coordinate through the shared artifact space rather than through messages. Like ants depositing pheromones: agents observe changes to shared artifacts and react to them. No central coordinator, no task queue, no explicit routing.

**Why it's impossible with humans:** Humans can't continuously monitor a codebase and react in real-time to every change. Agents can subscribe to artifact mutations and trigger automatically. Human stigmergy (leaving notes on a whiteboard) is lossy and slow; agent stigmergy is precise and instant.

**Mechanism:**
1. **Artifact observation** — every agent subscribes to a set of artifact patterns (files, code regions, knowledge graph nodes). Changes trigger the observer.
2. **Reactive production** — when an agent detects a relevant change, it *produces new artifacts* in response, which may trigger other agents.
3. **Pheromone markers** — agents tag artifacts with metadata (confidence, completeness, needs-review) that influence other agents' prioritization. Markers decay over time if not refreshed.
4. **Emergent workflow** — no predefined pipeline or task graph. The workflow emerges from agent reaction patterns. A coding agent produces code → a testing agent detects new untested code → a docs agent detects undocumented API → a security agent detects un-audited endpoints. Each reaction is autonomous.

**Not event-driven architecture.** Event-driven systems have predefined event types and handlers. Stigmergic coordination has agents that *autonomously decide* what artifact changes are relevant and what to do about them. The same artifact change might trigger different agents differently depending on their current state.

```yaml
fleet:
  stigmergic:
    agents:
      - id: implementer
        watches: ["specs/*.md", "design/*.md"]
        produces: ["src/**/*.rs"]
        markers: [confidence, completeness]
      - id: tester
        watches: ["src/**/*.rs"]
        produces: ["tests/**/*.rs"]
        markers: [coverage, edge-case-depth]
      - id: documenter
        watches: ["src/**/*.rs"]
        produces: ["docs/**/*.md"]
        markers: [completeness, accuracy]
      - id: security-auditor
        watches: ["src/**/*.rs"]
        produces: ["security/*.report"]
        markers: [threat-level, audit-depth]
    marker_decay: 3600s
    reaction_debounce: 5s
```

## Claude Code Implementation Notes

This section documents how each AI-native primitive can be implemented using Claude Code's native agent system. For conformance tier definitions and the full gap analysis, see spec 092.

### Primitive 1: Speculative Swarm — Claude Code

**Conformance tier:** ⚠️ Partial

**What works natively:**
- N variant prompts → N sequential `Agent` tool calls, each with a different strategy suffix
- Parent agent performs fragment fusion in its context window after all subagents complete
- Each subagent writes to its own scoped output file (e.g., `output-strategy-N.md`)

**What doesn't work:**
- Claude Code subagents run sequentially, not in parallel — losing the cost advantage of early pruning
- No mid-execution cross-pollination between branches
- No convergence-based pruning: all N branches run to completion regardless of similarity
- No typed `fragment-fusion` merge: parent must express the fusion strategy as natural language prompt instructions

**Canonical emulation pattern:**

```python
# Pseudocode — expressed as parent agent orchestration prompt
strategies = [
    "approach via divide-and-conquer",
    "approach via constraint propagation",
    "approach via first-principles decomposition",
]
outputs = []
for i, strategy in enumerate(strategies):
    result = spawn_subagent(
        prompt=f"{base_prompt}\n\nYour specific approach: {strategy}",
        output_file=f"output-swarm-{i}.md"
    )
    outputs.append(result)

# Fragment fusion in parent
synthesize(outputs, strategy="select best fragment per sub-problem section")
```

**When to use ClawDen fleet instead:** For latency-sensitive tasks or large N (>4 branches), use spec 068's `AINativeCoordination::fork()` through the ClawDen fleet layer, which achieves true parallelism.

---

### Primitive 2: Context Mesh — Claude Code

**Conformance tier:** ⚠️ Partial

**What works natively:**
- Shared filesystem IS the mesh substrate — all agents read/write the same workspace
- MCP servers (e.g., `@leanspec/mcp`) provide shared structured knowledge
- CLAUDE.md functions as a persistent, shared context declaration
- Agents can detect knowledge gaps via semantic search returning no results

**What doesn't work:**
- No reactive propagation — agent B is not notified when agent A writes a file
- Parent must explicitly orchestrate information flow: read updated files, inject into next subagent prompt
- No pheromone markers or confidence metadata on knowledge nodes
- No conflict resolution protocol — concurrent writes require manual management

**Canonical emulation pattern:**

Structure the knowledge graph as a directory of markdown files. Each subagent reads the full knowledge directory before producing output. Parent stitches updates manually:

```
knowledge/
  requirements.md     # produced by researcher agent
  constraints.md      # produced by researcher agent
  design.md           # produced by architect agent (reads requirements.md)
  interfaces.md       # produced by architect agent
  implementation.md   # produced by coder agent (reads design.md + interfaces.md)
```

---

### Primitive 3: Fractal Decomposition — Claude Code

**Conformance tier:** ⚠️ Partial (strongest AI-native fit)

**What works natively:**
- `Agent` tool natively models the split/reunify cycle
- Children have full workspace access (all files, all tools)
- Children can recursively spawn their own subagents (depth-bounded by token budget)
- Results reunify in parent via file artifacts + returned messages

**What doesn't work:**
- Children start from system-prompt context only, **not** from the parent's full mid-execution state
- This breaks the "lossless inheritance" guarantee: children receive a *textual snapshot* of context, not true state clone
- In practice: parent must carefully craft the child prompt to include all relevant context explicitly

**Canonical emulation pattern:**

```
Parent agent:
1. Analyze task, identify 3 orthogonal sub-problems: [auth, storage, API]
2. Write context-snapshot.md: full reasoning, constraints, decisions so far
3. Spawn child-auth with prompt: "[context-snapshot content]\nYou handle only: auth subsystem"
4. Spawn child-storage with prompt: "[context-snapshot content]\nYou handle only: storage subsystem"
5. Spawn child-api with prompt: "[context-snapshot content]\nYou handle only: API layer"
6. After all children complete, read their output files and reunify
```

**Depth limit:** Each level of recursion adds the full parent context to the child prompt. At depth 3–4, prompts become very large. Budget accordingly.

---

### Primitive 4: Generative-Adversarial — Claude Code

**Conformance tier:** ✅ Full (achievable with minimal adaptation)

**What works natively:**
- Generator + critic as separate `Agent` calls in a parent-managed loop
- No fatigue — each critic subagent is fresh and can apply maximum adversarial pressure
- Escalation implemented by enriching the critic's prompt each round with prior attack history
- Loop terminates when critic returns no new issues for K rounds, or `max_rounds` reached

**Canonical implementation:**

```
round = 0
critique_history = []
artifact = spawn_generator(base_prompt)

while round < max_rounds:
    critique = spawn_critic(
        artifact=artifact,
        history=critique_history,
        mode=escalation_modes[round]  # round 0: syntax, round 1: edge cases, etc.
    )
    if critique.is_clean():
        consecutive_clean += 1
        if consecutive_clean >= 2: break
    else:
        consecutive_clean = 0
        critique_history.append(critique)
        artifact = spawn_generator(artifact=artifact, critique=critique)
    round += 1
```

**No native escalation ladder:** The escalation mode progression (syntax → edge-cases → concurrency → adversarial-inputs) must be implemented as an explicit list in the parent orchestrator. There is no built-in mechanism for automatic escalation.

---

### Primitive 5: Stigmergic Coordination — Claude Code

**Conformance tier:** 🔧 Emulated

**What works natively:**
- Shared filesystem as pheromone surface — agents leave markers (status files, TODO comments, metadata files)
- Other agents can observe markers via `Grep`/`Glob` searches
- File modification timestamps serve as implicit pheromone decay approximation

**What doesn't work:**
- No reactive triggering — agents do not automatically activate when a file changes
- No debounce mechanism — parent must implement polling to avoid triggering storms
- No structured pheromone metadata (confidence, completeness decay rates)

**Canonical emulation pattern:**

```
# Marker format: .clawden-marker/<artifact-path>.json
{
  "agent": "tester",
  "marker": "coverage",
  "value": 0.72,
  "timestamp": "2026-03-10T00:00:00Z",
  "artifact": "src/auth.rs"
}

# Parent polling loop (imperative stigmergy):
while not_done:
    check_markers()
    dispatch_reactive_agents_based_on_markers()
    sleep(reaction_debounce)  # e.g., 5s
```

**For true reactive stigmergy:** Use ClawDen's fleet layer (spec 068), which provides the file-watch mechanism that triggers Claude Code agents on artifact changes without parent polling.

---

### Composability

These primitives compose per the rules defined in spec 072 Part 4. ClawDen enforces composability at config parse time — anti-patterns (swarm-in-swarm, adversarial-in-adversarial, stigmergic without debounce) are rejected with actionable error messages.

The `CoordinationPattern` trait from spec 067 remains the extension point — AI-native patterns implement a richer surface area.

## Notes

For the abstract primitive algorithms, composability rules, and the industrial-revolution analysis motivating this design, see **spec 072**.

This spec deliberately does not cover distributed execution of AI-native patterns. Running a speculative swarm across multiple hosts requires spec 062's remote control channel. That's a future extension — get the single-host primitives right first.

The boundary with spec 067: that spec owns the `CoordinationPattern` trait and organizational patterns (hierarchy, pipeline, committee, departmental, marketplace, matrix). This spec owns the `AINativeCoordination` extension and the five AI-native primitive implementations. Both share the same `AgentEnvelope` protocol and `MessageBus` from spec 065.

The boundary with spec 067 Phase 1: that spec owns the `CoordinationPattern` trait and organizational patterns (hierarchy, pipeline, committee, departmental, marketplace, matrix). This spec owns the `AINativeCoordination` extension and the five AI-native primitives. Both share the same `AgentEnvelope` protocol and `MessageBus` from spec 065.