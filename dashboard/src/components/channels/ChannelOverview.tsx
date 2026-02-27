import { useState, useEffect } from 'react';
import { toast } from 'sonner';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Skeleton } from '../ui/skeleton';
import { Separator } from '../ui/separator';
import {
  MessageCircle,
  Plus,
  CheckCircle,
  XCircle,
  AlertTriangle,
  ArrowLeftRight,
  Shield,
} from 'lucide-react';
import { cn } from '../../lib/utils';

// --- Types ---

interface ChannelTypeSummary {
  channel_type: string;
  instance_count: number;
  connected: number;
  disconnected: number;
}

interface ChannelInstanceConfig {
  instance_name: string;
  channel_type: string;
  credentials: Record<string, string>;
  options: Record<string, unknown>;
}

interface MatrixRow {
  channel_instance: string;
  channel_type: string;
  cells: MatrixCell[];
}

interface MatrixCell {
  agent_id: string;
  runtime: string;
  status: 'connected' | 'disconnected' | 'rate_limited' | 'proxied';
}

interface ChannelBinding {
  instance_id: string;
  channel_type: string;
  bot_token_hash: string;
  status: 'active' | 'draining' | 'released';
  bound_at_unix_ms: number;
}

interface BindingConflict {
  channel_type: string;
  bot_token_hash: string;
  instance_ids: string[];
}

interface SupportMatrix {
  [runtime: string]: {
    [channel: string]: string | { Via: string };
  };
}

// --- Channel icons ---

const CHANNEL_ICONS: Record<string, string> = {
  telegram: '✈️',
  discord: '🎮',
  slack: '💬',
  whatsapp: '📱',
  signal: '🔒',
  matrix: '🔗',
  email: '📧',
  feishu: '🪶',
  dingtalk: '🔔',
  mattermost: '💭',
  irc: '📺',
  teams: '👥',
  imessage: '🍎',
  google_chat: '🔍',
  qq: '🐧',
  line: '🟢',
  nostr: '🔐',
};

// --- Per-channel required fields ---

const CHANNEL_FIELDS: Record<string, { required: string[]; optional: string[] }> = {
  telegram: { required: ['token'], optional: ['allowed_users', 'group_mode'] },
  discord: { required: ['token'], optional: ['guild', 'allowed_roles'] },
  slack: { required: ['bot_token', 'app_token'], optional: ['allowed_channels'] },
  whatsapp: { required: ['token'], optional: [] },
  signal: { required: ['phone'], optional: ['signal_cli_path'] },
  feishu: { required: ['app_id', 'app_secret'], optional: [] },
  dingtalk: { required: ['token'], optional: [] },
};

// --- Main Channel Overview ---

