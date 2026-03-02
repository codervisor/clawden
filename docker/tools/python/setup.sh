#!/usr/bin/env bash
# Tool setup: python
# Validates Python runtime binaries and exports convenience variables.

if ! command -v python3 >/dev/null 2>&1; then
    echo "[clawden/tools/python] Error: python3 is not installed" >&2
    return 1
fi

if ! command -v pip3 >/dev/null 2>&1; then
    echo "[clawden/tools/python] Error: pip3 is not installed" >&2
    return 1
fi

export CLAWDEN_PYTHON="$(command -v python3)"
echo "[clawden/tools/python] python $(python3 --version 2>&1 | awk '{print $2}') ready"
