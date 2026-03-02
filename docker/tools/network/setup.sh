#!/usr/bin/env bash
# Tool setup: network
# Validates diagnostics and network probing binaries.

if ! command -v nc >/dev/null 2>&1; then
    echo "[clawden/tools/network] Error: nc is not installed" >&2
    return 1
fi

if ! command -v dig >/dev/null 2>&1; then
    echo "[clawden/tools/network] Error: dig is not installed" >&2
    return 1
fi

if ! command -v traceroute >/dev/null 2>&1; then
    echo "[clawden/tools/network] Error: traceroute is not installed" >&2
    return 1
fi

echo "[clawden/tools/network] nc, dig, traceroute ready"
