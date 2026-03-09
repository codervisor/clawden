---
status: planned
created: 2026-03-09
priority: critical
tags:
- coordination
- ai-native
- model
- specification
- agnostic
created_at: 2026-03-09T13:42:39.548740Z
updated_at: 2026-03-09T13:42:39.548740Z
---

# AI-Native Agent Coordination Model — Implementation-Agnostic Specification

## Overview

This spec defines the **abstract coordination model** for AI agent fleets — the primitives, operations, patterns, and composition rules that any agent orchestration framework can implement. It is deliberately free of language bindings, wire formats, and runtime choices.

The ClawDen implementation of this model lives in specs 067–071 under the 054 umbrella. Other frameworks (Python, TypeScript, Go) can implement the same model independently.

### Why a Separate Model Spec?

Every technology revolution follows the same two-wave pattern:

| Revolution | First wave (old model + new tech) | Second wave (new model only possible with new tech) |
|---|---|---|
| Steam (1760s) | Steam pumps replacing hand-pumps in mines | Factory system — centralized production with power-driven machinery |
| Electricity (1880s) | Electric motors replacing steam belts in same factory layouts | Assembly line — unit drive motors at each station enable Ford Model T |
| Information (1970s) | Computerized paper forms, digital filing cabinets | Internet-native business — Amazon, Google, SaaS, the long tail |
| AI Agents (now) | **Agent fleets mimicking org charts** (hierarchy, departments, committees) | **AI-native primitives** — speculative swarm, context mesh, fractal decomposition, stigmergy |

The first wave maps old constraints onto new capabilities. The second wave discards inapplicable constraints and invents production models that are impossible without the new technology. Ford didn't just replace horses — he invented the assembly line, which only works because electric motors deliver power on demand at any point.

This spec defines the **second wave** for agent coordination. Separating it from any implementation ensures the model is portable, debatable on its own merits, and implementable by any framework.

## Design

### Part 1: Agent Properties That Enable New Coordination Models

Human coordination patterns exist because of human constraints. AI agents lack these constraints:

| Human constraint | Org pattern it created | AI agents don't have this constraint |
|---|---|---|
| Can't be cloned | Fixed team rosters, hiring | **Zero fork cost** — spawn/destroy agents freely |
| Communicate in lossy natural language | Meetings, reports, handoff docs | **Lossless context transfer** — share full state, not summaries |
| Fixed identities and skills | Job titles, departments, training | **Elastic identity** — mutate role/expertise instantly |
| Can only do one thing at a time | Sequential task assignment, scheduling | **Speculative parallelism** — execute N strategies, keep the best |
| Ego, status, politics | Management layers, conflict resolution | **No social overhead** — zero coordination tax |
| Tire and context-switch | Sprints, focus time, 8-hour days | **Tireless and stateless** — no degradation over time |
| Thought is opaque | Status meetings, standups, reports | **Perfect observability** — inspect any agent's full internal state |

Any agent coordination framework that can provide these properties can implement the primitives defined below. No specific language, transport, or storage technology is required.

### Part 2: Abstract Operations

Six abstract operations form the foundation of AI-native coordination. Any conforming runtime must support all six:

**`spawn(template, context) → agent_id`**
Create a new agent from a template with injected context. The agent is immediately available for coordination. Cost is near-zero compared to human onboarding.

**`fork(agent_id, variants) → [agent_id]`**
Clone an existing agent into N copies, each receiving the parent's full state plus a divergent parameter set. Copies execute independently. This is the foundational operation that enables speculative parallelism — impossible with humans because a person cannot be cloned mid-task.

**`merge(agent_ids, strategy) → agent_id`**
Combine multiple agents' states or outputs into a single result. Merge strategies include:
- *Fragment fusion* — decompose outputs into scored fragments, assemble the best from each source
- *Winner-take-all* — one agent's output wins based on a quality metric
- *Weighted blend* — combine proportionally based on confidence or quality scores

