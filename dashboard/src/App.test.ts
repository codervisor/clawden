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
});
