export interface SkillContext {
  runtime: string;
  input: string;
}

export interface SkillDefinition {
  name: string;
  version: string;
  runtimes: string[];
  tools: string[];
  execute: (context: SkillContext) => Promise<unknown>;
  adapters?: Record<string, unknown>;
}

export function defineSkill(definition: SkillDefinition): SkillDefinition {
  return definition;
}

// ---------------------------------------------------------------------------
// Cross-runtime skill test harness
// ---------------------------------------------------------------------------

export interface SkillTestResult {
  runtime: string;
  passed: boolean;
  output: unknown;
  error?: string;
  durationMs: number;
}

export interface SkillTestSuite {
  skill: string;
  results: SkillTestResult[];
  allPassed: boolean;
}

/**
 * Run a skill's `execute` against each of the skill's declared runtimes.
 * Returns a test suite with individual results per runtime.
 */
export async function testSkill(
  skill: SkillDefinition,
  input: string,
): Promise<SkillTestSuite> {
  const results: SkillTestResult[] = [];

  for (const runtime of skill.runtimes) {
    const start = Date.now();
    try {
      const output = await skill.execute({ runtime, input });
      results.push({
        runtime,
        passed: true,
        output,
        durationMs: Date.now() - start,
      });
    } catch (err) {
      results.push({
        runtime,
        passed: false,
        output: null,
        error: err instanceof Error ? err.message : String(err),
        durationMs: Date.now() - start,
      });
    }
  }

  return {
    skill: skill.name,
    results,
    allPassed: results.every((r) => r.passed),
  };
}

// ---------------------------------------------------------------------------
// Marketplace registry protocol types
// ---------------------------------------------------------------------------

export interface MarketplaceEntry {
  name: string;
  version: string;
  description: string;
  author: string;
  runtimes: string[];
  tools: string[];
  downloads: number;
  publishedAt: string;
}

export interface MarketplaceSearchQuery {
  query?: string;
  runtime?: string;
  tool?: string;
  limit?: number;
  offset?: number;
}

export interface MarketplaceSearchResult {
  entries: MarketplaceEntry[];
  total: number;
}

/**
 * Marketplace client stub. In production, this would call the ClawDen
 * marketplace registry API. Currently provides the type surface and
 * validates query construction.
 */
export function buildSearchUrl(
  baseUrl: string,
  query: MarketplaceSearchQuery,
): string {
  const params = new URLSearchParams();
  if (query.query) params.set('q', query.query);
  if (query.runtime) params.set('runtime', query.runtime);
  if (query.tool) params.set('tool', query.tool);
  if (query.limit !== undefined) params.set('limit', String(query.limit));
  if (query.offset !== undefined) params.set('offset', String(query.offset));
  return `${baseUrl}/api/v1/skills/search?${params.toString()}`;
}
