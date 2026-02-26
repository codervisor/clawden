#!/usr/bin/env tsx
/**
 * Add platform-specific optional dependencies to the CLI package.json.
 *
 * Runs AFTER generate-platform-manifests.ts and BEFORE prepare-publish.ts.
 *
 * Usage:
 *   tsx scripts/add-platform-deps.ts
 */

import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

const PLATFORMS = ['darwin-x64', 'darwin-arm64', 'linux-x64', 'windows-x64'];
const CLI_PKG_PATH = path.join(ROOT, 'npm', 'clawden', 'package.json');

async function main() {
  console.log('🔗 Adding platform-specific optional dependencies...\n');

  const pkg = JSON.parse(await fs.readFile(CLI_PKG_PATH, 'utf-8'));
  console.log(`📦 Processing ${pkg.name}...`);

  const optionalDeps: Record<string, string> = {};
  let found = 0;

  for (const platform of PLATFORMS) {
    const platformPkgPath = path.join(ROOT, 'npm', 'clawden', 'binaries', platform, 'package.json');
    try {
      await fs.access(platformPkgPath);
      const depName = `@clawden/cli-${platform}`;
      optionalDeps[depName] = pkg.version;
      found++;
      console.log(`  ✓ Found ${depName}`);
    } catch {
      // Platform package doesn't exist yet
    }
  }

  if (found === 0) {
    console.log('  ⏭️  No platform packages found');
    return;
  }

  pkg.optionalDependencies = optionalDeps;
  await fs.writeFile(CLI_PKG_PATH, JSON.stringify(pkg, null, 2) + '\n');
  console.log(`  ✅ Added ${found} platform dependencies`);
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