export function ChannelOverview() {
  const [summaries, setSummaries] = useState<ChannelTypeSummary[]>([]);
  const [supportMatrix, setSupportMatrix] = useState<SupportMatrix>({});
  const [bindings, setBindings] = useState<ChannelBinding[]>([]);
  const [conflicts, setConflicts] = useState<BindingConflict[]>([]);
  const [loading, setLoading] = useState(true);
  const [configTarget, setConfigTarget] = useState<string | null>(null);
  const [showMatrix, setShowMatrix] = useState(false);

  const fetchData = async () => {
    try {
      const [summaryRes, matrixRes, bindingsRes, conflictsRes] = await Promise.all([
        fetch('/api/channels'),
        fetch('/api/channels/support-matrix'),
        fetch('/api/channels/bindings'),
        fetch('/api/channels/bindings/conflicts'),
      ]);

      if (summaryRes.ok) setSummaries(await summaryRes.json());
      if (matrixRes.ok) setSupportMatrix(await matrixRes.json());
      if (bindingsRes.ok) setBindings(await bindingsRes.json());
      if (conflictsRes.ok) setConflicts(await conflictsRes.json());
    } catch {
      /* ignore */
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void fetchData();
  }, []);

  // All known channel types from the support matrix
  const allChannelTypes = [
    ...new Set([
      ...Object.values(supportMatrix).flatMap((r) => Object.keys(r)),
      ...summaries.map((s) => s.channel_type),
    ]),
  ].sort();

  return (
    <div className="space-y-6">
      {/* Conflicts Warning */}
      {conflicts.length > 0 && (
        <div className="rounded-lg border border-amber-500/50 bg-amber-500/10 p-4">
          <div className="flex items-center gap-2 text-amber-600 font-semibold text-sm mb-2">
            <AlertTriangle className="h-4 w-4" />
            Token Conflicts Detected
          </div>
          {conflicts.map((conflict, i) => (
            <div key={i} className="text-sm text-muted-foreground">
              {conflict.channel_type}: same token used by instances{' '}
              {conflict.instance_ids.join(', ')}
            </div>
          ))}
        </div>
      )}

      {/* Channel Type Grid */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
            Channels
          </h3>
          <div className="flex gap-2">
            <Button
              variant={showMatrix ? 'default' : 'outline'}
              size="sm"
              onClick={() => setShowMatrix(!showMatrix)}
            >
              Support Matrix
            </Button>
          </div>
        </div>

        {loading ? (
          <div className="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-4">
            {Array.from({ length: 6 }).map((_, i) => (
              <Skeleton key={i} className="h-32 rounded-xl" />
            ))}
          </div>
        ) : (
          <div className="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-4">
            {allChannelTypes.map((channelType) => {
              const summary = summaries.find((s) => s.channel_type === channelType);
              const runtimesSupporting = Object.entries(supportMatrix).filter(
                ([, channels]) => channelType in channels,
              ).length;

              return (
                <ChannelCard
                  key={channelType}
                  channelType={channelType}
                  instanceCount={summary?.instance_count ?? 0}
                  connected={summary?.connected ?? 0}
                  disconnected={summary?.disconnected ?? 0}
                  runtimesSupporting={runtimesSupporting}
                  onConfigure={() => setConfigTarget(channelType)}
                />
              );
            })}
          </div>
        )}
      </div>

      {/* Support Matrix */}
      {showMatrix && (
        <ChannelSupportMatrix supportMatrix={supportMatrix} />
      )}

      {/* Active Bindings */}
      {bindings.length > 0 && (
        <ChannelBindings bindings={bindings} onRefresh={fetchData} />
      )}

      {/* Config Form */}
      {configTarget && (
        <ChannelConfigForm
          channelType={configTarget}
          onClose={() => setConfigTarget(null)}
          onSaved={() => {
            setConfigTarget(null);
            void fetchData();
            toast.success('Channel configured');
          }}
        />
      )}
    </div>
  );
}

// --- Channel Card ---

function ChannelCard({
  channelType,
  instanceCount,
  connected,
  disconnected,
  runtimesSupporting,
  onConfigure,
}: {
  channelType: string;
  instanceCount: number;
  connected: number;
  disconnected: number;
  runtimesSupporting: number;
  onConfigure: () => void;
}) {
  const icon = CHANNEL_ICONS[channelType] ?? '📡';
  const hasInstances = instanceCount > 0;

  return (
    <Card
      className={cn(
        'cursor-pointer transition-all hover:shadow-md',
        hasInstances ? 'border-primary/30' : '',
      )}
      onClick={onConfigure}
    >
      <CardContent className="p-4">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            <span className="text-xl">{icon}</span>
            <span className="font-medium capitalize text-sm">{channelType.replace('_', ' ')}</span>
          </div>
          {hasInstances && (
            <Badge variant="secondary" className="text-xs">
              {instanceCount}
            </Badge>
          )}
        </div>

        <div className="mt-3 flex items-center gap-3 text-xs text-muted-foreground">
          <span className="flex items-center gap-1">
            <Shield className="h-3 w-3" />
            {runtimesSupporting} runtimes
          </span>
        </div>

        {hasInstances && (
          <div className="mt-2 flex gap-2">
            {connected > 0 && (
              <span className="flex items-center gap-1 text-xs text-green-600">
                <CheckCircle className="h-3 w-3" />
                {connected}
              </span>
            )}
            {disconnected > 0 && (
              <span className="flex items-center gap-1 text-xs text-red-500">
                <XCircle className="h-3 w-3" />
                {disconnected}
              </span>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// --- Channel Support Matrix ---

function ChannelSupportMatrix({ supportMatrix }: { supportMatrix: SupportMatrix }) {
  const runtimes = Object.keys(supportMatrix).sort();
  const allChannels = [
    ...new Set(runtimes.flatMap((r) => Object.keys(supportMatrix[r]))),
  ].sort();

  if (runtimes.length === 0) return null;

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-base">Channel Support Matrix</CardTitle>
      </CardHeader>
      <CardContent className="p-0 overflow-x-auto">
        <table className="w-full text-xs">
          <thead>
            <tr className="border-b bg-muted/50">
              <th className="px-4 py-2 text-left font-semibold">Channel</th>
              {runtimes.map((rt) => (
                <th key={rt} className="px-3 py-2 text-center font-semibold">
                  {rt}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y">
            {allChannels.map((channel) => (
              <tr key={channel} className="hover:bg-muted/30">
                <td className="px-4 py-2 capitalize font-medium">
                  <span className="mr-1">{CHANNEL_ICONS[channel] ?? ''}</span>
                  {channel.replace('_', ' ')}
                </td>
                {runtimes.map((rt) => {
                  const support = supportMatrix[rt]?.[channel];
                  return (
                    <td key={rt} className="px-3 py-2 text-center">
                      <SupportBadge support={support} />
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </CardContent>
    </Card>
  );
}

function SupportBadge({ support }: { support?: string | { Via: string } }) {
  if (!support) return <span className="text-muted-foreground">—</span>;

  if (support === 'native') {
    return (
      <span className="inline-flex items-center gap-0.5 text-green-600" title="Native support">
        <CheckCircle className="h-3 w-3" />
      </span>
    );
  }

  if (typeof support === 'object' && 'Via' in support) {
    return (
      <span
        className="inline-flex items-center gap-0.5 text-blue-500"
        title={`Via: ${support.Via}`}
      >
        <ArrowLeftRight className="h-3 w-3" />
      </span>
    );
  }

  return <span className="text-muted-foreground">?</span>;
}

// --- Channel Bindings ---

function ChannelBindings({
  bindings,
  onRefresh,
}: {
  bindings: ChannelBinding[];
  onRefresh: () => void;
}) {
  const activeBindings = bindings.filter((b) => b.status === 'active');

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base">Channel Bindings</CardTitle>
          <Badge variant="secondary">{activeBindings.length} active</Badge>
        </div>
      </CardHeader>
      <CardContent className="p-0 overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b bg-muted/50 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              <th className="px-6 py-3 text-left">Instance</th>
              <th className="px-6 py-3 text-left">Channel</th>
              <th className="px-6 py-3 text-left">Token Hash</th>
              <th className="px-6 py-3 text-left">Status</th>
              <th className="px-6 py-3 text-left">Bound At</th>
            </tr>
          </thead>
          <tbody className="divide-y">
            {bindings.map((binding, i) => (
              <tr key={i} className="hover:bg-muted/30">
                <td className="px-6 py-3 font-medium">{binding.instance_id}</td>
                <td className="px-6 py-3 capitalize">{binding.channel_type}</td>
                <td className="px-6 py-3 font-mono text-xs text-muted-foreground">
                  {binding.bot_token_hash.slice(0, 12)}...
                </td>
                <td className="px-6 py-3">
                  <Badge
                    variant={
                      binding.status === 'active'
                        ? 'success'
                        : binding.status === 'draining'
                          ? 'warning'
                          : 'secondary'
                    }
                    className="capitalize"
                  >
                    {binding.status}
                  </Badge>
                </td>
                <td className="px-6 py-3 text-xs text-muted-foreground">
                  {new Date(binding.bound_at_unix_ms).toLocaleString()}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </CardContent>
    </Card>
  );
}

// --- Channel Config Form ---

function ChannelConfigForm({
  channelType,
  onClose,
  onSaved,
}: {
  channelType: string;
  onClose: () => void;
  onSaved: () => void;
}) {
  const fields = CHANNEL_FIELDS[channelType] ?? { required: ['token'], optional: [] };
  const [instanceName, setInstanceName] = useState(channelType);
  const [credentials, setCredentials] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);

  const handleSave = async () => {
    // Validate required fields
    for (const field of fields.required) {
      if (!credentials[field]?.trim()) {
        toast.error(`${field} is required`);
        return;
      }
    }

    setSaving(true);
    try {
      const res = await fetch(`/api/channels/${channelType}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          instance_name: instanceName,
          channel_type: channelType,
          credentials,
          options: {},
        }),
      });

      if (res.ok) {
        onSaved();
      } else {
        const err = await res.text();
        toast.error(`Failed to save: ${err}`);
      }
    } catch {
      toast.error('Failed to save channel config');
    } finally {
      setSaving(false);
    }
  };

  const handleTest = async () => {
    try {
      const res = await fetch(`/api/channels/${channelType}/test`, { method: 'POST' });
      if (res.ok) {
        const result = await res.json();
        if (result.status === 'ok') {
          toast.success('Connection test passed');
        } else {
          toast.warning('No instances configured for this channel');
        }
      }
    } catch {
      toast.error('Connection test failed');
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="fixed inset-0 bg-black/50" onClick={onClose} />
      <Card className="relative z-10 w-full max-w-md mx-4">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <span>{CHANNEL_ICONS[channelType] ?? '📡'}</span>
            Configure {channelType.charAt(0).toUpperCase() + channelType.slice(1)}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <label className="text-sm font-medium">Instance Name</label>
            <input
              type="text"
              value={instanceName}
              onChange={(e) => setInstanceName(e.target.value)}
              className="mt-1 w-full rounded-md border bg-background px-3 py-2 text-sm"
            />
          </div>

          <Separator />

          <div className="space-y-3">
            <h4 className="text-sm font-medium">Credentials</h4>
            {fields.required.map((field) => (
              <div key={field}>
                <label className="text-xs font-medium text-muted-foreground uppercase">
                  {field} <span className="text-destructive">*</span>
                </label>
                <input
                  type="password"
                  placeholder={`$${channelType.toUpperCase()}_${field.toUpperCase()}`}
                  value={credentials[field] ?? ''}
                  onChange={(e) =>
                    setCredentials({ ...credentials, [field]: e.target.value })
                  }
                  className="mt-1 w-full rounded-md border bg-background px-3 py-2 text-sm font-mono"
                />
              </div>
            ))}
            {fields.optional.map((field) => (
              <div key={field}>
                <label className="text-xs font-medium text-muted-foreground uppercase">
                  {field}
                </label>
                <input
                  type="text"
                  placeholder={field}
                  value={credentials[field] ?? ''}
                  onChange={(e) =>
                    setCredentials({ ...credentials, [field]: e.target.value })
                  }
                  className="mt-1 w-full rounded-md border bg-background px-3 py-2 text-sm"
                />
              </div>
            ))}
          </div>

          <div className="flex justify-between pt-2">
            <Button variant="outline" size="sm" onClick={handleTest}>
              Test Connection
            </Button>
            <div className="flex gap-2">
              <Button variant="outline" onClick={onClose}>
                Cancel
              </Button>
              <Button onClick={handleSave} disabled={saving}>
                {saving ? 'Saving...' : 'Save'}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
