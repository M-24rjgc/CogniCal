import React, { useCallback, useEffect } from 'react';
import { ReactFlowProvider } from '@xyflow/react';
import { DependencyGraph } from './DependencyGraph';
import { useDependencies } from '../../hooks/useDependencies';
import { Task } from '../../types/task';
import { DependencyFilter } from '../../types/dependency';

interface DependencyGraphContainerProps {
  tasks: Task[];
  onTaskClick?: (taskId: string) => void;
  className?: string;
}

export const DependencyGraphContainer: React.FC<DependencyGraphContainerProps> = ({
  tasks,
  onTaskClick,
  className,
}) => {
  const { dependencies, isLoading, fetchDependencies } = useDependencies();

  // Fetch dependencies when component mounts or tasks change
  useEffect(() => {
    const filter: DependencyFilter = {
      taskIds: tasks.map((task) => task.id),
      includeCompleted: true,
    };
    fetchDependencies(filter);
  }, [tasks, fetchDependencies]);

  const handleAddDependency = useCallback(async (from: string, to: string) => {
    // This is handled by the DependencyGraph component's onConnect
    console.log('Dependency added:', from, '->', to);
  }, []);

  const handleRemoveDependency = useCallback(async (dependencyId: string) => {
    // This is handled by the DependencyGraph component's delete handler
    console.log('Dependency removed:', dependencyId);
  }, []);

  if (isLoading && dependencies.length === 0) {
    return (
      <div className="flex h-[600px] items-center justify-center">
        <div className="text-center">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
          <p className="mt-2 text-sm text-muted-foreground">加载依赖图...</p>
        </div>
      </div>
    );
  }

  return (
    <ReactFlowProvider>
      <DependencyGraph
        tasks={tasks}
        dependencies={dependencies}
        onAddDependency={handleAddDependency}
        onRemoveDependency={handleRemoveDependency}
        onTaskClick={onTaskClick}
        className={className}
      />
    </ReactFlowProvider>
  );
};
