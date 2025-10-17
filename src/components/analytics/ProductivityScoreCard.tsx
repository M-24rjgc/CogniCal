import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { RefreshCw, TrendingUp, TrendingDown, Minus } from 'lucide-react';
import { ProductivityScoreRecord } from '../../types/productivity.ts';
import {
  getScoreLevel,
  getScoreColor,
  getScoreBgColor,
  formatScoreExplanation,
} from '../../hooks/useProductivityScore';

interface ProductivityScoreCardProps {
  score: ProductivityScoreRecord | null;
  isLoading?: boolean;
  error?: Error | null;
  onRefresh?: () => void;
  isRefreshing?: boolean;
}

export function ProductivityScoreCard({
  score,
  isLoading = false,
  error = null,
  onRefresh,
  isRefreshing = false,
}: ProductivityScoreCardProps) {
  if (isLoading) {
    return (
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">生产力评分</CardTitle>
          <div className="h-4 w-4 animate-pulse rounded bg-muted" />
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            <div className="h-8 w-16 animate-pulse rounded bg-muted" />
            <div className="space-y-2">
              <div className="h-4 w-full animate-pulse rounded bg-muted" />
              <div className="h-4 w-3/4 animate-pulse rounded bg-muted" />
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">生产力评分</CardTitle>
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isRefreshing}>
            <RefreshCw className={`h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
          </Button>
        </CardHeader>
        <CardContent>
          <div className="text-center text-sm text-muted-foreground">无法加载生产力评分</div>
        </CardContent>
      </Card>
    );
  }

  if (!score) {
    return (
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">生产力评分</CardTitle>
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isRefreshing}>
            <RefreshCw className={`h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
          </Button>
        </CardHeader>
        <CardContent>
          <div className="text-center text-sm text-muted-foreground">暂无生产力评分数据</div>
        </CardContent>
      </Card>
    );
  }

  const scoreLevel = getScoreLevel(score.compositeScore);
  const scoreColor = getScoreColor(score.compositeScore);
  const scoreBgColor = getScoreBgColor(score.compositeScore);
  const { summary, insights, recommendations } = formatScoreExplanation(score);

  const getTrendIcon = () => {
    switch (scoreLevel) {
      case 'excellent':
      case 'good':
        return <TrendingUp className="h-4 w-4 text-green-600" />;
      case 'needs-improvement':
        return <TrendingDown className="h-4 w-4 text-red-600" />;
      default:
        return <Minus className="h-4 w-4 text-yellow-600" />;
    }
  };

  const getLevelBadgeVariant = () => {
    switch (scoreLevel) {
      case 'excellent':
        return 'default';
      case 'good':
        return 'secondary';
      case 'moderate':
        return 'outline';
      case 'needs-improvement':
        return 'destructive';
      default:
        return 'outline';
    }
  };

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
        <div className="flex items-center space-x-2">
          <CardTitle className="text-lg font-semibold">生产力评分</CardTitle>
          {getTrendIcon()}
        </div>
        <div className="flex items-center space-x-2">
          <Badge variant={getLevelBadgeVariant()} className="text-xs">
            {scoreLevel === 'excellent' && '优秀'}
            {scoreLevel === 'good' && '良好'}
            {scoreLevel === 'moderate' && '中等'}
            {scoreLevel === 'needs-improvement' && '需改进'}
          </Badge>
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isRefreshing}>
            <RefreshCw className={`h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="grid gap-6 md:grid-cols-3">
          {/* Main Score Display */}
          <div className="flex flex-col justify-center space-y-2">
            <div className="flex items-baseline space-x-2">
              <div className={`text-5xl font-bold ${scoreColor}`}>
                {score.compositeScore.toFixed(1)}
              </div>
              <div className="text-lg text-muted-foreground">/ 100</div>
            </div>
            <div
              className={`w-fit px-3 py-1 rounded-full text-sm font-medium ${scoreBgColor} ${scoreColor}`}
            >
              {scoreLevel === 'excellent' && '表现优异'}
              {scoreLevel === 'good' && '表现良好'}
              {scoreLevel === 'moderate' && '表现中等'}
              {scoreLevel === 'needs-improvement' && '需要改进'}
            </div>
            {summary && (
              <div className="text-xs text-muted-foreground leading-relaxed">{summary}</div>
            )}
          </div>

          {/* Dimension Breakdown */}
          <div className="space-y-3">
            <h4 className="text-sm font-medium">维度评分</h4>
            <div className="space-y-2 text-xs">
              <div className="flex justify-between items-center">
                <span className="text-muted-foreground">完成率</span>
                <span className="font-semibold text-foreground">
                  {score.dimensionScores.completionRate.toFixed(1)}%
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-muted-foreground">准时率</span>
                <span className="font-semibold text-foreground">
                  {score.dimensionScores.onTimeRatio.toFixed(1)}%
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-muted-foreground">专注一致性</span>
                <span className="font-semibold text-foreground">
                  {score.dimensionScores.focusConsistency.toFixed(1)}%
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-muted-foreground">休息平衡</span>
                <span className="font-semibold text-foreground">
                  {score.dimensionScores.restBalance.toFixed(1)}%
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-muted-foreground">效率评级</span>
                <span className="font-semibold text-foreground">
                  {score.dimensionScores.efficiencyRating.toFixed(1)}%
                </span>
              </div>
            </div>
          </div>

          {/* Insights & Recommendations */}
          <div className="space-y-3">
            {insights.length > 0 && (
              <div className="space-y-2">
                <h4 className="text-sm font-medium">优势表现</h4>
                <ul className="text-xs text-muted-foreground space-y-1">
                  {insights.slice(0, 2).map((insight, index) => (
                    <li key={index} className="flex items-start">
                      <span className="mr-2 text-green-600">✓</span>
                      <span>{insight}</span>
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {recommendations.length > 0 && (
              <div className="space-y-2">
                <h4 className="text-sm font-medium">改进建议</h4>
                <ul className="text-xs text-muted-foreground space-y-1">
                  {recommendations.slice(0, 2).map((recommendation, index) => (
                    <li key={index} className="flex items-start">
                      <span className="mr-2 text-amber-600">→</span>
                      <span>{recommendation}</span>
                    </li>
                  ))}
                </ul>
              </div>
            )}

            <div className="text-xs text-muted-foreground pt-2">
              更新:{' '}
              {new Date(score.createdAt).toLocaleString('zh-CN', {
                month: '2-digit',
                day: '2-digit',
                hour: '2-digit',
                minute: '2-digit',
              })}
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
