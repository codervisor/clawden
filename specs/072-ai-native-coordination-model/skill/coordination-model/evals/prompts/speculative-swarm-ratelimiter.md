# Eval: Speculative Swarm — Rate Limiter

## Setup

Load the coordination SKILL.md into the agent's context before giving this prompt.

## Prompt

```
You have the coordination-model skill loaded. Use the **speculative swarm** primitive
to design a rate limiter module for an HTTP API server.

Follow the speculative swarm lifecycle:
1. **Fork**: Create at least 4 divergent design strategies (e.g., token bucket,
   sliding window, leaky bucket, fixed window with burst). Each strategy should be
   a separate, concrete implementation sketch — not just a name.
2. **Observe**: For each strategy, note its characteristics — throughput fairness,
   burst handling, memory usage, implementation complexity.
3. **Convergence**: Compare the strategies. Which ones are converging toward similar
   behavior? Which are genuinely different?
4. **Prune**: Eliminate the weakest strategies with explicit reasoning about why
   they lose.
5. **Merge**: Take the best fragments from surviving strategies and fuse them into
   a single final design. The merge should combine specific strengths — not just
   pick a winner.

Produce all 5 phases as separate sections in your output. The final merged design
should reference which fragments came from which strategy.

Write the final implementation as a Rust module (rate_limiter.rs).
```

## Expected structure

The agent's output should contain these phases as identifiable sections:

1. **Fork phase** — ≥4 strategy sketches, each meaningfully different
2. **Observe phase** — characteristic comparison (table or structured notes)
3. **Convergence phase** — similarity assessment between strategies
4. **Prune phase** — ≥1 strategy explicitly eliminated with reasoning
5. **Merge phase** — final design citing fragments from ≥2 surviving strategies

## Grading markers

```json
{
  "primitive": "speculative-swarm",
  "markers": {
    "fork_evidence": {
      "check": "≥4 distinct strategy sections with concrete implementation details",
      "required": true
    },
    "observe_evidence": {
      "check": "characteristic comparison across strategies (table, list, or structured comparison)",
      "required": true
    },
    "convergence_evidence": {
      "check": "explicit similarity/difference assessment between strategies",
      "required": true
    },
    "prune_evidence": {
      "check": "≥1 strategy explicitly eliminated with stated reasoning",
      "required": true
    },
    "merge_evidence": {
      "check": "final design cites specific fragments from ≥2 strategies",
      "required": true
    }
  },
  "pass_threshold": "all required markers present"
}
```

## Anti-signal (what failure looks like)

- Agent just picks token bucket and implements it (no fork)
- Agent lists 4 names but doesn't sketch each one (shallow fork)
- Agent compares strategies but picks one winner wholesale (no merge)
- Agent skips convergence/prune — goes straight from fork to "I'll combine them"
