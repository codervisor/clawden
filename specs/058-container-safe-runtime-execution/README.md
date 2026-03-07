---
status: in-progress
created: 2026-03-07
priority: critical
tags:
- container
- docker
- exec
- refactor
- process-lifecycle
- tech-debt
- cli
- railway
created_at: 2026-03-07T15:02:39.232914Z
updated_at: 2026-03-07T15:02:39.232914Z
---

# Container-Safe Runtime Execution — Exec Mode, Pre-Launch Unification & Process Lifecycle Overhaul

## Overview

ClawDen's runtime execution path has accumulated significant tech debt across specs 017–057. The `clawden run` command, `clawden up` command, Docker entrypoint, and ProcessManager all evolved independently under different specs, resulting in duplicated pre-launch logic, hardcoded startup timing, log-file race conditions, and a process model fundamentally unsuited for containers.

The immediate trigger is a crash loop on Railway (and any container platform) when using `CLAWDEN_USE_CLI=1`: `clawden run` spawns the runtime as a managed child with piped stdout/stderr and a 500ms startup check that causes rapid cascade failures. But the underlying issue is structural — container execution was never a first-class lifecycle mode.

This spec consolidates six categories of tech debt into one focused overhaul:

1. **Exec mode for containers** — `clawden run --exec` replaces the clawden process with the runtime via `exec()`, making the runtime PID 1 with direct stdio
2. **Pre-launch unification** — extract the duplicated config resolution, env-var building, and credential validation from `run.rs` and `up.rs` into a shared `PreLaunchContext`
3. **Startup verification tuning** — make `verify_runtime_startup` timing configurable and container-aware
4. **Entrypoint simplification** — remove duplicated env-var-to-flag mapping from `entrypoint.sh` now that `--exec` handles everything
5. **Process lifecycle fixes** — address log-file race conditions, PID detection in containers, and stale config cleanup
6. **Audit logging completeness** — capture config source, resolved env keys, and execution mode in the audit trail

## Problem

### Container crash loop (Critical — triggered the overhaul)

When deployed to Railway/Render/Fly with `CLAWDEN_USE_CLI=1`:
- Entrypoint invokes `clawden run --channel telegram --token ... --allowed-users ... openclaw`
- `clawden run` spawns openclaw as a background child with piped stdout/stderr
- `verify_runtime_startup` checks if the child is alive after 500ms
- If openclaw exits quickly (config issue, port conflict, slow Node.js startup), clawden prints "exited immediately" and exits
- Container orchestrator restarts; same cycle repeats every ~1–2 seconds
- No useful error output reaches the container logs because stdout/stderr are piped through tee threads that may not flush before the process dies

### Pre-launch code duplication (High)

`run.rs` (357 lines of pre-launch logic) and `up.rs` (261 lines) both independently:
- Call `build_runtime_env_vars` then layer on host-env tokens, memory vars, shortcut overrides
- Resolve channels, tools, config-dir generation, credential validation
- Handle env-file loading, provider inference, Telegram username resolution
- Build the final args list with default subcommands

Changes to one command silently diverge from the other. Six specs in a row (034–039) each required parallel fixes in both files.

### Docker entrypoint duplication (High)

`entrypoint.sh` manually reconstructs `--channel`, `--token`, `--allowed-users`, `--provider`, `--model`, `--api-key`, `--system-prompt` flags from environment variables — duplicating precedence logic that `run.rs` already handles via `apply_run_overrides` and `apply_shortcut_env_overrides`. The entrypoint doesn't support `--tools`, `--with`, or recent credential flags, causing feature drift.

### Startup timing (High)

`verify_runtime_startup` uses hardcoded delays: 500ms initial + 3×500ms crash checks + 5×1s health probes. Not configurable. Not container-aware. Runtimes that legitimately take 3–5 seconds to start (OpenClaw with large model configs, ZeroClaw rebuilding index) are incorrectly reported as crashed.

### Process lifecycle (Medium)

- **Log race**: `tee_reader_to_log` writes from background threads while `stream_logs`/`tail_logs` reads the same file — no synchronization
- **PID detection**: `is_pid_running` checks `/proc/<pid>/stat` (Linux-only), falls back to `ps`/`kill -0` — unreliable for containers with PID namespaces
- **Restart supervisor**: stores supervisor PID, not runtime PID — `stop` kills the wrong process after a restart cycle
- **Config cleanup**: `cleanup_project_config_dir` is defined but never called — stale config accumulates

## Design

### 1. Exec mode (`--exec` flag)

Already implemented in the initial fix. `clawden run --exec` performs all config translation (provider detection, username resolution, config-dir generation, env-var assembly) then calls `std::os::unix::process::CommandExt::exec()` to replace the clawden process with the runtime binary.

**Container benefits**:
- Runtime is PID 1 → receives signals directly, no zombie children
- stdout/stderr go directly to container log collector → no tee race
- No startup verification needed → container orchestrator handles restarts
- No PID files, no log files, no background threads

**Entrypoint update**: `entrypoint.sh` CLI mode now uses `clawden run --exec` instead of `clawden run`.

### 2. PreLaunchContext (shared pre-execution pipeline)

Extract into `crates/clawden-cli/src/commands/pre_launch.rs`:

```rust
pub struct PreLaunchContext {
    pub runtime: String,
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub env_vars: Vec<(String, String)>,
    pub channels: Vec<String>,
    pub tools: Vec<String>,
    pub project_hash: String,
    pub config: Option<ClawDenYaml>,
}

impl PreLaunchContext {
    pub async fn resolve(opts: &RunOptions, installer: &RuntimeInstaller) -> Result<Self>;
}
```

