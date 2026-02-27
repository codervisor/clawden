#!/usr/bin/env bash
# Tool setup: git
# Installs git and SSH client for runtime use.

if ! command -v git &>/dev/null; then
    echo "[clawden/tools/git] Installing git..."
    sudo apt-get update -qq && sudo apt-get install -y -qq git openssh-client
fi

echo "[clawden/tools/git] git $(git --version | awk '{print $3}') ready"
