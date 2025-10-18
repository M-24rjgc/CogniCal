import { Component, type ErrorInfo, Suspense, type ReactNode, type FC, useMemo } from 'react';
import { AlertCircle, RefreshCw } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Button } from '../ui/button';
import { Skeleton } from '../ui/skeleton';
import { cn } from '../../lib/utils';
import type { DashboardModuleId } from '../../types/dashboard';

type ModuleErrorFallbackRenderer = (error: Error, reset: () => void) => ReactNode;

interface ModuleErrorBoundaryProps {
  fallback: ModuleErrorFallbackRenderer;
  resetKeys: unknown[];
  children: ReactNode;
}

interface ModuleErrorBoundaryState {
  error: Error | null;
}

class ModuleErrorBoundary extends Component<ModuleErrorBoundaryProps, ModuleErrorBoundaryState> {
  state: ModuleErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): ModuleErrorBoundaryState {
    return { error };
  }

  componentDidUpdate(prevProps: ModuleErrorBoundaryProps) {
    if (this.state.error) {
      const { resetKeys } = this.props;
      const { resetKeys: previousKeys } = prevProps;
      if (
        resetKeys.length !== previousKeys.length ||
        resetKeys.some((value, index) => value !== previousKeys[index])
      ) {
        this.reset();
      }
    }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    if (typeof console !== 'undefined' && typeof console.error === 'function') {
      console.error('[ModuleContainer] 模块渲染失败', error, info);
    }
  }

  reset = () => {
    this.setState({ error: null });
  };

  render() {
    const { error } = this.state;
    if (error) {
      return this.props.fallback(error, this.reset);
    }
    return this.props.children;
  }
}

const DEFAULT_LOADING_PLACEHOLDER = (
  <div className="grid gap-3">
    <Skeleton className="h-5 w-40" />
    <Skeleton className="h-3.5 w-2/3" />
    <Skeleton className="h-24 w-full" />
  </div>
);

const DEFAULT_EMPTY_STATE = (
  <div className="flex flex-col items-start gap-2 rounded-xl border border-dashed border-border/60 bg-muted/20 p-6 text-sm text-muted-foreground">
    <span>当前暂无数据，稍后再来看看或尝试刷新模块。</span>
  </div>
);

export interface ModuleContainerProps {
  moduleId: DashboardModuleId;
  title: string;
  description?: string;
  actions?: ReactNode;
  children: ReactNode;
  className?: string;
  bodyClassName?: string;
  isLoading?: boolean;
  loadingFallback?: ReactNode;
  isEmpty?: boolean;
  emptyState?: ReactNode;
  error?: Error | null;
  onRetry?: () => void | Promise<void>;
  lazy?: boolean;
  suspenseFallback?: ReactNode;
}

const ModuleContainer: FC<ModuleContainerProps> = ({
  moduleId,
  title,
  description,
  actions,
  children,
  className,
  bodyClassName,
  isLoading = false,
  loadingFallback = DEFAULT_LOADING_PLACEHOLDER,
  isEmpty = false,
  emptyState = DEFAULT_EMPTY_STATE,
  error = null,
  onRetry,
  lazy = false,
  suspenseFallback,
}) => {
  const loadingContent = useMemo(
    () => loadingFallback ?? DEFAULT_LOADING_PLACEHOLDER,
    [loadingFallback],
  );
  const emptyContent = useMemo(() => emptyState ?? DEFAULT_EMPTY_STATE, [emptyState]);

  const renderErrorFallback: ModuleErrorFallbackRenderer = (caughtError, reset) => (
    <div className="flex flex-col gap-3 rounded-2xl border border-destructive/40 bg-destructive/10 p-4">
      <div className="flex items-start gap-3">
        <AlertCircle className="mt-0.5 h-4 w-4 text-destructive" />
        <div className="space-y-1 text-sm">
          <p className="font-medium text-destructive">模块加载失败</p>
          <p className="text-destructive/80">
            {caughtError?.message?.trim() || '发生未知错误，请稍后重试。'}
          </p>
        </div>
      </div>
      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          size="sm"
          variant="outline"
          onClick={() => {
            reset();
            void onRetry?.();
          }}
          className="inline-flex items-center gap-1"
        >
          <RefreshCw className="h-3.5 w-3.5" />
          重试
        </Button>
      </div>
    </div>
  );

  const renderContent = () => {
    if (isLoading) {
      return loadingContent;
    }

    if (isEmpty) {
      return emptyContent;
    }

    const content = lazy ? (
      <Suspense fallback={suspenseFallback ?? loadingContent}>{children}</Suspense>
    ) : (
      children
    );

    return (
      <ModuleErrorBoundary fallback={renderErrorFallback} resetKeys={[moduleId, isLoading]}>
        {content}
      </ModuleErrorBoundary>
    );
  };

  const effectiveContent = error
    ? renderErrorFallback(error, () => {
        void onRetry?.();
      })
    : renderContent();

  return (
    <Card className={cn('dashboard-module', className)} data-module-id={moduleId}>
      <CardHeader className="gap-2">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="space-y-1">
            <CardTitle className="text-base font-semibold text-foreground">{title}</CardTitle>
            {description ? (
              <p className="text-sm text-muted-foreground leading-relaxed">{description}</p>
            ) : null}
          </div>
          {actions ? <div className="flex shrink-0 items-center gap-2">{actions}</div> : null}
        </div>
      </CardHeader>
      <CardContent className={cn('space-y-3', bodyClassName)}>{effectiveContent}</CardContent>
    </Card>
  );
};

export default ModuleContainer;