**`observe(agent_id) → agent_state`**
Inspect another agent's full internal state — context, reasoning trace, intermediate outputs. Unlike human communication (lossy summaries), observation is lossless and instantaneous.

**`convergence(agent_ids, threshold) → convergence_result`**
Measure output similarity across agents to detect when parallel paths are producing redundant work. Returns a convergence score and identifies redundant vs. divergent agents.

**`prune(agent_ids, criterion) → [pruned_ids]`**
Remove agents that are no longer contributing novel progress. Pruning criteria include: convergence-based (too similar to a better agent), budget-based (cost ceiling reached), quality-based (below minimum threshold).

### Part 3: Coordination Pattern Taxonomy

Coordination patterns fall into two categories, both built on the same abstract operations.

#### Category A: Organizational Patterns

These map real-world organizational structures onto agent fleets. They are the "first wave" — valuable, intuitive, but bounded by the human metaphors they borrow.

**Hierarchical Delegation** — Recursive leader-worker trees. Leaders decompose tasks, sub-leaders further decompose, leaf workers execute. Results aggregate upward.
- *Operations used*: spawn, observe
- *Key invariant*: depth-bounded to prevent unbounded recursion
- *When to use*: well-understood domain with clear decomposition

**Pipeline (Assembly Line)** — Sequential stages where output of one feeds the next. Each stage may use internal parallelism.
- *Operations used*: spawn
- *Key invariant*: stage failure halts downstream (configurable retry/skip)
- *When to use*: workflows with natural sequential dependencies

**Committee (Peer Consensus)** — Equal-rank agents deliberate in rounds. Each round, agents see all previous responses and may revise. Terminates on consensus, quorum, or max rounds.
- *Operations used*: spawn, observe
- *Key invariant*: convergence toward agreement or explicit max-rounds termination
- *When to use*: decisions requiring diverse perspectives and deliberation

**Departmental (Cross-Team Routing)** — Specialist teams with a top-level router directing task categories to the right department. Departments have independent internal coordination.
- *Operations used*: spawn, observe
- *Key invariant*: clear capability boundaries between departments
- *When to use*: broad-scope tasks requiring different specializations

**Marketplace (Task Bidding)** — Tasks posted on a board; qualified agents bid; allocation selects a winner. Self-organizing, no explicit leader.
- *Operations used*: spawn
- *Key invariant*: timeout on bids triggers re-auction
- *When to use*: heterogeneous agent capabilities with dynamic availability

**Matrix (Multi-Team Membership)** — Agents belong to multiple teams with declared capacity splits. Scheduling resolves conflicts via priority-based preemption.
- *Operations used*: spawn, observe
- *Key invariant*: no deadlocks via capacity reservation and timeout-based release
- *When to use*: agents with cross-cutting expertise

#### Category B: AI-Native Primitives

These exploit properties unique to AI agents. They have no human organizational analogue and produce outcomes impossible with human teams.

**Primitive 1: Speculative Swarm**

Fork N agents to explore different strategies simultaneously, with midpoint cross-pollination and convergence-based pruning. Final output is assembled from the best fragments across surviving branches.

*Mechanism:*
1. **Seed** — fork the originating agent N times, each with a different strategy parameter
2. **Explore** — all forks execute independently; at configurable checkpoints, each fork's intermediate state is broadcast to all others (cross-pollination); forks may incorporate useful fragments from siblings
3. **Converge** — continuously measure output similarity; when two branches exceed the overlap threshold, prune the lower-quality branch
4. **Fuse** — decompose surviving branches' outputs into scored fragments; a merge agent assembles the final output by selecting the highest-scoring fragment for each sub-problem

*Operations used*: fork, observe, convergence, prune, merge (fragment-fusion)

*Not a committee, not an ensemble.* Committees discuss and vote on one solution. Ensembles average independent predictions. Speculative swarm *executes divergently and fuses selectively* — it produces outputs no single agent could have produced alone.

