#!/usr/bin/env bash
# Tool setup: http
# Installs curl and wget for runtime use.

if ! command -v curl &>/dev/null; then
    echo "[clawden/tools/http] Installing curl..."
    sudo apt-get update -qq && sudo apt-get install -y -qq curl
fi

if ! command -v wget &>/dev/null; then
    echo "[clawden/tools/http] Installing wget..."
    sudo apt-get update -qq && sudo apt-get install -y -qq wget
fi

echo "[clawden/tools/http] HTTP tools ready (curl, wget)"
