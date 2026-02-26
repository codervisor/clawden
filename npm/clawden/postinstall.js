#!/usr/bin/env node
/**
 * Postinstall: ensure the platform binary is executable.
 * npm strips file permissions, so we fix them after install.
 */

import { chmodSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

const PLATFORM_MAP = {
  darwin: { x64: 'darwin-x64', arm64: 'darwin-arm64' },
  linux: { x64: 'linux-x64' },
  win32: { x64: 'windows-x64' },
};

const platformKey = PLATFORM_MAP[process.platform]?.[process.arch];
if (!platformKey || process.platform === 'win32') {
  process.exit(0);
}

const binaryPath = join(__dirname, 'binaries', platformKey, 'clawden-cli');
try {
  chmodSync(binaryPath, 0o755);
} catch {
  // Binary may not exist locally (resolved via optional dep instead)
}
