# ClawDen

Unified orchestration platform for heterogeneous claw runtimes.

## Quick start

### Guided onboarding

1. Initialize a project with the setup wizard:
	- `cargo run -p clawden-cli -- init`
2. For CI or scripts, use non-interactive mode:
	- `cargo run -p clawden-cli -- init --yes --runtime zeroclaw`
3. Use a starter template when needed:
	- `cargo run -p clawden-cli -- init --template telegram-bot --yes`
4. Validate local setup before startup:
	- `cargo run -p clawden-cli -- doctor`
5. Start runtimes:
	- `cargo run -p clawden-cli -- up`

Provider key management:
	- List configured providers: `cargo run -p clawden-cli -- providers`
	- Test provider credentials: `cargo run -p clawden-cli -- providers test`
	- Store a key in local encrypted vault: `cargo run -p clawden-cli -- providers set-key openai`

### Rust backend and CLI

- Build: `cargo build`
- Test: `cargo test`
- Run server: `cargo run -p clawden-server`
- Run CLI: `cargo run -p clawden-cli -- --help`

### Direct install quickstart (no Docker)

1. Install one runtime:
	- `cargo run -p clawden-cli -- install zeroclaw`
	- `cargo run -p clawden-cli -- install openclaw`
2. Verify local prerequisites and installed runtimes:
	- `cargo run -p clawden-cli -- doctor`
	- `cargo run -p clawden-cli -- install --list`
3. Run directly on host:
	- `cargo run -p clawden-cli -- --no-docker run zeroclaw`
4. Manage runtime processes:
	- `cargo run -p clawden-cli -- up`
	- `cargo run -p clawden-cli -- ps`
	- `cargo run -p clawden-cli -- logs zeroclaw --lines 50`
	- `cargo run -p clawden-cli -- stop`

Notes:
- Direct installs are stored under `~/.clawden/runtimes/`.
- Set `CLAWDEN_NO_DOCKER=1` to always prefer direct mode.
- To enable health checks in `clawden ps`, set runtime-specific health env vars such as `CLAWDEN_HEALTH_PORT_ZEROCLAW=8080` (or `CLAWDEN_HEALTH_URL_ZEROCLAW=http://127.0.0.1:8080/health`).

### Dashboard and SDK

- Install deps: `pnpm install`
- Dashboard dev: `pnpm --filter @clawden/dashboard dev`
- Dashboard test: `pnpm --filter @clawden/dashboard test`
- SDK build: `pnpm --filter @clawden/sdk build`
- SDK test: `pnpm --filter @clawden/sdk test`
