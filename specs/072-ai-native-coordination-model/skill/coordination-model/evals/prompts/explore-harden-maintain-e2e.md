# Eval: Explore-Harden-Maintain — Full Playbook E2E

## Setup

Load the coordination SKILL.md into the agent's context before giving this prompt.

## Prompt

```
You have the coordination-model skill loaded. Execute the **explore-harden-maintain**
playbook to design and implement a CLI argument parser library.

This playbook has 3 stages. Execute them in order:

### Stage 1: Explore (speculative swarm)

Fork ≥3 divergent design strategies for a CLI arg parser:
- e.g., derive-macro based, builder pattern, declarative DSL, positional-first
- For each: sketch the user-facing API and internal parsing approach
- Observe characteristics, measure convergence, prune weak strategies
- Merge the best fragments into a unified design

### Stage 2: Harden (generative-adversarial)

Take the merged design from Stage 1 and run adversarial hardening:
- Round 1: Surface scan — obvious API usability issues
- Round 2: Edge cases — conflicting flags, unicode args, empty input, -- separator
- Round 3: Adversarial — injection via env vars, extremely long args, null bytes
- Fix each issue before proceeding to next round
- Terminate when 2 consecutive rounds find no new issues

### Stage 3: Maintain (stigmergic)

Given the hardened implementation from Stage 2, demonstrate maintenance readiness:
- Define what artifacts a stigmergic watcher would monitor
- Show a hypothetical change (e.g., adding subcommand support) and the
  reactive patches it would trigger
- Include debounce reasoning for related changes

Show clear handoff boundaries between stages: Stage 1 output → Stage 2 input,
Stage 2 output → Stage 3 input.

Produce the final implementation with a short summary of what each stage contributed.
```

## Expected structure

1. **Stage 1 (Explore)** — ≥3 strategies, comparison, pruning, merged design
2. **Stage 1→2 handoff** — explicit transition with the merged design as input
3. **Stage 2 (Harden)** — ≥2 adversarial rounds with critic/generator exchange
4. **Stage 2→3 handoff** — hardened implementation as input
5. **Stage 3 (Maintain)** — artifact watch definitions, hypothetical reactive patch, debounce
6. **Summary** — per-stage contribution to the final result

## Grading markers

```json
{
  "primitive": "composed-playbook",
  "markers": {
    "stage_sequence": {
      "check": "3 stages executed in order: explore → harden → maintain",
      "required": true
    },
    "stage_handoff": {
      "check": "explicit data flow between stages (output of N is input of N+1)",
      "required": true
    },
    "swarm_evidence": {
      "check": "Stage 1 shows fork (≥3 strategies), observe, prune, merge",
      "required": true
    },
    "adversarial_evidence": {
      "check": "Stage 2 shows ≥2 rounds of critic-generator exchange",
      "required": true
    },
    "stigmergic_evidence": {
      "check": "Stage 3 shows artifact watching, reactive patches, debounce",
      "required": true
    },
    "composition_coherence": {
      "check": "each stage builds on previous stage's output, not independent work",
      "required": true
    }
  },
  "pass_threshold": "all required markers present"
}
```

## Anti-signal

- Agent treats stages independently (no handoff — restarts from scratch each stage)
- Stage 1 skips to a single design without forking
- Stage 2 lists issues without round structure
- Stage 3 is generic maintenance advice unconnected to the specific implementation
- No summary of per-stage contribution
