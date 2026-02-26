import { describe, expect, it } from 'vitest';
import { buildSearchUrl, defineSkill, testSkill } from './index';

describe('defineSkill', () => {
  it('returns the same skill definition', async () => {
    const skill = defineSkill({
      name: 'echo',
      version: '0.1.0',
      runtimes: ['openclaw'],
      tools: [],
      execute: async () => 'ok',
    });

    expect(skill.name).toBe('echo');
    await expect(skill.execute({ runtime: 'openclaw', input: 'hi' })).resolves.toBe('ok');
  });
});

describe('testSkill', () => {
  it('runs skill across all declared runtimes', async () => {
    const skill = defineSkill({
      name: 'echo',
      version: '0.1.0',
      runtimes: ['openclaw', 'zeroclaw'],
      tools: [],
      execute: async (ctx) => `${ctx.runtime}: ${ctx.input}`,
    });

    const suite = await testSkill(skill, 'hello');
    expect(suite.skill).toBe('echo');
    expect(suite.allPassed).toBe(true);
    expect(suite.results).toHaveLength(2);
    expect(suite.results[0].runtime).toBe('openclaw');
    expect(suite.results[0].output).toBe('openclaw: hello');
    expect(suite.results[1].runtime).toBe('zeroclaw');
  });

  it('captures errors for failing runtimes', async () => {
    const skill = defineSkill({
      name: 'flaky',
      version: '0.1.0',
      runtimes: ['openclaw', 'zeroclaw'],
      tools: [],
      execute: async (ctx) => {
        if (ctx.runtime === 'zeroclaw') throw new Error('unsupported');
        return 'ok';
      },
    });

    const suite = await testSkill(skill, 'test');
    expect(suite.allPassed).toBe(false);
    expect(suite.results[0].passed).toBe(true);
    expect(suite.results[1].passed).toBe(false);
    expect(suite.results[1].error).toBe('unsupported');
  });
});

describe('buildSearchUrl', () => {
  it('builds URL with query params', () => {
    const url = buildSearchUrl('https://marketplace.clawden.dev', {
      query: 'scraper',
      runtime: 'openclaw',
      limit: 10,
    });
    expect(url).toContain('/api/v1/skills/search?');
    expect(url).toContain('q=scraper');
    expect(url).toContain('runtime=openclaw');
    expect(url).toContain('limit=10');
  });

  it('omits unset params', () => {
    const url = buildSearchUrl('https://marketplace.clawden.dev', {});
    expect(url).toBe('https://marketplace.clawden.dev/api/v1/skills/search?');
  });
});
