import { useMemo } from 'react';
import { Link } from 'react-router-dom';
import { AlertTriangle, BarChart3, CalendarClock, Clock3, Layers3, ListChecks } from 'lucide-react';
import { usePlanning } from '../hooks/usePlanning';
import { useTasks } from '../hooks/useTasks';
import { Badge } from '../components/ui/badge';
import { Button } from '../components/ui/button';
import type { PlanningOptionView } from '../types/planning';
import { useAnalyticsStore } from '../stores/analyticsStore';
import { useSettingsStore } from '../stores/settingsStore';
import { HelpPopover } from '../components/help/HelpPopover';

const DATE_FORMATTER = new Intl.DateTimeFormat('zh-CN', {
  month: '2-digit',
  day: '2-digit',
  weekday: 'short',
});

const TIME_FORMATTER = new Intl.DateTimeFormat('zh-CN', {
  hour: '2-digit',
  minute: '2-digit',
});

const STATUS_COPY: Record<string, { label: string; className: string }> = {
  draft: { label: '草稿', className: 'bg-muted text-muted-foreground' },
  planned: { label: '已规划', className: 'bg-sky-500/15 text-sky-600 dark:text-sky-400' },
  applied: {
    label: '已应用',
    className: 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400',
  },
  completed: { label: '已完成', className: 'bg-primary/15 text-primary' },
  skipped: { label: '已跳过', className: 'bg-muted text-muted-foreground' },
  conflicted: { label: '存在冲突', className: 'bg-rose-500/15 text-rose-600 dark:text-rose-400' },
};

