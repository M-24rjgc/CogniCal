export interface Goal {
  id: string;
  title: string;
  description?: string;
  parentGoalId?: string;
  status: GoalStatus;
  priority: string;
  targetDate?: string;
  createdAt: string;
  updatedAt: string;
}

export type GoalStatus = 'not_started' | 'in_progress' | 'completed' | 'on_hold' | 'cancelled';

export interface GoalWithProgress extends Goal {
  progressPercentage: number;
  totalTasks: number;
  completedTasks: number;
  inProgressTasks: number;
  blockedTasks: number;
  childGoals: GoalWithProgress[];
  isOnTrack: boolean;
  daysUntilTarget?: number;
}

export interface CreateGoalRequest {
  title: string;
  description?: string;
  parentGoalId?: string;
  priority: string;
  targetDate?: string;
}

export interface UpdateGoalRequest {
  title?: string;
  description?: string;
  status?: GoalStatus;
  priority?: string;
  targetDate?: string;
}

export interface GoalTemplate {
  id: string;
  name: string;
  description: string;
  type: 'sequential' | 'parallel' | 'milestone-based';
  taskStructure: TaskTemplate[];
}

export interface TaskTemplate {
  title: string;
  description?: string;
  dependencies?: number[]; // Indices of tasks this depends on
  estimatedMinutes?: number;
}
