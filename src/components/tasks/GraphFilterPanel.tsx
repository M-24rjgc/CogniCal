import React, { useState, useCallback } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Badge } from '../ui/badge';
import { 
  Search, 
  Filter, 
  X, 
  CheckCircle2, 
  AlertTriangle,
  Target
} from 'lucide-react';
import { Task, TaskStatus, TaskPriority } from '../../types/task';
import { cn } from '../../lib/utils';

interface GraphFilterOptions {
  search?: string;
  statuses?: TaskStatus[];
  priorities?: TaskPriority[];
  tags?: string[];
  showCompleted?: boolean;
  showBlocked?: boolean;
  showReady?: boolean;
  showCriticalPath?: boolean;
  dateRange?: {
    start?: Date;
    end?: Date;
  };
  complexityRange?: {
    min?: number;
    max?: number;
  };
}

interface GraphFilterPanelProps {
  tasks: Task[];
  filters: GraphFilterOptions;
  onFiltersChange: (filters: GraphFilterOptions) => void;
  onReset: () => void;
  className?: string;
}

const STATUS_LABELS: Record<TaskStatus, string> = {
  backlog: '待整理',
  todo: '待开始',
  in_progress: '进行中',
  blocked: '受阻',
  done: '已完成',
  archived: '已归档',
};

const PRIORITY_LABELS: Record<TaskPriority, string> = {
  low: '低',
  medium: '中',
  high: '高',
  urgent: '紧急',
};