**`resolve()` consolidates**:
1. Config loading (`load_config_with_env_file`)
2. Override application (`ensure_config_for_run_overrides`, `apply_run_overrides`)
3. Telegram username resolution
4. Provider inference from host env
5. Channel/tool resolution and config population
6. Env-var building (`build_runtime_env_vars` + host injection + shortcut overrides + env overrides)
7. Runtime installation and version pinning
8. Default subcommand injection
9. Config-dir generation and injection
10. Credential validation
11. State-dir env var injection

Both `exec_run` and `exec_up` call `PreLaunchContext::resolve()` then diverge only at the execution step (exec, spawn-managed, spawn-detached, Docker adapter).

### 3. Configurable startup verification

Add environment variables to tune `verify_runtime_startup`:

- `CLAWDEN_STARTUP_INITIAL_DELAY_MS` (default: 500) — initial sleep before first check
- `CLAWDEN_STARTUP_POLL_INTERVAL_MS` (default: 500) — interval between crash checks
- `CLAWDEN_STARTUP_POLL_COUNT` (default: 3) — number of crash checks
- `CLAWDEN_HEALTH_TIMEOUT_SECS` (default: 5) — total health-probe timeout

Skip all timing when `--exec` is used (no child process to verify).

### 4. Entrypoint simplification

With `--exec`, the entrypoint's CLI mode reduces to:

```bash
exec clawden run --exec "$RUNTIME" "$@"
```

All channel/token/credential env vars are auto-detected by `clawden run` via `infer_provider_from_host_env` and `inject_host_env_channel_tokens`. The entrypoint only needs to pass through extra Docker CMD args.

In direct mode (CLAWDEN_USE_CLI=0), keep the current behavior unchanged — it already execs into the runtime binary directly.

### 5. Process lifecycle fixes

**Log access**: Replace dual tee threads + file read with a single-writer append pattern. `stream_logs` reads from the log file using `seek` to the last known offset. Add a mutex or use atomic file appending so writes and reads don't interleave.

**PID detection**: Add waitpid-based detection for direct children. For non-child PIDs, prefer `kill(pid, 0)` over `/proc` stat parsing. Document that container PID namespace limits apply.

**Restart supervisor**: Store the actual runtime PID (not the supervisor's) in the PID file. Update the supervisor script to write a secondary `.child.pid` file after each spawn.

**Config cleanup**: Call `cleanup_project_config_dir` in `exec_stop` and on graceful shutdown in the foreground run loop.

### 6. Audit logging

Add audit entries for:
- Config source: "config.source" → "clawden.yaml" | "env-only" | "flags-only"
- Execution mode: "runtime.mode" → "exec" | "direct" | "detached" | "docker"
- Resolved env keys (not values): "runtime.env_keys" → comma-separated key list
- Config dir path: "runtime.config_dir" → path or "none"

## Plan

- [x] Add `--exec` flag to `clawden run` CLI definition
- [x] Implement exec mode in `exec_run` using `CommandExt::exec()`
- [x] Update `docker/entrypoint.sh` to use `--exec` in CLI mode
- [x] Add CLI test for `--exec` flag parsing
- [ ] Extract `PreLaunchContext` from `run.rs` and `up.rs` into shared module
- [ ] Refactor `exec_run` to use `PreLaunchContext::resolve()`
- [ ] Refactor `exec_up` to use `PreLaunchContext::resolve()`
- [ ] Simplify entrypoint CLI mode to pass-through (remove manual flag construction)
- [ ] Add `CLAWDEN_STARTUP_*` env vars to `verify_runtime_startup`
- [ ] Fix log-file access pattern in ProcessManager (single-writer)
- [ ] Fix restart supervisor to track child PID, not supervisor PID
- [ ] Call `cleanup_project_config_dir` on shutdown
- [ ] Add config-source and mode audit entries
- [ ] Add integration test: exec mode launches runtime as replacement process
- [ ] Add integration test: entrypoint CLI mode with minimal env vars

## Test

- [x] `clawden run --exec openclaw` exits only on exec() error (unit test for flag parsing)
- [ ] `PreLaunchContext::resolve()` produces identical env vars to current `exec_run` for all test cases
- [ ] `exec_up` using `PreLaunchContext` passes existing up integration tests
- [ ] Simplified entrypoint CLI mode produces the same `clawden run` invocation as current behavior
- [ ] `CLAWDEN_STARTUP_INITIAL_DELAY_MS=2000` delays first crash check to 2s
- [ ] Config dir is cleaned up after `clawden stop`
- [ ] Audit log includes config source and execution mode entries
- [ ] `cargo test -p clawden-cli --quiet` — all existing tests pass with refactored code
- [ ] `cargo build -p clawden-core -p clawden-cli --no-default-features --quiet` — no-default-features build succeeds

## Notes

This spec supersedes the quick-fix `--exec` implementation and elevates it into a proper architectural change. The initial `--exec` commit is preserved as the starting point — this spec builds on top of it.

Related completed specs:
- 017 (docker-runtime-images) — original Docker architecture
- 023 (cli-direct-architecture) — introduced ProcessManager's direct mode
- 033 (product-positioning) — defined run.rs as the primary CLI entry
- 035 (run-command-ux-polish) — added credential resolution and smart defaults
- 036 (run-clean-passthrough) — removed implicit subcommands (then re-added as smart defaults)
- 043 (rust-codebase-structural-refactor) — prior dedup effort
- 050 (docker-runtime-direct-run-ux) — self-describing entrypoint
- 051 (openclaw-docker-silent-exit) — partial container exit fix

Related planned specs:
- 039 (zeroclaw-security-defaults-compat) — may benefit from PreLaunchContext for security env injection