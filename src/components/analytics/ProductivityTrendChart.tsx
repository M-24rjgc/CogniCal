import { useMemo } from 'react';
import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Button } from '../ui/button';
import { Skeleton } from '../ui/skeleton';
import { type AnalyticsGrouping, type AnalyticsHistoryPoint } from '../../types/analytics';
import { type ProductivityScoreRecord } from '../../types/productivity.ts';

interface ProductivityTrendChartProps {
  analyticsData?: AnalyticsHistoryPoint[];
  productivityScores?: ProductivityScoreRecord[];
  grouping: AnalyticsGrouping;
  isLoading: boolean;
  rangeLabel: string;
  onDateRangeChange?: (range: '7d' | '30d' | '90d') => void;
  currentRange?: '7d' | '30d' | '90d';
  viewMode?: 'analytics' | 'productivity';
}

export function ProductivityTrendChart({
  analyticsData,
  productivityScores,
  grouping,
  isLoading,
  rangeLabel,
  onDateRangeChange,
  currentRange = '7d',
  viewMode = 'analytics',
}: ProductivityTrendChartProps) {
  const chartData = useMemo(() => {
    if (viewMode === 'productivity' && productivityScores) {
      return productivityScores.map((score) => ({
        date: new Date(score.snapshotDate).toLocaleDateString('zh-CN', {
          month: 'short',
          day: 'numeric',
        }),
        fullDate: score.snapshotDate,
        score: score.compositeScore,
        completionRate: score.dimensionScores.completionRate,
        onTimeRatio: score.dimensionScores.onTimeRatio,
        focusConsistency: score.dimensionScores.focusConsistency,
        restBalance: score.dimensionScores.restBalance,
        efficiencyRating: score.dimensionScores.efficiencyRating,
      }));
    }
    return analyticsData ?? [];
  }, [analyticsData, productivityScores, viewMode]);

  return (
    <Card className="h-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="text-lg">
              {viewMode === 'productivity' ? '生产力评分趋势' : '生产力走势'}
            </CardTitle>
            <p className="text-sm text-muted-foreground">
              {viewMode === 'productivity'
                ? `查看${rangeLabel}内的生产力评分变化趋势`
                : `查看${rangeLabel}内${grouping === 'day' ? '每日' : '每周'}的完成率与专注度变化。`}
            </p>
          </div>
          {viewMode === 'productivity' && onDateRangeChange && (
            <div className="flex space-x-1">
              {(['7d', '30d', '90d'] as const).map((range) => (
                <Button
                  key={range}
                  variant={currentRange === range ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => onDateRangeChange(range)}
                >
                  {range === '7d' && '7天'}
                  {range === '30d' && '30天'}
                  {range === '90d' && '90天'}
                </Button>
              ))}
            </div>
          )}
        </div>
      </CardHeader>
      <CardContent className="h-[400px] pt-6">
        {isLoading ? (
          <div className="flex h-full flex-col justify-between py-4">
            {[...Array(4)].map((_, index) => (
              <Skeleton key={index} className="h-8 w-full" />
            ))}
          </div>
        ) : chartData.length === 0 ? (
          <EmptyState />
        ) : (
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={chartData}>
              <CartesianGrid strokeDasharray="4 4" className="stroke-border" />
              <XAxis
                dataKey="date"
                tickFormatter={(value) => formatAxisLabel(value, grouping)}
                className="text-xs text-muted-foreground"
              />
              <YAxis
                yAxisId="left"
                tickFormatter={(value) => `${Math.round(Number(value) * 100)}%`}
                className="text-xs text-muted-foreground"
                domain={[0, 1]}
              />
              <YAxis
                yAxisId="right"
                orientation="right"
                className="text-xs text-muted-foreground"
              />
              <Tooltip content={<TrendTooltip />} />
              <Legend />
              <Line
                type="monotone"
                dataKey="completionRate"
                yAxisId="left"
                name="完成率"
                stroke="#6366f1"
                strokeWidth={2}
                dot={false}
              />
              <Line
                type="monotone"
                dataKey="productivityScore"
                yAxisId="right"
                name="生产力得分"
                stroke="#22c55e"
                strokeWidth={2}
              />
              <Line
                type="monotone"
                dataKey="focusMinutes"
                yAxisId="right"
                name="专注时长 (分钟)"
                stroke="#0ea5e9"
                strokeWidth={2}
                dot={false}
                strokeDasharray="6 6"
              />
            </LineChart>
          </ResponsiveContainer>
        )}
      </CardContent>
    </Card>
  );
}

function formatAxisLabel(date: string, grouping: AnalyticsGrouping) {
  const formatter = new Intl.DateTimeFormat(
    'zh-CN',
    grouping === 'day'
      ? { month: 'numeric', day: 'numeric' }
      : { month: 'numeric', day: 'numeric' },
  );
  return formatter.format(new Date(date));
}

function EmptyState() {
  return (
    <div className="flex h-full flex-col items-center justify-center text-center text-sm text-muted-foreground">
      <p>尚无历史数据，完成更多任务即可查看趋势。</p>
    </div>
  );
}

type TrendTooltipEntry = {
  value: number;
  name: string;
  color: string;
  dataKey: keyof AnalyticsHistoryPoint;
};

type TrendTooltipProps = {
  active?: boolean;
  payload?: TrendTooltipEntry[];
  label?: string;
};

function TrendTooltip({ active, payload, label }: TrendTooltipProps) {
  if (!active || !payload || payload.length === 0) return null;

  const formatter = new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: 'numeric',
    day: 'numeric',
  });
  const formattedLabel = label ? formatter.format(new Date(label)) : '';

  const entries = payload.map((entry) => {
    if (entry.dataKey === 'completionRate') {
      return { ...entry, displayValue: `${Math.round(entry.value * 100)}%` };
    }
    if (entry.dataKey === 'focusMinutes') {
      return { ...entry, displayValue: `${Math.round(entry.value)} 分钟` };
    }
    return { ...entry, displayValue: entry.value.toFixed(1) };
  });

  return (
    <div className="rounded-md border bg-popover px-3 py-2 text-xs shadow">
      <p className="mb-1 font-medium text-foreground">{formattedLabel}</p>
      <div className="space-y-1">
        {entries.map((entry) => (
          <div key={String(entry.dataKey)} className="flex items-center gap-2">
            <span className="h-2 w-2 rounded-full" style={{ backgroundColor: entry.color }} />
            <span className="text-muted-foreground">{entry.name}</span>
            <span className="font-medium text-foreground">{entry.displayValue}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
