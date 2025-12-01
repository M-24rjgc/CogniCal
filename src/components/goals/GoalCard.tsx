import React from 'react';
import { Goal } from '../../types/goal';
import { Target, Calendar, AlertCircle } from 'lucide-react';
import { useGoals } from '../../hooks/useGoals';
import type { GoalWithProgress } from '../../types/goal';

interface GoalCardProps {
  goal: Goal;
  onClick: () => void;
}

export function GoalCard({ goal, onClick }: GoalCardProps) {
  const { getGoalWithProgress } = useGoals();
  const [progress, setProgress] = React.useState<GoalWithProgress | null>(null);

  React.useEffect(() => {
    const fetchProgress = async () => {
      try {
        const data = await getGoalWithProgress(goal.id);
        setProgress(data);
      } catch (error) {
        console.error('加载进度失败:', error);
      }
    };

    fetchProgress();
  }, [goal.id, getGoalWithProgress]);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'bg-green-500';
      case 'in_progress':
        return 'bg-blue-500';
      case 'on_hold':
        return 'bg-yellow-500';
      case 'cancelled':
        return 'bg-red-500';
      default:
        return 'bg-gray-500';
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'high':
        return 'text-red-600';
      case 'medium':
        return 'text-yellow-600';
      case 'low':
        return 'text-green-600';
      default:
        return 'text-gray-600';
    }
  };

  const formatDate = (dateStr?: string) => {
    if (!dateStr) return null;
    const date = new Date(dateStr);
    return date.toLocaleDateString();
  };

  return (
    <button
      onClick={onClick}
      className="w-full p-4 border rounded-lg hover:border-primary hover:shadow-md transition-all cursor-pointer bg-card text-left"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onClick();
        }
      }}
    >
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-2">
          <Target className="w-5 h-5 text-primary" />
          <h3 className="font-semibold line-clamp-1">{goal.title}</h3>
        </div>
        <div className={`w-2 h-2 rounded-full ${getStatusColor(goal.status)}`} />
      </div>

      {goal.description && (
        <p className="text-sm text-muted-foreground mb-3 line-clamp-2">{goal.description}</p>
      )}

      {progress && (
        <div className="mb-3">
          <div className="flex items-center justify-between text-xs mb-1">
            <span className="text-muted-foreground">进度</span>
            <span className="font-medium">{Math.round(progress.progressPercentage)}%</span>
          </div>
          <div className="w-full bg-secondary rounded-full h-2">
            <div
              className={`h-2 rounded-full transition-all ${
                progress.isOnTrack ? 'bg-green-500' : 'bg-yellow-500'
              }`}
              style={{ width: `${progress.progressPercentage}%` }}
            />
          </div>
          <div className="flex items-center gap-3 mt-2 text-xs text-muted-foreground">
            <span>
              {progress.completedTasks}/{progress.totalTasks} 任务
            </span>
            {progress.childGoals.length > 0 && <span>{progress.childGoals.length} 子目标</span>}
          </div>
        </div>
      )}

      <div className="flex items-center justify-between text-xs">
        <span className={`font-medium ${getPriorityColor(goal.priority)}`}>
          {goal.priority.toUpperCase()}
        </span>
        {goal.targetDate && (
          <div className="flex items-center gap-1 text-muted-foreground">
            <Calendar className="w-3 h-3" />
            <span>{formatDate(goal.targetDate)}</span>
          </div>
        )}
      </div>

      {progress && !progress.isOnTrack && progress.daysUntilTarget !== undefined && (
        <div className="mt-2 flex items-center gap-1 text-xs text-yellow-600">
          <AlertCircle className="w-3 h-3" />
          <span>进度滞后</span>
        </div>
      )}
    </button>
  );
}
