import { Menu, Moon, Sun } from 'lucide-react';
import { Button } from '../ui/button';
import { Tooltip } from '../ui/tooltip';
import { useTheme } from '../../hooks/useTheme';
import { cn } from '../../lib/utils';

interface HeaderProps {
  title: string;
  onMenuClick: () => void;
  wsConnected: boolean;
}

export function Header({ title, onMenuClick, wsConnected }: HeaderProps) {
  const { resolvedTheme, setTheme } = useTheme();

  return (
    <header className="sticky top-0 z-30 flex h-14 items-center gap-4 border-b bg-background px-4 md:px-6">
      <Button
        variant="ghost"
        size="icon"
        onClick={onMenuClick}
        aria-label="Toggle sidebar"
        className="shrink-0"
      >
        <Menu className="h-5 w-5" />
      </Button>

      <div className="flex-1">
        <h2 className="text-base font-semibold">{title}</h2>
      </div>

      <div className="flex items-center gap-3">
        <Tooltip content={wsConnected ? 'Live connection' : 'Polling mode'}>
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <div
              className={cn(
                'h-2 w-2 rounded-full',
                wsConnected ? 'bg-green-500 animate-pulse' : 'bg-slate-400',
              )}
            />
            <span className="hidden sm:inline">{wsConnected ? 'Live' : 'Polling'}</span>
          </div>
        </Tooltip>

        <Tooltip content={resolvedTheme === 'dark' ? 'Switch to light' : 'Switch to dark'}>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setTheme(resolvedTheme === 'dark' ? 'light' : 'dark')}
            aria-label="Toggle theme"
          >
            {resolvedTheme === 'dark' ? (
              <Sun className="h-4 w-4" />
            ) : (
              <Moon className="h-4 w-4" />
            )}
          </Button>
        </Tooltip>
      </div>
    </header>
  );
}
