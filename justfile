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
    cargo run -p clawlab-server

run-cli:
    cargo run -p clawlab-cli -- --help
