---
status: planned
created: 2026-03-10
priority: medium
tags:
- coordination
- ai-native
- primitives
- group
- umbrella
parent: 072-ai-native-coordination-model
created_at: 2026-03-10T08:41:52.765819229Z
updated_at: 2026-03-10T08:41:52.765819229Z
---

# Coordination Model — Primitive Deep Dives

## Overview

Group spec for all per-primitive analysis and reference material under the coordination model.

Collects deep dives for the 5 AI-native primitives, the 6 organizational primitives, and the visual reference diagram spec. Each child is independently workable once the core theory (073) is complete.

## Design

| Child | Scope |
|-------|-------|
| `076-coordination-primitives-visual-reference` | Visual diagrams of all 11 primitives — agent flow, operations used, key structural insight |
| `078-speculative-swarm-primitive` | Operation lifecycle, config surface, merge strategies, composability, failure modes |
| `079-context-mesh-primitive` | Knowledge DAG structure, gap detection, conflict resolution, reactive propagation |
| `080-fractal-decomposition-primitive` | Scope isolation, recursive splitting, reunification, depth management |
| `081-generative-adversarial-primitive` | Escalation ladder, generator-critic contract, termination conditions |
| `082-stigmergic-coordination-primitive` | Pheromone markers, debounce, O(artifacts) cost model, emergent workflow |
| `083-organizational-coordination-primitives` | All 6 Category A primitives — config, failure modes, cross-category composition |

## Plan

- [ ] Complete 076 (visual reference) to establish diagrammatic language
- [ ] Complete 078–082 (AI-native primitives) — parallelizable
- [ ] Complete 083 (organizational primitives)

## Test

- [ ] All 7 child specs complete
- [ ] Each primitive deep dive covers: config surface, failure modes, composability, cost model impact