*Configuration surface* (implementation-agnostic):
- `strategies`: list of divergent strategy parameters
- `checkpoint_interval`: how often cross-pollination occurs
- `convergence_threshold`: similarity score triggering pruning
- `merge_strategy`: fragment-fusion | winner-take-all | weighted-blend
- `budget`: max agents, max cost, max time

**Primitive 2: Context Mesh**

A shared reactive knowledge graph where agents observe knowledge gaps and fill them autonomously — no routing, no handoffs, no manager deciding who knows what.

*Mechanism:*
1. **Shared graph** — a DAG where nodes are knowledge claims (facts, artifacts, decisions) and edges are dependencies
2. **Gap detection** — agents scan for missing dependencies ("node X depends on Y, but Y doesn't exist"); any capable agent can claim the gap
3. **Reactive propagation** — when a node is filled or updated, all dependent agents receive the delta (no polling)
4. **Conflict resolution** — concurrent fills trigger compete-and-compare to select the higher-confidence version

*Operations used*: spawn, observe, merge (compete-and-compare)

*Not departmental routing.* Departments gate knowledge through managers. Context mesh makes all knowledge visible to all agents simultaneously — coordination emerges from information availability, not organizational structure.

*Configuration surface:*
- `graph_storage`: the backing store for the shared DAG (implementation chooses concrete backend)
- `propagation`: reactive | polling (reactive preferred)
- `conflict_strategy`: compete-and-compare | last-write-wins | manual
- `agent_subscriptions`: per-agent watch patterns (which graph regions each agent cares about)

**Primitive 3: Fractal Decomposition**

An agent facing a complex task splits itself into scoped sub-agents, each inheriting the parent's full context but narrowing to a specific sub-problem. Sub-agents may recursively split. On completion, they reunify into the original.

*Mechanism:*
1. **Split** — agent identifies N orthogonal sub-problems, forks itself N times; each fork receives full parent context plus a scoping constraint
2. **Recurse** — children may further split if their sub-problem exceeds complexity threshold; depth bounded by config
3. **Reunify** — children's outputs merge back into the parent losslessly; because children were forks (not strangers), the parent integrates with full understanding of each child's reasoning
4. **Scope isolation** — during split, children can only modify artifacts within their scoped sub-problem (prevents conflicting writes without locks)

*Operations used*: fork, observe, merge (lossless-merge), prune

*Not hierarchical delegation.* Hierarchy has information loss at every level (manager briefs worker). Fractal decomposition has zero information loss because the children ARE the parent.

*Configuration surface:*
- `split_strategy`: how to identify orthogonal sub-problems
- `max_depth`: maximum recursion depth
- `max_children_per_level`: fan-out limit
- `reunification`: lossless-merge | summary-merge
- `scope_isolation`: boolean (prevent cross-scope writes)

**Primitive 4: Generative-Adversarial Coordination**

Two agent roles — generator and critic — locked in an escalating quality loop. The critic actively tries to break the generator's output. Quality emerges from adversarial pressure, not checklist compliance.

*Mechanism:*
1. **Generate** — generator produces initial artifact
2. **Attack** — critic attempts to break it: adversarial inputs, logical flaws, edge cases, invariant violations
3. **Escalate** — each round, critic sophistication increases (surface → deep semantic analysis); generator sees full attack history and adapts
4. **Terminate** — when critic fails to find new issues for K rounds, quality score exceeds threshold, or max rounds reached

*Operations used*: spawn, observe

*Configuration surface:*
- `escalation_modes`: ordered list of increasing attack sophistication
- `max_rounds`: upper bound on adversarial cycles
- `termination`: consecutive clean rounds, quality threshold, or both
- `progressive_difficulty`: boolean (critic increases effort budget each round)

**Primitive 5: Stigmergic Coordination**

