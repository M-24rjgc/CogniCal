import React from 'react';
import { cn } from '../../lib/utils';
import { ArrowRight, CheckCircle2 } from 'lucide-react';

interface DragConnectionOverlayProps {
  isConnecting: boolean;
  sourceNodeId?: string;
  hoveredNodeId?: string;
  draggedConnection?: {
    sourceX: number;
    sourceY: number;
    targetX: number;
    targetY: number;
  } | null;
  className?: string;
}

export const DragConnectionOverlay: React.FC<DragConnectionOverlayProps> = ({
  isConnecting,
  sourceNodeId,
  hoveredNodeId,
  draggedConnection,
  className,
}) => {
  if (!isConnecting) {
    return null;
  }

  return (
    <>
      {/* Background overlay */}
      <div className={cn(
        'pointer-events-none absolute inset-0 z-40 bg-black/5',
        className
      )} />
      
      {/* Connection instructions */}
      <div className="pointer-events-none absolute left-4 top-4 z-50 rounded-lg border border-blue-200 bg-blue-50 p-3 text-sm text-blue-700 shadow-lg">
        <div className="flex items-center gap-2">
          <ArrowRight className="h-4 w-4" />
          <span className="font-medium">创建依赖关系</span>
        </div>
        <p className="mt-1 text-xs">
          从 <span className="font-mono font-medium">{sourceNodeId}</span> 拖拽到目标任务
        </p>
        {hoveredNodeId && (
          <div className="mt-2 flex items-center gap-2 rounded border border-green-200 bg-green-50 p-2 text-green-700">
            <CheckCircle2 className="h-3 w-3" />
            <span className="text-xs">
              目标: <span className="font-mono font-medium">{hoveredNodeId}</span>
            </span>
          </div>
        )}
        <p className="mt-1 text-xs text-blue-600">
          松开鼠标完成连接，按 ESC 取消
        </p>
      </div>

      {/* Visual connection line (if dragging) */}
      {draggedConnection && (
        <svg className="pointer-events-none absolute inset-0 z-45">
          <defs>
            <marker
              id="arrowhead-temp"
              markerWidth="10"
              markerHeight="7"
              refX="9"
              refY="3.5"
              orient="auto"
            >
              <polygon
                points="0 0, 10 3.5, 0 7"
                fill="#3b82f6"
                opacity="0.7"
              />
            </marker>
          </defs>
          <line
            x1={draggedConnection.sourceX}
            y1={draggedConnection.sourceY}
            x2={draggedConnection.targetX}
            y2={draggedConnection.targetY}
            stroke="#3b82f6"
            strokeWidth="2"
            strokeDasharray="5,5"
            opacity="0.7"
            markerEnd="url(#arrowhead-temp)"
          />
        </svg>
      )}

      {/* Node highlight styles */}
      <style>{`
        .react-flow__node[data-id="${sourceNodeId}"] {
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.3);
          z-index: 1000;
        }
        
        ${hoveredNodeId ? `
        .react-flow__node[data-id="${hoveredNodeId}"] {
          box-shadow: 0 0 0 3px rgba(34, 197, 94, 0.5);
          z-index: 1000;
        }
        ` : ''}
      `}</style>
    </>
  );
};