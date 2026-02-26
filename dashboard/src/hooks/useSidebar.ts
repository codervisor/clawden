import { useState } from 'react';

function getInitial(): boolean {
  try {
    const stored = localStorage.getItem('clawden-sidebar-collapsed');
    if (stored !== null) return stored === 'true';
  } catch { /* ignore */ }
  return false;
}

export function useSidebar() {
  const [collapsed, setCollapsed] = useState(getInitial);

  const toggle = () => {
    setCollapsed((prev) => {
      const next = !prev;
      try {
        localStorage.setItem('clawden-sidebar-collapsed', String(next));
      } catch { /* ignore */ }
      return next;
    });
  };

  return { collapsed, toggle };
}
