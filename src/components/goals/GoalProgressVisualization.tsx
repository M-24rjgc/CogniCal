import { GoalWithProgress } from '../../types/goal';

interface GoalProgressVisualizationProps {
  goal: GoalWithProgress;
}

export function GoalProgressVisualization({ goal }: GoalProgressVisualizationProps) {
  return (
    <div className="border rounded-lg p-6">
      <h3 className="text-lg font-semibold mb-4">进度概览</h3>

      <div className="space-y-6">
        {/* Overall Progress Bar */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium">整体进度</span>
            <span className="text-sm font-bold">{Math.round(goal.progressPercentage)}%</span>
          </div>
          <div className="w-full bg-secondary rounded-full h-4">
            <div
              className={`h-4 rounded-full transition-all ${
                goal.isOnTrack ? 'bg-green-500' : 'bg-yellow-500'
              }`}
              style={{ width: `${goal.progressPercentage}%` }}
            />
          </div>
        </div>

        {/* Task Breakdown */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="text-center p-3 bg-accent rounded-lg">
            <div className="text-2xl font-bold text-gray-600">{goal.totalTasks}</div>
            <div className="text-xs text-muted-foreground">总任务数</div>
          </div>
          <div className="text-center p-3 bg-green-50 rounded-lg">
            <div className="text-2xl font-bold text-green-600">{goal.completedTasks}</div>
            <div className="text-xs text-muted-foreground">已完成</div>
          </div>
          <div className="text-center p-3 bg-blue-50 rounded-lg">
            <div className="text-2xl font-bold text-blue-600">{goal.inProgressTasks}</div>
            <div className="text-xs text-muted-foreground">进行中</div>
          </div>
          <div className="text-center p-3 bg-red-50 rounded-lg">
            <div className="text-2xl font-bold text-red-600">{goal.blockedTasks}</div>
            <div className="text-xs text-muted-foreground">受阻</div>
          </div>
        </div>

        {/* Timeline Indicator */}
        {goal.targetDate && (
          <div className="pt-4 border-t">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium">时间线</span>
              <span
                className={`text-sm font-medium ${
                  goal.isOnTrack ? 'text-green-600' : 'text-yellow-600'
                }`}
              >
                {goal.isOnTrack ? '按时进行' : '进度滞后'}
              </span>
            </div>
            <div className="relative">
              <div className="w-full bg-secondary rounded-full h-2">
                <div
                  className="h-2 bg-primary rounded-full"
                  style={{
                    width: `${Math.min(
                      100,
                      ((new Date().getTime() - new Date(goal.createdAt).getTime()) /
                        (new Date(goal.targetDate).getTime() -
                          new Date(goal.createdAt).getTime())) *
                        100,
                    )}%`,
                  }}
                />
              </div>
              <div className="flex justify-between text-xs text-muted-foreground mt-1">
                <span>开始</span>
                <span>今天</span>
                <span>目标</span>
              </div>
            </div>
          </div>
        )}

        {/* Health Indicators */}
        <div className="pt-4 border-t">
          <h4 className="text-sm font-medium mb-3">健康指标</h4>
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">进度速率</span>
              <span
                className={`text-sm font-medium ${
                  goal.isOnTrack ? 'text-green-600' : 'text-yellow-600'
                }`}
              >
                {goal.isOnTrack ? '良好' : '需要关注'}
              </span>
            </div>
            {goal.blockedTasks > 0 && (
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">受阻任务</span>
                <span className="text-sm font-medium text-red-600">
                  {goal.blockedTasks} 个任务受阻
                </span>
              </div>
            )}
            {goal.childGoals.length > 0 && (
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">子目标</span>
                <span className="text-sm font-medium">
                  {goal.childGoals.filter((g) => g.status === 'completed').length}/
                  {goal.childGoals.length} 已完成
                </span>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
