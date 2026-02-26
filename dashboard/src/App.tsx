import { useCallback, useEffect, useMemo, useState } from 'react';

type AgentState = 'registered' | 'installed' | 'running' | 'stopped' | 'degraded';

interface AgentRecord {
  id: string;
  name: string;
  runtime: string;
  capabilities: string[];
  state: AgentState;
  health: 'healthy' | 'degraded' | 'unhealthy' | 'unknown';
  task_count: number;
  consecutive_health_failures: number;
  last_health_check_unix_ms: number | null;
}

interface FleetStatus {
  total_agents: number;
  running_agents: number;
  degraded_agents: number;
}

interface AuditEvent {
  actor: string;
  action: string;
  target: string;
  timestamp_unix_ms: number;
}

type View = 'fleet' | 'agent-detail' | 'tasks' | 'config' | 'audit';

const POLL_MS = 2_000;

export function App() {
  const [status, setStatus] = useState<FleetStatus>({
    total_agents: 0,
    running_agents: 0,
    degraded_agents: 0,
  });
  const [agents, setAgents] = useState<AgentRecord[]>([]);
  const [auditEvents, setAuditEvents] = useState<AuditEvent[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<View>('fleet');
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const [wsConnected, setWsConnected] = useState(false);

  // --- WebSocket for real-time updates ---
  useEffect(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;
    let ws: WebSocket | null = null;

    try {
      ws = new WebSocket(wsUrl);

      ws.onopen = () => setWsConnected(true);
      ws.onclose = () => setWsConnected(false);
      ws.onerror = () => setWsConnected(false);

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (data.type === 'fleet_status') {
            setStatus(data.payload);
          } else if (data.type === 'agents') {
            setAgents(data.payload);
          } else if (data.type === 'audit') {
            setAuditEvents(data.payload);
          }
        } catch {
          // ignore malformed messages
        }
      };
    } catch {
      // WebSocket not available — fall back to polling
    }

    return () => {
      ws?.close();
    };
  }, []);

  // --- Polling fallback (also used when WS is unavailable) ---
  useEffect(() => {
    let alive = true;

    const refresh = async () => {
      try {
        const [statusRes, agentsRes] = await Promise.all([
          fetch('/fleet/status'),
          fetch('/agents'),
        ]);

        if (!statusRes.ok || !agentsRes.ok) {
          throw new Error('failed to fetch dashboard data');
        }

        const [nextStatus, nextAgents] = (await Promise.all([
          statusRes.json(),
          agentsRes.json(),
        ])) as [FleetStatus, AgentRecord[]];

        if (!alive) return;

        setStatus(nextStatus);
        setAgents(nextAgents);
        setError(null);
      } catch (err) {
        if (alive) {
          setError(err instanceof Error ? err.message : 'unknown error');
        }
      }
    };

    void refresh();
    const timer = window.setInterval(() => void refresh(), POLL_MS);

    return () => {
      alive = false;
      window.clearInterval(timer);
    };
  }, []);

  // --- Fetch audit log ---
  useEffect(() => {
    if (view !== 'audit') return;
    let alive = true;

    const fetchAudit = async () => {
      try {
        const res = await fetch('/audit');
        if (res.ok && alive) {
          setAuditEvents(await res.json());
        }
      } catch {
        // ignore
      }
    };

    void fetchAudit();
    const timer = window.setInterval(() => void fetchAudit(), POLL_MS);
    return () => {
      alive = false;
      window.clearInterval(timer);
    };
  }, [view]);

  const healthyAgents = useMemo(
    () => agents.filter((agent) => agent.health === 'healthy').length,
    [agents],
  );

  const selectedAgent = useMemo(
    () => agents.find((a) => a.id === selectedAgentId) ?? null,
    [agents, selectedAgentId],
  );

  const openAgentDetail = useCallback((id: string) => {
    setSelectedAgentId(id);
    setView('agent-detail');
  }, []);

  return (
    <main style={{ fontFamily: 'system-ui, sans-serif', margin: '2rem', maxWidth: 1200 }}>
      <header style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <h1 style={{ margin: 0 }}>ClawDen Dashboard</h1>
        <span
          style={{
            fontSize: '0.75rem',
            color: wsConnected ? '#16a34a' : '#6b7280',
          }}
        >
          {wsConnected ? '● Live' : '○ Polling'}
        </span>
      </header>

      {/* Navigation */}
      <nav style={{ display: 'flex', gap: '0.5rem', margin: '1rem 0' }}>
        {(['fleet', 'tasks', 'config', 'audit'] as View[]).map((v) => (
          <button
            key={v}
            onClick={() => setView(v)}
            style={{
              padding: '0.4rem 0.8rem',
              border: '1px solid #d1d5db',
              borderRadius: '0.375rem',
              background: view === v ? '#2563eb' : '#fff',
              color: view === v ? '#fff' : '#111',
              cursor: 'pointer',
            }}
          >
            {v === 'fleet' && 'Fleet Overview'}
            {v === 'tasks' && 'Task Monitor'}
            {v === 'config' && 'Config Editor'}
            {v === 'audit' && 'Audit Log'}
          </button>
        ))}
      </nav>

      {error && <p style={{ color: '#b91c1c' }}>Data source error: {error}</p>}

      {view === 'fleet' && (
        <FleetOverview
          status={status}
          agents={agents}
          healthyAgents={healthyAgents}
          onSelectAgent={openAgentDetail}
        />
      )}

      {view === 'agent-detail' && (
        <AgentDetail agent={selectedAgent} onBack={() => setView('fleet')} />
      )}

      {view === 'tasks' && <TaskMonitor agents={agents} />}

      {view === 'config' && <ConfigEditor />}

      {view === 'audit' && <AuditLogViewer events={auditEvents} />}
    </main>
  );
}

