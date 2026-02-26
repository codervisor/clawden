#!/usr/bin/env tsx
/**
 * Publish main packages (clawden CLI wrapper and @clawden/sdk).
 *
 * IMPORTANT: Run publish-platform-packages.ts FIRST!
 * Platform packages must be available on npm before publishing main packages.
 *
 * ⚠️  This script should ONLY be run in CI/CD!
 *
 * Usage:
 *   tsx scripts/publish-main-packages.ts [--dry-run] [--tag <tag>] [--allow-local]
 */

import { execSync } from 'node:child_process';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

function checkCIEnvironment(allowLocal: boolean): void {
  const isCI = process.env.CI === 'true' || process.env.GITHUB_ACTIONS === 'true';

  if (!isCI && !allowLocal) {
    console.error('❌ ERROR: This script should only be run in CI/CD!');
    console.error('');
    console.error('Override: tsx scripts/publish-main-packages.ts --allow-local');
    console.error('Recommended: gh workflow run publish.yml');
    process.exit(1);
  }

  if (!isCI && allowLocal) {
    console.warn('⚠️  WARNING: Running in local mode (--allow-local)\n');
  }
}

interface PublishResult {
  package: string;
  success: boolean;
  error?: string;
}

async function publishPackage(packageDir: string, dryRun: boolean, tag?: string): Promise<PublishResult> {
  const packageJsonPath = path.join(packageDir, 'package.json');

  try {
    const packageJson = JSON.parse(await fs.readFile(packageJsonPath, 'utf-8'));
    const packageName = packageJson.name;

    let command = 'npm publish --access public';
    if (tag) command += ` --tag ${tag}`;
    if (dryRun) command += ' --dry-run';

    console.log(`  📦 Publishing ${packageName}${tag ? ` (tag: ${tag})` : ''}...`);

    try {
      execSync(command, { cwd: packageDir, stdio: 'pipe' });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      if (message.includes('You cannot publish over the previously published versions')) {
        console.log(`  ⚠️  ${packageName} already published (skipped)`);
        return { package: packageName, success: true };
      }
      throw error;
    }

    return { package: packageName, success: true };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { package: packageDir, success: false, error: message };
  }
}

async function verifyPlatformPackages(): Promise<boolean> {
  console.log('🔍 Verifying platform packages are published...\n');

  const packagesToCheck = [
    '@clawden/cli-darwin-arm64',
    '@clawden/cli-linux-x64',
  ];

  for (const pkg of packagesToCheck) {
    try {
      execSync(`npm view ${pkg} version`, { stdio: 'pipe' });
      console.log(`  ✓ ${pkg} available on npm`);
    } catch {
      console.log(`  ✗ ${pkg} not found on npm`);
      console.log('\n❌ Platform packages must be published first!');
      console.log('   Run: pnpm publish:platforms');
      return false;
    }
  }

  console.log('');
  return true;
}

async function main() {
  const args = process.argv.slice(2);
  const dryRun = args.includes('--dry-run');
  const allowLocal = args.includes('--allow-local');
  const tagIndex = args.indexOf('--tag');
  const tag = tagIndex !== -1 ? args[tagIndex + 1] : undefined;

  checkCIEnvironment(allowLocal);

  console.log('📤 Publishing main packages...\n');
  if (dryRun) {
    console.log('🔍 DRY RUN - No packages will be published\n');
  } else {
    const verified = await verifyPlatformPackages();
    if (!verified) process.exit(1);
  }

  const results: PublishResult[] = [];

  console.log('📁 Main Packages:');

  // Publish CLI wrapper
  const cliResult = await publishPackage(path.join(ROOT, 'npm', 'clawden'), dryRun, tag);
  results.push(cliResult);
  console.log(cliResult.success ? `  ✓ ${cliResult.package}` : `  ✗ ${cliResult.package}: ${cliResult.error}`);

  // Publish SDK
  const sdkResult = await publishPackage(path.join(ROOT, 'sdk'), dryRun, tag);
  results.push(sdkResult);
  console.log(sdkResult.success ? `  ✓ ${sdkResult.package}` : `  ✗ ${sdkResult.package}: ${sdkResult.error}`);

  const successful = results.filter((r) => r.success);
  const failed = results.filter((r) => !r.success);

  console.log(`\n${'='.repeat(50)}`);
  console.log(`Summary: Published: ${successful.length}, Failed: ${failed.length}`);

  if (failed.length > 0) {
    console.log('\n❌ Failed packages:');
    for (const r of failed) console.log(`  - ${r.package}: ${r.error}`);
    process.exit(1);
  }

  if (!dryRun && successful.length > 0) {
    console.log('\n✅ Main packages published successfully!');
  }
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
