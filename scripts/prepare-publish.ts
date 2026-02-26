#!/usr/bin/env tsx
/**
 * Prepare packages for npm publish by replacing workspace:* dependencies
 * with actual versions. Also copies root README.md to CLI package.
 *
 * Usage:
 *   tsx scripts/prepare-publish.ts
 */

import { readFileSync, writeFileSync, existsSync, copyFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');

interface PackageJson {
  name: string;
  version: string;
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
  peerDependencies?: Record<string, string>;
  optionalDependencies?: Record<string, string>;
}

function readPackageJson(pkgPath: string): PackageJson {
  return JSON.parse(readFileSync(pkgPath, 'utf-8'));
}

function resolveWorkspaceVersion(depName: string): string | null {
  const pkgMap: Record<string, string> = {
    '@clawden/sdk': 'sdk/package.json',
    'clawden': 'npm/clawden/package.json',
    '@clawden/cli-darwin-x64': 'npm/clawden/binaries/darwin-x64/package.json',
    '@clawden/cli-darwin-arm64': 'npm/clawden/binaries/darwin-arm64/package.json',
    '@clawden/cli-linux-x64': 'npm/clawden/binaries/linux-x64/package.json',
    '@clawden/cli-windows-x64': 'npm/clawden/binaries/windows-x64/package.json',
  };

  const pkgPath = pkgMap[depName];
  if (!pkgPath) return null;

  const fullPath = join(ROOT, pkgPath);
  if (!existsSync(fullPath)) return null;

  return readPackageJson(fullPath).version;
}

function replaceWorkspaceDeps(deps: Record<string, string> | undefined, depType: string): boolean {
  if (!deps) return false;

  let changed = false;
  for (const [name, version] of Object.entries(deps)) {
    if (version.startsWith('workspace:')) {
      const resolved = resolveWorkspaceVersion(name);
      if (resolved) {
        deps[name] = resolved;
        console.log(`  ✓ ${depType}.${name}: workspace:* → ${resolved}`);
        changed = true;
      }
    }
  }
  return changed;
}

function processPackage(pkgPath: string): boolean {
  const fullPath = join(ROOT, pkgPath);
  if (!existsSync(fullPath)) {
    console.warn(`⚠️  Package not found: ${fullPath}`);
    return false;
  }

  const pkg = readPackageJson(fullPath);
  console.log(`\n📦 Processing ${pkg.name}...`);

  let changed = false;
  changed = replaceWorkspaceDeps(pkg.dependencies, 'dependencies') || changed;
  changed = replaceWorkspaceDeps(pkg.devDependencies, 'devDependencies') || changed;
  changed = replaceWorkspaceDeps(pkg.peerDependencies, 'peerDependencies') || changed;
  changed = replaceWorkspaceDeps(pkg.optionalDependencies, 'optionalDependencies') || changed;

  if (changed) {
    const backupPath = fullPath + '.backup';
    writeFileSync(backupPath, readFileSync(fullPath, 'utf-8'));
    console.log(`  💾 Backup saved to ${pkgPath}.backup`);
    writeFileSync(fullPath, JSON.stringify(pkg, null, 2) + '\n');
    console.log(`  ✅ Updated ${pkgPath}`);
    return true;
  } else {
    console.log(`  ⏭️  No workspace:* dependencies found`);
    return false;
  }
}

function main() {
  console.log('🚀 Preparing packages for npm publish...\n');

  const packages = [
    'npm/clawden/package.json',
    'sdk/package.json',
  ];

  const modified: string[] = [];
  for (const pkg of packages) {
    if (processPackage(pkg)) {
      modified.push(pkg);
    }
  }

  // Copy root README to CLI package for npm display
  const rootReadme = join(ROOT, 'README.md');
  const cliReadme = join(ROOT, 'npm', 'clawden', 'README.md');
  if (existsSync(rootReadme)) {
    copyFileSync(rootReadme, cliReadme);
    console.log('\n📄 Copied README.md to npm/clawden/');
  }

  console.log(`\n✅ Prepared ${modified.length} package(s) for publish`);
}

main();
