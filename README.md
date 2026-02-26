# ClawLab

Unified orchestration platform for heterogeneous claw runtimes.

## Quick start

### Rust backend and CLI

- Build: `cargo build`
- Test: `cargo test`
- Run server: `cargo run -p clawlab-server`
- Run CLI: `cargo run -p clawlab-cli -- --help`

### Dashboard and SDK

- Install deps: `pnpm install`
- Dashboard dev: `pnpm --filter @clawlab/dashboard dev`
- Dashboard test: `pnpm --filter @clawlab/dashboard test`
- SDK build: `pnpm --filter @clawlab/sdk build`
- SDK test: `pnpm --filter @clawlab/sdk test`
