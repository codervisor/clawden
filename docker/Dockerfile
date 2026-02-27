# ClawDen Runtime Container
# Single generic image supporting all Phase 1 runtimes:
# - OpenClaw (Node.js, JSON5 config, gateway on port 18789)
# - NanoClaw (Node.js, code-driven, Claude Agent SDK)
# - ZeroClaw (Rust binary, TOML + env vars)
# - PicoClaw (Go binary, JSON config)
#
# Image: ghcr.io/codervisor/clawden-runtime:latest
# Base: Debian slim + Node.js 22 LTS (for OpenClaw and NanoClaw)

FROM debian:bookworm-slim AS base

# Install system dependencies + Node.js 22 LTS
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    dnsutils \
    && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && npm install -g pnpm \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Create non-root clawden user (UID 10000)
RUN groupadd -g 10000 clawden && \
    useradd -u 10000 -g clawden -m -s /bin/bash clawden

# Create directory structure
RUN mkdir -p /opt/clawden/runtimes/openclaw \
             /opt/clawden/runtimes/nanoclaw \
             /opt/clawden/tools/git \
             /opt/clawden/tools/http \
             /opt/clawden/tools/browser \
             /opt/clawden/tools/gui \
             /home/clawden/workspace \
    && chown -R clawden:clawden /opt/clawden /home/clawden

# Copy entrypoint and tool setup scripts
COPY docker/entrypoint.sh /opt/clawden/entrypoint.sh
COPY docker/tools/ /opt/clawden/tools/
RUN chmod +x /opt/clawden/entrypoint.sh \
    && find /opt/clawden/tools -name "setup.sh" -exec chmod +x {} \;

# Copy internal defaults
COPY docker/defaults.toml /etc/clawden/defaults.toml

# Phase 1 runtime binaries would be copied here during CI build:
# COPY --from=zeroclaw-builder /usr/local/bin/zeroclaw /opt/clawden/runtimes/zeroclaw
# COPY --from=picoclaw-builder /usr/local/bin/picoclaw /opt/clawden/runtimes/picoclaw
# For OpenClaw and NanoClaw, their Node.js apps go into /opt/clawden/runtimes/{name}/

WORKDIR /home/clawden/workspace
VOLUME /home/clawden/workspace

# Health check — ClawDen or the runtime exposes GET /health
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -f http://localhost:${RUNTIME_PORT:-8080}/health || exit 1

USER clawden

ENTRYPOINT ["/opt/clawden/entrypoint.sh"]
