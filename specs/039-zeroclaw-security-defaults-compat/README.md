---
status: planned
created: 2026-03-05
priority: high
tags:
- security
- zeroclaw
- config
- rlimit
- sandbox
- compatibility
- developer-experience
created_at: 2026-03-05T01:56:04.595663369Z
updated_at: 2026-03-05T01:56:04.595663369Z
---

# ZeroClaw Security Defaults Compatibility — Relaxed Limits for ClawDen-Managed Execution

## Overview

ZeroClaw recent versions (≥0.1.7) ship with very strict security defaults: aggressive resource limits (`rlimit`), restrictive seccomp profiles, capability dropping, and sandboxed tool execution. While these defaults are appropriate for standalone multi-tenant deployments, they cause failures and degraded performance when ZeroClaw is launched via ClawDen — where ClawDen itself provides the trust boundary and orchestration layer.

## Context

### Problem

When `clawden run zeroclaw` or `clawden up` launches ZeroClaw, users hit unexpected failures:

1. **Memory limits (`RLIMIT_AS` / `RLIMIT_DATA`)**: ZeroClaw's default memory cap (e.g., 512MB) is too low for LLM-heavy workloads that buffer large context windows, tool outputs, or multi-turn conversation history. The runtime OOM-kills itself or child tool processes.

2. **File descriptor limits (`RLIMIT_NOFILE`)**: The default cap (e.g., 256 FDs) is insufficient when ZeroClaw connects to multiple channels simultaneously (Telegram polling + Discord gateway + Slack socket mode), each requiring persistent connections plus the LLM provider HTTP client pool.

3. **Seccomp profile**: ZeroClaw's default seccomp filter blocks syscalls used by common tools — `clone3` (needed by modern glibc), `io_uring_*` (used by some async runtimes), `execve` restrictions that interact poorly with ClawDen's sandbox wrapper (`clawden-sandbox` / bwrap).

4. **Capability dropping**: ZeroClaw drops `CAP_NET_RAW` by default, which breaks ICMP-based health checks and network diagnostic tools. It also drops `CAP_SYS_PTRACE`, which prevents ClawDen's process monitoring from attaching to inspect stuck child processes.

5. **Sandboxed tool execution conflicts**: ZeroClaw has its own sandboxing layer for tool calls, which conflicts with ClawDen's `clawden-sandbox` (bwrap-based). Running bwrap-inside-bwrap fails with `EPERM` because the inner namespace creation is denied by the outer namespace's seccomp filter.

### Why This Matters for ClawDen

ClawDen's architecture (spec 024, spec 033) positions it as the **trust boundary and orchestration layer**:

- ClawDen provides its own sandbox (`clawden-sandbox`, bubblewrap) for tool execution isolation (spec 024)
- ClawDen manages process lifecycle, health checks, and process monitoring (spec 031)
- ClawDen handles credential injection and config generation — it already controls the security perimeter (specs 025, 029, 031)
- In Docker mode, the container itself provides isolation; ZeroClaw's inner restrictions are redundant

When ZeroClaw independently enforces strict limits, it creates a **double-sandboxing** problem where two layers conflict and neither works correctly.

### Affected Failure Modes

| Symptom | Root Cause | Impact |
|---------|-----------|--------|
| OOM crash during long conversations | `RLIMIT_AS` too low for LLM context | Runtime dies, user loses session |
| "Too many open files" with multi-channel | `RLIMIT_NOFILE` too low | Channel connections drop |
| Tool execution `EPERM` errors | Nested bwrap (ClawDen sandbox inside ZeroClaw sandbox) | Tools fail silently |
| Health check probe fails | `CAP_NET_RAW` dropped, ICMP blocked | ClawDen thinks runtime is down, triggers restart loop |
| `clone3` SIGSYS in tool subprocess | Seccomp blocks modern glibc syscalls | Cryptic crash in tool child processes |

### Affected Runtimes

| Runtime | Has own security limits | Conflicts with ClawDen |
|---------|------------------------|----------------------|
| ZeroClaw | Yes (strict defaults) | **Yes — this spec** |
| NullClaw | Likely (similar codebase) | Probable |
| OpenFang | Unknown | Needs audit |
| OpenClaw | Minimal (env-only) | No |
| NanoClaw | Minimal (env-only) | No |
| PicoClaw | Unknown | Needs audit |

## Solution

### Strategy: "ClawDen-Managed" Security Profile

When ZeroClaw is launched by ClawDen (vs. standalone), inject a relaxed security profile that defers trust-boundary enforcement to ClawDen. ZeroClaw still runs its own application-level security (allowlists, auth), but resource limits and process isolation are managed by ClawDen's outer layer.

### 1. Security Profile Config Injection (`config_gen.rs`)

Add `inject_security_profile()` to `generate_toml_config()`, emitting a `[security]` section:

```toml
[security]
profile = "managed"           # vs. "strict" (ZeroClaw default)
rlimit_as = 0                 # 0 = inherit from parent (no override)
rlimit_nofile = 0             # 0 = inherit from parent
rlimit_nproc = 0              # 0 = inherit from parent
seccomp = "disabled"          # ClawDen's sandbox handles this
drop_capabilities = false     # ClawDen manages capabilities
sandbox_tools = false         # Use ClawDen's clawden-sandbox instead
```

