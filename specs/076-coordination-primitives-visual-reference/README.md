---
status: planned
created: 2026-03-09
priority: medium
tags:
- coordination
- ai-native
- model
- reference
- visual
parent: 072-ai-native-coordination-model
created_at: 2026-03-09T14:50:13.395487Z
updated_at: 2026-03-09T14:50:13.395487Z
---

# Coordination Primitives Visual Reference

## Overview

Visual reference for all 11 coordination primitives — 5 AI-native (Category B) and 6 organizational (Category A). Each primitive shows the agent flow, operations used, and key structural insight.

This spec is a reference companion to spec 073 (theory). No implementation, no schemas — just diagrams.

## Design

### Category B: AI-Native Primitives

#### Speculative Swarm

Fork N divergent strategies from a single agent, cross-pollinate insights, measure convergence, prune losers, fuse the best fragments.

```mermaid
flowchart LR
    A[Agent] -->|fork| B1[Branch 1]
    A -->|fork| B2[Branch 2]
    A -->|fork| B3[Branch 3]
    A -->|fork| B4[Branch N]
    B1 & B2 & B3 & B4 -->|observe| CONV{convergence}
    CONV -->|prune| X1[❌ B2]
    CONV -->|prune| X2[❌ B4]
    B1 & B3 -->|merge| R[Fused result]

    style A fill:#2d6a4f,color:#fff
    style CONV fill:#354f52,color:#fff
    style R fill:#1b4965,color:#fff
    style X1 fill:#832f2f,color:#fff
    style X2 fill:#832f2f,color:#fff
```

**Key insight**: The result contains fragments from multiple branches — not "best branch wins" but "best pieces from each branch fused together."

#### Context Mesh

Shared knowledge DAG where any agent's discovery is instantly available to all others. Knowledge gaps trigger reactive spawning.

```mermaid
flowchart TD
    K[Knowledge DAG]
    A1[Agent 1] -->|observe| K
    A2[Agent 2] -->|observe| K
    A3[Agent 3] -->|observe| K
    A1 -->|merge| K
    A2 -->|merge| K
    K -->|gap detected| S{spawn}
    S --> A3

    style K fill:#1b4965,color:#fff
    style S fill:#2d6a4f,color:#fff
```

**Key insight**: Coordination is implicit — agents read/write a shared DAG. No point-to-point messages. Gap detection drives agent creation.

#### Fractal Decomposition

An agent splits itself into scoped sub-agents recursively. Each child inherits full context and specializes on a subset.

```mermaid
flowchart TD
    A[Agent] -->|fork| C1[Child: scope A]
    A -->|fork| C2[Child: scope B]
    C1 -->|fork| G1[Grandchild: A.1]
    C1 -->|fork| G2[Grandchild: A.2]
    G1 & G2 -->|merge| C1R[Result A]
    C2 -->|merge| C2R[Result B]
    C1R & C2R -->|merge| R[Final result]

    A -.->|observe + prune| C1 & C2

    style A fill:#2d6a4f,color:#fff
    style R fill:#1b4965,color:#fff
```

**Key insight**: Recursion depth is dynamic — each child decides whether to split further based on scope complexity. Prune collapses branches that converge early.

#### Generative-Adversarial

Generator and critic agents escalate quality in a tight loop. Critic difficulty increases each round until quality threshold is met.

```mermaid
flowchart LR
    G[Generator] -->|produce| A[Artifact v1]
    A --> C[Critic]
    C -->|observe| G
    G -->|improve| A2[Artifact v2]
    A2 --> C
    C -->|observe| G
    G -->|improve| AN[Artifact vN ✓]

    style G fill:#2d6a4f,color:#fff
    style C fill:#832f2f,color:#fff
    style AN fill:#1b4965,color:#fff
```

**Key insight**: Only two agents, but round count is unbounded. Termination is quality-driven (consecutive clean rounds), not time-driven.

#### Stigmergic

Agents coordinate purely through shared artifact changes — like ants leaving pheromone trails. No direct messaging.

```mermaid
flowchart TD
    ENV[Shared artifacts]
    A1[Agent 1] -->|"change artifact"| ENV
    ENV -->|"observe change"| A2[Agent 2]
    A2 -->|"react: change artifact"| ENV
    ENV -->|"observe change"| A3[Agent 3]
    A3 -->|"react: spawn"| A4[Agent 4]

    ENV -.->|"marker decay"| D["🕐 Stale markers expire"]

    style ENV fill:#1b4965,color:#fff
    style D fill:#52796f,color:#fff
```

**Key insight**: Coordination cost is O(artifacts), not O(agents²). Debounce is structurally required to prevent reaction storms.

### Category A: Organizational Primitives

#### Hierarchical

Manager delegates to workers, observes results, aggregates.

```mermaid
flowchart TD
    M[Manager] -->|spawn| W1[Worker 1]
    M -->|spawn| W2[Worker 2]
    M -->|spawn| W3[Worker 3]
    W1 & W2 & W3 -->|observe| M

    style M fill:#354f52,color:#fff
```

#### Pipeline

Sequential stages — output of one becomes input of the next.

```mermaid
flowchart LR
    S1[Stage 1] -->|spawn| S2[Stage 2]
    S2 -->|spawn| S3[Stage 3]
    S3 -->|spawn| S4[Stage N]

    style S1 fill:#354f52,color:#fff
    style S4 fill:#354f52,color:#fff
```

#### Committee

Peers deliberate independently, then vote on the outcome.

```mermaid
flowchart TD
    CHAIR[Chair] -->|spawn| P1[Peer 1]
    CHAIR -->|spawn| P2[Peer 2]
    CHAIR -->|spawn| P3[Peer 3]
    P1 & P2 & P3 -->|observe: vote| CHAIR
    CHAIR --> V[Decision by majority]

    style CHAIR fill:#354f52,color:#fff
    style V fill:#1b4965,color:#fff
```

#### Departmental

Functional groups work in parallel, with cross-group sync points.

```mermaid
flowchart TD
    COORD[Coordinator] -->|spawn| D1[Dept: Frontend]
    COORD -->|spawn| D2[Dept: Backend]
    COORD -->|spawn| D3[Dept: QA]
    D1 & D2 -->|observe: sync| COORD
    COORD -->|observe: handoff| D3

    style COORD fill:#354f52,color:#fff
```

#### Marketplace

Tasks posted to a shared queue. Agents bid or claim based on capability.

```mermaid
flowchart LR
    Q[Task queue] -->|spawn: claim| A1[Specialist A]
    Q -->|spawn: claim| A2[Specialist B]
    Q -->|spawn: claim| A3[Generalist C]

    style Q fill:#354f52,color:#fff
```

#### Matrix

Dual reporting — agents serve both a functional specialty and a project goal.

```mermaid
flowchart TD
    FUNC[Function lead] -->|spawn| A1[Agent]
    PROJ[Project lead] -->|spawn| A1
    FUNC -->|observe| A1
    PROJ -->|observe| A1
    A1 --> R[Serves both]

    style FUNC fill:#354f52,color:#fff
    style PROJ fill:#52796f,color:#fff
    style R fill:#1b4965,color:#fff
```

## Plan

- [x] Diagram all 5 AI-native primitives with flow + key insight
- [x] Diagram all 6 organizational primitives

## Test

- [ ] Every diagram renders correctly in Mermaid
- [ ] Every primitive from spec 073 is represented