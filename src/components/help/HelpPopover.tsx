import { useEffect, useId, useMemo, useState, type ReactElement } from 'react';
import { useLocation } from 'react-router-dom';
import * as PopoverPrimitive from '@radix-ui/react-popover';
import { ExternalLink, X } from 'lucide-react';
import { HelpIconButton, type HelpIconButtonProps } from './HelpIconButton';
import {
  CONTEXTUAL_HELP_ENTRIES,
  type ContextualHelpEntry,
  getOnboardingStepById,
} from '../../utils/onboarding';
import { Button } from '../ui/button';
import { cn } from '../../lib/utils';

export type HelpEntryId = keyof typeof CONTEXTUAL_HELP_ENTRIES;

type HelpLink = NonNullable<ContextualHelpEntry['links']>[number];

const FALLBACK_MAX_WIDTH = 320;

const isExternalLink = (link: HelpLink) =>
  Boolean(link.external) || /^https?:/i.test(link.href ?? '');

export interface HelpPopoverProps {
  entryId: HelpEntryId;
  triggerLabel?: string;
  className?: string;
  triggerClassName?: string;
  side?: PopoverPrimitive.PopoverContentProps['side'];
  align?: PopoverPrimitive.PopoverContentProps['align'];
  sideOffset?: number;
  alignOffset?: number;
  children?: ReactElement;
  triggerProps?: Partial<HelpIconButtonProps>;
}

export function HelpPopover({
  entryId,
  triggerLabel,
  className,
  triggerClassName,
  side = 'top',
  align = 'end',
  sideOffset = 12,
  alignOffset = -4,
  children,
  triggerProps,
}: HelpPopoverProps) {
  const location = useLocation();
  const [open, setOpen] = useState(false);
  const headingId = useId();
  const descriptionId = useId();

  const entry = CONTEXTUAL_HELP_ENTRIES[entryId];

  useEffect(() => {
    setOpen(false);
  }, [location.key]);

  const relatedStepLabel = useMemo(() => {
    if (!entry?.relatedStepId) return null;
    const step = getOnboardingStepById(entry.relatedStepId);
    return step?.title ?? null;
  }, [entry?.relatedStepId]);

  if (!entry) {
    if (process.env.NODE_ENV !== 'production') {
      console.warn('[HelpPopover] 未找到帮助条目', entryId);
    }
    return null;
  }

  const ariaLabel = triggerLabel ?? `查看「${entry.title}」模块帮助`;

  const triggerElement = children ? (
    children
  ) : (
    <HelpIconButton
      aria-label={ariaLabel}
      className={triggerClassName}
      size="sm"
      data-contextual-help-trigger={entry.id}
      {...triggerProps}
    />
  );

  return (
    <PopoverPrimitive.Root open={open} onOpenChange={setOpen}>
      <span
        data-contextual-help-entry={entry.id}
        data-state={open ? 'open' : 'closed'}
        className="inline-flex"
      >
        <PopoverPrimitive.Trigger asChild>{triggerElement}</PopoverPrimitive.Trigger>
      </span>
      <PopoverPrimitive.Portal>
        <PopoverPrimitive.Content
          side={side}
          align={align}
          sideOffset={sideOffset}
          alignOffset={alignOffset}
          className={cn(
            'z-50 w-[var(--popover-width,320px)] max-w-[min(var(--popover-width,320px),calc(100vw-2rem))] rounded-2xl border border-border/70 bg-popover p-4 text-sm shadow-xl backdrop-blur-sm focus:outline-none',
            'data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=open]:fade-in data-[state=closed]:fade-out data-[state=open]:zoom-in-95 data-[state=closed]:zoom-out-95',
            'data-[side=top]:slide-in-from-bottom-1 data-[side=bottom]:slide-in-from-top-1 data-[side=left]:slide-in-from-right-1 data-[side=right]:slide-in-from-left-1',
            className,
          )}
          style={{
            ['--popover-width' as string]: `${FALLBACK_MAX_WIDTH}px`,
          }}
          aria-labelledby={headingId}
          aria-describedby={descriptionId}
          collisionPadding={16}
          onOpenAutoFocus={(event: Event) => {
            event.preventDefault();
          }}
          onCloseAutoFocus={(event: Event) => {
            event.preventDefault();
          }}
        >
          <div className="flex items-start justify-between gap-3">
            <div className="space-y-2">
              <h3 id={headingId} className="text-sm font-semibold text-foreground">
                {entry.title}
              </h3>
              <p id={descriptionId} className="text-xs leading-relaxed text-muted-foreground">
                {entry.description}
              </p>
            </div>
            <PopoverPrimitive.Close asChild>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                className="h-7 w-7 rounded-full text-muted-foreground hover:text-foreground"
                aria-label="关闭帮助"
              >
                <X className="h-4 w-4" aria-hidden="true" />
              </Button>
            </PopoverPrimitive.Close>
          </div>

          {relatedStepLabel ? (
            <div className="mt-3 rounded-lg border border-border/60 bg-muted/40 px-3 py-2 text-[11px] text-muted-foreground">
              <span className="font-semibold text-foreground">相关引导：</span>
              <span>{relatedStepLabel}</span>
            </div>
          ) : null}

          {entry.links && entry.links.length > 0 ? (
            <div className="mt-3 space-y-2">
              <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                深入了解
              </p>
              <ul className="space-y-1.5">
                {entry.links.map((link) => {
                  const external = isExternalLink(link);
                  return (
                    <li key={`${link.href}-${link.label}`}>
                      <a
                        href={link.href}
                        target={external ? '_blank' : undefined}
                        rel={external ? 'noreferrer' : undefined}
                        className="inline-flex items-center gap-1 text-sm text-primary transition hover:text-primary/80 hover:underline"
                      >
                        {link.label}
                        {external ? (
                          <ExternalLink className="h-3.5 w-3.5" aria-hidden="true" />
                        ) : null}
                      </a>
                    </li>
                  );
                })}
              </ul>
            </div>
          ) : null}

          <PopoverPrimitive.Arrow className="fill-popover" height={10} width={18} />
        </PopoverPrimitive.Content>
      </PopoverPrimitive.Portal>
    </PopoverPrimitive.Root>
  );
}