Agents coordinate through the shared artifact space rather than through messages. Like ants depositing pheromones: agents observe changes and react. No central coordinator, no task queue.

*Mechanism:*
1. **Artifact observation** — agents subscribe to artifact patterns; changes trigger the observer
2. **Reactive production** — on relevant change, agent produces new artifacts, which may trigger other agents
3. **Pheromone markers** — agents tag artifacts with metadata (confidence, completeness, needs-review) that influence others' prioritization; markers decay over time if not refreshed
4. **Emergent workflow** — no predefined pipeline; workflow emerges from agent reaction patterns

*Operations used*: observe, spawn

*Not event-driven architecture.* Event-driven systems have predefined handlers. Stigmergic agents autonomously decide what changes are relevant and how to respond.

*Configuration surface:*
- `agent_subscriptions`: per-agent artifact watch patterns and production targets
- `marker_types`: metadata tags agents can attach to artifacts
- `marker_decay`: time-to-live for pheromone markers
- `reaction_debounce`: minimum interval between reactions to prevent storms

### Part 4: Composability Rules

Primitives compose. The following compositions are well-defined:

| Outer | Inner | Result |
|---|---|---|
| Pipeline | Speculative swarm | Each pipeline stage explores strategies independently |
| Stigmergic | Fractal decomposition | An agent's reaction to an artifact change may trigger self-splitting |
| Speculative swarm | Generative-adversarial | Each swarm branch is adversarially hardened before fragment fusion |
| Context mesh | Speculative swarm | Mesh gap detection triggers swarm exploration of candidate fills |
| Fractal decomposition | Committee | Children deliberate before reunifying |

**Anti-patterns** (compositions that produce poor results):

| Composition | Why it fails |
|---|---|
| Swarm inside swarm | Exponential agent count; typically hits budget limits before producing value |
| Adversarial inside adversarial | Critic of a critic produces meta-critique without grounding; quality doesn't improve |
| Stigmergic with no debounce | Reaction storm — agents endlessly triggering each other |

### Part 5: Domain Playbook Schema

A **playbook** is a declarative composition of primitives for a concrete domain workflow. Playbooks are implementation-agnostic — they describe *what* coordination to apply, not *how* to wire it up.

```yaml
# Abstract playbook schema (no runtime-specific fields)
playbook:
  name: <string>
  domain: <string>
  description: <string>
  stages:
    - name: <string>
      primitive: <speculative-swarm | context-mesh | fractal-decomposition | generative-adversarial | stigmergic | hierarchical | pipeline | committee | departmental | marketplace | matrix>
      config:
        <primitive-specific configuration surface>
      trigger: manual | auto
      condition: <optional expression>
      lifecycle: one-shot | persistent
      budget:
        max_agents: <int>
        max_cost: <float>
        max_time: <duration>
  composition_rules:
    - outer: <stage_name>
      inner: <stage_name>
      binding: <how inner feeds outer>
```

**Reference playbook compositions** (domain-agnostic skeletons):

**Explore-Harden-Maintain** — for producing and sustaining high-quality artifacts:
1. Speculative swarm (divergent creation) → 2. Generative-adversarial (quality escalation) → 3. Stigmergic (continuous maintenance)

**Mesh-Fractal-Swarm** — for deep analysis of complex knowledge domains:
1. Context mesh (shared knowledge graph) → 2. Fractal decomposition (deep-dive into sub-problems) → 3. Speculative swarm (hypothesis generation)

**Swarm-Mesh-Stigmergy** — for incident response and reactive systems:
1. Speculative swarm (multi-hypothesis exploration) → 2. Context mesh (shared knowledge accumulation) → 3. Stigmergic (continuous monitoring)

### Part 6: Cost Optimization Model

AI-native coordination is economically viable only with cost optimization. The abstract cost model:

**Observation**: most agent work within a fleet is repetitive pattern execution, not novel reasoning.

