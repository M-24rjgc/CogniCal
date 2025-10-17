import { AlertTriangle, ArrowRight, Bot, CheckCircle2, Info, Lightbulb } from 'lucide-react';
import { Link } from 'react-router-dom';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Badge } from '../ui/badge';
import { Skeleton } from '../ui/skeleton';
import {
  type AnalyticsEfficiency,
  type EfficiencySuggestion,
  type InsightCard,
} from '../../types/analytics';

interface EfficiencyInsightsProps {
  efficiency: AnalyticsEfficiency | null;
  insights: InsightCard[];
  isLoading: boolean;
}

export function EfficiencyInsights({ efficiency, insights, isLoading }: EfficiencyInsightsProps) {
  const stats = efficiency ? buildStats(efficiency) : [];

  return (
    <section className="grid gap-6 lg:grid-cols-1 xl:grid-cols-2">
      <Card className="h-full">
        <CardHeader className="space-y-1">
          <CardTitle className="text-lg">效率洞察</CardTitle>
          <p className="text-sm text-muted-foreground">
            根据近期任务执行情况自动生成改进建议，帮助你持续优化规划节奏。
          </p>
        </CardHeader>
        <CardContent className="space-y-4">
          {isLoading ? (
            <LoadingState />
          ) : stats.length === 0 ? (
            <EmptyState message="暂无足够数据生成效率指标，请先完成更多任务。" />
          ) : (
            <div className="grid gap-4 sm:grid-cols-3">
              {stats.map((item) => (
                <article
                  key={item.id}
                  className="rounded-xl border border-border/70 bg-muted/40 p-4"
                >
                  <div className="mb-2 inline-flex items-center gap-2 text-sm font-medium text-foreground">
                    <item.icon className="h-4 w-4 text-primary" />
                    {item.label}
                  </div>
                  <p className="text-2xl font-semibold text-foreground">{item.value}</p>
                  <p className="mt-1 text-xs text-muted-foreground">{item.description}</p>
                </article>
              ))}
            </div>
          )}

          {efficiency?.suggestions?.length ? (
            <SuggestionList suggestions={efficiency.suggestions} />
          ) : null}
        </CardContent>
      </Card>

      <Card className="h-full">
        <CardHeader className="space-y-1">
          <CardTitle className="text-lg">重点提醒</CardTitle>
          <p className="text-sm text-muted-foreground">
            AI 与规则引擎生成的重点提醒，帮助你及时处理风险或机会。
          </p>
        </CardHeader>
        <CardContent className="space-y-4">
          {isLoading ? (
            <LoadingState />
          ) : insights.length === 0 ? (
            <EmptyState message="暂无提醒。完成更多计划与任务，会让洞察更加精准。" />
          ) : (
            <ul className="space-y-3 text-sm">
              {insights.map((insight) => (
                <li
                  key={insight.id}
                  className="rounded-xl border border-border/80 bg-background/80 p-4 shadow-sm"
                >
                  <div className="mb-1 flex items-center gap-2">
                    <SeverityBadge severity={insight.severity} source={insight.source} />
                    <span className="text-xs text-muted-foreground">
                      {new Date(insight.generatedAt).toLocaleString('zh-CN')}
                    </span>
                  </div>
                  <p className="text-sm font-semibold text-foreground">{insight.headline}</p>
                  <p className="mt-1 text-xs text-muted-foreground">{insight.detail}</p>
                  {insight.actionHref && insight.actionLabel ? (
                    <Link
                      to={insight.actionHref}
                      className="mt-2 inline-flex items-center gap-1 text-xs font-medium text-primary underline-offset-4 hover:underline"
                    >
                      {insight.actionLabel}
                      <ArrowRight className="h-3.5 w-3.5" />
                    </Link>
                  ) : null}
                </li>
              ))}
            </ul>
          )}
        </CardContent>
      </Card>
    </section>
  );
}

