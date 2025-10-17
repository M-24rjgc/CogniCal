import { Activity, Clock, TrendingUp, Zap } from 'lucide-react';
import { useWeeklySummary } from '@/hooks/useWellness';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

export function WeeklySummaryPanel() {
  const { data: summary, isLoading, error } = useWeeklySummary();

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>每周健康概览</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">加载中...</p>
        </CardContent>
      </Card>
    );
  }

  if (error || !summary) {
    return null; // Silently fail if no data
  }

  const complianceRate = summary.rest_compliance_rate ?? 0;
  const rhythmScore = summary.focus_rhythm_score ?? 0;
  const completedCount = summary.completed_count ?? 0;
  const totalNudges = summary.total_nudges ?? 0;
  const snoozedCount = summary.snoozed_count ?? 0;
  const ignoredCount = summary.ignored_count ?? 0;
  const peakHours = summary.peak_hours ?? [];

  const getComplianceColor = (rate: number) => {
    if (rate >= 0.8) return 'text-green-600';
    if (rate >= 0.5) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getRhythmColor = (score: number) => {
    if (score >= 80) return 'text-green-600';
    if (score >= 60) return 'text-yellow-600';
    return 'text-red-600';
  };

  const formatPeakHour = (hour: number) => {
    if (hour === 0) return '12:00 AM';
    if (hour < 12) return `${hour}:00 AM`;
    if (hour === 12) return '12:00 PM';
    return `${hour - 12}:00 PM`;
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Activity className="h-5 w-5" />
              每周健康概览
            </CardTitle>
            <CardDescription>过去 7 天的工作习惯分析</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
          {/* Statistics Cards */}
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <Zap className="h-4 w-4 text-blue-500" />
              <span className="text-sm font-medium">休息遵从率</span>
            </div>
            <p className={`text-3xl font-bold ${getComplianceColor(complianceRate)}`}>
              {(complianceRate * 100).toFixed(0)}%
            </p>
            <p className="text-xs text-muted-foreground">
              {completedCount} / {totalNudges} 次休息
            </p>
          </div>

          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <TrendingUp className="h-4 w-4 text-green-500" />
              <span className="text-sm font-medium">节奏得分</span>
            </div>
            <p className={`text-3xl font-bold ${getRhythmColor(rhythmScore)}`}>
              {rhythmScore.toFixed(0)}
            </p>
            <p className="text-xs text-muted-foreground">工作节奏健康度</p>
          </div>

          {/* Response Breakdown */}
          <div className="space-y-2">
            <div className="text-sm font-medium">响应分布</div>
            <div className="flex flex-col gap-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">完成</span>
                <Badge variant="default" className="bg-green-500">
                  {completedCount}
                </Badge>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">延迟</span>
                <Badge variant="secondary" className="bg-yellow-500 text-white">
                  {snoozedCount}
                </Badge>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">忽略</span>
                <Badge variant="outline">{ignoredCount}</Badge>
              </div>
            </div>
          </div>

          {/* Peak Hours */}
          {peakHours.length > 0 && (
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <Clock className="h-4 w-4 text-purple-500" />
                <span className="text-sm font-medium">高峰时段</span>
              </div>
              <div className="flex flex-wrap gap-2">
                {peakHours.slice(0, 3).map((hour: number, index: number) => (
                  <Badge key={index} variant="secondary" className="bg-purple-100 text-purple-700">
                    {formatPeakHour(hour)}
                  </Badge>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Recommendations - 独立一行 */}
        {summary.recommendations && summary.recommendations.length > 0 && (
          <div className="mt-6 space-y-2 border-t pt-4">
            <div className="text-sm font-medium">健康建议</div>
            <div className="grid gap-2 md:grid-cols-2 lg:grid-cols-3">
              {summary.recommendations.map((rec: string, index: number) => (
                <div
                  key={index}
                  className="flex items-start gap-2 rounded-lg border border-border/60 bg-muted/30 p-3 text-sm"
                >
                  <span className="text-blue-500 mt-0.5">•</span>
                  <span className="text-muted-foreground">{rec}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
