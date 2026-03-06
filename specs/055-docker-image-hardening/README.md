---
status: planned
created: 2026-03-06
priority: high
tags:
- docker
- security
- supply-chain
- infra
- container
- hardening
depends_on:
- 052-docker-image-runtime-binary-compat
parent: 017-docker-runtime-images
created_at: 2026-03-06T08:17:36.969275199Z
updated_at: 2026-03-06T08:17:36.969275199Z
---

# Docker Image Hardening — Security, Consistency & Supply-Chain Fixes

## Overview

A comprehensive audit of the ClawDen Docker runtime image (`docker/Dockerfile`, `docker/entrypoint.sh`, `docker/tools/`) revealed **1 critical, 6 high, and 14 medium** issues across security, supply-chain integrity, spec consistency, correctness, and UX. This spec tracks all actionable fixes.

### Why Now

- **S1 (critical)**: Node.js is installed via `curl | bash` with no integrity verification — a supply-chain attack vector.
- **S2**: `PICOCLAW_VERSION=latest` and `OPENCLAW_VERSION=latest` break reproducibility and allow compromised upstream releases.
- **S3**: `nullclaw` appears in `runtime_default_args()` but is not an installed or supported runtime — dead code that misleads debugging.
- **C1**: `core-utils` spec says `jq, yq, tree, file, zip/unzip, gzip` but `yq`, `file`, and `gzip` are never installed.

## Design

### Phase 1 — Critical & High Severity

#### 1. Eliminate `curl | bash` for Node.js (S1)

Replace the NodeSource setup script with a multi-stage `COPY --from`:

```dockerfile
COPY --from=node:22-bookworm-slim /usr/local/bin/node /usr/local/bin/node
COPY --from=node:22-bookworm-slim /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -s /usr/local/lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s /usr/local/lib/node_modules/corepack/dist/corepack.js /usr/local/bin/corepack \
    && corepack enable pnpm
```

This eliminates remote script execution and pins to a Docker-verified Node image.

#### 2. Pin all runtime versions (S2, S9)

```dockerfile
ARG OPENCLAW_VERSION=0.4.7      # was: latest
ARG PICOCLAW_VERSION=0.2.3      # was: latest
ARG NANOCLAW_REF=v0.1.0         # was: main (floating git ref)
```

All five runtimes must have pinned, reproducible version ARGs.

#### 3. Remove dead `nullclaw` case (S3)

Delete the `nullclaw)` branch from `runtime_default_args()` in `entrypoint.sh`. NullClaw is not installed, not in `SUPPORTED_RUNTIMES`, and not smoke-tested. Add it back when NullClaw is actually shipped.

#### 4. Fix tool setup.sh scripts that call `sudo apt-get` (S4, S5)

`git/setup.sh` and `http/setup.sh` have `sudo apt-get install` fallbacks that always fail (no sudo for `clawden` user). These packages are already baked into the base image. Replace install branches with validate-only checks like `python/setup.sh` does.

#### 5. Install missing `core-utils` components (C1)

Add `yq` (mikefarah/yq binary), `file`, and `gzip` to the Dockerfile's apt-get layer. Update `core-utils/setup.sh` to validate all six components, not just `jq`.

#### 6. Fix `$DEFAULT_ARGS` word-splitting (R1)

Replace:
```bash
exec "$LAUNCHER" $DEFAULT_ARGS
```
With array expansion:
```bash
read -ra default_args <<< "$DEFAULT_ARGS"
exec "$LAUNCHER" "${default_args[@]}"
```

### Phase 2 — Medium Severity

#### 7. Runtime-aware HEALTHCHECK (S6)

Set `RUNTIME_PORT` per runtime in entrypoint.sh before exec:
```bash
case "$RUNTIME" in
    openclaw)  export RUNTIME_PORT="${RUNTIME_PORT:-18789}" ;;
    openfang)  export RUNTIME_PORT="${RUNTIME_PORT:-4200}" ;;
    *)         export RUNTIME_PORT="${RUNTIME_PORT:-8080}" ;;
esac
```

#### 8. Sandbox `/proc` exposure (S7)

Replace `--ro-bind / /` with explicit path binds (`/usr`, `/lib`, `/bin`, `/etc`) and `--proc /proc` so `/proc/1/environ` (API keys) is not leaked to sandboxed processes.

#### 9. Add signal trap in entrypoint (R2)

Add near the top of `entrypoint.sh`:
```bash
trap 'echo "[clawden] Interrupted during setup"; exit 130' INT TERM
```

#### 10. Validate env vars for all runtimes (R4)

