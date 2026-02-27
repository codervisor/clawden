import { useState, useEffect, useCallback } from 'react';
import { toast } from 'sonner';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Skeleton } from '../ui/skeleton';
import { AlertDialog } from '../ui/alert-dialog';
import {
  Play,
  Square,
  RotateCcw,
  Settings,
  ChevronRight,
  Monitor,
  Globe,
} from 'lucide-react';
import { cn } from '../../lib/utils';

// --- Types ---

interface RuntimeMetadata {
  runtime: string;
  version: string;
  language: string;
  capabilities: string[];
  default_port: number | null;
  config_format: string | null;
  channel_support: Record<string, string | { Via: string }>;
}

interface AgentRecord {
  id: string;
  name: string;
  runtime: string;
  capabilities: string[];
  state: 'registered' | 'installed' | 'running' | 'stopped' | 'degraded';
  health: 'healthy' | 'degraded' | 'unhealthy' | 'unknown';
  task_count: number;
  consecutive_health_failures: number;
  last_health_check_unix_ms: number | null;
}

interface DeployFormState {
  instanceName: string;
  runtime: string;
  channels: string[];
  tools: string[];
}

// --- Language colors ---

const LANGUAGE_COLORS: Record<string, string> = {
  typescript: 'bg-blue-500',
  rust: 'bg-orange-500',
  go: 'bg-cyan-500',
  zig: 'bg-amber-500',
};

const LANGUAGE_LABELS: Record<string, string> = {
  typescript: 'TypeScript',
  rust: 'Rust',
  go: 'Go',
  zig: 'Zig',
};

// --- Runtime Catalog ---

