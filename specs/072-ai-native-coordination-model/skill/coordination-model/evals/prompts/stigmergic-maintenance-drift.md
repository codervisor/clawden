# Eval: Stigmergic Maintenance — Config Drift Detection

## Setup

Load the coordination SKILL.md into the agent's context before giving this prompt.
Provide these two files as context:

**config_schema.rs** (the "artifact" being watched):
```rust
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}
```

**config_schema_updated.rs** (the artifact AFTER a change):
```rust
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,        // NEW
    pub telemetry: TelemetryConfig, // NEW
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tls_cert: Option<String>,  // NEW
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub statement_timeout: Option<String>, // NEW
}

#[derive(Debug, Deserialize)]
pub struct CacheConfig {           // NEW
    pub backend: String,
    pub ttl_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct TelemetryConfig {       // NEW
    pub endpoint: String,
    pub sample_rate: f64,
}
```

## Prompt

```
You have the coordination-model skill loaded. Use the **stigmergic** primitive
to react to a configuration schema change.

The config schema artifact has changed (diff between config_schema.rs and
config_schema_updated.rs above). You are a stigmergic maintenance agent that
watches config artifacts.

Follow the stigmergic lifecycle:

1. **Observe (detect markers)**: Analyze the diff. What changed? Classify each
   change as a marker type: "needs-review", "needs-docs", "needs-test",
   "needs-migration", "confidence:high/low".

2. **React (produce patches)**: For each marker, produce the minimum targeted
   response. Don't rewrite everything — react proportionally to each change:
   - New struct → generate default config TOML section + doc comment
   - New optional field → generate migration note
   - New required field → flag as breaking change

3. **Debounce reasoning**: Explain which reactions you're suppressing. If
   adding CacheConfig triggers a need for cache docs, and TelemetryConfig
   triggers a need for telemetry docs, those are separate reactions — don't
   combine them into a monolithic "rewrite all docs" reaction.

4. **Marker decay**: Note which of your markers are time-sensitive (should
   someone review within 1h? 24h? or is it informational?).

Produce: the marker inventory, the targeted patches, the debounce decisions,
and the decay annotations.
```

## Expected structure

1. **Change detection** — itemized list of what changed in the diff
2. **Marker inventory** — each change classified with a marker type
3. **Targeted reactions** — small, focused patches (not a monolithic rewrite)
4. **Debounce reasoning** — explicit decisions about what NOT to combine
5. **Decay annotations** — time-sensitivity per marker

## Grading markers

```json
{
  "primitive": "stigmergic",
  "markers": {
    "observe_evidence": {
      "check": "itemized change detection with marker classification",
      "required": true
    },
    "reactive_evidence": {
      "check": "targeted patches per change, not a monolithic rewrite",
      "required": true
    },
    "proportionality": {
      "check": "reaction size is proportional to change size (small change → small patch)",
      "required": true
    },
    "debounce_reasoning": {
      "check": "explicit reasoning about which reactions are kept separate vs. could storm",
      "required": true
    },
    "decay_evidence": {
      "check": "markers annotated with time-sensitivity / urgency",
      "required": false
    }
  },
  "pass_threshold": "all required markers present"
}
```

## Anti-signal

- Agent rewrites all docs/tests from scratch (no proportionality)
- Agent lists changes but doesn't classify with markers
- No debounce reasoning — treats all changes as one big reaction
- No distinction between urgent and informational markers