export default function CalendarPage() {
  const { session, selectedOption, hasEventBridge } = usePlanning({
    autoAttachEvents: true,
    autoLoadPreferences: false,
  });
  const { tasks } = useTasks({ autoFetch: true });

  const taskById = useMemo(() => {
    const map = new Map<string, string>();
    for (const task of tasks) {
      map.set(task.id, task.title);
    }
    return map;
  }, [tasks]);

  const conflictsByBlock = useMemo(() => {
    if (!selectedOption) return new Map<string, number>();
    const acc = new Map<string, number>();
    for (const conflict of selectedOption.conflicts) {
      if (!conflict.relatedBlockId) continue;
      acc.set(conflict.relatedBlockId, (acc.get(conflict.relatedBlockId) ?? 0) + 1);
    }
    return acc;
  }, [selectedOption]);

  type PlanningBlock = PlanningOptionView['blocks'][number];

  type BlockGroup = {
    dateKey: string;
    label: string;
    blocks: PlanningBlock[];
  };

  const blockGroups = useMemo<BlockGroup[]>(() => {
    if (!selectedOption) return [];
    const groups = new Map<string, { label: string; blocks: PlanningBlock[] }>();
    const sortedBlocks = [...selectedOption.blocks].sort(
      (a, b) => new Date(a.startAt).getTime() - new Date(b.startAt).getTime(),
    );

    for (const block of sortedBlocks) {
      const start = new Date(block.startAt);
      const dateKey = start.toISOString().split('T')[0] ?? block.startAt;
      const label = DATE_FORMATTER.format(start);
      const entry = groups.get(dateKey);
      if (entry) {
        entry.blocks.push(block);
      } else {
        groups.set(dateKey, { label, blocks: [block] });
      }
    }

    return Array.from(groups.entries()).map(([dateKey, value]) => ({
      dateKey,
      label: value.label,
      blocks: value.blocks,
    }));
  }, [selectedOption]);

  const conflicts = selectedOption?.conflicts ?? session?.conflicts ?? [];

  const selectedLabel = selectedOption?.option.summary ?? '当前方案';

  const statusSummary = session?.session.status === 'applied' ? '已生效' : '待应用';
  const updatedAt = session ? new Date(session.session.updatedAt) : null;

  const analyticsLastRefreshed = useAnalyticsStore((state) => state.lastRefreshedAt);
  const analyticsIsDemo = useAnalyticsStore((state) => state.isDemoData);
  const hasDeepseekKey = useSettingsStore((state) => state.settings?.hasDeepseekKey ?? false);

  const analyticsSummaryCopy = analyticsLastRefreshed
    ? `最新仪表盘数据更新于 ${new Date(analyticsLastRefreshed).toLocaleString('zh-CN')}。`
    : hasDeepseekKey
      ? '完成规划后，可前往仪表盘查看专注时间与冲突洞察。'
      : '配置 DeepSeek API Key 才能启用智能分析与冲突洞察。';

  return (
    <section className="flex h-full flex-1 flex-col gap-6">
      <header className="flex flex-col gap-3 rounded-3xl border border-border/60 bg-background/80 p-6 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <Badge variant="secondary" className="bg-secondary/15 text-xs">
                <CalendarClock className="mr-1.5 h-3.5 w-3.5" /> 智能日程面板
              </Badge>
              <Badge variant={hasEventBridge ? 'secondary' : 'outline'} className="text-xs">
                {hasEventBridge ? '实时同步已开启' : '事件同步未连接'}
              </Badge>
            </div>
            <div className="flex items-center gap-2">
              <h1 className="text-2xl font-semibold text-foreground">规划时间线</h1>
              <HelpPopover
                entryId="planning-center"
                triggerLabel="查看规划时间线帮助说明"
                triggerClassName="ml-1"
              />
            </div>
            <p className="text-sm text-muted-foreground">
              查看最新应用的规划时间块，追踪冲突并保持与任务列表同步。
            </p>
          </div>
          <div className="flex flex-col items-end gap-2 text-xs text-muted-foreground">
            <Badge variant="outline" className="flex items-center gap-1">
              <Layers3 className="h-3.5 w-3.5" /> {selectedOption ? selectedLabel : '暂无方案'}
            </Badge>
            {updatedAt ? <span>最近更新：{TIME_FORMATTER.format(updatedAt)}</span> : null}
            {session ? <span>状态：{statusSummary}</span> : null}
          </div>
        </div>
      </header>

      <section className="flex flex-col gap-2 rounded-3xl border border-primary/40 bg-primary/5 p-5 text-sm text-primary">
        <header className="flex flex-wrap items-center gap-2">
          <BarChart3 className="h-4 w-4" />
          <span className="font-semibold">规划 & 分析联动</span>
          {analyticsIsDemo ? (
            <Badge variant="outline" className="border-primary/40 text-[11px] text-primary">
              示例数据
            </Badge>
          ) : null}
        </header>
        <p className="text-xs text-primary/80">{analyticsSummaryCopy}</p>
        <div className="flex flex-wrap gap-2 pt-1">
          <Button asChild size="sm" className="h-8 px-3 text-[12px]">
            <Link to="/">查看智能分析</Link>
          </Button>
          <Button asChild size="sm" variant="ghost" className="h-8 px-3 text-[12px]">
            <Link to="/settings">配置 AI 偏好</Link>
          </Button>
        </div>
      </section>

      {selectedOption ? (
        <div className="grid gap-6 lg:grid-cols-[2fr_1fr]">
          <section className="flex flex-col gap-4 rounded-3xl border border-border/60 bg-card/80 p-5 shadow-sm">
            <header className="flex items-center justify-between gap-2">
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <ListChecks className="h-4 w-4 text-primary" />
                <span>已排程任务 · {selectedOption.blocks.length} 个时间块</span>
              </div>
              <Badge variant="outline" className="text-xs">
                {conflicts.length ? `${conflicts.length} 项冲突等待处理` : '冲突已清空'}
              </Badge>
            </header>

            <div className="grid gap-4">
              {blockGroups.map((group) => (
                <article
                  key={group.dateKey}
                  className="rounded-2xl border border-border/60 bg-background/80 p-4 shadow-inner"
                >
                  <header className="mb-3 flex items-center justify-between">
                    <h3 className="text-sm font-semibold text-foreground">{group.label}</h3>
                    <Badge variant="secondary" className="text-[11px]">
                      {group.blocks.length} 个时间块
                    </Badge>
                  </header>
                  <ol className="grid gap-3">
                    {group.blocks.map((block) => {
                      const start = new Date(block.startAt);
                      const end = new Date(block.endAt);
                      const taskTitle = taskById.get(block.taskId) ?? `任务 ${block.taskId}`;
                      const statusMeta = STATUS_COPY[block.status] ?? STATUS_COPY.planned;
                      const conflictCount = conflictsByBlock.get(block.id) ?? 0;
                      return (
                        <li
                          key={block.id}
                          className="rounded-2xl border border-border/70 bg-card/70 p-4 transition hover:border-primary/40"
                        >
                          <div className="flex flex-col gap-2 sm:flex-row sm:items-baseline sm:justify-between">
                            <div className="flex flex-col gap-1">
                              <span className="text-base font-medium text-foreground">
                                {taskTitle}
                              </span>
                              <span className="flex items-center gap-2 text-xs text-muted-foreground">
                                <Clock3 className="h-3.5 w-3.5 text-primary" />
                                {TIME_FORMATTER.format(start)} — {TIME_FORMATTER.format(end)}
                              </span>
                            </div>
                            <div className="flex flex-wrap items-center gap-2">
                              <Badge className={`text-[11px] ${statusMeta.className}`}>
                                {statusMeta.label}
                              </Badge>
                              {typeof block.confidence === 'number' ? (
                                <Badge variant="outline" className="text-[11px]">
                                  置信 {Math.round(block.confidence * 100)}%
                                </Badge>
                              ) : null}
                              {block.flexibility ? (
                                <Badge variant="outline" className="text-[11px]">
                                  灵活度 · {block.flexibility}
                                </Badge>
                              ) : null}
                              {conflictCount ? (
                                <Badge
                                  variant="destructive"
                                  className="flex items-center gap-1 text-[11px]"
                                >
                                  <AlertTriangle className="h-3 w-3" /> 冲突 {conflictCount}
                                </Badge>
                              ) : null}
                            </div>
                          </div>
                        </li>
                      );
                    })}
                  </ol>
                </article>
              ))}
            </div>
          </section>

          <aside className="flex flex-col gap-4 rounded-3xl border border-border/60 bg-card/70 p-5 shadow-sm">
            <section className="space-y-2">
              <h3 className="flex items-center gap-2 text-sm font-semibold text-foreground">
                <AlertTriangle className="h-4 w-4 text-amber-500" /> 冲突监控
              </h3>
              {conflicts.length ? (
                <ul className="grid gap-3 text-xs text-muted-foreground">
                  {conflicts.map((conflict, index) => (
                    <li
                      key={`${conflict.conflictType}-${conflict.message}-${index}`}
                      className="rounded-2xl border border-amber-400/50 bg-amber-500/10 p-3"
                    >
                      <div className="flex items-center gap-2 text-amber-600">
                        <Badge variant="outline" className="border-amber-400 text-[11px]">
                          {conflictSeverityLabel(conflict.severity)}
                        </Badge>
                        <span className="text-muted-foreground">{conflict.conflictType}</span>
                      </div>
                      <p className="mt-2 text-amber-700">{conflict.message}</p>
                    </li>
                  ))}
                </ul>
              ) : (
                <div className="flex flex-col items-center justify-center gap-2 rounded-2xl border border-emerald-400/40 bg-emerald-500/10 p-6 text-sm text-emerald-600">
                  <span>暂无冲突，排程一切正常</span>
                </div>
              )}
            </section>

            <section className="space-y-2">
              <h3 className="flex items-center gap-2 text-sm font-semibold text-foreground">
                <CalendarClock className="h-4 w-4 text-primary" /> 会话概览
              </h3>
              <div className="rounded-2xl border border-border/60 bg-background/80 p-4 text-xs text-muted-foreground">
                {session ? (
                  <ul className="space-y-2">
                    <li>会话 ID：{session.session.id.slice(0, 8)}</li>
                    <li>任务数量：{session.session.taskIds.length}</li>
                    <li>方案数量：{session.options.length}</li>
                    <li>
                      生成时间：{TIME_FORMATTER.format(new Date(session.session.generatedAt))}
                    </li>
                  </ul>
                ) : (
                  <p>尚未生成规划方案。</p>
                )}
              </div>
            </section>
          </aside>
        </div>
      ) : (
        <div className="flex flex-1 items-center justify-center rounded-3xl border border-dashed border-border/60 bg-background/60 p-10 text-sm text-muted-foreground">
          当前还没有可显示的规划时间块，请在任务页面生成并应用方案后回来查看。
        </div>
      )}
    </section>
  );
}

function conflictSeverityLabel(value: string) {
  switch (value) {
    case 'high':
      return '高';
    case 'medium':
      return '中';
    case 'low':
    default:
      return '低';
  }
}