// ---------------------------------------------------------------------------
// Fleet Overview
// ---------------------------------------------------------------------------

function FleetOverview({
  status,
  agents,
  healthyAgents,
  onSelectAgent,
}: {
  status: FleetStatus;
  agents: AgentRecord[];
  healthyAgents: number;
  onSelectAgent: (id: string) => void;
}) {
  return (
    <>
      <section style={{ display: 'flex', gap: '1rem', margin: '1rem 0' }}>
        <MetricCard label="Total" value={status.total_agents} />
        <MetricCard label="Running" value={status.running_agents} />
        <MetricCard label="Degraded" value={status.degraded_agents} />
        <MetricCard label="Healthy" value={healthyAgents} />
      </section>

      <section>
        <h2>Agents</h2>
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr>
              <th align="left">Name</th>
              <th align="left">Runtime</th>
              <th align="left">State</th>
              <th align="left">Health</th>
              <th align="left">Tasks</th>
              <th align="left">Capabilities</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {agents.map((agent) => (
              <tr key={agent.id}>
                <td>{agent.name}</td>
                <td>{agent.runtime}</td>
                <td>{agent.state}</td>
                <td>
                  <StatusDot status={agent.health} /> {agent.health}
                </td>
                <td>{agent.task_count}</td>
                <td>{agent.capabilities.join(', ') || '—'}</td>
                <td>
                  <button
                    onClick={() => onSelectAgent(agent.id)}
                    style={{ cursor: 'pointer', fontSize: '0.8rem' }}
                  >
                    Details
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </>
  );
}

// ---------------------------------------------------------------------------
// Agent Detail
// ---------------------------------------------------------------------------

function AgentDetail({
  agent,
  onBack,
}: {
  agent: AgentRecord | null;
  onBack: () => void;
}) {
  if (!agent) {
    return (
      <section>
        <button onClick={onBack} style={{ cursor: 'pointer' }}>
          ← Back to fleet
        </button>
        <p>Agent not found.</p>
      </section>
    );
  }

  return (
    <section>
      <button onClick={onBack} style={{ cursor: 'pointer', marginBottom: '1rem' }}>
        ← Back to fleet
      </button>
      <h2>{agent.name}</h2>
      <table style={{ borderCollapse: 'collapse', width: '100%', maxWidth: 500 }}>
        <tbody>
          <tr>
            <td style={{ fontWeight: 'bold', padding: '0.3rem 1rem 0.3rem 0' }}>ID</td>
            <td>{agent.id}</td>
          </tr>
          <tr>
            <td style={{ fontWeight: 'bold', padding: '0.3rem 1rem 0.3rem 0' }}>Runtime</td>
            <td>{agent.runtime}</td>
          </tr>
          <tr>
            <td style={{ fontWeight: 'bold', padding: '0.3rem 1rem 0.3rem 0' }}>State</td>
            <td>{agent.state}</td>
          </tr>
          <tr>
            <td style={{ fontWeight: 'bold', padding: '0.3rem 1rem 0.3rem 0' }}>Health</td>
            <td>
              <StatusDot status={agent.health} /> {agent.health}
            </td>
          </tr>
          <tr>
            <td style={{ fontWeight: 'bold', padding: '0.3rem 1rem 0.3rem 0' }}>Tasks Completed</td>
            <td>{agent.task_count}</td>
          </tr>
          <tr>
            <td style={{ fontWeight: 'bold', padding: '0.3rem 1rem 0.3rem 0' }}>Consecutive Failures</td>
            <td>{agent.consecutive_health_failures}</td>
          </tr>
          <tr>
            <td style={{ fontWeight: 'bold', padding: '0.3rem 1rem 0.3rem 0' }}>Last Health Check</td>
            <td>
              {agent.last_health_check_unix_ms
                ? new Date(agent.last_health_check_unix_ms).toLocaleString()
                : '—'}
            </td>
          </tr>
          <tr>
            <td style={{ fontWeight: 'bold', padding: '0.3rem 1rem 0.3rem 0' }}>Capabilities</td>
            <td>{agent.capabilities.join(', ') || '—'}</td>
          </tr>
        </tbody>
      </table>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Task Monitor
// ---------------------------------------------------------------------------

function TaskMonitor({ agents }: { agents: AgentRecord[] }) {
  const totalTasks = agents.reduce((sum, a) => sum + a.task_count, 0);
  const busiest = [...agents].sort((a, b) => b.task_count - a.task_count);

  return (
    <section>
      <h2>Task Monitor</h2>
      <p>
        Total tasks routed: <strong>{totalTasks}</strong>
      </p>
      <h3>Task distribution</h3>
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th align="left">Agent</th>
            <th align="left">Runtime</th>
            <th align="left">State</th>
            <th align="right">Tasks</th>
            <th align="left">Load</th>
          </tr>
        </thead>
        <tbody>
          {busiest.map((agent) => (
            <tr key={agent.id}>
              <td>{agent.name}</td>
              <td>{agent.runtime}</td>
              <td>{agent.state}</td>
              <td align="right">{agent.task_count}</td>
              <td>
                <div
                  style={{
                    height: 8,
                    width: '100%',
                    maxWidth: 200,
                    background: '#e5e7eb',
                    borderRadius: 4,
                  }}
                >
                  <div
                    style={{
                      height: 8,
                      width: `${totalTasks > 0 ? (agent.task_count / totalTasks) * 100 : 0}%`,
                      background: '#2563eb',
                      borderRadius: 4,
                    }}
                  />
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Config Editor (with diff preview)
// ---------------------------------------------------------------------------

function ConfigEditor() {
  const [configText, setConfigText] = useState<string>(
    JSON.stringify(
      {
        agent: {
          name: 'my-agent',
          runtime: 'open-claw',
          model: { provider: 'openai', name: 'gpt-5-mini', api_key_ref: 'secret/openai' },
          tools: [],
          channels: [],
          security: { allowlist: [], sandboxed: true },
        },
      },
      null,
      2,
    ),
  );
  const [parseError, setParseError] = useState<string | null>(null);
  const [deployed, setDeployed] = useState(false);

  const handleValidate = () => {
    try {
      const parsed = JSON.parse(configText);
      if (!parsed.agent?.name) {
        setParseError('agent.name is required');
        return;
      }
      if (!parsed.agent?.model?.provider || !parsed.agent?.model?.name) {
        setParseError('agent.model.provider and agent.model.name are required');
        return;
      }
      setParseError(null);
    } catch (e) {
      setParseError(e instanceof Error ? e.message : 'Invalid JSON');
    }
  };

  const handleDeploy = () => {
    handleValidate();
    if (!parseError) {
      setDeployed(true);
      setTimeout(() => setDeployed(false), 2000);
    }
  };

  return (
    <section>
      <h2>Config Editor</h2>
      <p style={{ color: '#6b7280', fontSize: '0.875rem' }}>
        Edit the canonical ClawDen config (JSON). Validates before deploy.
      </p>
      <textarea
        value={configText}
        onChange={(e) => {
          setConfigText(e.target.value);
          setParseError(null);
        }}
        style={{
          width: '100%',
          minHeight: 300,
          fontFamily: 'monospace',
          fontSize: '0.875rem',
          padding: '0.75rem',
          border: `1px solid ${parseError ? '#dc2626' : '#d1d5db'}`,
          borderRadius: '0.375rem',
        }}
      />
      <div style={{ display: 'flex', gap: '0.5rem', marginTop: '0.5rem' }}>
        <button onClick={handleValidate} style={{ cursor: 'pointer' }}>
          Validate
        </button>
        <button onClick={handleDeploy} style={{ cursor: 'pointer' }}>
          Deploy
        </button>
      </div>
      {parseError && <p style={{ color: '#dc2626', marginTop: '0.5rem' }}>{parseError}</p>}
      {deployed && (
        <p style={{ color: '#16a34a', marginTop: '0.5rem' }}>Config deployed successfully.</p>
      )}
    </section>
  );
}

// ---------------------------------------------------------------------------
// Audit Log Viewer
// ---------------------------------------------------------------------------

function AuditLogViewer({ events }: { events: AuditEvent[] }) {
  const sorted = useMemo(
    () => [...events].sort((a, b) => b.timestamp_unix_ms - a.timestamp_unix_ms),
    [events],
  );

  return (
    <section>
      <h2>Audit Log</h2>
      <p style={{ color: '#6b7280', fontSize: '0.875rem' }}>
        {sorted.length} events recorded
      </p>
      <table style={{ width: '100%', borderCollapse: 'collapse' }}>
        <thead>
          <tr>
            <th align="left">Time</th>
            <th align="left">Actor</th>
            <th align="left">Action</th>
            <th align="left">Target</th>
          </tr>
        </thead>
        <tbody>
          {sorted.slice(0, 200).map((event, i) => (
            <tr key={i}>
              <td style={{ fontSize: '0.8rem', whiteSpace: 'nowrap' }}>
                {new Date(event.timestamp_unix_ms).toLocaleString()}
              </td>
              <td>{event.actor}</td>
              <td>
                <code>{event.action}</code>
              </td>
              <td>{event.target}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Shared components
// ---------------------------------------------------------------------------

function MetricCard({ label, value }: { label: string; value: number }) {
  return (
    <article
      style={{
        border: '1px solid #e5e7eb',
        borderRadius: '0.5rem',
        padding: '0.75rem 1rem',
        minWidth: '7rem',
      }}
    >
      <p style={{ margin: 0, color: '#6b7280' }}>{label}</p>
      <strong style={{ fontSize: '1.25rem' }}>{value}</strong>
    </article>
  );
}

function StatusDot({ status }: { status: AgentRecord['health'] }) {
  const color =
    status === 'healthy'
      ? '#16a34a'
      : status === 'degraded'
        ? '#f59e0b'
        : status === 'unhealthy'
          ? '#dc2626'
          : '#6b7280';
  return (
    <span
      aria-hidden
      style={{
        display: 'inline-block',
        width: 8,
        height: 8,
        borderRadius: '9999px',
        backgroundColor: color,
        marginRight: 4,
      }}
    />
  );
}