**Teacher-student distillation** — a high-capability "teacher" model executes a task with full reasoning. Execution traces are captured and distilled into artifacts (skills, schemas, instructions) that enable a cheaper "student" model to replicate the behavior.

**Model tier abstraction:**
- *Frontier* — highest capability, highest cost (novel reasoning, creative exploration)
- *Mid-tier* — balanced capability/cost (moderate complexity tasks)
- *Student* — lowest cost, pattern replay (repetitive tasks with distilled skills)

**Scheduler model selection logic** (abstract):
1. Check skill registry for a matching `(role, primitive)` pair
2. If a distilled skill exists with sufficient quality → spawn student-tier agent
3. If no skill or quality below threshold → spawn frontier teacher, enable trace capture
4. For novel tasks → always teacher, always capture

**Cost reduction by primitive:**

| Primitive | Without distillation | With distillation | Reduction |
|---|---|---|---|
| Speculative swarm (8 forks) | 8× frontier | 1–2× frontier + 6× student | 60–80% |
| Stigmergic (5 watchers) | 5× frontier | 5× student (after training) | ~90% |
| Generative-adversarial (6 rounds) | 12× frontier | 2× frontier + 4× student | ~50% |
| Fractal (15 agents) | 15× frontier | 1–3× frontier + 12× student | ~70% |

### Part 7: Conformance Requirements

A runtime claiming to implement this coordination model must support:

1. **All six abstract operations** — spawn, fork, merge, observe, convergence, prune
2. **Dynamic agent lifecycle** — agents created and destroyed mid-task (not fixed rosters)
3. **State observability** — any agent's full state inspectable by the coordinator
4. **Budget enforcement** — hard limits on agent count, cost, and time
5. **Composable patterns** — primitives can be nested (inner/outer) without framework changes
6. **Trace capture** — execution traces for any agent are capturable for distillation
7. **Declarative playbooks** — playbook configurations can be loaded and executed without code changes

A runtime MAY additionally support:
- Distributed execution (agents across multiple hosts)
- Persistent state (surviving crashes)
- Hot-swappable coordination patterns (changing patterns mid-execution)

## Plan

- [ ] Validate the six abstract operations are sufficient for all five AI-native primitives
- [ ] Validate organizational patterns (hierarchy, pipeline, committee, departmental, marketplace, matrix) can be expressed using the same operation set
- [ ] Formalize composability rules with prohibited combinations
- [ ] Define the playbook schema with reference compositions
- [ ] Define the cost optimization model and model-tier abstraction
- [ ] Define conformance requirements for implementing runtimes
- [ ] Ensure ClawDen's implementation specs (067–071) reference this model correctly

## Test

- [ ] Each AI-native primitive's mechanism can be expressed purely in terms of the six abstract operations
- [ ] Each organizational pattern's mechanism can be expressed using the same operations (possibly a subset)
- [ ] The playbook schema can represent all six domain playbooks from spec 069 and all four SDD playbooks from spec 070
- [ ] Composability rules correctly identify the documented anti-patterns
- [ ] The conformance requirements are satisfiable by at least one concrete runtime architecture (ClawDen)
- [ ] The cost optimization model is coherent without reference to any specific distillation tool

## Notes

This spec is intentionally free of:
- Programming language (no Rust traits, no TypeScript interfaces)
- Wire format (no JSON-Lines, no AgentEnvelope, no gRPC)
- Storage backend (no SQLite, no Redis)
- Configuration format (YAML examples are illustrative, not normative)
- Specific LLM providers or model names

The ClawDen implementation lives in specs 067–071 under the 054 umbrella. Those specs translate this abstract model into concrete Rust traits, an AgentEnvelope wire protocol, SQLite persistence, and `clawden.yaml` configuration.

Other implementations are possible and encouraged. The abstract operations, primitive algorithms, composability rules, and conformance requirements are the stable contract; everything else is an implementation choice.