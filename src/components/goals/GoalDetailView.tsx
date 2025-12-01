import { useState, useEffect, useCallback } from 'react';
import { ArrowLeft, Target, Calendar, TrendingUp, Plus, Trash2, CheckCircle2 } from 'lucide-react';
import { Button } from '../ui/button';
import { GoalProgressVisualization } from './GoalProgressVisualization';
import { GoalTaskList } from './GoalTaskList';
import { GoalCreationWizard } from './GoalCreationWizard';
import { useGoals } from '../../hooks/useGoals';
import { GoalWithProgress, GoalStatus } from '../../types/goal';

interface GoalDetailViewProps {
  goalId: string;
  onClose: () => void;
}

export function GoalDetailView({ goalId, onClose }: GoalDetailViewProps) {
  const { getGoalWithProgress, deleteGoal, updateGoal } = useGoals();
  const [goal, setGoal] = useState<GoalWithProgress | null>(null);
  const [loading, setLoading] = useState(true);
  const [showSubGoalWizard, setShowSubGoalWizard] = useState(false);

  const loadGoal = useCallback(async () => {
    try {
      setLoading(true);
      const data = await getGoalWithProgress(goalId);
      setGoal(data);
    } catch (error) {
      console.error('加载目标失败:', error);
    } finally {
      setLoading(false);
    }
  }, [goalId, getGoalWithProgress]);

  useEffect(() => {
    loadGoal();
  }, [loadGoal]);

  const handleDelete = async () => {
    if (confirm('确定要删除这个目标吗？这也将删除所有子目标。')) {
      try {
        await deleteGoal(goalId);
        onClose();
      } catch (error) {
        console.error('删除目标失败:', error);
      }
    }
  };

  const handleStatusChange = async (newStatus: GoalStatus) => {
    try {
      await updateGoal(goalId, { status: newStatus });
      await loadGoal();
    } catch (error) {
      console.error('更新目标状态失败:', error);
    }
  };

  const formatDate = (dateStr?: string) => {
    if (!dateStr) return '未设置目标日期';
    const date = new Date(dateStr);
    return date.toLocaleDateString();
  };

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-muted-foreground">加载中...</div>
      </div>
    );
  }

  if (!goal) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-muted-foreground">目标未找到</div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={onClose}>
            <ArrowLeft className="w-5 h-5" />
          </Button>
          <div>
            <h1 className="text-3xl font-bold flex items-center gap-2">
              <Target className="w-8 h-8" />
              {goal.title}
            </h1>
            {goal.description && <p className="text-muted-foreground mt-1">{goal.description}</p>}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={handleDelete}>
            <Trash2 className="w-4 h-4 mr-2" />
            删除
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <div className="p-4 border rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <TrendingUp className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm text-muted-foreground">进度</span>
          </div>
          <div className="text-2xl font-bold">{Math.round(goal.progressPercentage)}%</div>
          <div className="text-xs text-muted-foreground mt-1">
            {goal.completedTasks} / {goal.totalTasks} 任务已完成
          </div>
        </div>

        <div className="p-4 border rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <Calendar className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm text-muted-foreground">目标日期</span>
          </div>
          <div className="text-lg font-semibold">{formatDate(goal.targetDate)}</div>
          {goal.daysUntilTarget !== undefined && (
            <div className="text-xs text-muted-foreground mt-1">
              {goal.daysUntilTarget > 0
                ? `还有 ${goal.daysUntilTarget} 天`
                : goal.daysUntilTarget === 0
                  ? '今天到期'
                  : `已逾期 ${Math.abs(goal.daysUntilTarget)} 天`}
            </div>
          )}
        </div>

        <div className="p-4 border rounded-lg">
          <div className="flex items-center gap-2 mb-2">
            <CheckCircle2 className="w-4 h-4 text-muted-foreground" />
            <span className="text-sm text-muted-foreground">状态</span>
          </div>
          <select
            value={goal.status}
            onChange={(e) => handleStatusChange(e.target.value as GoalStatus)}
            className="w-full p-2 border rounded text-sm"
          >
            <option value="not_started">未开始</option>
            <option value="in_progress">进行中</option>
            <option value="completed">已完成</option>
            <option value="on_hold">暂停</option>
            <option value="cancelled">已取消</option>
          </select>
          {!goal.isOnTrack && goal.status !== 'completed' && (
            <div className="text-xs text-yellow-600 mt-1">进度滞后</div>
          )}
        </div>
      </div>

      <GoalProgressVisualization goal={goal} />

      <div className="mt-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold">任务</h2>
          <Button variant="outline" size="sm" onClick={() => setShowSubGoalWizard(true)}>
            <Plus className="w-4 h-4 mr-2" />
            添加子目标
          </Button>
        </div>
        <GoalTaskList goalId={goalId} onTasksChange={loadGoal} />
      </div>

      {goal.childGoals.length > 0 && (
        <div className="mt-6">
          <h2 className="text-xl font-semibold mb-4">子目标</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {goal.childGoals.map((childGoal) => (
              <div key={childGoal.id} className="p-4 border rounded-lg">
                <h3 className="font-semibold mb-2">{childGoal.title}</h3>
                <div className="w-full bg-secondary rounded-full h-2 mb-2">
                  <div
                    className={`h-2 rounded-full ${
                      childGoal.isOnTrack ? 'bg-green-500' : 'bg-yellow-500'
                    }`}
                    style={{ width: `${childGoal.progressPercentage}%` }}
                  />
                </div>
                <div className="text-xs text-muted-foreground">
                  {childGoal.completedTasks}/{childGoal.totalTasks} 任务 •{' '}
                  {Math.round(childGoal.progressPercentage)}%
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      <GoalCreationWizard
        open={showSubGoalWizard}
        onClose={() => setShowSubGoalWizard(false)}
        onGoalCreated={() => {
          setShowSubGoalWizard(false);
          loadGoal();
        }}
        parentGoalId={goalId}
      />
    </div>
  );
}
