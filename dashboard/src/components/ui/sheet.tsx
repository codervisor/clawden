import * as React from 'react';
import { cn } from '../../lib/utils';

interface SheetProps {
  open: boolean;
  onClose: () => void;
  children: React.ReactNode;
  side?: 'left' | 'right';
}

function Sheet({ open, onClose, children, side = 'left' }: SheetProps) {
  React.useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    if (open) document.addEventListener('keydown', handleKey);
    return () => document.removeEventListener('keydown', handleKey);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <>
      <div
        className="fixed inset-0 z-40 bg-black/50"
        onClick={onClose}
        aria-hidden="true"
      />
      <div
        className={cn(
          'fixed z-50 top-0 h-full w-72 bg-[hsl(var(--sidebar-background))] text-[hsl(var(--sidebar-foreground))] shadow-xl transition-transform duration-200',
          side === 'left' ? 'left-0' : 'right-0',
        )}
        role="dialog"
        aria-modal="true"
      >
        {children}
      </div>
    </>
  );
}

export { Sheet };
