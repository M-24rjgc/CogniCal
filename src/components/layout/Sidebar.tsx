import { type LucideIcon, Menu, X } from 'lucide-react';
import { ReactNode } from 'react';
import { NavLink } from 'react-router-dom';
import { cn } from '../../lib/utils';

export interface SidebarItem {
  key: string;
  label: string;
  to: string;
  icon: LucideIcon;
  description?: string;
  badge?: string;
}

interface SidebarProps {
  items: SidebarItem[];
  footer?: ReactNode;
  onNavigate?: () => void;
  onClose?: () => void;
}

export function Sidebar({ items, footer, onNavigate, onClose }: SidebarProps) {
  return (
    <div className="flex h-full w-full flex-col gap-6 px-4 py-6">
      <div className="flex items-center justify-between px-2">
        <div className="flex items-center gap-2">
          <Menu className="h-4 w-4 text-primary" />
          <span className="font-heading text-sm font-semibold uppercase tracking-[0.35em] text-muted-foreground">
            CogniCal
          </span>
        </div>
        {onClose ? (
          <button
            type="button"
            aria-label="关闭导航"
            className="inline-flex h-9 w-9 items-center justify-center rounded-lg border border-border/60 text-foreground transition hover:bg-muted/70 lg:hidden"
            onClick={onClose}
          >
            <X className="h-4 w-4" />
          </button>
        ) : null}
      </div>
      <nav className="flex flex-1 flex-col gap-1" aria-label="主导航">
        {items.map((item) => {
          const Icon = item.icon;
          return (
            <NavLink
              key={item.key}
              to={item.to}
              end={item.to === '/'}
              onClick={onNavigate}
              className={({ isActive }) =>
                cn(
                  'group relative flex items-center gap-3 rounded-xl px-3 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-muted/70 hover:text-foreground',
                  isActive && 'bg-muted/90 text-foreground shadow-sm',
                )
              }
            >
              <Icon className="h-4 w-4 text-primary transition group-hover:text-primary/80" />
              <span className="flex-1">{item.label}</span>
              {item.badge ? (
                <span className="rounded-full bg-primary/10 px-2 py-0.5 text-xs font-semibold text-primary">
                  {item.badge}
                </span>
              ) : null}
            </NavLink>
          );
        })}
      </nav>

      {footer ? <div className="mt-auto px-2 text-xs text-muted-foreground">{footer}</div> : null}
    </div>
  );
}
