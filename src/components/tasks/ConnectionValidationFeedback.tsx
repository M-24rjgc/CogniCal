import React from 'react';
import { AlertTriangle, CheckCircle2, Loader2 } from 'lucide-react';
import { cn } from '../../lib/utils';
import { DependencyValidation } from '../../types/dependency';

interface ConnectionValidationFeedbackProps {
  isValidating: boolean;
  validation: DependencyValidation | null;
  className?: string;
}

export const ConnectionValidationFeedback: React.FC<ConnectionValidationFeedbackProps> = ({
  isValidating,
  validation,
  className,
}) => {
  if (isValidating) {
    return (
      <div className={cn(
        'flex items-center gap-2 rounded-lg border border-blue-200 bg-blue-50 p-3 text-sm text-blue-700',
        className
      )}>
        <Loader2 className="h-4 w-4 animate-spin" />
        <span>正在验证连接...</span>
      </div>
    );
  }

  if (!validation) {
    return null;
  }

  if (validation.isValid) {
    return (
      <div className={cn(
        'flex items-center gap-2 rounded-lg border border-green-200 bg-green-50 p-3 text-sm text-green-700',
        className
      )}>
        <CheckCircle2 className="h-4 w-4" />
        <span>连接有效，可以创建依赖关系</span>
      </div>
    );
  }

  return (
    <div className={cn(
      'flex flex-col gap-2 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700',
      className
    )}>
      <div className="flex items-center gap-2">
        <AlertTriangle className="h-4 w-4" />
        <span className="font-medium">无法创建依赖关系</span>
      </div>
      
      {validation.errorMessage && (
        <p className="text-xs">{validation.errorMessage}</p>
      )}
      
      {validation.wouldCreateCycle && validation.cyclePath && (
        <div className="mt-1">
          <p className="text-xs font-medium">检测到循环依赖:</p>
          <p className="text-xs font-mono">
            {validation.cyclePath.join(' → ')}
          </p>
        </div>
      )}
    </div>
  );
};