Extend `validate_runtime_env` to cover ZeroClaw, PicoClaw, NanoClaw, and OpenFang required config.

#### 11. Fix tools.json output (C8)

Write actual binary paths and versions instead of `"bin": "$setup_script"` and `"version": "unknown"`.

#### 12. Spec parity fixes (C2, C3, C4, C6)

- Add `hexyl` to `code-tools` (C2).
- Correct Python version in manifest to `3.11` to match Bookworm (C3).
- Remove or stub `browser`/`gui` tool references from comments (C4).
- Add `env` table to `python/manifest.toml` (C6).

#### 13. Remove vestigial `browser`/`gui` empty directories (C7)

Delete `mkdir` lines for `/opt/clawden/tools/browser` and `/opt/clawden/tools/gui` from Dockerfile. Spec 024 clarifies these are image variants, not tool-layer entries.

#### 14. Eliminate redundant `apt-get update` for p7zip purge (R5)

Either don't install `p7zip-full` at all, or if needed for runtime install only, combine install and purge in a single `RUN` layer.

### Phase 3 — Low Severity & Optimization

#### 15. Add `EXPOSE` directives (P4)

```dockerfile
EXPOSE 8080 18789 4200
```

#### 16. Replace pnpm global install with corepack (P1)

`corepack enable pnpm` is zero-install and built into Node 22. Remove `npm install -g pnpm`.

#### 17. Reduce runtime list duplication (M1)

The runtime list is maintained in 3 places (Dockerfile ARGs, entrypoint `SUPPORTED_RUNTIMES`, smoke-test loop). Consider extracting to a shared file that all three consume.

#### 18. Add `phase` field to tool manifests (M2)

Spec 024 defines `phase` in the manifest schema but no manifest includes it. Add `phase = 1` to core tools and `phase = 2` to standard tools.

## Plan

- [ ] **Phase 1**: S1 — Replace `curl | bash` with `COPY --from=node:22-bookworm-slim`
- [ ] **Phase 1**: S2/S9 — Pin OpenClaw, PicoClaw, NanoClaw versions
- [ ] **Phase 1**: S3 — Remove `nullclaw` dead code from entrypoint
- [ ] **Phase 1**: S4/S5 — Fix `git/setup.sh` and `http/setup.sh` sudo fallbacks
- [ ] **Phase 1**: C1 — Install `yq`, `file`, `gzip`; update `core-utils/setup.sh`
- [ ] **Phase 1**: R1 — Fix `$DEFAULT_ARGS` word-splitting
- [ ] **Phase 2**: S6 — Runtime-aware HEALTHCHECK port
- [ ] **Phase 2**: S7 — Sandbox path bind hardening
- [ ] **Phase 2**: R2 — Add signal trap to entrypoint
- [ ] **Phase 2**: R4 — Env validation for all runtimes
- [ ] **Phase 2**: C8 — Fix tools.json binary/version output
- [ ] **Phase 2**: C2/C3/C4/C6/C7 — Spec parity fixes
- [ ] **Phase 2**: R5 — Remove p7zip round-trip layer
- [ ] **Phase 3**: P4 — Add EXPOSE directives
- [ ] **Phase 3**: P1 — Replace pnpm global install with corepack
- [ ] **Phase 3**: M1 — Reduce runtime list duplication
- [ ] **Phase 3**: M2 — Add phase field to tool manifests

## Test

- [ ] `docker build --target latest .` succeeds with no `curl | bash`
- [ ] All runtime version ARGs are pinned (no `latest`, no floating refs)
- [ ] `entrypoint.sh` has no `nullclaw` references
- [ ] `git/setup.sh` and `http/setup.sh` contain no `sudo` or `apt-get install` calls
- [ ] `yq --version`, `file --version`, `gzip --version` all succeed inside `:latest` image
- [ ] Sandbox `clawden-sandbox` does not expose `/proc/1/environ`
- [ ] `tools.json` contains actual binary paths and version strings
- [ ] `HEALTHCHECK` uses correct port for openclaw (18789) and openfang (4200)
- [ ] Build-time smoke test passes for all 5 runtimes
- [ ] `cargo test -p clawden-core --quiet && cargo test -p clawden-cli --quiet` pass

## Notes

- Audit performed 2026-03-06. Full finding list: 1 critical, 6 high, 14 medium, 9 low across security, consistency, correctness, performance, UX, and maintenance.
- Spec 052 (in-progress) handles the binary compatibility subset (glibc mismatch, arch errors). This spec covers all remaining findings.
- Phase 2 runtimes (IronClaw, NullClaw, MicroClaw) are out of scope — they'll need their own Dockerfile entries when shipped.