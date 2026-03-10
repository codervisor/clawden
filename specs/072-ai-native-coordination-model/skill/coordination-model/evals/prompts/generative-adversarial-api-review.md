# Eval: Generative-Adversarial — API Security Review

## Setup

Load the coordination SKILL.md into the agent's context before giving this prompt.

## Prompt

```
You have the coordination-model skill loaded. Use the **generative-adversarial**
primitive to review and harden this HTTP endpoint handler:

```rust
async fn create_user(
    Json(payload): Json<CreateUserRequest>,
    State(db): State<DbPool>,
) -> impl IntoResponse {
    let hash = bcrypt::hash(&payload.password, 10).unwrap();
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (name, email, password_hash) VALUES ($1, $2, $3) RETURNING *",
        payload.name,
        payload.email,
        hash,
    )
    .fetch_one(&db)
    .await
    .unwrap();

    Json(user)
}
```

Follow the generative-adversarial lifecycle:

**Round 1 — Surface scan**: Generator proposes the initial implementation as-is.
Critic identifies obvious issues (error handling, validation, response codes).
Generator fixes them.

**Round 2 — Edge cases**: Critic attacks with edge cases (duplicate emails, empty
fields, extremely long inputs, unicode edge cases). Generator hardens against each.

**Round 3 — Adversarial inputs**: Critic attempts injection attacks, timing attacks,
enumeration attacks. Generator adds defenses.

**Round 4 — Semantic analysis**: Critic reviews the overall design for architectural
issues (information leakage, rate limiting needs, audit logging gaps). Generator
addresses them.

Each round must show: (a) the critic's specific attack/finding, (b) the generator's
concrete fix. Terminate when 2 consecutive rounds produce no new issues OR after
round 4.

Produce the final hardened implementation.
```

## Expected structure

1. **Round 1** — Critic findings + Generator fixes (surface level)
2. **Round 2** — Critic findings + Generator fixes (edge cases)
3. **Round 3** — Critic findings + Generator fixes (adversarial)
4. **Round 4** — Critic findings + Generator fixes (semantic/architectural)
5. **Final implementation** — All fixes integrated
6. **Termination reason** — Why the loop stopped

## Grading markers

```json
{
  "primitive": "generative-adversarial",
  "markers": {
    "escalation_evidence": {
      "check": "≥3 rounds with increasing attack sophistication",
      "required": true
    },
    "critic_specificity": {
      "check": "critic findings are concrete (specific attack vectors, not vague 'improve security')",
      "required": true
    },
    "generator_response": {
      "check": "generator produces code fixes for each critic finding, not just acknowledgment",
      "required": true
    },
    "termination_condition": {
      "check": "explicit termination reasoning (consecutive clean rounds or max reached)",
      "required": true
    },
    "progressive_difficulty": {
      "check": "later rounds address harder issues than earlier rounds",
      "required": true
    }
  },
  "pass_threshold": "all required markers present"
}
```

## Anti-signal

- Agent lists all issues at once without rounds (no escalation)
- Critic is vague ("add better error handling") instead of specific attacks
- Generator acknowledges issues but doesn't show code changes per round
- No termination reasoning — just stops after listing issues
