export const DEPENDENCY_TYPES = [
  'finish_to_start',
  'start_to_start', 
  'finish_to_finish',
  'start_to_finish'
] as const;

export type DependencyType = (typeof DEPENDENCY_TYPES)[number];

export interface TaskDependency {
  id: string;
  predecessorId: string;
  successorId: string;
  dependencyType: DependencyType;
  createdAt: string;
}

export interface DependencyCreateInput {
  predecessorId: string;
  successorId: string;
  dependencyType?: DependencyType;
}

export interface DependencyValidation {
  isValid: boolean;
  errorMessage?: string;
  wouldCreateCycle: boolean;
  cyclePath?: string[];
}

export interface TaskNode {
  taskId: string;
  status: string;
  dependencies: string[];
  dependents: string[];
  isReady: boolean;
}

export interface DependencyEdge {
  id: string;
  source: string;
  target: string;
  dependencyType: DependencyType;
}

export interface DependencyGraph {
  nodes: Record<string, TaskNode>;
  edges: DependencyEdge[];
  topologicalOrder: string[];
  criticalPath: string[];
}

export interface DependencyFilter {
  taskIds?: string[];
  includeCompleted?: boolean;
  maxDepth?: number;
}

export interface ReadyTask {
  id: string;
  title: string;
  status: string;
  priority: string;
  dueAt?: string;
}

// React Flow specific types
export interface GraphNode {
  id: string;
  data: {
    task: import('./task').Task;
    isReady: boolean;
    isOnCriticalPath: boolean;
    dependencies: string[];
    dependents: string[];
  };
  position: { x: number; y: number };
  type: 'taskNode';
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  type: 'dependencyEdge';
  selected?: boolean;
  data: {
    dependency: TaskDependency;
    onDelete?: (dependencyId: string) => void;
    onUpdateType?: (dependencyId: string, newType: DependencyType) => void;
  };
}

export interface GraphLayoutOptions {
  algorithm: 'dagre' | 'elk' | 'manual';
  direction: 'TB' | 'BT' | 'LR' | 'RL';
  nodeSpacing: number;
  rankSpacing: number;
}