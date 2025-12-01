import { useState, useMemo } from 'react';
import { format, parseISO } from 'date-fns';
import { zhCN } from 'date-fns/locale';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { Card, CardContent } from '../ui/card';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../ui/table';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu';
import {
  type TaskInstance,
  type RecurringTaskTemplate,
  type TaskStatus,
  TASK_STATUSES,
} from '../../types/task';

interface TaskInstanceManagerProps {
  open: boolean;
  template: RecurringTaskTemplate | undefined;
  instances: TaskInstance[];
  onOpenChange: (open: boolean) => void;
  onEditInstance: (instance: TaskInstance, editType: 'single' | 'series') => void;
  onDeleteInstance: (instanceId: string, deleteType: 'single' | 'series') => void;
  onBulkStatusUpdate: (instanceIds: string[], status: TaskStatus) => void;
  onBulkDelete: (instanceIds: string[]) => void;
  isLoading?: boolean;
}

export function TaskInstanceManager({
  open,
  template,
  instances,
  onOpenChange,
  onEditInstance,
  onDeleteInstance,
  onBulkStatusUpdate,
  onBulkDelete,
}: TaskInstanceManagerProps) {
  const [selectedInstances, setSelectedInstances] = useState<Set<string>>(new Set());
  const [filterStatus, setFilterStatus] = useState<TaskStatus | 'all'>('all');
  const [showExceptionsOnly, setShowExceptionsOnly] = useState(false);

  // Filter instances based on current filters
  const filteredInstances = useMemo(() => {
    return instances.filter((instance) => {
      if (filterStatus !== 'all' && instance.status !== filterStatus) {
        return false;
      }
      if (showExceptionsOnly && !instance.isException) {
        return false;
      }
      return true;
    });
  }, [instances, filterStatus, showExceptionsOnly]);

  // Group instances by status for summary
  const statusSummary = useMemo(() => {
    const summary = instances.reduce(
      (acc, instance) => {
        acc[instance.status] = (acc[instance.status] || 0) + 1;
        return acc;
      },
      {} as Record<TaskStatus, number>,
    );

    return summary;
  }, [instances]);

  const handleSelectAll = () => {
    if (selectedInstances.size === filteredInstances.length) {
      setSelectedInstances(new Set());
    } else {
      setSelectedInstances(new Set(filteredInstances.map((i) => i.id)));
    }
  };

  const handleSelectInstance = (instanceId: string) => {
    const newSelected = new Set(selectedInstances);
    if (newSelected.has(instanceId)) {
      newSelected.delete(instanceId);
    } else {
      newSelected.add(instanceId);
    }
    setSelectedInstances(newSelected);
  };

  const handleBulkAction = (action: 'status' | 'delete', value?: TaskStatus) => {
    const selectedIds = Array.from(selectedInstances);
    if (selectedIds.length === 0) return;

    if (action === 'status' && value) {
      onBulkStatusUpdate(selectedIds, value);
    } else if (action === 'delete') {
      onBulkDelete(selectedIds);
    }

    setSelectedInstances(new Set());
  };

  if (!template) return null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-6xl max-h-[90vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle>管理重复任务实例</DialogTitle>
          <DialogDescription>
            管理「{template.title}」的所有任务实例，可以单独编辑或批量操作。
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-hidden flex flex-col space-y-4">
          {/* Summary Cards */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold">{instances.length}</div>
                <div className="text-sm text-muted-foreground">总实例数</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-green-600">{statusSummary.done || 0}</div>
                <div className="text-sm text-muted-foreground">已完成</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-blue-600">
                  {statusSummary.in_progress || 0}
                </div>
                <div className="text-sm text-muted-foreground">进行中</div>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-4">
                <div className="text-2xl font-bold text-orange-600">
                  {instances.filter((i) => i.isException).length}
                </div>
                <div className="text-sm text-muted-foreground">例外实例</div>
              </CardContent>
            </Card>
          </div>

          {/* Filters and Bulk Actions */}
          <div className="flex flex-wrap items-center gap-4 p-4 bg-muted/50 rounded-lg">
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium">筛选:</span>
              <select
                className="h-8 rounded border border-border/60 bg-background px-2 text-sm"
                value={filterStatus}
                onChange={(e) => setFilterStatus(e.target.value as TaskStatus | 'all')}
              >
                <option value="all">全部状态</option>
                {TASK_STATUSES.map((status) => (
                  <option key={status} value={status}>
                    {STATUS_LABELS[status]}
                  </option>
                ))}
              </select>
            </div>

            <Button
              variant={showExceptionsOnly ? 'default' : 'outline'}
              size="sm"
              onClick={() => setShowExceptionsOnly(!showExceptionsOnly)}
            >
              仅显示例外
            </Button>

            {selectedInstances.size > 0 && (
              <>
                <div className="h-4 w-px bg-border" />
                <span className="text-sm text-muted-foreground">
                  已选择 {selectedInstances.size} 个实例
                </span>

                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button variant="outline" size="sm">
                      批量操作
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent>
                    <DropdownMenuItem onClick={() => handleBulkAction('status', 'todo')}>
                      标记为待开始
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => handleBulkAction('status', 'in_progress')}>
                      标记为进行中
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={() => handleBulkAction('status', 'done')}>
                      标记为已完成
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem
                      onClick={() => handleBulkAction('delete')}
                      className="text-destructive"
                    >
                      批量删除
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </>
            )}
          </div>

          {/* Instance Table */}
          <div className="flex-1 overflow-auto border rounded-lg">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-12">
                    <input
                      type="checkbox"
                      checked={
                        selectedInstances.size === filteredInstances.length &&
                        filteredInstances.length > 0
                      }
                      onChange={handleSelectAll}
                      className="rounded"
                    />
                  </TableHead>
                  <TableHead>日期</TableHead>
                  <TableHead>状态</TableHead>
                  <TableHead>截止时间</TableHead>
                  <TableHead>标记</TableHead>
                  <TableHead className="w-32">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredInstances.map((instance) => (
                  <TableRow key={instance.id}>
                    <TableCell>
                      <input
                        type="checkbox"
                        checked={selectedInstances.has(instance.id)}
                        onChange={() => handleSelectInstance(instance.id)}
                        className="rounded"
                      />
                    </TableCell>
                    <TableCell>
                      <div className="font-medium">
                        {format(parseISO(instance.instanceDate), 'yyyy年M月d日', { locale: zhCN })}
                      </div>
                      <div className="text-sm text-muted-foreground">
                        {format(parseISO(instance.instanceDate), 'EEEE', { locale: zhCN })}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge variant={getStatusVariant(instance.status)}>
                        {STATUS_LABELS[instance.status]}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      {instance.dueAt ? (
                        <div className="text-sm">
                          {format(parseISO(instance.dueAt), 'M月d日 HH:mm', { locale: zhCN })}
                        </div>
                      ) : (
                        <span className="text-muted-foreground">未设置</span>
                      )}
                    </TableCell>
                    <TableCell>
                      <div className="flex gap-1">
                        {instance.isException && (
                          <Badge variant="outline" className="text-xs">
                            例外
                          </Badge>
                        )}
                        {instance.completedAt && (
                          <Badge variant="outline" className="text-xs">
                            已完成
                          </Badge>
                        )}
                      </div>
                    </TableCell>
                    <TableCell>
                      <InstanceActionMenu
                        instance={instance}
                        onEdit={onEditInstance}
                        onDelete={onDeleteInstance}
                      />
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>

            {filteredInstances.length === 0 && (
              <div className="p-8 text-center text-muted-foreground">
                {instances.length === 0 ? '暂无任务实例' : '没有符合筛选条件的实例'}
              </div>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            关闭
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

// Instance Action Menu Component
interface InstanceActionMenuProps {
  instance: TaskInstance;
  onEdit: (instance: TaskInstance, editType: 'single' | 'series') => void;
  onDelete: (instanceId: string, deleteType: 'single' | 'series') => void;
}

function InstanceActionMenu({ instance, onEdit, onDelete }: InstanceActionMenuProps) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="sm">
          操作
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onClick={() => onEdit(instance, 'single')}>编辑此实例</DropdownMenuItem>
        <DropdownMenuItem onClick={() => onEdit(instance, 'series')}>编辑整个系列</DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          onClick={() => onDelete(instance.id, 'single')}
          className="text-destructive"
        >
          删除此实例
        </DropdownMenuItem>
        <DropdownMenuItem
          onClick={() => onDelete(instance.id, 'series')}
          className="text-destructive"
        >
          删除整个系列
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

// Helper functions
function getStatusVariant(status: TaskStatus): 'default' | 'secondary' | 'destructive' | 'outline' {
  switch (status) {
    case 'done':
      return 'default';
    case 'in_progress':
      return 'secondary';
    case 'blocked':
      return 'destructive';
    default:
      return 'outline';
  }
}

// Labels
const STATUS_LABELS: Record<TaskStatus, string> = {
  backlog: '待整理',
  todo: '待开始',
  in_progress: '进行中',
  blocked: '受阻',
  done: '已完成',
  archived: '已归档',
};
