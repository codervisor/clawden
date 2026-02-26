#!/usr/bin/env tsx
/**
 * Publish platform-specific binary packages to npm.
 * Platform packages MUST be published before the main packages.
 *
 * ⚠️  This script should ONLY be run in CI/CD!
 * Publishing from a local machine may result in wrong platform binaries.
 *
 * Usage:
 *   tsx scripts/publish-platform-packages.ts [--dry-run] [--tag <tag>] [--allow-local]
 */

import { execSync } from 'node:child_process';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

const PLATFORMS = ['darwin-x64', 'darwin-arm64', 'linux-x64', 'windows-x64'];

function checkCIEnvironment(allowLocal: boolean): void {
  const isCI = process.env.CI === 'true' || process.env.GITHUB_ACTIONS === 'true';

  if (!isCI && !allowLocal) {
    console.error('❌ ERROR: This script should only be run in CI/CD!');
    console.error('');
    console.error('Publishing from a local machine may result in wrong platform binaries.');
    console.error('');
    console.error('Override (dangerous):');
    console.error('  tsx scripts/publish-platform-packages.ts --allow-local');
    console.error('');
    console.error('Recommended: Use the GitHub Actions workflow:');
    console.error('  gh workflow run publish.yml');
    process.exit(1);
  }

  if (!isCI && allowLocal) {
    console.warn('⚠️  WARNING: Running in local mode (--allow-local)');
    console.warn('⚠️  Make sure all platform binaries are correctly cross-compiled!\n');
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

    const binaryPath = path.join(packageDir, packageJson.main);
    try {
      await fs.access(binaryPath);
    } catch {
      return { package: packageName, success: false, error: `Binary not found: ${packageJson.main}` };
    }

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

async function main() {
  const args = process.argv.slice(2);
  const dryRun = args.includes('--dry-run');
  const allowLocal = args.includes('--allow-local');
  const tagIndex = args.indexOf('--tag');
  const tag = tagIndex !== -1 ? args[tagIndex + 1] : undefined;

  checkCIEnvironment(allowLocal);

  console.log('📤 Publishing platform packages...\n');
  if (dryRun) console.log('🔍 DRY RUN - No packages will be published\n');

  const packagesToPublish = PLATFORMS.map((platform) => ({
    dir: path.join(ROOT, 'npm', 'clawden', 'binaries', platform),
  }));

  console.log(`Publishing ${packagesToPublish.length} platform packages in parallel...\n`);

  const results = await Promise.all(
    packagesToPublish.map(({ dir }) => publishPackage(dir, dryRun, tag))
  );

  console.log('\n📁 CLI Platform Packages:');
  for (const result of results) {
    if (result.success) {
      console.log(`  ✓ ${result.package}`);
    } else {
      console.log(`  ✗ ${result.package}: ${result.error}`);
    }
  }

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
    console.log('\n✅ Platform packages published successfully!');
  }
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
