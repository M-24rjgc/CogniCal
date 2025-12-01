import React from 'react';
import { getBezierPath } from '@xyflow/react';
import { cn } from '../../lib/utils';

interface ConnectionPreviewProps {
  sourceX: number;
  sourceY: number;
  targetX: number;
  targetY: number;
  isValid?: boolean;
  className?: string;
}

export const ConnectionPreview: React.FC<ConnectionPreviewProps> = ({
  sourceX,
  sourceY,
  targetX,
  targetY,
  isValid = true,
  className,
}) => {
  const [edgePath] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition: 'bottom' as any,
    targetX,
    targetY,
    targetPosition: 'top' as any,
  });

  return (
    <svg className={cn('pointer-events-none absolute inset-0 z-40', className)}>
      <defs>
        <marker
          id="arrowhead-preview"
          markerWidth="10"
          markerHeight="7"
          refX="9"
          refY="3.5"
          orient="auto"
        >
          <polygon
            points="0 0, 10 3.5, 0 7"
            fill={isValid ? '#10b981' : '#ef4444'}
            opacity="0.8"
          />
        </marker>
      </defs>
      <path
        d={edgePath}
        stroke={isValid ? '#10b981' : '#ef4444'}
        strokeWidth="3"
        strokeDasharray="8,4"
        opacity="0.8"
        fill="none"
        markerEnd="url(#arrowhead-preview)"
        className="animate-pulse"
      />
    </svg>
  );
};