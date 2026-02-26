#!/usr/bin/env tsx
/**
 * Validate that no package.json files contain workspace:* protocol references.
 *
 * Run after prepare-publish to catch any leaks before npm publish.
 *
 * Usage:
 *   tsx scripts/validate-no-workspace-protocol.ts
 */

import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

interface WorkspaceRef {
  file: string;
  depType: string;
  depName: string;
}

function checkDeps(deps: Record<string, string> | undefined, depType: string, file: string, refs: WorkspaceRef[]): void {
  if (!deps) return;
  for (const [name, version] of Object.entries(deps)) {
    if (version.startsWith('workspace:')) {
      refs.push({ file, depType, depName: name });
    }
  }
}

async function main() {
  console.log('🔍 Checking for workspace:* protocol references...\n');

  const packageFiles = [
    'npm/clawden/package.json',
    'sdk/package.json',
  ];

  const refs: WorkspaceRef[] = [];

  for (const file of packageFiles) {
    const fullPath = path.join(ROOT, file);
    try {
      const pkg = JSON.parse(await fs.readFile(fullPath, 'utf-8'));
      checkDeps(pkg.dependencies, 'dependencies', file, refs);
      checkDeps(pkg.devDependencies, 'devDependencies', file, refs);
      checkDeps(pkg.peerDependencies, 'peerDependencies', file, refs);
      checkDeps(pkg.optionalDependencies, 'optionalDependencies', file, refs);
      console.log(`  ✓ ${file}`);
    } catch {
      console.log(`  ⏭️  ${file} (not found, skipping)`);
    }
  }

  if (refs.length > 0) {
    console.log('\n❌ Found workspace:* references that would break npm install:\n');
    for (const ref of refs) {
      console.log(`  ${ref.file}: ${ref.depType}.${ref.depName}`);
    }
    console.log('\nRun prepare-publish first: tsx scripts/prepare-publish.ts');
    process.exit(1);
  }

  console.log('\n✅ No workspace:* references found');
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
