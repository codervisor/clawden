import { useState, useEffect } from 'react';
import { Badge } from '../ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Skeleton } from '../ui/skeleton';
import {
  CheckCircle,
  XCircle,
  AlertTriangle,
  ArrowLeftRight,
} from 'lucide-react';

// --- Types ---

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

// --- Status icons ---

const STATUS_CONFIG: Record<
  string,
  { icon: typeof CheckCircle; color: string; label: string }
> = {
  connected: { icon: CheckCircle, color: 'text-green-600', label: 'Connected' },
  disconnected: { icon: XCircle, color: 'text-red-500', label: 'Disconnected' },
  rate_limited: { icon: AlertTriangle, color: 'text-amber-500', label: 'Rate Limited' },
  proxied: { icon: ArrowLeftRight, color: 'text-blue-500', label: 'Proxied' },
};

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

/**
 * Real-time instance × channel status matrix grid.
 * Per-cell: Connected ✅ / Disconnected ❌ / Rate limited ⚠️ / Proxied 🔄
 */
export function ChannelStatusMatrix() {
  const [matrix, setMatrix] = useState<MatrixRow[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchMatrix = async () => {
      try {
        const res = await fetch('/api/channels/matrix');
        if (res.ok) {
          setMatrix(await res.json());
        }
      } catch {
        /* ignore */
      } finally {
        setLoading(false);
      }
    };

    void fetchMatrix();
    // Poll for status updates every 5 seconds
    const interval = setInterval(() => void fetchMatrix(), 5000);
    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base">Channel Status Matrix</CardTitle>
        </CardHeader>
        <CardContent>
          <Skeleton className="h-48 w-full rounded" />
        </CardContent>
      </Card>
    );
  }

  if (matrix.length === 0) {
    return (
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base">Channel Status Matrix</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">
            No channel instances configured. Configure channels and assign them to runtimes to see
            the status matrix.
          </p>
        </CardContent>
      </Card>
    );
  }

  // Collect all unique agents from cells
  const agents = matrix.length > 0
    ? matrix[0].cells.map((c) => ({ agent_id: c.agent_id, runtime: c.runtime }))
    : [];

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base">Channel Status Matrix</CardTitle>
          <StatusLegend />
        </div>
      </CardHeader>
      <CardContent className="p-0 overflow-x-auto">
        <table className="w-full text-xs">
          <thead>
            <tr className="border-b bg-muted/50">
              <th className="px-4 py-2 text-left font-semibold sticky left-0 bg-muted/50">
                Channel Instance
              </th>
              {agents.map((agent) => (
                <th key={agent.agent_id} className="px-3 py-2 text-center font-semibold">
                  <div className="text-xs">{agent.agent_id}</div>
                  <div className="text-[10px] text-muted-foreground">{agent.runtime}</div>
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y">
            {matrix.map((row) => (
              <tr key={row.channel_instance} className="hover:bg-muted/30">
                <td className="px-4 py-2 font-medium sticky left-0 bg-background">
                  <div className="flex items-center gap-1.5">
                    <span>{CHANNEL_ICONS[row.channel_type] ?? '📡'}</span>
                    <span>{row.channel_instance}</span>
                    <Badge variant="outline" className="text-[10px] ml-1">
                      {row.channel_type}
                    </Badge>
                  </div>
                </td>
                {row.cells.map((cell) => (
                  <td key={cell.agent_id} className="px-3 py-2 text-center">
                    <StatusCell status={cell.status} />
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </CardContent>
    </Card>
  );
}

function StatusCell({ status }: { status: string }) {
  const config = STATUS_CONFIG[status];
  if (!config) return <span className="text-muted-foreground">—</span>;

  const Icon = config.icon;
  return (
    <span className={`inline-flex items-center gap-0.5 ${config.color}`} title={config.label}>
      <Icon className="h-3.5 w-3.5" />
    </span>
  );
}

function StatusLegend() {
  return (
    <div className="flex gap-3 text-[10px] text-muted-foreground">
      {Object.entries(STATUS_CONFIG).map(([key, cfg]) => {
        const Icon = cfg.icon;
        return (
          <span key={key} className={`flex items-center gap-0.5 ${cfg.color}`}>
            <Icon className="h-3 w-3" />
            {cfg.label}
          </span>
        );
      })}
    </div>
  );
}
