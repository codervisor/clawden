import { describe, expect, it } from 'vitest';
import { defineSkill } from './index';

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
