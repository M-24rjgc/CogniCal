import { ExternalLink, HelpCircle, RefreshCcw, RotateCcw, Sparkles } from 'lucide-react';
import type { ShortcutGroup } from '../keyboard/KeyboardShortcutsHelp';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { Progress } from '../ui/progress';

export interface HelpResourceLinkItem {
  label: string;
  href: string;
  external?: boolean;
}

export interface HelpCenterContentProps {
  onboarding: {
    totalSteps: number;
    completedSteps: number;
    hasCompletedTour: boolean;
    pendingStepTitle: string | null;
    lastCompletedTitle: string | null;
    dismissedAtLabel: string | null;
  };
  settings: {
    hasDeepseekKey: boolean;
    maskedKey: string | null;
    lastUpdatedLabel: string;
    isLoading: boolean;
    errorMessage?: string | null;
  };
  shortcutGroups: ShortcutGroup[];
  resources: HelpResourceLinkItem[];
  onReplayTour: () => void;
  onResetProgress: () => void;
  onOpenDocs: () => void;
  onNavigateToSettings: () => void;
  onOpenShortcuts: () => void;
}

export function HelpCenterContent({
  onboarding,
  settings,
  shortcutGroups,
  resources,
  onReplayTour,
  onResetProgress,
  onOpenDocs,
  onNavigateToSettings,
  onOpenShortcuts,
}: HelpCenterContentProps) {
  const progressPercent = onboarding.totalSteps
    ? Math.min(100, Math.round((onboarding.completedSteps / onboarding.totalSteps) * 100))
    : 0;

  return (
    <div className="space-y-6">
      <header className="flex flex-col gap-2 border-b border-border/40 pb-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="space-y-1">
          <h2 className="text-2xl font-semibold text-foreground">帮助与支持中心</h2>
          <p className="text-sm text-muted-foreground">
            快速查看引导进度、DeepSeek 配置状态与常用快捷键，随时重新播放引导或进入文档。
          </p>
        </div>
        <Badge variant={onboarding.hasCompletedTour ? 'secondary' : 'outline'} className="w-fit">
          {onboarding.hasCompletedTour ? '引导完成' : '引导未完成'}
        </Badge>
      </header>

      <section className="grid gap-4 md:grid-cols-2">
        <article className="rounded-2xl border border-border/60 bg-muted/20 p-5">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Sparkles className="h-5 w-5 text-primary" />
              <h3 className="text-sm font-semibold text-foreground">互动引导进度</h3>
            </div>
            <span className="text-xs text-muted-foreground">
              {onboarding.completedSteps}/{onboarding.totalSteps} 步
            </span>
          </div>
          <div className="mt-4 space-y-2">
            <Progress value={progressPercent} aria-label="引导完成度" />
            <p className="text-xs text-muted-foreground">
              {onboarding.pendingStepTitle
                ? `下一步：${onboarding.pendingStepTitle}`
                : onboarding.hasCompletedTour
                  ? '已完成全部引导步骤，可随时重新播放。'
                  : '引导进度将帮助你快速掌握核心功能。'}
            </p>
            {onboarding.lastCompletedTitle ? (
              <p className="text-xs text-muted-foreground">
                最近完成：{onboarding.lastCompletedTitle}
              </p>
            ) : null}
            {onboarding.dismissedAtLabel ? (
              <p className="text-[11px] text-muted-foreground/80">
                上次关闭：{onboarding.dismissedAtLabel}
              </p>
            ) : null}
          </div>
          <div className="mt-4 flex flex-wrap gap-2">
            <Button type="button" size="sm" onClick={onReplayTour}>
              <RefreshCcw className="mr-2 h-4 w-4" /> 重新播放引导
            </Button>
            <Button type="button" size="sm" variant="ghost" onClick={onResetProgress}>
              <RotateCcw className="mr-2 h-4 w-4" /> 重置进度
            </Button>
          </div>
        </article>

        <article className="rounded-2xl border border-border/60 bg-background/90 p-5 shadow-sm">
          <div className="flex items-start justify-between gap-2">
            <div className="space-y-1">
              <h3 className="flex items-center gap-2 text-sm font-semibold text-foreground">
                <HelpCircle className="h-4 w-4 text-primary" /> DeepSeek API 配置提醒
              </h3>
              <p className="text-xs text-muted-foreground">
                {settings.hasDeepseekKey
                  ? 'DeepSeek API 已启用，可正常使用解析、规划与分析功能。'
                  : '尚未配置 DeepSeek API Key，建议尽快前往设置完成配置。'}
              </p>
            </div>
            <Badge variant={settings.hasDeepseekKey ? 'secondary' : 'destructive'}>
              {settings.hasDeepseekKey ? '已配置' : '未配置'}
            </Badge>
          </div>
          <div className="mt-3 space-y-2 rounded-xl border border-border/60 bg-muted/30 p-3 text-xs text-muted-foreground">
            <div className="flex items-center justify-between">
              <span>密钥预览</span>
              <span className="font-mono text-[11px] text-muted-foreground/80">
                {settings.maskedKey ?? '••••••••'}
              </span>
            </div>
            <div className="flex items-center justify-between text-[11px]">
              <span>最近更新</span>
              <span>{settings.lastUpdatedLabel}</span>
            </div>
          </div>
          <div className="mt-4 flex flex-wrap gap-2">
            <Button
              type="button"
              size="sm"
              onClick={onNavigateToSettings}
              disabled={settings.isLoading}
            >
              前往设置
            </Button>
            <Button type="button" size="sm" variant="outline" onClick={onOpenDocs}>
              查看配置指南
            </Button>
          </div>
          {settings.errorMessage ? (
            <p className="mt-3 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-xs text-destructive">
              {settings.errorMessage}
            </p>
          ) : null}
        </article>
      </section>

      <section className="grid gap-4 lg:grid-cols-[2fr_3fr]">
        <article className="rounded-2xl border border-border/60 bg-muted/20 p-5">
          <header className="flex items-center justify-between">
            <h3 className="text-sm font-semibold text-foreground">资源与操作</h3>
            <Button type="button" variant="ghost" size="sm" onClick={onOpenDocs}>
              <ExternalLink className="mr-2 h-4 w-4" /> 打开文档
            </Button>
          </header>
          <ul className="mt-3 space-y-2">
            {resources.length > 0 ? (
              resources.map((resource) => (
                <li key={`${resource.href}-${resource.label}`}>
                  <a
                    href={resource.href}
                    target={resource.external ? '_blank' : undefined}
                    rel={resource.external ? 'noreferrer' : undefined}
                    className="flex items-center justify-between gap-3 rounded-xl border border-transparent bg-background/70 px-3 py-2 text-sm text-muted-foreground transition hover:border-primary/40 hover:text-foreground"
                  >
                    <span>{resource.label}</span>
                    {resource.external ? (
                      <ExternalLink className="h-4 w-4" aria-hidden="true" />
                    ) : null}
                  </a>
                </li>
              ))
            ) : (
              <li className="rounded-xl border border-dashed border-border/60 bg-background/70 px-3 py-6 text-center text-xs text-muted-foreground">
                暂无更多资源，文档将持续更新。
              </li>
            )}
          </ul>
        </article>

        <article className="rounded-2xl border border-border/60 bg-background/95 p-5 shadow-sm">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold text-foreground">常用快捷键</h3>
            <Button type="button" variant="outline" size="sm" onClick={onOpenShortcuts}>
              查看全部
            </Button>
          </div>
          <div className="mt-4 grid gap-3 md:grid-cols-2">
            {shortcutGroups.map((group) => (
              <div
                key={group.title}
                className="space-y-2 rounded-xl border border-border/60 bg-muted/20 p-3"
              >
                <h4 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                  {group.title}
                </h4>
                <div className="space-y-2">
                  {group.shortcuts.map((shortcut) => (
                    <div
                      key={`${group.title}-${shortcut.keys}`}
                      className="flex items-center justify-between gap-3 rounded-lg bg-background/80 px-3 py-2 text-xs text-muted-foreground"
                    >
                      <span className="max-w-[70%]">{shortcut.description}</span>
                      <kbd className="whitespace-nowrap rounded border border-border/60 bg-muted px-2 py-1 text-[11px] font-medium text-muted-foreground">
                        {shortcut.keys}
                      </kbd>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </article>
      </section>
    </div>
  );
}

export default HelpCenterContent;
