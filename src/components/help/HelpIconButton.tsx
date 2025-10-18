import { forwardRef, type ButtonHTMLAttributes } from 'react';
import { HelpCircle } from 'lucide-react';
import { Button } from '../ui/button';
import { cn } from '../../lib/utils';

export type HelpIconButtonSize = 'default' | 'sm';

export interface HelpIconButtonProps
  extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'> {
  srLabel?: string;
  size?: HelpIconButtonSize;
}

export const HelpIconButton = forwardRef<HTMLButtonElement, HelpIconButtonProps>(
  ({ className, size = 'default', srLabel, ...rest }, ref) => {
    const rawProps = rest as ButtonHTMLAttributes<HTMLButtonElement> & { 'aria-label'?: string };
    const ariaLabel = rawProps['aria-label'] ?? srLabel ?? '查看帮助';
    const finalProps: ButtonHTMLAttributes<HTMLButtonElement> = {
      ...rawProps,
      'aria-label': ariaLabel,
    };

    const iconSizeClass = size === 'sm' ? 'h-4 w-4' : 'h-5 w-5';
    const dimensionsClass = size === 'sm' ? 'h-7 w-7' : 'h-8 w-8';

    return (
      <Button
        ref={ref}
        type="button"
        variant="ghost"
        size="icon"
        className={cn(
          'relative inline-flex items-center justify-center rounded-full border border-border/60 bg-background/80 text-muted-foreground transition hover:border-primary/50 hover:text-foreground focus-visible:ring-2 focus-visible:ring-primary/70 focus-visible:ring-offset-1',
          dimensionsClass,
          className,
        )}
        {...finalProps}
      >
        <HelpCircle className={cn(iconSizeClass)} aria-hidden="true" />
        <span className="sr-only">{ariaLabel}</span>
      </Button>
    );
  },
);

HelpIconButton.displayName = 'HelpIconButton';