**Behavior:**
- Only injected when ClawDen launches the runtime (presence of `--config-dir` flag or `CLAWDEN_MANAGED=1` env var)
- Not injected for standalone `zeroclaw daemon` usage
- User can override any field via `clawden.yaml` config overrides (same merge pattern as proxy injection in spec 038)

### 2. `CLAWDEN_MANAGED` Environment Variable

Set `CLAWDEN_MANAGED=1` in the runtime's environment when launched via `clawden run` or `clawden up`. This gives runtimes a universal signal that an outer orchestrator is managing security:

```rust
// In build_runtime_env_vars() or process spawning
env_vars.insert("CLAWDEN_MANAGED".into(), "1".into());
```

Runtimes that understand this variable can switch to a relaxed profile even without config file injection (fallback for runtimes that don't use `--config-dir`).

### 3. Resource Limit Propagation

For Docker mode, ensure container resource limits are set appropriately:

```rust
// When building docker run args
docker_args.push("--memory=4g".into());        // Generous default
docker_args.push("--ulimit=nofile=65536:65536".into());
docker_args.push("--security-opt=seccomp=unconfined".into());  // ClawDen sandbox handles this
```

For Direct mode, ClawDen can set `setrlimit()` before `exec()` if the user requests specific limits in `clawden.yaml`:

```yaml
runtimes:
  zeroclaw:
    security:
      memory_limit: "4g"    # Optional, default = no limit
      max_open_files: 65536  # Optional, default = system default
```

### 4. Sandbox Delegation

When `security.sandbox_tools = false` is injected into ZeroClaw's config:

- ZeroClaw skips its internal bwrap/namespace isolation for tool calls
- ClawDen's `clawden-sandbox` wrapper remains active if the `sandbox` tool is enabled (spec 024)
- Tools execute with ClawDen's isolation boundary, not a nested one

This eliminates the double-sandbox conflict while maintaining isolation.

### 5. SecurityConfig Extension

Extend the existing `SecurityConfig` in `clawden-config`:

```rust
pub struct SecurityConfig {
    pub allowlist: Vec<String>,
    pub sandboxed: bool,
    // New fields:
    pub profile: Option<String>,          // "strict" | "managed" | "permissive"
    pub memory_limit: Option<String>,     // e.g., "4g", "unlimited"
    pub max_open_files: Option<u64>,      // e.g., 65536
    pub seccomp_enabled: Option<bool>,    // Override runtime default
    pub drop_capabilities: Option<bool>,  // Override runtime default
    pub delegate_sandbox: Option<bool>,   // true = use ClawDen sandbox
}
```

### 6. Config Translator Update (`ZeroClawConfigTranslator`)

Update `to_native()` to emit security config:

```rust
if let Some(ref sec) = config.security {
    if sec.profile.as_deref() == Some("managed") {
        native["security"]["profile"] = "managed".into();
        native["security"]["rlimit_as"] = 0.into();
        native["security"]["rlimit_nofile"] = 0.into();
        native["security"]["seccomp"] = "disabled".into();
        native["security"]["drop_capabilities"] = false.into();
        native["security"]["sandbox_tools"] = false.into();
    }
}
```

## Alternatives Considered

### A. Patch ZeroClaw upstream to detect ClawDen
Rejected: Creates a coupling dependency. ZeroClaw shouldn't need to know about ClawDen.

### B. Only use Docker mode for isolation
Rejected: Direct mode is the default (spec 033) and many users prefer it. Can't require Docker.

### C. Override via environment variables only
Partially adopted (`CLAWDEN_MANAGED=1`), but insufficient alone — ZeroClaw's config file takes precedence over env vars for security settings (same class of issue as spec 038 proxy bug).

### D. Strip ZeroClaw's seccomp at the kernel level
Rejected: Requires root, fragile, and breaks the security model for standalone usage.

## Checklist

- [ ] Add `inject_security_profile()` to `generate_toml_config()` in `config_gen.rs`
- [ ] Set `security.profile = "managed"` with relaxed rlimits, seccomp, capabilities
- [ ] Set `CLAWDEN_MANAGED=1` env var in `build_runtime_env_vars()`
- [ ] Extend `SecurityConfig` struct with new fields (profile, memory_limit, max_open_files, etc.)
- [ ] Update `ZeroClawConfigTranslator::to_native()` to emit `[security]` section
- [ ] Add Docker `--ulimit` and `--security-opt` flags for Docker mode launches
- [ ] Add optional `security:` section to `clawden.yaml` runtime config schema
- [ ] Verify multi-channel launch (Telegram + Discord) stays within FD limits
- [ ] Verify tool execution works without double-sandbox EPERM
- [ ] Verify health check probes succeed (no capability-drop interference)
- [ ] Audit NullClaw and OpenFang for similar security default conflicts
- [ ] All clawden-cli tests pass

## Notes

- This is the same class of config-override bug as spec 038 (proxy) and spec 031 (config injection): the runtime's own config silently overrides what ClawDen intends. The fix pattern is the same — inject correct values during config generation.
- The `CLAWDEN_MANAGED` env var can be reused by future specs as a universal "orchestrator present" signal for any runtime.
- NullClaw likely has the same issue (similar Rust codebase) — audit separately and apply the same pattern.
- PicoClaw and OpenFang should be audited but may not have strict security defaults.
