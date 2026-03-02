#!/usr/bin/env bash
# Tool setup: database
# Validates SQLite CLI availability.

if ! command -v sqlite3 >/dev/null 2>&1; then
    echo "[clawden/tools/database] Error: sqlite3 is not installed" >&2
    return 1
fi

export CLAWDEN_SQLITE="$(command -v sqlite3)"
echo "[clawden/tools/database] sqlite3 ready"