function buildStats(efficiency: AnalyticsEfficiency) {
  return [
    {
      id: 'estimate-accuracy',
      label: '预估准确率',
      value: `${Math.round(efficiency.estimateAccuracy * 100)}%`,
      description: '任务耗时预估与实际表现的接近程度',
      icon: Lightbulb,
    },
    {
      id: 'on-time-rate',
      label: '准时完成率',
      value: `${Math.round(efficiency.onTimeRate * 100)}%`,
      description: '计划内按时完成任务的比例',
      icon: CheckCircle2,
    },
    {
      id: 'complexity-correlation',
      label: '复杂度相关性',
      value: `${Math.round(efficiency.complexityCorrelation * 100)}%`,
      description: '复杂度评估与实际耗时的匹配程度',
      icon: Bot,
    },
  ];
}

function SuggestionList({ suggestions }: { suggestions: EfficiencySuggestion[] }) {
  return (
    <div className="space-y-3">
      <h4 className="text-sm font-medium text-foreground">智能改进建议</h4>
      <ul className="space-y-2">
        {suggestions.map((suggestion) => (
          <li
            key={suggestion.id}
            className="rounded-lg border border-border/70 bg-background/60 p-4"
          >
            <div className="flex flex-wrap items-center justify-between gap-2">
              <p className="text-sm font-semibold text-foreground">{suggestion.title}</p>
              <Badge variant="secondary" className="text-xs">
                {impactLabel(suggestion.impact)} · 置信度 {Math.round(suggestion.confidence * 100)}%
              </Badge>
            </div>
            <p className="mt-1 text-xs text-muted-foreground">{suggestion.summary}</p>
            {suggestion.relatedTaskId ? (
              <Link
                to={`/tasks/${suggestion.relatedTaskId}`}
                className="mt-2 inline-flex items-center gap-1 text-xs text-primary"
              >
                查看相关任务
                <ArrowRight className="h-3.5 w-3.5" />
              </Link>
            ) : null}
            {suggestion.relatedPlanId ? (
              <Link
                to={`/planning/${suggestion.relatedPlanId}`}
                className="mt-2 ml-4 inline-flex items-center gap-1 text-xs text-primary"
              >
                查看规划详情
                <ArrowRight className="h-3.5 w-3.5" />
              </Link>
            ) : null}
          </li>
        ))}
      </ul>
    </div>
  );
}

function SeverityBadge({
  severity,
  source,
}: {
  severity: InsightCard['severity'];
  source: InsightCard['source'];
}) {
  const icon = severityIcon(severity);
  const label = severityLabel(severity);
  return (
    <Badge variant="outline" className="flex items-center gap-1 text-xs">
      {icon}
      {label}
      <span className="ml-1 rounded-full bg-muted px-2 py-0.5 text-[10px] uppercase tracking-widest text-muted-foreground">
        {source === 'ai' ? 'AI' : source === 'rule' ? 'RULE' : 'MANUAL'}
      </span>
    </Badge>
  );
}

function severityIcon(severity: InsightCard['severity']) {
  switch (severity) {
    case 'critical':
      return <AlertTriangle className="h-3.5 w-3.5 text-rose-500" />;
    case 'warning':
      return <AlertTriangle className="h-3.5 w-3.5 text-amber-500" />;
    case 'success':
      return <CheckCircle2 className="h-3.5 w-3.5 text-emerald-500" />;
    default:
      return <Info className="h-3.5 w-3.5 text-primary" />;
  }
}

function severityLabel(severity: InsightCard['severity']) {
  switch (severity) {
    case 'critical':
      return '高优先级';
    case 'warning':
      return '提醒';
    case 'success':
      return '积极信号';
    default:
      return '提示';
  }
}

function impactLabel(impact: EfficiencySuggestion['impact']) {
  switch (impact) {
    case 'high':
      return '高影响';
    case 'medium':
      return '中影响';
    default:
      return '低影响';
  }
}

function LoadingState() {
  return (
    <div className="space-y-3">
      {[...Array(3)].map((_, index) => (
        <Skeleton key={index} className="h-20 w-full" />
      ))}
    </div>
  );
}

function EmptyState({ message }: { message: string }) {
  return (
    <div className="flex min-h-[120px] flex-col items-center justify-center rounded-lg border border-dashed text-center text-sm text-muted-foreground">
      <p>{message}</p>
    </div>
  );
}
