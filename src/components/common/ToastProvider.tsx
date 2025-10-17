import { useEffect } from 'react';
import { cn } from '../../lib/utils';
import { useUIStore, type Toast, type ToastVariant } from '../../stores/uiStore';

const VARIANT_STYLES: Record<ToastVariant, string> = {
  default:
    'border-border/60 bg-background/95 text-foreground shadow-[0_10px_30px_-15px_rgba(15,23,42,0.45)] backdrop-blur',
  success:
    'border-emerald-500/50 bg-emerald-100/90 text-emerald-950 dark:bg-emerald-500/10 dark:text-emerald-50',
  error: 'border-destructive/60 bg-destructive/10 text-destructive-foreground',
  warning:
    'border-amber-500/60 bg-amber-100/90 text-amber-900 dark:bg-amber-500/10 dark:text-amber-100',
};

export function ToastViewport() {
  const toasts = useUIStore((state) => state.toasts);
  const dismissToast = useUIStore((state) => state.dismissToast);

  return (
    <div
      className="pointer-events-none fixed inset-x-0 top-6 z-[9999] flex flex-col items-center gap-3 px-4"
      role="presentation"
    >
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} onDismiss={dismissToast} />
      ))}
    </div>
  );
}

interface ToastItemProps {
  toast: Toast;
  onDismiss: (id: string) => void;
}

function ToastItem({ toast, onDismiss }: ToastItemProps) {
  useEffect(() => {
    const timer = window.setTimeout(() => onDismiss(toast.id), toast.duration);
    return () => window.clearTimeout(timer);
  }, [onDismiss, toast.duration, toast.id]);

  return (
    <div
      role="alert"
      aria-live="polite"
      className={cn(
        'pointer-events-auto flex w-full max-w-md items-start gap-3 rounded-xl border px-4 py-3 text-sm shadow-lg transition-all',
        VARIANT_STYLES[toast.variant],
      )}
    >
      <div className="flex-1 space-y-1">
        <p className="font-semibold leading-tight">{toast.title}</p>
        {toast.description ? (
          <p className="text-xs leading-snug text-muted-foreground/90 dark:text-muted-foreground">
            {toast.description}
          </p>
        ) : null}
      </div>
      <button
        type="button"
        aria-label="关闭通知"
        onClick={() => onDismiss(toast.id)}
        className="ml-2 rounded-full border border-transparent p-1 text-xs font-medium text-muted-foreground transition hover:border-muted-foreground/40 hover:text-foreground"
      >
        ×
      </button>
    </div>
  );
}
