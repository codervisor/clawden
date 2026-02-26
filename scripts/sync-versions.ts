#!/usr/bin/env tsx
/**
 * Sync versions across all packages from root package.json.
 *
 * Updates:
 * - npm/clawden/package.json
 * - sdk/package.json
 * - Cargo.toml workspace version
 * - Platform binary package.json files (if they exist)
 *
 * Usage:
 *   tsx scripts/sync-versions.ts [--dry-run]
 */

import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

const PLATFORMS = ['darwin-x64', 'darwin-arm64', 'linux-x64', 'windows-x64'];

async function fileExists(filePath: string): Promise<boolean> {
  try {
    await fs.access(filePath);
    return true;
  } catch {
    return false;
  }
}

async function resolveTargetVersion(): Promise<string> {
  const rootPkg = JSON.parse(await fs.readFile(path.join(ROOT, 'package.json'), 'utf-8'));
  if (!rootPkg.version) throw new Error('Root package.json missing version');
  return rootPkg.version;
}

async function syncJsonPackage(filePath: string, targetVersion: string, dryRun: boolean): Promise<boolean> {
  if (!(await fileExists(filePath))) return false;

  const pkg = JSON.parse(await fs.readFile(filePath, 'utf-8'));
  if (pkg.version === targetVersion) return false;

  const label = path.relative(ROOT, filePath);
  if (!dryRun) {
    pkg.version = targetVersion;
    await fs.writeFile(filePath, JSON.stringify(pkg, null, 2) + '\n');
    console.log(`  ✓ ${label}: ${pkg.version} → ${targetVersion}`);
  } else {
    console.log(`  ℹ ${label}: would update to ${targetVersion} (dry run)`);
  }
  return true;
}

async function syncCargoVersion(targetVersion: string, dryRun: boolean): Promise<boolean> {
  const cargoPath = path.join(ROOT, 'Cargo.toml');
  if (!(await fileExists(cargoPath))) return false;

  const content = await fs.readFile(cargoPath, 'utf-8');
  const re = /(\[workspace\.package\][\s\S]*?\nversion\s*=\s*")([^"]+)(")/m;
  const match = content.match(re);

  if (!match) {
    console.warn('  ⚠️  Could not find [workspace.package] version in Cargo.toml');
    return false;
  }

  if (match[2] === targetVersion) return false;

  if (!dryRun) {
    const updated = content.replace(re, `$1${targetVersion}$3`);
    await fs.writeFile(cargoPath, updated);
    console.log(`  ✓ Cargo.toml: ${match[2]} → ${targetVersion}`);
  } else {
    console.log(`  ℹ Cargo.toml: would update to ${targetVersion} (dry run)`);
  }
  return true;
}

async function main() {
  const dryRun = process.argv.includes('--dry-run');

  console.log('🔄 Syncing workspace package versions...\n');
  const targetVersion = await resolveTargetVersion();
  console.log(`📦 Root version: ${targetVersion}\n`);

  let updated = 0;

  // Sync Cargo.toml
  console.log('Rust workspace:');
  if (await syncCargoVersion(targetVersion, dryRun)) updated++;
  else console.log(`  ✓ Cargo.toml: already at ${targetVersion}`);

  // Sync npm packages
  console.log('\nnpm packages:');
  const npmPackages = [
    path.join(ROOT, 'npm', 'clawden', 'package.json'),
    path.join(ROOT, 'sdk', 'package.json'),
    path.join(ROOT, 'dashboard', 'package.json'),
  ];

  for (const pkgPath of npmPackages) {
    if (await syncJsonPackage(pkgPath, targetVersion, dryRun)) updated++;
    else {
      const label = path.relative(ROOT, pkgPath);
      if (await fileExists(pkgPath)) console.log(`  ✓ ${label}: already at ${targetVersion}`);
    }
  }

  // Sync platform packages
  console.log('\nPlatform packages:');
  for (const platform of PLATFORMS) {
    const pkgPath = path.join(ROOT, 'npm', 'clawden', 'binaries', platform, 'package.json');
    if (await syncJsonPackage(pkgPath, targetVersion, dryRun)) updated++;
  }

  console.log(`\n✅ Synced ${updated} package(s)`);
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
