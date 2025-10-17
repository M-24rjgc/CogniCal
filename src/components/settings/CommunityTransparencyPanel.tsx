import { useState } from 'react';
import { ExternalLink, Github, FileText, Users, Download, Shield, Check } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { useProjectInfo, useDetectPlugins, useGenerateExportBundle } from '@/hooks/useCommunity';
import { ExportReviewDialog } from './ExportReviewDialog';
import { useToast } from '@/providers/toast-provider';
import type { ExportBundle } from '@/hooks/useCommunity';

export function CommunityTransparencyPanel() {
  const { notify } = useToast();
  const { data: projectInfo, isLoading: isLoadingInfo } = useProjectInfo();
  const { data: plugins, isLoading: isLoadingPlugins } = useDetectPlugins();
  const generateExport = useGenerateExportBundle();

  const [showExportDialog, setShowExportDialog] = useState(false);
  const [generatedBundle, setGeneratedBundle] = useState<ExportBundle | null>(null);
  const [includeFeedback, setIncludeFeedback] = useState(false);

  const handleGenerateExport = async (includesFeedback: boolean) => {
    try {
      setIncludeFeedback(includesFeedback);
      const bundle = await generateExport.mutateAsync(includesFeedback);
      setGeneratedBundle(bundle);
      setShowExportDialog(true);
    } catch (error) {
      notify({
        title: '导出失败',
        description: error instanceof Error ? error.message : '无法生成导出包',
        variant: 'error',
      });
    }
  };

  return (
    <div className="space-y-6">
      {/* Project Information */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Shield className="h-5 w-5" />
                开源透明度
              </CardTitle>
              <CardDescription>CogniCal 是完全开源的项目,所有功能永久免费</CardDescription>
            </div>
            {projectInfo?.isOpenSource && (
              <Badge variant="default" className="gap-1">
                <Check className="h-3 w-3" />
                100% 开源
              </Badge>
            )}
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {isLoadingInfo ? (
            <div className="space-y-2">
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-3/4" />
            </div>
          ) : projectInfo ? (
            <>
              <div className="grid gap-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">版本</span>
                  <span className="text-sm font-medium">{projectInfo.version}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground">许可证</span>
                  <Badge variant="outline">{projectInfo.license}</Badge>
                </div>
              </div>

              <div className="flex flex-wrap gap-2 pt-2">
                <Button
                  variant="outline"
                  size="sm"
                  className="gap-2"
                  onClick={() => window.open(projectInfo.repositoryUrl, '_blank')}
                >
                  <Github className="h-4 w-4" />
                  源代码仓库
                  <ExternalLink className="h-3 w-3" />
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  className="gap-2"
                  onClick={() => window.open(projectInfo.docsUrl, '_blank')}
                >
                  <FileText className="h-4 w-4" />
                  文档
                  <ExternalLink className="h-3 w-3" />
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  className="gap-2"
                  onClick={() => window.open(projectInfo.contributingUrl, '_blank')}
                >
                  <Users className="h-4 w-4" />
                  贡献指南
                  <ExternalLink className="h-3 w-3" />
                </Button>
              </div>

              {projectInfo.featuresAlwaysFree && (
                <div className="rounded-lg bg-primary/10 p-3 text-sm">
                  <p className="font-medium text-primary">✨ 所有功能永久免费</p>
                  <p className="text-xs text-muted-foreground mt-1">
                    包括生产力评分、AI 建议、工作负载预测等高级功能,无需付费订阅
                  </p>
                </div>
              )}
            </>
          ) : null}
        </CardContent>
      </Card>

      {/* Plugins Section */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileText className="h-5 w-5" />
            社区插件
          </CardTitle>
          <CardDescription>检测到的插件和扩展模块</CardDescription>
        </CardHeader>
        <CardContent>
          {isLoadingPlugins ? (
            <Skeleton className="h-20 w-full" />
          ) : plugins && plugins.length > 0 ? (
            <div className="space-y-3">
              {plugins.map((plugin, index) => (
                <div key={index} className="flex items-start justify-between rounded-lg border p-3">
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{plugin.name}</span>
                      {plugin.version && (
                        <Badge variant="secondary" className="text-xs">
                          v{plugin.version}
                        </Badge>
                      )}
                      <Badge variant={plugin.enabled ? 'default' : 'outline'} className="text-xs">
                        {plugin.enabled ? '已启用' : '已禁用'}
                      </Badge>
                    </div>
                    <p className="text-xs text-muted-foreground mt-1">来源: {plugin.source}</p>
                    {plugin.permissions.length > 0 && (
                      <p className="text-xs text-muted-foreground mt-1">
                        权限: {plugin.permissions.join(', ')}
                      </p>
                    )}
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-8">
              <FileText className="h-12 w-12 text-muted-foreground/50 mx-auto mb-2" />
              <p className="text-sm text-muted-foreground">未检测到插件</p>
              <p className="text-xs text-muted-foreground mt-1">插件功能将在未来版本中提供</p>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Community Export Section */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Download className="h-5 w-5" />
            社区反馈导出
          </CardTitle>
          <CardDescription>
            导出匿名化的使用数据和反馈摘要,用于提交 GitHub Issue 或社区讨论
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="rounded-lg bg-muted/50 p-3 text-sm space-y-2">
            <p className="font-medium">🔒 隐私保护说明:</p>
            <ul className="text-xs text-muted-foreground space-y-1 ml-4 list-disc">
              <li>所有个人信息(任务名称、备注等)将被自动移除</li>
              <li>仅包含统计数据和聚合指标</li>
              <li>导出前可以预览所有内容</li>
              <li>数据完全本地生成,不会自动上传</li>
            </ul>
          </div>

          <div className="flex gap-2">
            <Button
              onClick={() => handleGenerateExport(false)}
              disabled={generateExport.isPending}
              className="gap-2"
            >
              <Download className="h-4 w-4" />
              {generateExport.isPending ? '生成中...' : '生成基础导出'}
            </Button>
            <Button
              variant="outline"
              onClick={() => handleGenerateExport(true)}
              disabled={generateExport.isPending}
              className="gap-2"
            >
              <Download className="h-4 w-4" />
              包含 AI 反馈
            </Button>
          </div>

          <p className="text-xs text-muted-foreground">
            导出包将包含系统信息、匿名化指标和 SHA-256 校验和,格式为 Markdown
          </p>
        </CardContent>
      </Card>

      {/* Export Review Dialog */}
      {generatedBundle && (
        <ExportReviewDialog
          open={showExportDialog}
          onOpenChange={setShowExportDialog}
          bundle={generatedBundle}
          includedFeedback={includeFeedback}
        />
      )}
    </div>
  );
}
