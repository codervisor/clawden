# Eval: Context Mesh — Codebase Research

## Setup

Load the coordination SKILL.md into the agent's context before giving this prompt.

## Prompt

```
You have the coordination-model skill loaded. Use the **context mesh** primitive
to research and document the error handling patterns across a Rust web application
that uses axum, sqlx, and tower.

Follow the context mesh lifecycle:

1. **Spawn knowledge nodes**: Create initial knowledge nodes for each domain:
   - axum error handling (extractors, handlers, middleware)
   - sqlx error handling (query errors, connection pool, migrations)
   - tower error handling (service errors, layer composition, timeouts)

2. **Observe & link**: For each knowledge node, identify what information it
   references from other nodes. Build a knowledge DAG showing dependencies
   (e.g., "axum handler errors often wrap sqlx errors").

3. **Gap detection**: Identify gaps in the knowledge graph — questions that
   arise from the connections but aren't answered by any node. Examples:
   - "How do tower timeout errors surface through axum?"
   - "What happens when sqlx connection pool exhaustion meets axum's backpressure?"

4. **Reactive spawning**: For each gap detected, spawn a focused research node
   that fills exactly that gap. Show what triggered the spawn.

5. **Merge**: Synthesize all nodes (original + gap-fillers) into a unified
   error handling guide with cross-references.

Produce the knowledge DAG (as mermaid or text), the gap analysis, and the
final unified guide.
```

## Expected structure

1. **Knowledge nodes** — ≥3 domain-specific knowledge blocks
2. **Knowledge DAG** — visual or described dependency links between nodes
3. **Gap detection** — ≥2 specific knowledge gaps identified from cross-node analysis
4. **Reactive spawns** — new nodes created specifically to fill detected gaps
5. **Merged guide** — unified output with cross-references to source nodes

## Grading markers

```json
{
  "primitive": "context-mesh",
  "markers": {
    "spawn_evidence": {
      "check": "≥3 distinct knowledge nodes with domain-specific content",
      "required": true
    },
    "dag_evidence": {
      "check": "explicit dependency/reference links between knowledge nodes",
      "required": true
    },
    "gap_detection": {
      "check": "≥2 knowledge gaps identified from cross-node analysis (not pre-known)",
      "required": true
    },
    "reactive_spawn": {
      "check": "new knowledge nodes created specifically to fill detected gaps",
      "required": true
    },
    "merge_evidence": {
      "check": "unified output that cross-references source nodes",
      "required": true
    }
  },
  "pass_threshold": "all required markers present"
}
```

## Anti-signal

- Agent writes a monolithic error handling guide (no nodes, no DAG)
- Agent creates nodes but doesn't link them (no mesh)
- No gap detection — agent only writes what it already knows
- Reactive spawns aren't actually triggered by gaps — just more content
