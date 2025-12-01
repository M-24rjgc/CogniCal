import React, { useState } from 'react';
import { Button } from '../ui/button';
import { 
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';
import { Card } from '../ui/card';
import { X, Check } from 'lucide-react';
import { DependencyType, DEPENDENCY_TYPES } from '../../types/dependency';

interface DependencyTypeEditorProps {
  currentType: DependencyType;
  onSave: (newType: DependencyType) => void;
  onCancel: () => void;
  position: { x: number; y: number };
}

const DEPENDENCY_TYPE_LABELS: Record<DependencyType, string> = {
  finish_to_start: '完成-开始 (FS)',
  start_to_start: '开始-开始 (SS)', 
  finish_to_finish: '完成-完成 (FF)',
  start_to_finish: '开始-完成 (SF)',
};

const DEPENDENCY_TYPE_DESCRIPTIONS: Record<DependencyType, string> = {
  finish_to_start: '前置任务完成后，后续任务才能开始',
  start_to_start: '前置任务开始后，后续任务才能开始',
  finish_to_finish: '前置任务完成后，后续任务才能完成',
  start_to_finish: '前置任务开始后，后续任务才能完成',
};

export const DependencyTypeEditor: React.FC<DependencyTypeEditorProps> = ({
  currentType,
  onSave,
  onCancel,
  position,
}) => {
  const [selectedType, setSelectedType] = useState<DependencyType>(currentType);

  const handleSave = () => {
    onSave(selectedType);
  };

  return (
    <div
      style={{
        position: 'absolute',
        left: position.x,
        top: position.y,
        zIndex: 1000,
      }}
      className="pointer-events-auto"
    >
      <Card className="p-4 shadow-lg border-2 border-blue-200 bg-white min-w-[300px]">
        <div className="flex items-center justify-between mb-3">
          <h4 className="font-medium text-sm">编辑依赖类型</h4>
          <Button
            size="sm"
            variant="ghost"
            className="h-6 w-6 p-0"
            onClick={onCancel}
          >
            <X className="h-3 w-3" />
          </Button>
        </div>

        <div className="space-y-3">
          <Select value={selectedType} onValueChange={(value) => setSelectedType(value as DependencyType)}>
            <SelectTrigger className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {DEPENDENCY_TYPES.map((type) => (
                <SelectItem key={type} value={type}>
                  <div className="flex flex-col">
                    <span className="font-medium">{DEPENDENCY_TYPE_LABELS[type]}</span>
                    <span className="text-xs text-gray-500">
                      {DEPENDENCY_TYPE_DESCRIPTIONS[type]}
                    </span>
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <div className="text-xs text-gray-600 p-2 bg-gray-50 rounded">
            {DEPENDENCY_TYPE_DESCRIPTIONS[selectedType]}
          </div>

          <div className="flex gap-2 justify-end">
            <Button size="sm" variant="outline" onClick={onCancel}>
              取消
            </Button>
            <Button 
              size="sm" 
              onClick={handleSave}
              disabled={selectedType === currentType}
            >
              <Check className="h-3 w-3 mr-1" />
              保存
            </Button>
          </div>
        </div>
      </Card>
    </div>
  );
};