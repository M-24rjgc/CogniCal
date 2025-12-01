import { useState } from 'react';
import { Plus, Target } from 'lucide-react';
import { Button } from '../components/ui/button';
import { GoalCreationWizard } from '../components/goals/GoalCreationWizard';
import { GoalCard } from '../components/goals/GoalCard';
import { GoalDetailView } from '../components/goals/GoalDetailView';
import { useGoals } from '../hooks/useGoals';
import { Goal } from '../types/goal';

export default function Goals() {
  const { goals, loading, refetch } = useGoals();
  const [showWizard, setShowWizard] = useState(false);
  const [selectedGoal, setSelectedGoal] = useState<Goal | null>(null);

  const handleGoalCreated = () => {
    refetch();
  };

  const handleGoalClick = (goal: Goal) => {
    setSelectedGoal(goal);
  };

  const handleCloseDetail = () => {
    setSelectedGoal(null);
    refetch();
  };

  if (selectedGoal) {
    return (
      <GoalDetailView
        goalId={selectedGoal.id}
        onClose={handleCloseDetail}
      />
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold flex items-center gap-2">
            <Target className="w-8 h-8" />
            目标
          </h1>
          <p className="text-muted-foreground mt-1">
            将大目标分解为可管理的任务，并建立清晰的依赖关系
          </p>
        </div>
        <Button onClick={() => setShowWizard(true)}>
          <Plus className="w-4 h-4 mr-2" />
          新建目标
        </Button>
      </div>

      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="text-muted-foreground">加载中...</div>
        </div>
      ) : goals.length === 0 ? (
        <div className="flex flex-col items-center justify-center h-64 border-2 border-dashed rounded-lg">
          <Target className="w-16 h-16 text-muted-foreground mb-4" />
          <h3 className="text-lg font-semibold mb-2">暂无目标</h3>
          <p className="text-muted-foreground mb-4">
            创建您的第一个目标，开始分解复杂项目
          </p>
          <Button onClick={() => setShowWizard(true)}>
            <Plus className="w-4 h-4 mr-2" />
            创建目标
          </Button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {goals.map((goal) => (
            <GoalCard
              key={goal.id}
              goal={goal}
              onClick={() => handleGoalClick(goal)}
            />
          ))}
        </div>
      )}

      <GoalCreationWizard
        open={showWizard}
        onClose={() => setShowWizard(false)}
        onGoalCreated={handleGoalCreated}
      />
    </div>
  );
}