export const GraphFilterPanel: React.FC<GraphFilterPanelProps> = ({
  tasks,
  filters,
  onFiltersChange,
  onReset,
  className,
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [searchTerm, setSearchTerm] = useState(filters.search || '');

  // Extract unique tags from tasks
  const availableTags = React.useMemo(() => {
    const tagSet = new Set<string>();
    tasks.forEach(task => {
      task.tags?.forEach(tag => tagSet.add(tag));
    });
    return Array.from(tagSet).sort();
  }, [tasks]);

  const handleSearchChange = useCallback((value: string) => {
    setSearchTerm(value);
    onFiltersChange({
      ...filters,
      search: value.trim() || undefined,
    });
  }, [filters, onFiltersChange]);

  const handleStatusToggle = useCallback((status: TaskStatus) => {
    const currentStatuses = filters.statuses || [];
    const newStatuses = currentStatuses.includes(status)
      ? currentStatuses.filter(s => s !== status)
      : [...currentStatuses, status];
    
    onFiltersChange({
      ...filters,
      statuses: newStatuses.length > 0 ? newStatuses : undefined,
    });
  }, [filters, onFiltersChange]);

  const handlePriorityToggle = useCallback((priority: TaskPriority) => {
    const currentPriorities = filters.priorities || [];
    const newPriorities = currentPriorities.includes(priority)
      ? currentPriorities.filter(p => p !== priority)
      : [...currentPriorities, priority];
    
    onFiltersChange({
      ...filters,
      priorities: newPriorities.length > 0 ? newPriorities : undefined,
    });
  }, [filters, onFiltersChange]);

  const handleTagToggle = useCallback((tag: string) => {
    const currentTags = filters.tags || [];
    const newTags = currentTags.includes(tag)
      ? currentTags.filter(t => t !== tag)
      : [...currentTags, tag];
    
    onFiltersChange({
      ...filters,
      tags: newTags.length > 0 ? newTags : undefined,
    });
  }, [filters, onFiltersChange]);

  const handleQuickFilterToggle = useCallback((filterType: keyof GraphFilterOptions) => {
    onFiltersChange({
      ...filters,
      [filterType]: !filters[filterType],
    });
  }, [filters, onFiltersChange]);

  const handleComplexityRangeChange = useCallback((type: 'min' | 'max', value: string) => {
    const numValue = value ? Number(value) : undefined;
    const currentRange = filters.complexityRange || {};
    
    onFiltersChange({
      ...filters,
      complexityRange: {
        ...currentRange,
        [type]: numValue,
      },
    });
  }, [filters, onFiltersChange]);

  // Count active filters
  const activeFilterCount = React.useMemo(() => {
    let count = 0;
    if (filters.search) count++;
    if (filters.statuses?.length) count++;
    if (filters.priorities?.length) count++;
    if (filters.tags?.length) count++;
    if (filters.showCompleted) count++;
    if (filters.showBlocked) count++;
    if (filters.showReady) count++;
    if (filters.showCriticalPath) count++;
    if (filters.complexityRange?.min !== undefined || filters.complexityRange?.max !== undefined) count++;
    return count;
  }, [filters]);

  return (
    <Card className={cn('border-l-4 border-l-blue-500', className)}>
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <div className="flex items-center gap-2">
          <Filter className="h-4 w-4 text-blue-600" />
          <span className="font-medium">图表筛选</span>
          {activeFilterCount > 0 && (
            <Badge variant="secondary" className="text-xs">
              {activeFilterCount} 个筛选条件
            </Badge>
          )}
        </div>
        
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="ghost"
            onClick={() => setIsExpanded(!isExpanded)}
            className="h-8"
          >
            {isExpanded ? '收起' : '展开'}
          </Button>
          
          {activeFilterCount > 0 && (
            <Button
              size="sm"
              variant="outline"
              onClick={onReset}
              className="h-8"
            >
              <X className="h-3 w-3 mr-1" />
              重置
            </Button>
          )}
        </div>
      </div>

      {/* Search */}
      <div className="p-4 border-b">
        <div className="flex items-center gap-2">
          <Search className="h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="搜索任务标题或描述..."
            value={searchTerm}
            onChange={(e) => handleSearchChange(e.target.value)}
            className="flex-1"
          />
        </div>
      </div>

      {/* Quick Filters */}
      <div className="p-4 border-b">
        <Label className="text-sm font-medium mb-2 block">快速筛选</Label>
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant={filters.showReady ? "default" : "outline"}
            onClick={() => handleQuickFilterToggle('showReady')}
            className="h-8"
          >
            <CheckCircle2 className="h-3 w-3 mr-1 text-green-600" />
            可执行任务
          </Button>
          
          <Button
            size="sm"
            variant={filters.showBlocked ? "default" : "outline"}
            onClick={() => handleQuickFilterToggle('showBlocked')}
            className="h-8"
          >
            <AlertTriangle className="h-3 w-3 mr-1 text-orange-600" />
            受阻任务
          </Button>
          
          <Button
            size="sm"
            variant={filters.showCompleted ? "default" : "outline"}
            onClick={() => handleQuickFilterToggle('showCompleted')}
            className="h-8"
          >
            <CheckCircle2 className="h-3 w-3 mr-1 text-blue-600" />
            包含已完成
          </Button>
          
          <Button
            size="sm"
            variant={filters.showCriticalPath ? "default" : "outline"}
            onClick={() => handleQuickFilterToggle('showCriticalPath')}
            className="h-8"
          >
            <Target className="h-3 w-3 mr-1 text-red-600" />
            关键路径
          </Button>
        </div>
      </div>

      {/* Expanded Filters */}
      {isExpanded && (
        <div className="p-4 space-y-4">
          {/* Status Filter */}
          <div>
            <Label className="text-sm font-medium mb-2 block">任务状态</Label>
            <div className="flex flex-wrap gap-2">
              {Object.entries(STATUS_LABELS).map(([status, label]) => (
                <Button
                  key={status}
                  size="sm"
                  variant={filters.statuses?.includes(status as TaskStatus) ? "default" : "outline"}
                  onClick={() => handleStatusToggle(status as TaskStatus)}
                  className="h-8 text-xs"
                >
                  {label}
                </Button>
              ))}
            </div>
          </div>

          {/* Priority Filter */}
          <div>
            <Label className="text-sm font-medium mb-2 block">优先级</Label>
            <div className="flex flex-wrap gap-2">
              {Object.entries(PRIORITY_LABELS).map(([priority, label]) => (
                <Button
                  key={priority}
                  size="sm"
                  variant={filters.priorities?.includes(priority as TaskPriority) ? "default" : "outline"}
                  onClick={() => handlePriorityToggle(priority as TaskPriority)}
                  className="h-8 text-xs"
                >
                  {label}
                </Button>
              ))}
            </div>
          </div>

          {/* Tags Filter */}
          {availableTags.length > 0 && (
            <div>
              <Label className="text-sm font-medium mb-2 block">标签</Label>
              <div className="flex flex-wrap gap-2 max-h-24 overflow-y-auto">
                {availableTags.map((tag) => (
                  <Button
                    key={tag}
                    size="sm"
                    variant={filters.tags?.includes(tag) ? "default" : "outline"}
                    onClick={() => handleTagToggle(tag)}
                    className="h-8 text-xs"
                  >
                    #{tag}
                  </Button>
                ))}
              </div>
            </div>
          )}

          {/* Complexity Range */}
          <div>
            <Label className="text-sm font-medium mb-2 block">复杂度范围</Label>
            <div className="flex items-center gap-2">
              <Input
                type="number"
                min={0}
                max={10}
                placeholder="最小值"
                value={filters.complexityRange?.min || ''}
                onChange={(e) => handleComplexityRangeChange('min', e.target.value)}
                className="w-20 h-8"
              />
              <span className="text-muted-foreground">-</span>
              <Input
                type="number"
                min={0}
                max={10}
                placeholder="最大值"
                value={filters.complexityRange?.max || ''}
                onChange={(e) => handleComplexityRangeChange('max', e.target.value)}
                className="w-20 h-8"
              />
            </div>
          </div>
        </div>
      )}
    </Card>
  );
};