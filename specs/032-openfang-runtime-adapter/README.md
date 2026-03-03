---
status: planned
created: 2026-03-03
priority: medium
tags:
- adapter
- openfang
- runtime
- integration
depends_on:
- 010-claw-runtime-interface
created_at: 2026-03-03T06:37:44.122195Z
updated_at: 2026-03-03T06:38:05.022125Z
---

# OpenFang Runtime Adapter — Full-Stack Integration

## Overview

OpenFang is a Rust-based Agent OS runtime with TOML configuration, a built-in dashboard on port 4200, and native channel support. It is already defined in the `ClawRuntime` enum and referenced across 6+ specs, but lacks the adapter implementation, feature flags, registry wiring, Docker integration, and install-layer support needed to actually run it through ClawDen.

This spec covers adding OpenFang as a fully functional runtime in ClawDen — on par with zeroclaw, openclaw, picoclaw, and nanoclaw.

## Context

### Current State (~15% complete)

| Layer | Status | Detail |
|-------|--------|--------|
| Core Enum (`ClawRuntime`) | ✅ Done | Variant, Display, from_str_loose, as_slug all present |
| Cost Tier | ✅ Done | Tier 2 in server manager |
| CLI Init | ✅ Done | Selectable in wizard |
| Adapter (`openfang.rs`) | ❌ Missing | No `ClawAdapter` implementation |
| Feature Flag | ❌ Missing | No `openfang = []` in adapters Cargo.toml |
| Registry | ❌ Missing | Not registered in `builtin_registry()` |
| Start Args | ❌ Missing | Falls through to empty Vec |
| Supported Extra Args | ❌ Missing | Falls through to empty slice |
| Health URL | ❌ Missing | No built-in default for port 4200 |
| Docker Build | ❌ Disabled | Commented out in Dockerfile |
| Docker Entrypoint | ❌ Disabled | Commented out in entrypoint.sh |
| Direct Install | ❌ Excluded | Explicitly excluded from spec 022 download sources |

### Why Now

- OpenFang is the only Rust runtime besides ZeroClaw — its TOML config format and architecture are well-understood
- Users can select it in `clawden init` but then `clawden up` has no adapter to dispatch to
- Spec 031 (config-dir injection) lists OpenFang as Phase 2 — having the adapter in place is a prerequisite
- Spec 018 (channel support matrix) documents OpenFang's native channel implementations

### OpenFang Runtime Details

| Property | Value |
|----------|-------|
| Language | Rust |
| Config Format | TOML |
| Config File | `~/.openfang/config.toml` |
| Dashboard Port | 4200 |
| Health Endpoint | `http://127.0.0.1:4200/health` |
| Default Start Command | `openfang daemon` (assumed, TBC) |
| Channels | Telegram (native Rust), Discord (native Rust), Slack (Cloud API), WhatsApp (Socket Mode) |

### Related Specs

- **010-claw-runtime-interface** (complete) — defines `ClawAdapter` trait
- **017-docker-runtime-images** (in-progress) — Docker build/deploy for runtimes
- **018-channel-support-matrix** (in-progress) — documents OpenFang channel support
- **022-direct-install** (complete) — download sources for direct install
- **031-direct-mode-config-injection** (draft) — Phase 2 depends on this adapter existing

## Design

Follow the runtime-sync full-stack checklist exactly. Steps 1 and 6 (core enum, dashboard) are already complete.

### Step 2: Adapter Module

Create `crates/clawden-adapters/src/openfang.rs` using the canonical adapter template:

- Struct: `OpenFangAdapter` (no fields)
- `metadata()`: language=Rust, port=4200, config_format=toml, channels=[telegram, discord, slack, whatsapp]
- `send()`: echo pattern (`"OpenFang echo: {msg}"`)
- `get_config()`: fallback includes `"runtime": "openfang"`
- All lifecycle methods use `ClawRuntime::OpenFang` and `"openfang"`
- Test: `start_persists_forwarded_runtime_config`

### Step 3: Feature Flag

In `crates/clawden-adapters/Cargo.toml`:
- Add `openfang = []` to `[features]`
- Add `"openfang"` to `default` list

### Step 4: Registry Wiring

In `crates/clawden-adapters/src/lib.rs`:
- `#[cfg(feature = "openfang")] mod openfang;`
- `#[cfg(feature = "openfang")] pub use openfang::OpenFangAdapter;`
- Register in `builtin_registry()`

### Step 5a: Install Layer (install.rs)

In `crates/clawden-core/src/install.rs`:
- `runtime_start_args("openfang")` → `vec!["daemon"]`
- `runtime_supported_extra_args("openfang")` → `&["--port", "--host"]` (no `--config-dir` yet — that's spec 031 Phase 2)

### Step 5b: Health URL

In `crates/clawden-core/src/process.rs`:
- Add built-in default: `"openfang" => Some("http://127.0.0.1:4200/health")`

### Step 5c: Docker Integration

In `docker/Dockerfile`:
- Uncomment / add `ARG OPENFANG_VERSION=0.2.0`
- Add install command

In `docker/entrypoint.sh`:
- Uncomment / add `openfang)` case with `DEFAULT_ARGS="daemon"`
- Add "openfang" to supported runtimes help text

### Step 5d: Direct Install Source

Add OpenFang to the download sources table in install registry (if binary distributions exist).

## Checklist

- [ ] Create `crates/clawden-adapters/src/openfang.rs` with full `ClawAdapter` impl
- [ ] Add `openfang` feature flag to `crates/clawden-adapters/Cargo.toml`
- [ ] Wire adapter in `crates/clawden-adapters/src/lib.rs` registry
- [ ] Add `runtime_start_args` case for openfang in `install.rs`
- [ ] Add `runtime_supported_extra_args` case for openfang in `install.rs`
- [ ] Add built-in health URL default in `process.rs`
- [ ] Enable OpenFang in `docker/Dockerfile`
- [ ] Add OpenFang case in `docker/entrypoint.sh`
- [ ] Add adapter unit test (`start_persists_forwarded_runtime_config`)
- [ ] `cargo build --features openfang` passes
- [ ] `cargo test -p clawden-adapters` passes
- [ ] `cargo clippy` clean