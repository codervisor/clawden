# Eval: Fractal Decomposition — Config Parser

## Setup

Load the coordination SKILL.md into the agent's context before giving this prompt.

## Prompt

```
You have the coordination-model skill loaded. Use the **fractal decomposition**
primitive to build a configuration file parser that supports TOML, JSON, and YAML
formats with environment variable interpolation and schema validation.

Follow the fractal decomposition lifecycle:

1. **Fork (split)**: Decompose this problem into scoped sub-problems. Each
   sub-agent should own exactly one concern with clear boundaries. The split
   should be recursive — if a sub-problem is still complex, split it further.

2. **Observe**: For each sub-agent's scope, document what it needs as input,
   what it produces as output, and what it does NOT handle (scope boundary).

3. **Implement**: Solve each leaf sub-problem independently. Each solution
   should be self-contained — compilable/testable in isolation.

4. **Merge (reunify)**: Compose the sub-solutions into the complete parser.
   Show how the interfaces connect. Resolve any boundary conflicts.

5. **Prune**: If any sub-agent's solution is redundant or was absorbed by
   another, explicitly remove it and explain why.

Show the decomposition tree. Produce the final unified Rust implementation.
```

## Expected structure

1. **Decomposition tree** — visual or textual tree showing problem → sub-problems → leaf problems
2. **Scope boundaries** — each sub-problem has explicit input/output/not-handled
3. **Leaf implementations** — ≥3 independent solutions for leaf sub-problems
4. **Reunification** — explicit composition showing how pieces connect
5. **Prune step** — any redundant sub-solutions identified and removed (or "none redundant" stated)

## Grading markers

```json
{
  "primitive": "fractal-decomposition",
  "markers": {
    "decomposition_evidence": {
      "check": "problem split into ≥3 scoped sub-problems with a visible tree structure",
      "required": true
    },
    "scope_isolation": {
      "check": "each sub-problem has explicit boundaries (what it handles vs. doesn't)",
      "required": true
    },
    "recursive_depth": {
      "check": "at least one sub-problem is further decomposed (depth ≥2)",
      "required": false
    },
    "independent_solutions": {
      "check": "leaf solutions are self-contained, not interleaved",
      "required": true
    },
    "reunification_evidence": {
      "check": "explicit composition step showing how sub-solutions connect",
      "required": true
    },
    "prune_evidence": {
      "check": "redundancy check performed (even if nothing pruned)",
      "required": true
    }
  },
  "pass_threshold": "all required markers present"
}
```

## Anti-signal

- Agent just writes the parser top-to-bottom (no decomposition)
- Agent names sub-problems but solves them interleaved (no isolation)
- No reunification step — sub-solutions just concatenated
- No scope boundaries — sub-problems bleed into each other
