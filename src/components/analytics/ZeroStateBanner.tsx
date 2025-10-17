import { ArrowRight, Lightbulb, PlayCircle, RefreshCw } from 'lucide-react';
import { Link } from 'react-router-dom';
import { Button } from '../ui/button';
import { Card, CardContent } from '../ui/card';
import { Badge } from '../ui/badge';
import { type ZeroStateMeta } from '../../types/analytics';

interface ZeroStateBannerProps {
  zeroState: ZeroStateMeta;
  onCreateTasksPath: string;
  onLoadSampleData: () => Promise<unknown> | void;
  onDismiss: () => void;
  onRemindLater: () => void;
}

export function ZeroStateBanner({
  zeroState,
  onCreateTasksPath,
  onLoadSampleData,
  onDismiss,
  onRemindLater,
}: ZeroStateBannerProps) {
  return (
    <Card className="border-dashed border-primary/40 bg-primary/5">
      <CardContent className="flex flex-col gap-6 p-6 lg:flex-row lg:items-center lg:justify-between">
        <div className="space-y-3">
          <Badge className="bg-primary text-primary-foreground">智能分析准备中</Badge>
          <h2 className="text-2xl font-semibold text-primary">欢迎体验智能分析仪表盘</h2>
          <p className="text-sm text-primary/80">
            当前还没有足够的任务完成数据。先去完成规划或导入示例数据，即可解锁趋势分析、效率洞察与自动化建议。
          </p>
          <ActionList actions={zeroState.recommendedActions} />
        </div>

        <div className="flex flex-col gap-3 text-sm">
          <div className="flex flex-wrap items-center gap-2">
            <Button asChild size="sm" className="inline-flex items-center gap-2">
              <Link to={onCreateTasksPath}>
                <PlayCircle className="h-4 w-4" /> 开始创建任务
              </Link>
            </Button>
            {zeroState.sampleDataAvailable ? (
              <Button
                type="button"
                variant="secondary"
                size="sm"
                className="inline-flex items-center gap-2"
                onClick={() => void onLoadSampleData()}
              >
                <RefreshCw className="h-4 w-4" /> 装载示例数据
              </Button>
            ) : null}
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="inline-flex items-center gap-2 text-primary"
              onClick={onDismiss}
            >
              <ArrowRight className="h-4 w-4" /> 已知晓
            </Button>
          </div>
          <div className="flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
            {zeroState.missingConfiguration?.length ? (
              <span>仍需配置：{zeroState.missingConfiguration.join('、')}</span>
            ) : null}
            <button
              type="button"
              onClick={onRemindLater}
              className="inline-flex items-center gap-1 text-primary underline-offset-4 hover:underline"
            >
              稍后提醒
            </button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function ActionList({ actions }: { actions: string[] }) {
  if (!actions?.length) return null;
  return (
    <ul className="space-y-2 text-sm text-primary/80">
      {actions.map((action, index) => (
        <li key={index} className="flex items-start gap-2">
          <Lightbulb className="mt-0.5 h-4 w-4 flex-shrink-0" />
          <span>{action}</span>
        </li>
      ))}
    </ul>
  );
}
