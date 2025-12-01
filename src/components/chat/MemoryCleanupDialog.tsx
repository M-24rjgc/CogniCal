import { useState } from 'react';
import { Trash2, Calendar, AlertTriangle, CheckCircle, X, Loader2 } from 'lucide-react';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';

interface MemoryCleanupDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onCleanup: (olderThanDays: number) => Promise<{ success: boolean; message: string; count?: number }>;
  totalMemories: number;
}

export function MemoryCleanupDialog({ 
  isOpen, 
  onClose, 
  onCleanup, 
  totalMemories 
}: MemoryCleanupDialogProps) {
  const [selectedDays, setSelectedDays] = useState(30);
  const [isProcessing, setIsProcessing] = useState(false);
  const [result, setResult] = useState<{ success: boolean; message: string; count?: number } | null>(null);

  const cleanupOptions = [
    { days: 7, label: '7 天前', description: '删除一周前的记录' },
    { days: 30, label: '30 天前', description: '删除一个月前的记录' },
    { days: 90, label: '90 天前', description: '删除三个月前的记录' },
    { days: 180, label: '180 天前', description: '删除半年前的记录' },
    { days: 365, label: '365 天前', description: '删除一年前的记录' },
  ];

  const handleCleanup = async () => {
    setIsProcessing(true);
    setResult(null);

    try {
      const cleanupResult = await onCleanup(selectedDays);
      setResult(cleanupResult);
    } catch (error) {
      setResult({
        success: false,
        message: error instanceof Error ? error.message : '清理失败',
      });
    } finally {
      setIsProcessing(false);
    }
  };

  const handleClose = () => {
    setResult(null);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-md rounded-2xl border border-border bg-background p-6 shadow-lg">
        {/* Header */}
        <div className="mb-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Trash2 className="h-5 w-5 text-destructive" />
            <h2 className="text-lg font-semibold">清理记忆数据</h2>
          </div>
          <Button variant="ghost" size="icon" onClick={handleClose}>
            <X className="h-4 w-4" />
          </Button>
        </div>

        {/* Content */}
        <div className="space-y-4">
          {/* Warning */}
          <div className="rounded-lg border border-amber-200 bg-amber-50 dark:border-amber-800 dark:bg-amber-950/30 p-4">
            <div className="flex items-start gap-3">
              <AlertTriangle className="h-5 w-5 text-amber-600 dark:text-amber-400 mt-0.5" />
              <div className="flex-1">
                <div className="font-medium text-amber-800 dark:text-amber-200">
                  注意事项
                </div>
                <div className="text-sm text-amber-700 dark:text-amber-300 mt-1">
                  清理操作将永久删除选定时间范围内的记忆数据，此操作不可撤销。
                </div>
              </div>
            </div>
          </div>

          {/* Current Status */}
          <div className="rounded-lg border border-border bg-card p-4">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium">当前记忆数据</span>
              <Badge variant="secondary">{totalMemories} 条记录</Badge>
            </div>
            <div className="text-xs text-muted-foreground">
              包含所有历史对话和上下文信息
            </div>
          </div>

          {/* Cleanup Options */}
          {!result && (
            <div className="space-y-3">
              <div className="text-sm font-medium">选择清理范围</div>
              <div className="space-y-2">
                {cleanupOptions.map((option) => (
                  <label
                    key={option.days}
                    className={`flex items-center gap-3 rounded-lg border p-3 cursor-pointer transition-colors ${
                      selectedDays === option.days
                        ? 'border-primary bg-primary/5'
                        : 'border-border hover:bg-accent/50'
                    }`}
                  >
                    <input
                      type="radio"
                      name="cleanup-days"
                      value={option.days}
                      checked={selectedDays === option.days}
                      onChange={(e) => setSelectedDays(Number(e.target.value))}
                      className="sr-only"
                    />
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <Calendar className="h-4 w-4 text-muted-foreground" />
                        <span className="font-medium">{option.label}</span>
                      </div>
                      <div className="text-sm text-muted-foreground mt-1">
                        {option.description}
                      </div>
                    </div>
                    {selectedDays === option.days && (
                      <div className="h-4 w-4 rounded-full border-2 border-primary bg-primary flex items-center justify-center">
                        <div className="h-2 w-2 rounded-full bg-white" />
                      </div>
                    )}
                  </label>
                ))}
              </div>
            </div>
          )}

          {/* Result Display */}
          {result && (
            <div className={`rounded-lg border p-4 ${
              result.success 
                ? 'border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950/30'
                : 'border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-950/30'
            }`}>
              <div className="flex items-start gap-3">
                {result.success ? (
                  <CheckCircle className="h-5 w-5 text-green-600 dark:text-green-400 mt-0.5" />
                ) : (
                  <X className="h-5 w-5 text-red-600 dark:text-red-400 mt-0.5" />
                )}
                <div className="flex-1">
                  <div className={`font-medium ${
                    result.success 
                      ? 'text-green-800 dark:text-green-200'
                      : 'text-red-800 dark:text-red-200'
                  }`}>
                    {result.success ? '清理完成' : '清理失败'}
                  </div>
                  <div className={`text-sm mt-1 ${
                    result.success 
                      ? 'text-green-700 dark:text-green-300'
                      : 'text-red-700 dark:text-red-300'
                  }`}>
                    {result.message}
                    {result.success && result.count !== undefined && (
                      <span className="block mt-1">
                        已清理 {result.count} 条记录
                      </span>
                    )}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-2 pt-2">
            <Button
              variant="outline"
              onClick={handleClose}
              className="flex-1"
            >
              {result ? '完成' : '取消'}
            </Button>
            
            {!result && (
              <Button
                variant="destructive"
                onClick={handleCleanup}
                disabled={isProcessing}
                className="flex-1"
              >
                {isProcessing ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    清理中...
                  </>
                ) : (
                  <>
                    <Trash2 className="mr-2 h-4 w-4" />
                    开始清理
                  </>
                )}
              </Button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}