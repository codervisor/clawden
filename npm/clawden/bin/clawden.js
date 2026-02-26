#!/usr/bin/env node
/**
 * ClawDen CLI Binary Wrapper
 *
 * Detects the current platform/architecture and spawns the appropriate
 * Rust binary with the provided arguments.
 *
 * Resolution order:
 * 1. target/debug/clawden-cli   (local cargo build)
 * 2. target/release/clawden-cli (local cargo build --release)
 * 3. @clawden/cli-{platform}    (npm-installed platform package)
 * 4. binaries/{platform}/       (local fallback)
 */

import { spawn } from 'child_process';
import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { accessSync, openSync, readSync, closeSync } from 'fs';

const require = createRequire(import.meta.url);
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const DEBUG = process.env.CLAWDEN_DEBUG === '1';
const debug = (...args) => DEBUG && console.error('[clawden debug]', ...args);

const PLATFORM_MAP = {
  darwin: { x64: 'darwin-x64', arm64: 'darwin-arm64' },
  linux: { x64: 'linux-x64' },
  win32: { x64: 'windows-x64' },
};

const MACHO_MAGICS = new Set([
  0xfeedface, 0xfeedfacf, 0xcefaedfe, 0xcffaedfe, 0xcafebabe, 0xbebafeca,
]);

function readHeaderBytes(filePath) {
  const fd = openSync(filePath, 'r');
  try {
    const buffer = Buffer.alloc(4);
    const bytesRead = readSync(fd, buffer, 0, 4, 0);
    return bytesRead === 4 ? buffer : null;
  } finally {
    closeSync(fd);
  }
}

function isValidBinaryHeader(filePath, platform) {
  try {
    const header = readHeaderBytes(filePath);
    if (!header) return false;

    if (platform === 'linux') {
      return header[0] === 0x7f && header[1] === 0x45 && header[2] === 0x4c && header[3] === 0x46;
    }
    if (platform === 'win32') {
      return header[0] === 0x4d && header[1] === 0x5a;
    }
    if (platform === 'darwin') {
      const magicBE = header.readUInt32BE(0);
      const magicLE = header.readUInt32LE(0);
      return MACHO_MAGICS.has(magicBE) || MACHO_MAGICS.has(magicLE);
    }
    return false;
  } catch {
    return false;
  }
}

function getBinaryPath() {
  const platform = process.platform;
  const arch = process.arch;

  debug('Platform detection:', { platform, arch });

  const platformKey = PLATFORM_MAP[platform]?.[arch];
  if (!platformKey) {
    console.error(`Unsupported platform: ${platform}-${arch}`);
    console.error('Supported: macOS (x64/arm64), Linux (x64), Windows (x64)');
    process.exit(1);
  }

  const isWindows = platform === 'win32';
  const binaryName = isWindows ? 'clawden-cli.exe' : 'clawden-cli';
  const packageName = `@clawden/cli-${platformKey}`;

  debug('Binary info:', { platformKey, binaryName, packageName });

  // 1. Try target/debug (local cargo build)
  try {
    const debugPath = join(__dirname, '..', '..', '..', 'target', 'debug', binaryName);
    debug('Trying cargo debug binary:', debugPath);
    accessSync(debugPath);
    if (isValidBinaryHeader(debugPath, platform)) {
      debug('Found cargo debug binary:', debugPath);
      return debugPath;
    }
  } catch (e) {
    debug('Cargo debug binary not found:', e.message);
  }

  // 2. Try target/release (local cargo build --release)
  try {
    const releasePath = join(__dirname, '..', '..', '..', 'target', 'release', binaryName);
    debug('Trying cargo release binary:', releasePath);
    accessSync(releasePath);
    if (isValidBinaryHeader(releasePath, platform)) {
      debug('Found cargo release binary:', releasePath);
      return releasePath;
    }
  } catch (e) {
    debug('Cargo release binary not found:', e.message);
  }

  // 3. Try npm-installed platform package
  try {
    const resolvedPath = require.resolve(`${packageName}/${binaryName}`);
    if (isValidBinaryHeader(resolvedPath, platform)) {
      debug('Found platform package binary:', resolvedPath);
      return resolvedPath;
    }
  } catch (e) {
    debug('Platform package not found:', packageName, '-', e.message);
  }

  // 4. Try local binaries directory (fallback)
  try {
    const localPath = join(__dirname, '..', 'binaries', platformKey, binaryName);
    debug('Trying local binary:', localPath);
    accessSync(localPath);
    if (isValidBinaryHeader(localPath, platform)) {
      debug('Found local binary:', localPath);
      return localPath;
    }
  } catch (e) {
    debug('Local binary not found:', e.message);
  }

  console.error(`Binary not found for ${platform}-${arch}`);
  console.error(`Expected package: ${packageName}`);
  console.error('');
  console.error('To install:');
  console.error('  npm install -g clawden');
  console.error('');
  console.error('If you installed globally, try reinstalling:');
  console.error('  npm uninstall -g clawden && npm install -g clawden');
  console.error('');
  console.error('If your npm config omits optional dependencies, enable them and reinstall.');
  process.exit(1);
}

const binaryPath = getBinaryPath();
const args = process.argv.slice(2);

debug('Spawning binary:', binaryPath);
debug('Arguments:', args);

const child = spawn(binaryPath, args, {
  stdio: 'inherit',
  windowsHide: true,
});

child.on('exit', (code) => {
  debug('Binary exited with code:', code);
  process.exit(code ?? 1);
});

child.on('error', (err) => {
  console.error('Failed to start clawden:', err.message);
  debug('Spawn error:', err);
  process.exit(1);
});
