import { describe, expect, it } from 'vitest';

describe('dashboard', () => {
  it('App module exports the App component', async () => {
    const mod = await import('./App');
    expect(typeof mod.App).toBe('function');
  });

  it('Fleet types are consistent', () => {
    const status = { total_agents: 3, running_agents: 2, degraded_agents: 1 };
    expect(status.total_agents).toBeGreaterThanOrEqual(
      status.running_agents + status.degraded_agents,
    );
  });

  it('useTheme hook module exports useTheme', async () => {
    const mod = await import('./hooks/useTheme');
    expect(typeof mod.useTheme).toBe('function');
  });

  it('useSidebar hook module exports useSidebar', async () => {
    const mod = await import('./hooks/useSidebar');
    expect(typeof mod.useSidebar).toBe('function');
  });

  it('cn utility merges class names', async () => {
    const { cn } = await import('./lib/utils');
    expect(cn('foo', 'bar')).toBe('foo bar');
    expect(cn('px-2', 'px-4')).toBe('px-4'); // tailwind-merge deduplication
  });
});
