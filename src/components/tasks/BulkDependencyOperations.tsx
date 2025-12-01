import React, { useState } from 'react';
import { Button } from '../ui/button';
import { Card } from '../ui/card';
import { Badge } from '../ui/badge';
import { 
  Trash2, 
  Edit3, 
  X, 
  Check,
  AlertTriangle,
  Layers
} from 'lucide-react';
import { TaskDependency, DependencyType, DEPENDENCY_TYPES } from '../../types/dependency';
import { 
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';

interface BulkDependencyOperationsProps {
  selectedDependencies: TaskDependency[];
  onDeleteSelected: (dependencyIds: string[]) => void;
  onUpdateTypeSelected: (dependencyIds: string[], newType: DependencyType) => void;
  onClearSelection: () => void;
  className?: string;
}

const DEPENDENCY_TYPE_LABELS: Record<DependencyType, string> = {
  finish_to_start: '完成-开始 (FS)',
  start_to_start: '开始-开始 (SS)', 
  finish_to_finish: '完成-完成 (FF)',
  start_to_finish: '开始-完成 (SF)',
};

export const BulkDependencyOperations: React.FC<BulkDependencyOperationsProps> = ({
  selectedDependencies,
  onDeleteSelected,
  onUpdateTypeSelected,
  onClearSelection,
  className,
}) => {
  const [isEditing, setIsEditing] = useState(false);
  const [selectedType, setSelectedType] = useState<DependencyType>('finish_to_start');

  if (selectedDependencies.length === 0) {
    return null;
  }

  const handleBulkDelete = () => {
    if (window.confirm(`确定要删除选中的 ${selectedDependencies.length} 个依赖关系吗？`)) {
      onDeleteSelected(selectedDependencies.map(dep => dep.id));
    }
  };

  const handleBulkTypeUpdate = () => {
    onUpdateTypeSelected(selectedDependencies.map(dep => dep.id), selectedType);
    setIsEditing(false);
  };

  // Group dependencies by type for display
  const typeGroups = selectedDependencies.reduce((groups, dep) => {
    const type = dep.dependencyType;
    groups[type] = (groups[type] || 0) + 1;
    return groups;
  }, {} as Record<DependencyType, number>);

  return (
    <Card className={`fixed bottom-4 left-1/2 transform -translate-x-1/2 z-50 p-4 shadow-lg border-2 border-blue-200 bg-white ${className}`}>
      <div className="flex items-center gap-4">
        {/* Selection info */}
        <div className="flex items-center gap-2">
          <Layers className="h-4 w-4 text-blue-600" />
          <span className="font-medium text-sm">
            已选择 {selectedDependencies.length} 个依赖关系
          </span>
        </div>

        {/* Type distribution */}
        <div className="flex gap-1">
          {Object.entries(typeGroups).map(([type, count]) => (
            <Badge key={type} variant="outline" className="text-xs">
              {DEPENDENCY_TYPE_LABELS[type as DependencyType]}: {count}
            </Badge>
          ))}
        </div>

        {/* Actions */}
        <div className="flex items-center gap-2 ml-auto">
          {!isEditing ? (
            <>
              <Button
                size="sm"
                variant="outline"
                onClick={() => setIsEditing(true)}
                className="h-8"
              >
                <Edit3 className="h-3 w-3 mr-1" />
                批量修改类型
              </Button>
              
              <Button
                size="sm"
                variant="destructive"
                onClick={handleBulkDelete}
                className="h-8"
              >
                <Trash2 className="h-3 w-3 mr-1" />
                批量删除
              </Button>
            </>
          ) : (
            <>
              <Select value={selectedType} onValueChange={(value) => setSelectedType(value as DependencyType)}>
                <SelectTrigger className="w-40 h-8">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {DEPENDENCY_TYPES.map((type) => (
                    <SelectItem key={type} value={type}>
                      {DEPENDENCY_TYPE_LABELS[type]}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              
              <Button
                size="sm"
                onClick={handleBulkTypeUpdate}
                className="h-8"
              >
                <Check className="h-3 w-3 mr-1" />
                应用
              </Button>
              
              <Button
                size="sm"
                variant="outline"
                onClick={() => setIsEditing(false)}
                className="h-8"
              >
                取消
              </Button>
            </>
          )}
          
          <Button
            size="sm"
            variant="ghost"
            onClick={onClearSelection}
            className="h-8 w-8 p-0"
          >
            <X className="h-3 w-3" />
          </Button>
        </div>
      </div>

      {/* Warning for mixed types */}
      {Object.keys(typeGroups).length > 1 && isEditing && (
        <div className="mt-2 flex items-center gap-2 text-xs text-amber-600">
          <AlertTriangle className="h-3 w-3" />
          <span>选中的依赖关系包含多种类型，批量修改将统一设置为选择的类型</span>
        </div>
      )}
    </Card>
  );
};