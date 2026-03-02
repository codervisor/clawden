#!/usr/bin/env bash
# Tool setup: code-tools
# Validates fast code navigation binaries.

if ! command -v rg >/dev/null 2>&1; then
    echo "[clawden/tools/code-tools] Error: ripgrep (rg) is not installed" >&2
    return 1
fi

if ! command -v fdfind >/dev/null 2>&1 && ! command -v fd >/dev/null 2>&1; then
    echo "[clawden/tools/code-tools] Error: fd-find/fd is not installed" >&2
    return 1
fi

if ! command -v bat >/dev/null 2>&1 && ! command -v batcat >/dev/null 2>&1; then
    echo "[clawden/tools/code-tools] Error: bat/batcat is not installed" >&2
    return 1
fi

echo "[clawden/tools/code-tools] rg, fd, bat ready"
