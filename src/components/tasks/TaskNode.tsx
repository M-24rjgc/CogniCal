import React from 'react';
import { Handle, Position, NodeProps } from '@xyflow/react';
import { Badge } from '../ui/badge';
import { Card } from '../ui/card';
import { cn } from '../../lib/utils';
import { Task, TaskStatus, TaskPriority } from '../../types/task';
import { 
  CheckCircle2, 
  Circle, 
  Clock, 
  AlertCircle, 
  Pause,
  Archive,
  Zap
} from 'lucide-react';

interface TaskNodeData {
  task: Task;
  isReady: boolean;
  isOnCriticalPath: boolean;
  isBlocked?: boolean;
  dependencies: string[];
  dependents: string[];
}

const getStatusIcon = (status: TaskStatus) => {
  switch (status) {
    case 'done':
      return <CheckCircle2 className="h-4 w-4 text-green-600" />;
    case 'in_progress':
      return <Clock className="h-4 w-4 text-blue-600" />;
    case 'blocked':
      return <AlertCircle className="h-4 w-4 text-red-600" />;
    case 'backlog':
      return <Pause className="h-4 w-4 text-gray-500" />;
    case 'archived':
      return <Archive className="h-4 w-4 text-gray-400" />;
    default:
      return <Circle className="h-4 w-4 text-gray-400" />;
  }
};

const getStatusColor = (status: TaskStatus) => {
  switch (status) {
    case 'done':
      return 'bg-green-50 border-green-200';
    case 'in_progress':
      return 'bg-blue-50 border-blue-200';
    case 'blocked':
      return 'bg-red-50 border-red-200';
    case 'backlog':
      return 'bg-gray-50 border-gray-200';
    case 'archived':
      return 'bg-gray-50 border-gray-200';
    default:
      return 'bg-white border-gray-200';
  }
};

const getPriorityColor = (priority: TaskPriority) => {
  switch (priority) {
    case 'urgent':
      return 'bg-red-100 text-red-800 border-red-200';
    case 'high':
      return 'bg-orange-100 text-orange-800 border-orange-200';
    case 'medium':
      return 'bg-yellow-100 text-yellow-800 border-yellow-200';
    case 'low':
      return 'bg-green-100 text-green-800 border-green-200';
    default:
      return 'bg-gray-100 text-gray-800 border-gray-200';
  }
};

export const TaskNode: React.FC<NodeProps> = ({ 
  data, 
  selected, 
  dragging 
}) => {
  const nodeData = data as unknown as TaskNodeData;
  const { task, isReady, isOnCriticalPath, isBlocked, dependencies, dependents } = nodeData;
  
  return (
    <>
      {/* Input handle for dependencies */}
      <Handle
        type="target"
        position={Position.Top}
        className={cn(
          "w-3 h-3 border-2 border-white transition-all duration-200",
          "!bg-gray-400 hover:!bg-blue-500 hover:scale-125"
        )}
        isConnectable={true}
      />
      
      <Card 
        className={cn(
          'min-w-[200px] max-w-[280px] p-3 cursor-pointer transition-all duration-200',
          getStatusColor(task.status),
          selected && 'ring-2 ring-blue-500 ring-offset-2',
          isOnCriticalPath && 'ring-2 ring-red-500 ring-offset-1 shadow-lg shadow-red-200',
          isReady && task.status !== 'done' && 'ring-2 ring-green-400 ring-offset-1 shadow-lg shadow-green-200',
          isBlocked && 'opacity-70 ring-2 ring-orange-400 ring-offset-1',
          dragging && 'rotate-2 scale-105 shadow-xl'
        )}
      >
        {/* Header with status and priority */}
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            {getStatusIcon(task.status)}
            {isOnCriticalPath && (
              <Zap className="h-3 w-3 text-red-500" />
            )}
            {isReady && task.status !== 'done' && (
              <CheckCircle2 className="h-3 w-3 text-green-500" />
            )}
            {isBlocked && (
              <AlertCircle className="h-3 w-3 text-orange-500" />
            )}
          </div>
          <Badge 
            variant="outline" 
            className={cn('text-xs', getPriorityColor(task.priority))}
          >
            {task.priority}
          </Badge>
        </div>

        {/* Task title */}
        <h4 className="font-medium text-sm text-gray-900 mb-1 line-clamp-2">
          {task.title}
        </h4>

        {/* Task description (if exists) */}
        {task.description && (
          <p className="text-xs text-gray-600 mb-2 line-clamp-2">
            {task.description}
          </p>
        )}

        {/* Tags */}
        {task.tags.length > 0 && (
          <div className="flex flex-wrap gap-1 mb-2">
            {task.tags.slice(0, 3).map((tag: string) => (
              <Badge key={tag} variant="secondary" className="text-xs px-1 py-0">
                {tag}
              </Badge>
            ))}
            {task.tags.length > 3 && (
              <Badge variant="secondary" className="text-xs px-1 py-0">
                +{task.tags.length - 3}
              </Badge>
            )}
          </div>
        )}

        {/* Due date */}
        {task.dueAt && (
          <div className="text-xs text-gray-500">
            Due: {new Date(task.dueAt).toLocaleDateString()}
          </div>
        )}

        {/* Dependency info */}
        <div className="flex justify-between text-xs text-gray-500 mt-2 pt-2 border-t border-gray-200">
          <span>{dependencies.length} deps</span>
          <span>{dependents.length} blocks</span>
        </div>

        {/* Status indicators */}
        {isOnCriticalPath && (
          <div className="absolute -top-1 -left-1 w-3 h-3 bg-red-500 rounded-full border-2 border-white" 
               title="关键路径" />
        )}
        {isReady && task.status !== 'done' && !isOnCriticalPath && (
          <div className="absolute -top-1 -right-1 w-3 h-3 bg-green-500 rounded-full border-2 border-white" 
               title="可执行任务" />
        )}
        {isBlocked && (
          <div className="absolute -top-1 -right-1 w-3 h-3 bg-orange-500 rounded-full border-2 border-white" 
               title="受依赖阻塞" />
        )}
      </Card>

      {/* Output handle for dependents */}
      <Handle
        type="source"
        position={Position.Bottom}
        className={cn(
          "w-3 h-3 border-2 border-white transition-all duration-200",
          "!bg-gray-400 hover:!bg-blue-500 hover:scale-125"
        )}
        isConnectable={true}
      />
    </>
  );
};