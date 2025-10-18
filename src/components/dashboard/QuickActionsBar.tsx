import { useCallback, useMemo, useRef, type KeyboardEvent } from 'react';
import { Sparkles, ArrowUpRight, AlertCircle } from 'lucide-react';
import { useNavigate } from 'react-router-dom';
import type { LucideIcon } from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Badge } from '../ui/badge';
import { Skeleton } from '../ui/skeleton';
import { cn } from '../../lib/utils';
import type { QuickAction } from '../../types/dashboard';

export interface QuickActionsBarItem extends Pick<QuickAction, 'id' | 'label' | 'tooltip'> {
  description?: string;
  icon?: LucideIcon;
  shortcut?: string;
  disabled?: boolean;
  to?: string;
  href?: string;
  external?: boolean;
  onSelect?: () => void | Promise<void>;
}

export interface QuickActionsBarProps {
  actions: QuickActionsBarItem[];
  title?: string;
  subtitle?: string;
  badgeText?: string;
  className?: string;
  isLoading?: boolean;
  emptyHint?: string;
}

const DEFAULT_EMPTY_HINT = '暂无快捷操作，前往设置页可自定义仪表盘行为。';

const QuickActionsBar = ({
  actions,
  title = '快捷操作',
  subtitle = '高频场景一键直达，保持当前的工作流畅度。',
  badgeText,
  className,
  isLoading = false,
  emptyHint = DEFAULT_EMPTY_HINT,
}: QuickActionsBarProps) => {
  const navigate = useNavigate();
  const buttonRefs = useRef<Array<HTMLButtonElement | null>>([]);

  const actionableItems = useMemo(() => actions.filter((action) => !action.disabled), [actions]);

  const focusItem = useCallback((index: number) => {
    const target = buttonRefs.current[index];
    if (target) {
      target.focus();
    }
  }, []);

  const findNextEnabledIndex = useCallback(
    (start: number, direction: 1 | -1): number => {
      const total = actions.length;
      let offset = start;
      for (let step = 0; step < total; step += 1) {
        offset = (offset + direction + total) % total;
        if (!actions[offset]?.disabled) {
          return offset;
        }
      }
      return start;
    },
    [actions],
  );

  const handleSelect = useCallback(
    (action: QuickActionsBarItem) => {
      if (action.disabled) return;
      if (action.onSelect) {
        void action.onSelect();
        return;
      }
      if (action.to) {
        navigate(action.to);
        return;
      }
      if (action.href) {
        const target = action.external ? '_blank' : '_self';
        window.open(action.href, target, 'noopener');
      }
    },
    [navigate],
  );

  const focusFirstEnabled = useCallback(() => {
    for (let index = 0; index < actions.length; index += 1) {
      if (!actions[index]?.disabled) {
        focusItem(index);
        break;
      }
    }
  }, [actions, focusItem]);

  const focusLastEnabled = useCallback(() => {
    for (let index = actions.length - 1; index >= 0; index -= 1) {
      if (!actions[index]?.disabled) {
        focusItem(index);
        break;
      }
    }
  }, [actions, focusItem]);

  const handleKeyDown = useCallback(
    (event: KeyboardEvent<HTMLButtonElement>, index: number) => {
      switch (event.key) {
        case 'ArrowRight':
        case 'ArrowDown': {
          event.preventDefault();
          const nextIndex = findNextEnabledIndex(index, 1);
          focusItem(nextIndex);
          break;
        }
        case 'ArrowLeft':
        case 'ArrowUp': {
          event.preventDefault();
          const prevIndex = findNextEnabledIndex(index, -1);
          focusItem(prevIndex);
          break;
        }
        case 'Home': {
          event.preventDefault();
          focusFirstEnabled();
          break;
        }
        case 'End': {
          event.preventDefault();
          focusLastEnabled();
          break;
        }
        default:
          break;
      }
    },
    [findNextEnabledIndex, focusFirstEnabled, focusLastEnabled, focusItem],
  );

  const renderShortcut = (shortcut?: string) => {
    if (!shortcut) return null;
    return (
      <kbd className="rounded border border-border/60 bg-muted px-1.5 py-0.5 text-[11px] font-medium text-muted-foreground">
        {shortcut}
      </kbd>
    );
  };

  return (
    <Card className={cn('dashboard-quick-actions', className)}>
      <CardHeader className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-start gap-2">
          <div className="mt-0.5 rounded-full bg-primary/10 p-1 text-primary">
            <Sparkles className="h-4 w-4" />
          </div>
          <div className="space-y-1">
            <CardTitle className="text-base font-semibold">{title}</CardTitle>
            {subtitle ? (
              <p className="text-sm text-muted-foreground leading-relaxed">{subtitle}</p>
            ) : null}
          </div>
        </div>
        {badgeText ? <Badge variant="outline">{badgeText}</Badge> : null}
      </CardHeader>
      <CardContent className="space-y-4">
        {isLoading ? (
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
            {Array.from({ length: 3 }).map((_, index) => (
              <Skeleton key={index} className="h-20 w-full rounded-2xl" />
            ))}
          </div>
        ) : null}

        {!isLoading && actions.length === 0 ? (
          <div className="flex items-center gap-2 rounded-2xl border border-dashed border-border/60 bg-muted/30 p-4 text-sm text-muted-foreground">
            <AlertCircle className="h-4 w-4" />
            <span>{emptyHint}</span>
          </div>
        ) : null}

        {!isLoading && actions.length > 0 ? (
          <div
            role="toolbar"
            aria-label="仪表盘快捷操作"
            className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3"
          >
            {actions.map((action, index) => {
              const Icon = action.icon;
              const showExternal = Boolean(action.external || action.href);
              return (
                <Button
                  key={action.id}
                  ref={(node) => {
                    buttonRefs.current[index] = node;
                  }}
                  type="button"
                  variant="secondary"
                  disabled={action.disabled}
                  onClick={() => handleSelect(action)}
                  onKeyDown={(event) => handleKeyDown(event, index)}
                  className={cn(
                    'group flex h-auto w-full flex-col items-start gap-2 rounded-2xl border border-border/60 bg-background/80 px-4 py-3 text-left transition hover:border-primary/50 hover:bg-primary/10 focus-visible:ring-2 focus-visible:ring-primary',
                    action.disabled
                      ? 'cursor-not-allowed opacity-60 hover:border-border/60 hover:bg-background/80'
                      : null,
                  )}
                  data-action-id={action.id}
                  title={action.tooltip ?? action.description ?? action.label}
                  aria-describedby={action.description ? `${action.id}-description` : undefined}
                >
                  <span className="flex w-full items-center justify-between gap-2 text-sm font-medium text-foreground">
                    <span className="flex items-center gap-2">
                      {Icon ? <Icon className="h-4 w-4" /> : null}
                      {action.label}
                    </span>
                    {showExternal ? <ArrowUpRight className="h-3.5 w-3.5 opacity-60" /> : null}
                  </span>
                  {action.description ? (
                    <span
                      id={`${action.id}-description`}
                      className="text-xs text-muted-foreground leading-relaxed"
                    >
                      {action.description}
                    </span>
                  ) : null}
                  {renderShortcut(action.shortcut)}
                </Button>
              );
            })}
          </div>
        ) : null}

        {actionableItems.length > 0 ? (
          <p className="text-[11px] text-muted-foreground">
            支持方向键快速切换操作，按 Enter 或空格键执行当前聚焦的快捷操作。
          </p>
        ) : null}
      </CardContent>
    </Card>
  );
};

export default QuickActionsBar;
