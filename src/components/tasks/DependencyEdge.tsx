import React from 'react';
import { 
  EdgeProps, 
  getBezierPath, 
  EdgeLabelRenderer,
  BaseEdge
} from '@xyflow/react';
import { Button } from '../ui/button';
import { X } from 'lucide-react';
import { TaskDependency, DependencyType } from '../../types/dependency';
import { cn } from '../../lib/utils';
import { DependencyTypeEditor } from './DependencyTypeEditor';

interface DependencyEdgeData {
  dependency: TaskDependency;
  onDelete?: (dependencyId: string) => void;
  onUpdateType?: (dependencyId: string, newType: DependencyType) => void;
}

const getDependencyTypeLabel = (type: DependencyType) => {
  switch (type) {
    case 'finish_to_start':
      return 'FS';
    case 'start_to_start':
      return 'SS';
    case 'finish_to_finish':
      return 'FF';
    case 'start_to_finish':
      return 'SF';
    default:
      return 'FS';
  }
};

const getDependencyTypeColor = (type: DependencyType) => {
  switch (type) {
    case 'finish_to_start':
      return 'stroke-blue-500';
    case 'start_to_start':
      return 'stroke-green-500';
    case 'finish_to_finish':
      return 'stroke-purple-500';
    case 'start_to_finish':
      return 'stroke-orange-500';
    default:
      return 'stroke-blue-500';
  }
};

export const DependencyEdge: React.FC<EdgeProps> = ({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  data,
  selected,
  markerEnd
}) => {
  const [isHovered, setIsHovered] = React.useState(false);
  const [isEditing, setIsEditing] = React.useState(false);
  const [editorPosition, setEditorPosition] = React.useState({ x: 0, y: 0 });
  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  const edgeData = data as unknown as DependencyEdgeData;
  const dependency = edgeData?.dependency;
  const onDelete = edgeData?.onDelete;
  const onUpdateType = edgeData?.onUpdateType;

  if (!dependency) {
    return null;
  }

  return (
    <>
      <BaseEdge
        id={id}
        path={edgePath}
        markerEnd={markerEnd}
        className={cn(
          'transition-all duration-200 cursor-pointer',
          getDependencyTypeColor(dependency.dependencyType),
          selected || isHovered ? 'stroke-4' : 'stroke-2',
          isHovered && 'drop-shadow-lg'
        )}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      />
      
      <EdgeLabelRenderer>
        <div
          style={{
            position: 'absolute',
            transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
            pointerEvents: 'all',
          }}
          className="flex items-center gap-1"
          onMouseEnter={() => setIsHovered(true)}
          onMouseLeave={() => setIsHovered(false)}
        >
          {/* Dependency type label */}
          <div className={cn(
            'px-2 py-1 text-xs font-medium rounded-md border bg-white shadow-sm transition-all duration-200',
            (selected || isHovered) && 'ring-2 ring-blue-500 ring-offset-1 shadow-md',
            'cursor-pointer hover:bg-blue-50'
          )}>
            {getDependencyTypeLabel(dependency.dependencyType)}
          </div>
          
          {/* Delete button (shown on selection or hover) */}
          {(selected || isHovered) && onDelete && (
            <Button
              size="sm"
              variant="destructive"
              className="h-6 w-6 p-0 rounded-full opacity-80 hover:opacity-100 transition-all duration-200"
              onClick={(e) => {
                e.stopPropagation();
                if (window.confirm('确定要删除这个依赖关系吗？')) {
                  onDelete(dependency.id);
                }
              }}
              title="删除依赖关系"
            >
              <X className="h-3 w-3" />
            </Button>
          )}
          
          {/* Edit button for dependency type (shown on hover) */}
          {isHovered && onUpdateType && (
            <Button
              size="sm"
              variant="outline"
              className="h-6 w-6 p-0 rounded-full opacity-80 hover:opacity-100 transition-all duration-200"
              onClick={(e) => {
                e.stopPropagation();
                const rect = e.currentTarget.getBoundingClientRect();
                setEditorPosition({
                  x: rect.left,
                  y: rect.bottom + 5,
                });
                setIsEditing(true);
              }}
              title="编辑依赖类型"
            >
              <span className="text-xs">✎</span>
            </Button>
          )}
        </div>

        {/* Dependency Type Editor */}
        {isEditing && onUpdateType && (
          <DependencyTypeEditor
            currentType={dependency.dependencyType}
            position={editorPosition}
            onSave={(newType) => {
              onUpdateType(dependency.id, newType);
              setIsEditing(false);
            }}
            onCancel={() => setIsEditing(false)}
          />
        )}
      </EdgeLabelRenderer>
    </>
  );
};