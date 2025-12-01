import React, { useState, useCallback } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { 
  Download, 
  Share2, 
  FileImage, 
  FileText, 
  Database, 
  Copy, 
  Check,
  ExternalLink,
  Image as ImageIcon,
  FileJson
} from 'lucide-react';
import { 
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '../ui/select';
import { Task } from '../../types/task';
import { TaskDependency } from '../../types/dependency';
import { pushToast } from '../../stores/uiStore';
import { cn } from '../../lib/utils';

interface ExportOptions {
  format: 'png' | 'svg' | 'json' | 'csv' | 'markdown';
  includeCompleted: boolean;
  includeMetadata: boolean;
  anonymize: boolean;
  quality: 'low' | 'medium' | 'high';
}

interface GraphExportPanelProps {
  tasks: Task[];
  dependencies: TaskDependency[];
  onExport: (options: ExportOptions) => Promise<void>;
  onShare: (shareUrl: string) => void;
  className?: string;
}

const EXPORT_FORMATS = [
  { value: 'png', label: 'PNG 图片', icon: FileImage, description: '高质量图片，适合文档和演示' },
  { value: 'svg', label: 'SVG 矢量图', icon: ImageIcon, description: '可缩放矢量图形，适合打印' },
  { value: 'json', label: 'JSON 数据', icon: FileJson, description: '结构化数据，可导入其他工具' },
  { value: 'csv', label: 'CSV 表格', icon: Database, description: '表格数据，可用 Excel 打开' },
  { value: 'markdown', label: 'Markdown 文档', icon: FileText, description: '文本格式，适合文档记录' },
] as const;

const QUALITY_OPTIONS = [
  { value: 'low', label: '低质量 (快速)', description: '1x 分辨率，文件较小' },
  { value: 'medium', label: '中等质量', description: '2x 分辨率，平衡质量和大小' },
  { value: 'high', label: '高质量 (推荐)', description: '4x 分辨率，最佳质量' },
] as const;

export const GraphExportPanel: React.FC<GraphExportPanelProps> = ({
  tasks,
  dependencies,
  onExport,
  onShare,
  className,
}) => {
  const [isExporting, setIsExporting] = useState(false);
  const [exportOptions, setExportOptions] = useState<ExportOptions>({
    format: 'png',
    includeCompleted: true,
    includeMetadata: true,
    anonymize: false,
    quality: 'high',
  });
  const [shareUrl, setShareUrl] = useState<string>('');
  const [isSharing, setIsSharing] = useState(false);
  const [copied, setCopied] = useState(false);

  const handleExport = useCallback(async () => {
    setIsExporting(true);
    try {
      await onExport(exportOptions);
      pushToast({
        title: '导出成功',
        description: `图表已导出为 ${exportOptions.format.toUpperCase()} 格式`,
        variant: 'success',
      });
    } catch (error) {
      console.error('Export failed:', error);
      pushToast({
        title: '导出失败',
        description: error instanceof Error ? error.message : '未知错误',
        variant: 'error',
      });
    } finally {
      setIsExporting(false);
    }
  }, [exportOptions, onExport]);

  const handleShare = useCallback(async () => {
    setIsSharing(true);
    try {
      // Generate a shareable URL (in a real implementation, this would call an API)
      const mockShareUrl = `https://cognical.app/shared/graph/${Date.now()}`;
      setShareUrl(mockShareUrl);
      onShare(mockShareUrl);
      
      pushToast({
        title: '分享链接已生成',
        description: '链接已复制到剪贴板',
        variant: 'success',
      });
    } catch (error) {
      console.error('Share failed:', error);
      pushToast({
        title: '分享失败',
        description: error instanceof Error ? error.message : '未知错误',
        variant: 'error',
      });
    } finally {
      setIsSharing(false);
    }
  }, [onShare]);

  const handleCopyShareUrl = useCallback(async () => {
    if (!shareUrl) return;
    
    try {
      await navigator.clipboard.writeText(shareUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
      
      pushToast({
        title: '链接已复制',
        variant: 'success',
      });
    } catch (error) {
      console.error('Copy failed:', error);
      pushToast({
        title: '复制失败',
        description: '请手动复制链接',
        variant: 'error',
      });
    }
  }, [shareUrl]);

  const updateExportOption = useCallback(<K extends keyof ExportOptions>(
    key: K,
    value: ExportOptions[K]
  ) => {
    setExportOptions(prev => ({
      ...prev,
      [key]: value,
    }));
  }, []);

  const selectedFormat = EXPORT_FORMATS.find(f => f.value === exportOptions.format);
  const selectedQuality = QUALITY_OPTIONS.find(q => q.value === exportOptions.quality);

  // Calculate export statistics
  const stats = React.useMemo(() => {
    const totalTasks = tasks.length;
    const completedTasks = tasks.filter(t => t.status === 'done').length;
    const totalDependencies = dependencies.length;
    
    return {
      totalTasks,
      completedTasks,
      totalDependencies,
      filteredTasks: exportOptions.includeCompleted ? totalTasks : totalTasks - completedTasks,
    };
  }, [tasks, dependencies, exportOptions.includeCompleted]);

  return (
    <Card className={cn('border-l-4 border-l-green-500', className)}>
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <div className="flex items-center gap-2">
          <Download className="h-4 w-4 text-green-600" />
          <span className="font-medium">导出与分享</span>
        </div>
        
        <div className="flex items-center gap-2">
          <Badge variant="outline" className="text-xs">
            {stats.filteredTasks} 任务
          </Badge>
          <Badge variant="outline" className="text-xs">
            {stats.totalDependencies} 依赖
          </Badge>
        </div>
      </div>

      <div className="p-4 space-y-4">
        {/* Export Format Selection */}
        <div>
          <label className="text-sm font-medium mb-2 block">导出格式</label>
          <Select 
            value={exportOptions.format} 
            onValueChange={(value) => updateExportOption('format', value as ExportOptions['format'])}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {EXPORT_FORMATS.map((format) => {
                const Icon = format.icon;
                return (
                  <SelectItem key={format.value} value={format.value}>
                    <div className="flex items-center gap-2">
                      <Icon className="h-4 w-4" />
                      <div>
                        <div className="font-medium">{format.label}</div>
                        <div className="text-xs text-muted-foreground">{format.description}</div>
                      </div>
                    </div>
                  </SelectItem>
                );
              })}
            </SelectContent>
          </Select>
        </div>

        {/* Quality Selection (for image formats) */}
        {(exportOptions.format === 'png' || exportOptions.format === 'svg') && (
          <div>
            <label className="text-sm font-medium mb-2 block">图片质量</label>
            <Select 
              value={exportOptions.quality} 
              onValueChange={(value) => updateExportOption('quality', value as ExportOptions['quality'])}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {QUALITY_OPTIONS.map((quality) => (
                  <SelectItem key={quality.value} value={quality.value}>
                    <div>
                      <div className="font-medium">{quality.label}</div>
                      <div className="text-xs text-muted-foreground">{quality.description}</div>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}

        {/* Export Options */}
        <div>
          <label className="text-sm font-medium mb-2 block">导出选项</label>
          <div className="space-y-2">
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={exportOptions.includeCompleted}
                onChange={(e) => updateExportOption('includeCompleted', e.target.checked)}
                className="rounded"
              />
              包含已完成任务
            </label>
            
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={exportOptions.includeMetadata}
                onChange={(e) => updateExportOption('includeMetadata', e.target.checked)}
                className="rounded"
              />
              包含任务元数据 (创建时间、标签等)
            </label>
            
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={exportOptions.anonymize}
                onChange={(e) => updateExportOption('anonymize', e.target.checked)}
                className="rounded"
              />
              匿名化处理 (隐藏敏感信息)
            </label>
          </div>
        </div>

        {/* Export Preview */}
        <div className="bg-muted/50 rounded-lg p-3">
          <div className="text-sm font-medium mb-2">导出预览</div>
          <div className="text-xs text-muted-foreground space-y-1">
            <div>格式: {selectedFormat?.label}</div>
            {(exportOptions.format === 'png' || exportOptions.format === 'svg') && (
              <div>质量: {selectedQuality?.label}</div>
            )}
            <div>任务数量: {stats.filteredTasks} / {stats.totalTasks}</div>
            <div>依赖关系: {stats.totalDependencies}</div>
            {exportOptions.anonymize && (
              <div className="text-amber-600">⚠️ 将进行匿名化处理</div>
            )}
          </div>
        </div>

        {/* Export Button */}
        <Button 
          onClick={handleExport} 
          disabled={isExporting}
          className="w-full"
        >
          <Download className="h-4 w-4 mr-2" />
          {isExporting ? '导出中...' : `导出为 ${exportOptions.format.toUpperCase()}`}
        </Button>

        {/* Share Section */}
        <div className="border-t pt-4">
          <div className="flex items-center gap-2 mb-3">
            <Share2 className="h-4 w-4 text-blue-600" />
            <span className="text-sm font-medium">在线分享</span>
          </div>
          
          <div className="space-y-2">
            <Button 
              onClick={handleShare} 
              disabled={isSharing}
              variant="outline"
              className="w-full"
            >
              <Share2 className="h-4 w-4 mr-2" />
              {isSharing ? '生成中...' : '生成分享链接'}
            </Button>
            
            {shareUrl && (
              <div className="flex items-center gap-2 p-2 bg-muted rounded-lg">
                <input
                  type="text"
                  value={shareUrl}
                  readOnly
                  className="flex-1 bg-transparent text-xs border-none outline-none"
                />
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={handleCopyShareUrl}
                  className="h-6 w-6 p-0"
                >
                  {copied ? (
                    <Check className="h-3 w-3 text-green-600" />
                  ) : (
                    <Copy className="h-3 w-3" />
                  )}
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => window.open(shareUrl, '_blank')}
                  className="h-6 w-6 p-0"
                >
                  <ExternalLink className="h-3 w-3" />
                </Button>
              </div>
            )}
          </div>
          
          <div className="text-xs text-muted-foreground mt-2">
            分享链接将在 30 天后过期，包含当前筛选条件下的图表数据
          </div>
        </div>
      </div>
    </Card>
  );
};