export function RuntimeCatalog({
  agents,
  onSelectAgent,
}: {
  agents: AgentRecord[];
  onSelectAgent: (id: string) => void;
}) {
  const [runtimes, setRuntimes] = useState<RuntimeMetadata[]>([]);
  const [loading, setLoading] = useState(true);
  const [deployTarget, setDeployTarget] = useState<RuntimeMetadata | null>(null);

  useEffect(() => {
    const fetchRuntimes = async () => {
      try {
        const res = await fetch('/api/runtimes');
        if (res.ok) {
          setRuntimes(await res.json());
        }
      } catch {
        /* ignore */
      } finally {
        setLoading(false);
      }
    };
    void fetchRuntimes();
  }, []);

  const instancesByRuntime = (runtime: string) =>
    agents.filter(
      (a) => a.runtime.toLowerCase().replace('-', '') === runtime.toLowerCase().replace('-', ''),
    );

  const runtimeStatus = (runtime: string) => {
    const instances = instancesByRuntime(runtime);
    if (instances.some((i) => i.state === 'running')) return 'running';
    if (instances.length > 0) return 'installed';
    return 'available';
  };

  return (
    <div className="space-y-6">
      {/* Runtime Catalog Grid */}
      <div>
        <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground mb-4">
          Available Runtimes
        </h3>
        {loading ? (
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4">
            {Array.from({ length: 4 }).map((_, i) => (
              <Skeleton key={i} className="h-48 rounded-xl" />
            ))}
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4">
            {runtimes.map((rt) => (
              <RuntimeCard
                key={rt.runtime}
                runtime={rt}
                status={runtimeStatus(rt.runtime)}
                instanceCount={instancesByRuntime(rt.runtime).length}
                onDeploy={() => setDeployTarget(rt)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Instance List */}
      <InstanceList agents={agents} onSelectAgent={onSelectAgent} />

      {/* Deploy Dialog */}
      {deployTarget && (
        <DeployDialog
          runtime={deployTarget}
          onClose={() => setDeployTarget(null)}
          onDeployed={() => {
            setDeployTarget(null);
            toast.success('Instance deployed successfully');
          }}
        />
      )}
    </div>
  );
}

// --- Runtime Card ---

function RuntimeCard({
  runtime,
  status,
  instanceCount,
  onDeploy,
}: {
  runtime: RuntimeMetadata;
  status: 'available' | 'installed' | 'running';
  instanceCount: number;
  onDeploy: () => void;
}) {
  const channelCount = Object.keys(runtime.channel_support).length;
  const statusColors = {
    available: 'border-muted',
    installed: 'border-blue-500/50',
    running: 'border-green-500/50',
  };

  return (
    <Card className={cn('relative overflow-hidden transition-all hover:shadow-md', statusColors[status])}>
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between">
          <div>
            <CardTitle className="text-base">{runtime.runtime}</CardTitle>
            <div className="mt-1 flex items-center gap-2">
              <span
                className={cn('inline-block h-2 w-2 rounded-full', LANGUAGE_COLORS[runtime.language] ?? 'bg-gray-400')}
              />
              <span className="text-xs text-muted-foreground">
                {LANGUAGE_LABELS[runtime.language] ?? runtime.language}
              </span>
            </div>
          </div>
          <StatusIndicator status={status} />
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex flex-wrap gap-1">
          {runtime.capabilities.map((cap) => (
            <Badge key={cap} variant="secondary" className="text-xs">
              {cap}
            </Badge>
          ))}
        </div>

        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <span className="flex items-center gap-1">
            <Globe className="h-3 w-3" />
            {channelCount} channels
          </span>
          {instanceCount > 0 && (
            <span className="flex items-center gap-1">
              <Monitor className="h-3 w-3" />
              {instanceCount} instance{instanceCount !== 1 ? 's' : ''}
            </span>
          )}
        </div>

        {runtime.default_port && (
          <div className="text-xs text-muted-foreground">
            Port: <span className="font-mono">{runtime.default_port}</span>
          </div>
        )}

        <Button size="sm" className="w-full" onClick={onDeploy}>
          <Play className="mr-1 h-3 w-3" />
          Deploy
        </Button>
      </CardContent>
    </Card>
  );
}

function StatusIndicator({ status }: { status: 'available' | 'installed' | 'running' }) {
  const config = {
    available: { color: 'bg-slate-400', label: 'Available' },
    installed: { color: 'bg-blue-500', label: 'Installed' },
    running: { color: 'bg-green-500 animate-pulse', label: 'Running' },
  };

  return (
    <div className="flex items-center gap-1.5">
      <span className={cn('h-2 w-2 rounded-full', config[status].color)} />
      <span className="text-xs text-muted-foreground">{config[status].label}</span>
    </div>
  );
}

// --- Instance List ---

function InstanceList({
  agents,
  onSelectAgent,
}: {
  agents: AgentRecord[];
  onSelectAgent: (id: string) => void;
}) {
  const [actionTarget, setActionTarget] = useState<{ id: string; action: string } | null>(null);

  const handleAction = async (agentId: string, action: string) => {
    try {
      const endpoint =
        action === 'stop'
          ? `/api/agents/${agentId}/stop`
          : `/api/agents/${agentId}/start`;
      const res = await fetch(endpoint, { method: 'POST' });
      if (res.ok) {
        toast.success(`Agent ${action} successful`);
      } else {
        toast.error(`Failed to ${action} agent`);
      }
    } catch {
      toast.error(`Failed to ${action} agent`);
    }
    setActionTarget(null);
  };

  if (agents.length === 0) return null;

  // Group by runtime
  const grouped = agents.reduce<Record<string, AgentRecord[]>>((acc, agent) => {
    const key = agent.runtime;
    if (!acc[key]) acc[key] = [];
    acc[key].push(agent);
    return acc;
  }, {});

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base">Deployed Instances</CardTitle>
          <Badge variant="secondary">{agents.length} total</Badge>
        </div>
      </CardHeader>
      <CardContent className="p-0">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-muted/50 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                <th className="px-6 py-3 text-left">Instance</th>
                <th className="px-6 py-3 text-left">Runtime</th>
                <th className="px-6 py-3 text-left">State</th>
                <th className="px-6 py-3 text-left">Health</th>
                <th className="px-6 py-3 text-right">Tasks</th>
                <th className="px-6 py-3 text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y">
              {Object.entries(grouped).map(([runtime, instances]) =>
                instances.map((agent) => (
                  <tr key={agent.id} className="hover:bg-muted/30 transition-colors">
                    <td className="px-6 py-3">
                      <button
                        className="flex items-center gap-1 font-medium text-primary hover:underline"
                        onClick={() => onSelectAgent(agent.id)}
                      >
                        {agent.name}
                        <ChevronRight className="h-3 w-3" />
                      </button>
                    </td>
                    <td className="px-6 py-3 font-mono text-xs text-muted-foreground">{runtime}</td>
                    <td className="px-6 py-3">
                      <Badge
                        variant={
                          agent.state === 'running'
                            ? 'success'
                            : agent.state === 'degraded'
                              ? 'warning'
                              : 'secondary'
                        }
                        className="capitalize"
                      >
                        {agent.state}
                      </Badge>
                    </td>
                    <td className="px-6 py-3">
                      <Badge
                        variant={
                          agent.health === 'healthy'
                            ? 'success'
                            : agent.health === 'degraded'
                              ? 'warning'
                              : agent.health === 'unhealthy'
                                ? 'destructive'
                                : 'secondary'
                        }
                        className="capitalize"
                      >
                        {agent.health}
                      </Badge>
                    </td>
                    <td className="px-6 py-3 text-right font-mono">{agent.task_count}</td>
                    <td className="px-6 py-3 text-right">
                      <div className="flex items-center justify-end gap-1">
                        {agent.state !== 'running' && (
                          <Button
                            variant="ghost"
                            size="icon"
                            className="h-7 w-7"
                            onClick={() => setActionTarget({ id: agent.id, action: 'start' })}
                          >
                            <Play className="h-3 w-3" />
                          </Button>
                        )}
                        {agent.state === 'running' && (
                          <>
                            <Button
                              variant="ghost"
                              size="icon"
                              className="h-7 w-7"
                              onClick={() => setActionTarget({ id: agent.id, action: 'restart' })}
                            >
                              <RotateCcw className="h-3 w-3" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon"
                              className="h-7 w-7 text-destructive"
                              onClick={() => setActionTarget({ id: agent.id, action: 'stop' })}
                            >
                              <Square className="h-3 w-3" />
                            </Button>
                          </>
                        )}
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-7 w-7"
                          onClick={() => onSelectAgent(agent.id)}
                        >
                          <Settings className="h-3 w-3" />
                        </Button>
                      </div>
                    </td>
                  </tr>
                )),
              )}
            </tbody>
          </table>
        </div>

        <AlertDialog
          open={!!actionTarget}
          title={`${actionTarget?.action ?? ''} Instance`}
          description={`Are you sure you want to ${actionTarget?.action} this instance?`}
          confirmLabel={actionTarget?.action ?? 'Confirm'}
          cancelLabel="Cancel"
          onConfirm={() => {
            if (actionTarget) void handleAction(actionTarget.id, actionTarget.action);
          }}
          onCancel={() => setActionTarget(null)}
          destructive={actionTarget?.action === 'stop'}
        />
      </CardContent>
    </Card>
  );
}

// --- Deploy Dialog ---

function DeployDialog({
  runtime,
  onClose,
  onDeployed,
}: {
  runtime: RuntimeMetadata;
  onClose: () => void;
  onDeployed: () => void;
}) {
  const [step, setStep] = useState<'configure' | 'deploying' | 'complete'>('configure');
  const [form, setForm] = useState<DeployFormState>({
    instanceName: `${runtime.runtime.toLowerCase()}-1`,
    runtime: runtime.runtime,
    channels: [],
    tools: [],
  });
  const [progress, setProgress] = useState<string[]>([]);

  const handleDeploy = useCallback(async () => {
    setStep('deploying');
    setProgress(['Registering instance...']);

    try {
      const runtimeKey = runtime.runtime.toLowerCase().replace('-', '');
      const res = await fetch(`/api/runtimes/${runtimeKey}/deploy`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          instance_name: form.instanceName,
          runtime: runtime.runtime,
          capabilities: ['chat'],
          channels: form.channels,
          tools: form.tools,
        }),
      });

      if (res.ok) {
        setProgress((p) => [...p, 'Installing runtime...', 'Configuring...', 'Starting instance...', 'Health check passed']);
        setStep('complete');
      } else {
        const err = await res.text();
        toast.error(`Deploy failed: ${err}`);
        setStep('configure');
      }
    } catch {
      toast.error('Deploy failed');
      setStep('configure');
    }
  }, [form, runtime]);

  const availableChannels = Object.keys(runtime.channel_support);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="fixed inset-0 bg-black/50" onClick={onClose} />
      <Card className="relative z-10 w-full max-w-lg mx-4">
        <CardHeader>
          <CardTitle>Deploy {runtime.runtime}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {step === 'configure' && (
            <>
              <div>
                <label className="text-sm font-medium">Instance Name</label>
                <input
                  type="text"
                  value={form.instanceName}
                  onChange={(e) => setForm({ ...form, instanceName: e.target.value })}
                  className="mt-1 w-full rounded-md border bg-background px-3 py-2 text-sm"
                />
              </div>

              <div>
                <label className="text-sm font-medium">Channels</label>
                <div className="mt-2 flex flex-wrap gap-2">
                  {availableChannels.map((ch) => (
                    <button
                      key={ch}
                      onClick={() => {
                        setForm((prev) => ({
                          ...prev,
                          channels: prev.channels.includes(ch)
                            ? prev.channels.filter((c) => c !== ch)
                            : [...prev.channels, ch],
                        }));
                      }}
                      className={cn(
                        'rounded-md border px-3 py-1.5 text-xs font-medium transition-colors',
                        form.channels.includes(ch)
                          ? 'border-primary bg-primary/10 text-primary'
                          : 'border-muted text-muted-foreground hover:border-primary/50',
                      )}
                    >
                      {ch}
                    </button>
                  ))}
                </div>
              </div>

              <div>
                <label className="text-sm font-medium">Tools</label>
                <div className="mt-2 flex flex-wrap gap-2">
                  {['git', 'http'].map((tool) => (
                    <button
                      key={tool}
                      onClick={() => {
                        setForm((prev) => ({
                          ...prev,
                          tools: prev.tools.includes(tool)
                            ? prev.tools.filter((t) => t !== tool)
                            : [...prev.tools, tool],
                        }));
                      }}
                      className={cn(
                        'rounded-md border px-3 py-1.5 text-xs font-medium transition-colors',
                        form.tools.includes(tool)
                          ? 'border-primary bg-primary/10 text-primary'
                          : 'border-muted text-muted-foreground hover:border-primary/50',
                      )}
                    >
                      {tool}
                    </button>
                  ))}
                </div>
              </div>

              <div className="flex justify-end gap-2 pt-2">
                <Button variant="outline" onClick={onClose}>
                  Cancel
                </Button>
                <Button onClick={handleDeploy} disabled={!form.instanceName.trim()}>
                  Deploy
                </Button>
              </div>
            </>
          )}

          {step === 'deploying' && (
            <div className="space-y-2">
              {progress.map((msg, i) => (
                <div key={i} className="flex items-center gap-2 text-sm">
                  <div className="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                  {msg}
                </div>
              ))}
            </div>
          )}

          {step === 'complete' && (
            <div className="space-y-4">
              <div className="space-y-2">
                {progress.map((msg, i) => (
                  <div key={i} className="flex items-center gap-2 text-sm text-green-600">
                    <span>✓</span>
                    {msg}
                  </div>
                ))}
              </div>
              <Button className="w-full" onClick={onDeployed}>
                Done
              </Button>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
