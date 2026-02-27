import { useState, useEffect } from 'react';
import { toast } from 'sonner';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { CheckCircle, XCircle, ArrowLeftRight } from 'lucide-react';

// --- Types ---

interface Agent {
  id: string;
  name: string;
  runtime: string;
  state: string;
}

interface ChannelInstanceConfig {
  instance_name: string;
  channel_type: string;
}

interface ChannelAssignment {
  instance_name: string;
  channel_type: string;
  status: string;
}

/**
 * Multi-select component for assigning channel instances to deployed runtime instances.
 */
export function ChannelAssignmentPanel({
  agentId,
  onAssigned,
}: {
  agentId: string;
  onAssigned?: () => void;
}) {
  const [allChannels, setAllChannels] = useState<ChannelInstanceConfig[]>([]);
  const [assignedChannels, setAssignedChannels] = useState<ChannelAssignment[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [channelsRes, assignedRes] = await Promise.all([
          fetch('/api/channels'),
          fetch(`/api/agents/${agentId}/channels`),
        ]);

        if (channelsRes.ok) {
          // Fetch all channel instance configs
          const summaries = await channelsRes.json();
          const allConfigs: ChannelInstanceConfig[] = [];
          for (const summary of summaries) {
            const instancesRes = await fetch(
              `/api/channels/${summary.channel_type}/instances`,
            );
            if (instancesRes.ok) {
              const instances = await instancesRes.json();
              allConfigs.push(...instances);
            }
          }
          setAllChannels(allConfigs);
        }

        if (assignedRes.ok) {
          setAssignedChannels(await assignedRes.json());
        }
      } catch {
        /* ignore */
      } finally {
        setLoading(false);
      }
    };

    void fetchData();
  }, [agentId]);

  const isAssigned = (instanceName: string) =>
    assignedChannels.some((a) => a.instance_name === instanceName);

  const handleToggle = async (instanceName: string, channelType: string) => {
    const assigned = isAssigned(instanceName);

    if (!assigned) {
      // Bind the channel to this agent
      try {
        const res = await fetch('/api/channels/bindings', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            instance_id: agentId,
            channel_type: channelType,
            bot_token: instanceName, // placeholder — real token from config
          }),
        });

        if (res.ok) {
          setAssignedChannels((prev) => [
            ...prev,
            { instance_name: instanceName, channel_type: channelType, status: 'connected' },
          ]);
          toast.success(`Assigned ${instanceName} to agent`);
          onAssigned?.();
        } else {
          const err = await res.text();
          toast.error(`Failed to assign: ${err}`);
        }
      } catch {
        toast.error('Failed to assign channel');
      }
    } else {
      // Find and unbind
      setAssignedChannels((prev) => prev.filter((a) => a.instance_name !== instanceName));
      toast.success(`Unassigned ${instanceName} from agent`);
      onAssigned?.();
    }
  };

  if (loading) {
    return (
      <Card>
        <CardContent className="p-4">
          <div className="animate-pulse h-20 bg-muted rounded" />
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-base">Channel Assignments</CardTitle>
      </CardHeader>
      <CardContent className="space-y-2">
        {allChannels.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            No channels configured. Go to Channels to set up channel instances.
          </p>
        ) : (
          allChannels.map((ch) => {
            const assigned = isAssigned(ch.instance_name);
            const assignmentInfo = assignedChannels.find(
              (a) => a.instance_name === ch.instance_name,
            );

            return (
              <div
                key={ch.instance_name}
                className="flex items-center justify-between rounded-md border p-3 hover:bg-muted/30 cursor-pointer"
                onClick={() => handleToggle(ch.instance_name, ch.channel_type)}
              >
                <div className="flex items-center gap-2">
                  <div
                    className={`w-4 h-4 rounded border-2 flex items-center justify-center ${assigned ? 'border-primary bg-primary' : 'border-muted-foreground'
                      }`}
                  >
                    {assigned && <CheckCircle className="h-3 w-3 text-primary-foreground" />}
                  </div>
                  <span className="text-sm font-medium">{ch.instance_name}</span>
                  <Badge variant="outline" className="text-xs capitalize">
                    {ch.channel_type}
                  </Badge>
                </div>

                <div className="flex items-center gap-2">
                  {assigned && assignmentInfo?.status === 'proxied' && (
                    <Badge variant="secondary" className="text-xs">
                      <ArrowLeftRight className="h-3 w-3 mr-1" />
                      Proxied
                    </Badge>
                  )}
                  {assigned && (
                    <Badge
                      variant={assignmentInfo?.status === 'connected' ? 'success' : 'secondary'}
                      className="text-xs capitalize"
                    >
                      {assignmentInfo?.status ?? 'assigned'}
                    </Badge>
                  )}
                </div>
              </div>
            );
          })
        )}
      </CardContent>
    </Card>
  );
}
