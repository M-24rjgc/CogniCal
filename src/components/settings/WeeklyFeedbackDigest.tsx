import { TrendingUp, TrendingDown, AlertCircle, CheckCircle2 } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { useWeeklyDigest } from '@/hooks/useFeedback';

export function WeeklyFeedbackDigest() {
  const { data: digest, isLoading, error } = useWeeklyDigest();

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>AI 反馈周报</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">加载中...</p>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>AI 反馈周报</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-destructive">加载失败: {error.message}</p>
        </CardContent>
      </Card>
    );
  }

  if (!digest) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>AI 反馈周报</CardTitle>
          <CardDescription>过去 7 天的反馈摘要</CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">
            反馈数据不足,至少需要 5 条反馈才能生成周报
          </p>
        </CardContent>
      </Card>
    );
  }

  const satisfactionRate =
    digest.totalFeedback > 0 ? (digest.positiveCount / digest.totalFeedback) * 100 : 0;

  const getSatisfactionColor = (rate: number) => {
    if (rate >= 80) return 'text-green-600';
    if (rate >= 50) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getSatisfactionBadge = (rate: number) => {
    if (rate >= 80)
      return (
        <Badge variant="default" className="bg-green-600">
          优秀
        </Badge>
      );
    if (rate >= 50)
      return (
        <Badge variant="default" className="bg-yellow-600">
          良好
        </Badge>
      );
    return <Badge variant="destructive">需改进</Badge>;
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>AI 反馈周报</CardTitle>
            <CardDescription>
              {new Date(digest.periodStart).toLocaleDateString()} -{' '}
              {new Date(digest.periodEnd).toLocaleDateString()}
            </CardDescription>
          </div>
          {getSatisfactionBadge(satisfactionRate)}
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Overall Satisfaction */}
        <div className="space-y-2">
          <div className="flex items-center justify-between text-sm">
            <span className="font-medium">整体满意度</span>
            <span className={`font-bold ${getSatisfactionColor(satisfactionRate)}`}>
              {satisfactionRate.toFixed(1)}%
            </span>
          </div>
          <Progress value={satisfactionRate} className="h-2" />
          <div className="flex items-center justify-between text-xs text-muted-foreground">
            <span className="flex items-center gap-1">
              <TrendingUp className="h-3 w-3" />
              {digest.positiveCount} 正面
            </span>
            <span className="flex items-center gap-1">
              <TrendingDown className="h-3 w-3" />
              {digest.negativeCount} 负面
            </span>
            <span>共 {digest.totalFeedback} 条反馈</span>
          </div>
        </div>

        {/* By Surface */}
        <div className="space-y-3">
          <h4 className="text-sm font-medium">功能反馈</h4>
          {digest.bySurface.map((surface) => (
            <div key={surface.surface} className="space-y-1">
              <div className="flex items-center justify-between text-sm">
                <span className="capitalize">{getSurfaceLabel(surface.surface)}</span>
                <span className="text-muted-foreground">
                  {surface.positive} 👍 / {surface.negative} 👎
                </span>
              </div>
              <Progress value={surface.satisfactionRate * 100} className="h-1.5" />
            </div>
          ))}
        </div>

        {/* Insights */}
        {digest.insights.length > 0 && (
          <div className="space-y-2">
            <h4 className="text-sm font-medium flex items-center gap-1.5">
              <AlertCircle className="h-4 w-4" />
              关键洞察
            </h4>
            <ul className="space-y-1.5">
              {digest.insights.map((insight, index) => (
                <li key={index} className="text-sm text-muted-foreground flex items-start gap-2">
                  <span className="text-primary mt-0.5">•</span>
                  <span>{insight}</span>
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Adjustments */}
        {digest.adjustmentsMade.length > 0 && (
          <div className="space-y-2">
            <h4 className="text-sm font-medium flex items-center gap-1.5">
              <CheckCircle2 className="h-4 w-4" />
              改进建议
            </h4>
            <ul className="space-y-1.5">
              {digest.adjustmentsMade.map((adjustment, index) => (
                <li key={index} className="text-sm text-muted-foreground flex items-start gap-2">
                  <span className="text-green-600 mt-0.5">✓</span>
                  <span>{adjustment}</span>
                </li>
              ))}
            </ul>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function getSurfaceLabel(surface: string): string {
  const labels: Record<string, string> = {
    score: '生产力评分',
    recommendation: '任务推荐',
    forecast: '工作负载预测',
  };
  return labels[surface] || surface;
}
