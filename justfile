set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    just --list

fmt:
    cargo fmt --all

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

run-server:
    cargo run -p clawden-server

run-cli:
    cargo run -p clawden-cli -- --help

# Publishing
sync-versions:
    tsx scripts/sync-versions.ts

publish-dry-run:
    tsx scripts/publish-platform-packages.ts --dry-run --allow-local
    tsx scripts/publish-main-packages.ts --dry-run --allow-local
