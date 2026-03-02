#!/usr/bin/env bash
# Tool setup: sandbox
# Validates bubblewrap and provides a thin execution wrapper.

if ! command -v bwrap >/dev/null 2>&1; then
    echo "[clawden/tools/sandbox] Error: bubblewrap (bwrap) is not installed" >&2
    return 1
fi

SANDBOX_BIN_DIR="${HOME}/.clawden/bin"
SANDBOX_WRAPPER="${SANDBOX_BIN_DIR}/clawden-sandbox"
mkdir -p "${SANDBOX_BIN_DIR}"

cat >"${SANDBOX_WRAPPER}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if [ "${1:-}" != "exec" ]; then
    echo "Usage: clawden-sandbox exec [--allow-network] [--timeout <seconds>] [--memory <mb>] -- <command...>" >&2
    exit 2
fi
shift

allow_network=0
timeout_secs=""
memory_mb=""

while [ "$#" -gt 0 ]; do
    case "$1" in
        --allow-network)
            allow_network=1
            shift
            ;;
        --timeout)
            timeout_secs="${2:-}"
            shift 2
            ;;
        --memory)
            memory_mb="${2:-}"
            shift 2
            ;;
        --)
            shift
            break
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 2
            ;;
    esac
done

if [ "$#" -eq 0 ]; then
    echo "No command provided" >&2
    exit 2
fi

bwrap_args=(
    --ro-bind / /
    --tmpfs /tmp
    --bind "$PWD" "$PWD"
    --dev /dev
    --unshare-pid
    --die-with-parent
)

if [ "${allow_network}" -eq 0 ]; then
    bwrap_args+=(--unshare-net)
fi

run_cmd=("$@")
if [ -n "${memory_mb}" ]; then
    max_kb=$((memory_mb * 1024))
    ulimit -v "${max_kb}"
fi

if [ -n "${timeout_secs}" ]; then
    exec timeout "${timeout_secs}" bwrap "${bwrap_args[@]}" -- "${run_cmd[@]}"
fi

exec bwrap "${bwrap_args[@]}" -- "${run_cmd[@]}"
EOF

chmod +x "${SANDBOX_WRAPPER}"
export PATH="${SANDBOX_BIN_DIR}:${PATH}"
export CLAWDEN_SANDBOX_BIN="${SANDBOX_WRAPPER}"

echo "[clawden/tools/sandbox] bwrap and clawden-sandbox wrapper ready"
