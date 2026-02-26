#!/usr/bin/env tsx
/**
 * Generate package.json and postinstall.js for platform-specific binary packages.
 *
 * Run AFTER binaries are copied to platform directories, BEFORE publishing.
 *
 * Usage:
 *   tsx scripts/generate-platform-manifests.ts
 */

import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

const PLATFORMS = ['darwin-x64', 'darwin-arm64', 'linux-x64', 'windows-x64'];

interface PlatformInfo {
  label: string;
  os: string;
  cpu: string;
}

function getPlatformInfo(platformKey: string): PlatformInfo {
  const [os, arch] = platformKey.split('-');
  const osMap: Record<string, string> = { darwin: 'macOS', linux: 'Linux', windows: 'Windows' };
  const archMap: Record<string, string> = { x64: 'x64', arm64: 'ARM64' };
  return {
    label: `${osMap[os] || os} ${archMap[arch] || arch}`,
    os: os === 'windows' ? 'win32' : os,
    cpu: arch,
  };
}

async function resolveTargetVersion(): Promise<string> {
  const rootPkg = JSON.parse(await fs.readFile(path.join(ROOT, 'package.json'), 'utf-8'));
  return rootPkg.version;
}

async function generateManifests(platformKey: string, version: string): Promise<void> {
  const isWindows = platformKey.startsWith('windows-');
  const binaryFileName = isWindows ? 'clawden-cli.exe' : 'clawden-cli';
  const platformInfo = getPlatformInfo(platformKey);
  const packageName = `@clawden/cli-${platformKey}`;

  const destDir = path.join(ROOT, 'npm', 'clawden', 'binaries', platformKey);
  const binaryPath = path.join(destDir, binaryFileName);

  try {
    await fs.access(binaryPath);
  } catch {
    console.warn(`⚠️  Binary not found: ${binaryPath}, skipping`);
    return;
  }

  // Generate package.json
  const packageJson = {
    name: packageName,
    version,
    description: `ClawDen CLI binary for ${platformInfo.label}`,
    os: [platformInfo.os],
    cpu: [platformInfo.cpu],
    main: binaryFileName,
    files: [binaryFileName, 'postinstall.js'],
    scripts: {
      postinstall: 'node postinstall.js',
    },
    repository: {
      type: 'git',
      url: 'https://github.com/codervisor/clawden.git',
    },
    license: 'MIT',
  };

  await fs.writeFile(path.join(destDir, 'package.json'), JSON.stringify(packageJson, null, 2) + '\n');

  // Generate postinstall.js
  const postinstallContent = isWindows
    ? `#!/usr/bin/env node\nconsole.log('✓ ${binaryFileName} ready');\n`
    : `#!/usr/bin/env node
const { chmodSync } = require('fs');
const { join } = require('path');
try {
  chmodSync(join(__dirname, '${binaryFileName}'), 0o755);
  console.log('✓ Set execute permissions on ${binaryFileName}');
} catch (err) {
  console.error('Warning: Could not set execute permissions:', err.message);
}
`;

  await fs.writeFile(path.join(destDir, 'postinstall.js'), postinstallContent);
  console.log(`  ✓ Generated manifests for ${packageName}`);
}

async function main() {
  console.log('📝 Generating platform package manifests...\n');
  const version = await resolveTargetVersion();
  console.log(`Version: ${version}\n`);

  for (const platformKey of PLATFORMS) {
    console.log(`Platform: ${platformKey}`);
    await generateManifests(platformKey, version);
  }

  console.log('\n✅ Manifest generation complete');
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
