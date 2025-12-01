import { useState } from 'react';
import { Download, Loader2, X, FileText, Archive, CheckCircle } from 'lucide-react';
import { Button } from '../ui/button';

interface ConversationExportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onExport: () => Promise<string>;
  conversationId: string;
  messageCount: number;
}

export function ConversationExportDialog({ 
  isOpen, 
  onClose, 
  onExport, 
  conversationId,
  messageCount 
}: ConversationExportDialogProps) {
  const [isExporting, setIsExporting] = useState(false);
  const [exportPath, setExportPath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleExport = async () => {
    setIsExporting(true);
    setError(null);
    setExportPath(null);

    try {
      const path = await onExport();
      setExportPath(path);
    } catch (err) {
      setError(err instanceof Error ? err.message : '导出失败');
    } finally {
      setIsExporting(false);
    }
  };

  const handleClose = () => {
    setExportPath(null);
    setError(null);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-md rounded-2xl border border-border bg-background p-6 shadow-lg">
        {/* Header */}
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-lg font-semibold">导出对话</h2>
          <Button variant="ghost" size="icon" onClick={handleClose}>
            <X className="h-4 w-4" />
          </Button>
        </div>

        {/* Content */}
        <div className="space-y-4">
          {/* Conversation Info */}
          <div className="rounded-lg border border-border bg-card p-4">
            <div className="flex items-center gap-3 mb-3">
              <FileText className="h-5 w-5 text-muted-foreground" />
              <div>
                <div className="font-medium">当前对话</div>
                <div className="text-sm text-muted-foreground">
                  {messageCount} 条消息
                </div>
              </div>
            </div>
            
            <div className="text-xs text-muted-foreground font-mono bg-muted/50 rounded px-2 py-1">
              ID: {conversationId}
            </div>
          </div>

          {/* Export Options */}
          <div className="space-y-3">
            <div className="text-sm font-medium">导出格式</div>
            <div className="rounded-lg border border-border bg-card p-4">
              <div className="flex items-center gap-3">
                <Archive className="h-5 w-5 text-blue-500" />
                <div className="flex-1">
                  <div className="font-medium">Markdown 归档</div>
                  <div className="text-sm text-muted-foreground">
                    包含完整对话历史和元数据的 Markdown 文件
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Success State */}
          {exportPath && (
            <div className="rounded-lg border border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950/30 p-4">
              <div className="flex items-start gap-3">
                <CheckCircle className="h-5 w-5 text-green-600 dark:text-green-400 mt-0.5" />
                <div className="flex-1">
                  <div className="font-medium text-green-800 dark:text-green-200">
                    导出成功
                  </div>
                  <div className="text-sm text-green-700 dark:text-green-300 mt-1">
                    文件已保存到: {exportPath}
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Error State */}
          {error && (
            <div className="rounded-lg border border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-950/30 p-4">
              <div className="flex items-start gap-3">
                <X className="h-5 w-5 text-red-600 dark:text-red-400 mt-0.5" />
                <div className="flex-1">
                  <div className="font-medium text-red-800 dark:text-red-200">
                    导出失败
                  </div>
                  <div className="text-sm text-red-700 dark:text-red-300 mt-1">
                    {error}
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
              {exportPath ? '完成' : '取消'}
            </Button>
            
            {!exportPath && (
              <Button
                onClick={handleExport}
                disabled={isExporting}
                className="flex-1"
              >
                {isExporting ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    导出中...
                  </>
                ) : (
                  <>
                    <Download className="mr-2 h-4 w-4" />
                    开始导出
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