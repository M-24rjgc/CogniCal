import React, { useState, useCallback } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { Input } from '../ui/input';
import { Textarea } from '../ui/textarea';
import { 
  Layers, 
  Plus, 
  ArrowRight, 
  GitBranch, 
  Target,
  Clock,
  Code,
  Palette,
  Save
} from 'lucide-react';
import { 
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '../ui/dialog';
import { Task } from '../../types/task';
import { TaskDependency } from '../../types/dependency';
import { pushToast } from '../../stores/uiStore';
import { cn } from '../../lib/utils';

interface ProjectTemplate {
  id: string;
  name: string;
  description: string;
  category: 'software' | 'marketing' | 'research' | 'general';
  icon: React.ComponentType<{ className?: string }>;
  tasks: Omit<Task, 'id' | 'createdAt' | 'updatedAt'>[];
  dependencies: Omit<TaskDependency, 'id' | 'createdAt'>[];
  estimatedDuration: string;
  complexity: 'simple' | 'medium' | 'complex';
}

interface GraphTemplatePanelProps {
  onApplyTemplate: (template: ProjectTemplate) => Promise<void>;
  onSaveAsTemplate: (name: string, description: string) => Promise<void>;
  className?: string;
}

const BUILT_IN_TEMPLATES: ProjectTemplate[] = [
  {
    id: 'sequential-project',
    name: '顺序项目',
    description: '任务按顺序执行，每个任务完成后才能开始下一个',
    category: 'general',
    icon: ArrowRight,
    estimatedDuration: '2-4 周',
    complexity: 'simple',
    tasks: [
      {
        title: '项目启动',
        description: '定义项目目标和范围',
        status: 'todo',
        priority: 'high',
        tags: ['启动'],
        estimatedMinutes: 120,
        isRecurring: false,
      },
      {
        title: '需求分析',
        description: '收集和分析项目需求',
        status: 'todo',
        priority: 'high',
        tags: ['分析'],
        estimatedMinutes: 240,
        isRecurring: false,
      },
      {
        title: '设计方案',
        description: '制定详细的设计方案',
        status: 'todo',
        priority: 'medium',
        tags: ['设计'],
        estimatedMinutes: 360,
        isRecurring: false,
      },
      {
        title: '实施执行',
        description: '按照设计方案执行项目',
        status: 'todo',
        priority: 'high',
        tags: ['执行'],
        estimatedMinutes: 720,
        isRecurring: false,
      },
      {
        title: '测试验收',
        description: '测试项目成果并进行验收',
        status: 'todo',
        priority: 'medium',
        tags: ['测试'],
        estimatedMinutes: 180,
        isRecurring: false,
      },
      {
        title: '项目收尾',
        description: '整理文档，总结经验',
        status: 'todo',
        priority: 'low',
        tags: ['收尾'],
        estimatedMinutes: 120,
        isRecurring: false,
      },
    ],
    dependencies: [
      { predecessorId: '0', successorId: '1', dependencyType: 'finish_to_start' },
      { predecessorId: '1', successorId: '2', dependencyType: 'finish_to_start' },
      { predecessorId: '2', successorId: '3', dependencyType: 'finish_to_start' },
      { predecessorId: '3', successorId: '4', dependencyType: 'finish_to_start' },
      { predecessorId: '4', successorId: '5', dependencyType: 'finish_to_start' },
    ],
  },
  {
    id: 'parallel-project',
    name: '并行项目',
    description: '多个任务可以同时进行，最后汇总到一个里程碑',
    category: 'general',
    icon: GitBranch,
    estimatedDuration: '1-3 周',
    complexity: 'medium',
    tasks: [
      {
        title: '项目启动',
        description: '项目启动会议和资源分配',
        status: 'todo',
        priority: 'high',
        tags: ['启动'],
        estimatedMinutes: 90,
        isRecurring: false,
      },
      {
        title: '前端开发',
        description: '用户界面和交互开发',
        status: 'todo',
        priority: 'high',
        tags: ['前端'],
        estimatedMinutes: 480,
        isRecurring: false,
      },
      {
        title: '后端开发',
        description: 'API 和数据库开发',
        status: 'todo',
        priority: 'high',
        tags: ['后端'],
        estimatedMinutes: 480,
        isRecurring: false,
      },
      {
        title: '测试用例编写',
        description: '编写自动化测试用例',
        status: 'todo',
        priority: 'medium',
        tags: ['测试'],
        estimatedMinutes: 240,
        isRecurring: false,
      },
      {
        title: '文档编写',
        description: '编写用户文档和技术文档',
        status: 'todo',
        priority: 'medium',
        tags: ['文档'],
        estimatedMinutes: 180,
        isRecurring: false,
      },
      {
        title: '集成测试',
        description: '前后端集成和系统测试',
        status: 'todo',
        priority: 'high',
        tags: ['集成'],
        estimatedMinutes: 240,
        isRecurring: false,
      },
      {
        title: '项目交付',
        description: '最终交付和部署',
        status: 'todo',
        priority: 'high',
        tags: ['交付'],
        estimatedMinutes: 120,
        isRecurring: false,
      },
    ],
    dependencies: [
      { predecessorId: '0', successorId: '1', dependencyType: 'finish_to_start' },
      { predecessorId: '0', successorId: '2', dependencyType: 'finish_to_start' },
      { predecessorId: '0', successorId: '3', dependencyType: 'finish_to_start' },
      { predecessorId: '0', successorId: '4', dependencyType: 'finish_to_start' },
      { predecessorId: '1', successorId: '5', dependencyType: 'finish_to_start' },
      { predecessorId: '2', successorId: '5', dependencyType: 'finish_to_start' },
      { predecessorId: '3', successorId: '5', dependencyType: 'finish_to_start' },
      { predecessorId: '5', successorId: '6', dependencyType: 'finish_to_start' },
    ],
  },
  {
    id: 'milestone-project',
    name: '里程碑项目',
    description: '以关键里程碑为节点的项目结构',
    category: 'general',
    icon: Target,
    estimatedDuration: '3-6 周',
    complexity: 'complex',
    tasks: [
      {
        title: '里程碑 1: 项目规划',
        description: '完成项目规划和资源准备',
        status: 'todo',
        priority: 'urgent',
        tags: ['里程碑', '规划'],
        estimatedMinutes: 180,
        isRecurring: false,
      },
      {
        title: '需求调研',
        description: '深入调研用户需求',
        status: 'todo',
        priority: 'high',
        tags: ['调研'],
        estimatedMinutes: 360,
        isRecurring: false,
      },
      {
        title: '竞品分析',
        description: '分析竞争对手产品',
        status: 'todo',
        priority: 'medium',
        tags: ['分析'],
        estimatedMinutes: 240,
        isRecurring: false,
      },
      {
        title: '里程碑 2: 设计完成',
        description: '完成所有设计工作',
        status: 'todo',
        priority: 'urgent',
        tags: ['里程碑', '设计'],
        estimatedMinutes: 120,
        isRecurring: false,
      },
      {
        title: '原型设计',
        description: '制作产品原型',
        status: 'todo',
        priority: 'high',
        tags: ['原型'],
        estimatedMinutes: 480,
        isRecurring: false,
      },
      {
        title: '技术架构',
        description: '设计技术架构方案',
        status: 'todo',
        priority: 'high',
        tags: ['架构'],
        estimatedMinutes: 360,
        isRecurring: false,
      },
      {
        title: '里程碑 3: 开发完成',
        description: '完成核心功能开发',
        status: 'todo',
        priority: 'urgent',
        tags: ['里程碑', '开发'],
        estimatedMinutes: 120,
        isRecurring: false,
      },
      {
        title: '核心功能开发',
        description: '开发产品核心功能',
        status: 'todo',
        priority: 'high',
        tags: ['开发'],
        estimatedMinutes: 960,
        isRecurring: false,
      },
      {
        title: '里程碑 4: 项目交付',
        description: '项目最终交付',
        status: 'todo',
        priority: 'urgent',
        tags: ['里程碑', '交付'],
        estimatedMinutes: 180,
        isRecurring: false,
      },
    ],
    dependencies: [
      { predecessorId: '1', successorId: '0', dependencyType: 'finish_to_start' },
      { predecessorId: '2', successorId: '0', dependencyType: 'finish_to_start' },
      { predecessorId: '0', successorId: '4', dependencyType: 'finish_to_start' },
      { predecessorId: '0', successorId: '5', dependencyType: 'finish_to_start' },
      { predecessorId: '4', successorId: '3', dependencyType: 'finish_to_start' },
      { predecessorId: '5', successorId: '3', dependencyType: 'finish_to_start' },
      { predecessorId: '3', successorId: '7', dependencyType: 'finish_to_start' },
      { predecessorId: '7', successorId: '6', dependencyType: 'finish_to_start' },
      { predecessorId: '6', successorId: '8', dependencyType: 'finish_to_start' },
    ],
  },
  {
    id: 'software-development',
    name: '软件开发项目',
    description: '标准的软件开发生命周期模板',
    category: 'software',
    icon: Code,
    estimatedDuration: '4-8 周',
    complexity: 'complex',
    tasks: [
      {
        title: '需求分析',
        description: '收集和分析软件需求',
        status: 'todo',
        priority: 'high',
        tags: ['需求', '分析'],
        estimatedMinutes: 480,
        isRecurring: false,
      },
      {
        title: '系统设计',
        description: '设计系统架构和数据库',
        status: 'todo',
        priority: 'high',
        tags: ['设计', '架构'],
        estimatedMinutes: 360,
        isRecurring: false,
      },
      {
        title: 'UI/UX 设计',
        description: '设计用户界面和用户体验',
        status: 'todo',
        priority: 'medium',
        tags: ['UI', 'UX'],
        estimatedMinutes: 480,
        isRecurring: false,
      },
      {
        title: '数据库设计',
        description: '设计数据库结构',
        status: 'todo',
        priority: 'high',
        tags: ['数据库'],
        estimatedMinutes: 240,
        isRecurring: false,
      },
      {
        title: '前端开发',
        description: '开发用户界面',
        status: 'todo',
        priority: 'high',
        tags: ['前端', '开发'],
        estimatedMinutes: 720,
        isRecurring: false,
      },
      {
        title: '后端开发',
        description: '开发服务器端逻辑',
        status: 'todo',
        priority: 'high',
        tags: ['后端', '开发'],
        estimatedMinutes: 720,
        isRecurring: false,
      },
      {
        title: '单元测试',
        description: '编写和执行单元测试',
        status: 'todo',
        priority: 'medium',
        tags: ['测试', '单元'],
        estimatedMinutes: 360,
        isRecurring: false,
      },
      {
        title: '集成测试',
        description: '系统集成测试',
        status: 'todo',
        priority: 'high',
        tags: ['测试', '集成'],
        estimatedMinutes: 240,
        isRecurring: false,
      },
      {
        title: '部署上线',
        description: '部署到生产环境',
        status: 'todo',
        priority: 'high',
        tags: ['部署', '上线'],
        estimatedMinutes: 180,
        isRecurring: false,
      },
    ],
    dependencies: [
      { predecessorId: '0', successorId: '1', dependencyType: 'finish_to_start' },
      { predecessorId: '1', successorId: '2', dependencyType: 'finish_to_start' },
      { predecessorId: '1', successorId: '3', dependencyType: 'finish_to_start' },
      { predecessorId: '2', successorId: '4', dependencyType: 'finish_to_start' },
      { predecessorId: '3', successorId: '5', dependencyType: 'finish_to_start' },
      { predecessorId: '4', successorId: '6', dependencyType: 'start_to_start' },
      { predecessorId: '5', successorId: '6', dependencyType: 'start_to_start' },
      { predecessorId: '4', successorId: '7', dependencyType: 'finish_to_start' },
      { predecessorId: '5', successorId: '7', dependencyType: 'finish_to_start' },
      { predecessorId: '6', successorId: '7', dependencyType: 'finish_to_start' },
      { predecessorId: '7', successorId: '8', dependencyType: 'finish_to_start' },
    ],
  },
  {
    id: 'marketing-campaign',
    name: '营销活动',
    description: '完整的营销活动策划和执行模板',
    category: 'marketing',
    icon: Palette,
    estimatedDuration: '3-5 周',
    complexity: 'medium',
    tasks: [
      {
        title: '市场调研',
        description: '分析目标市场和用户群体',
        status: 'todo',
        priority: 'high',
        tags: ['调研', '市场'],
        estimatedMinutes: 360,
        isRecurring: false,
      },
      {
        title: '策略制定',
        description: '制定营销策略和目标',
        status: 'todo',
        priority: 'high',
        tags: ['策略'],
        estimatedMinutes: 240,
        isRecurring: false,
      },
      {
        title: '内容创作',
        description: '创作营销内容和素材',
        status: 'todo',
        priority: 'medium',
        tags: ['内容', '创作'],
        estimatedMinutes: 480,
        isRecurring: false,
      },
      {
        title: '渠道准备',
        description: '准备营销渠道和平台',
        status: 'todo',
        priority: 'medium',
        tags: ['渠道'],
        estimatedMinutes: 180,
        isRecurring: false,
      },
      {
        title: '活动执行',
        description: '执行营销活动',
        status: 'todo',
        priority: 'high',
        tags: ['执行', '活动'],
        estimatedMinutes: 360,
        isRecurring: false,
      },
      {
        title: '效果监控',
        description: '监控活动效果和数据',
        status: 'todo',
        priority: 'medium',
        tags: ['监控', '数据'],
        estimatedMinutes: 120,
        isRecurring: false,
      },
      {
        title: '效果分析',
        description: '分析活动效果和ROI',
        status: 'todo',
        priority: 'medium',
        tags: ['分析', 'ROI'],
        estimatedMinutes: 180,
        isRecurring: false,
      },
    ],
    dependencies: [
      { predecessorId: '0', successorId: '1', dependencyType: 'finish_to_start' },
      { predecessorId: '1', successorId: '2', dependencyType: 'finish_to_start' },
      { predecessorId: '1', successorId: '3', dependencyType: 'finish_to_start' },
      { predecessorId: '2', successorId: '4', dependencyType: 'finish_to_start' },
      { predecessorId: '3', successorId: '4', dependencyType: 'finish_to_start' },
      { predecessorId: '4', successorId: '5', dependencyType: 'start_to_start' },
      { predecessorId: '5', successorId: '6', dependencyType: 'finish_to_start' },
    ],
  },
];

const CATEGORY_LABELS = {
  general: '通用',
  software: '软件开发',
  marketing: '营销推广',
  research: '研究分析',
};

const COMPLEXITY_LABELS = {
  simple: '简单',
  medium: '中等',
  complex: '复杂',
};

export const GraphTemplatePanel: React.FC<GraphTemplatePanelProps> = ({
  onApplyTemplate,
  onSaveAsTemplate,
  className,
}) => {
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [isApplying, setIsApplying] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [saveDialogOpen, setSaveDialogOpen] = useState(false);
  const [templateName, setTemplateName] = useState('');
  const [templateDescription, setTemplateDescription] = useState('');

  const filteredTemplates = React.useMemo(() => {
    if (selectedCategory === 'all') {
      return BUILT_IN_TEMPLATES;
    }
    return BUILT_IN_TEMPLATES.filter(template => template.category === selectedCategory);
  }, [selectedCategory]);

  const handleApplyTemplate = useCallback(async (template: ProjectTemplate) => {
    setIsApplying(true);
    try {
      await onApplyTemplate(template);
      pushToast({
        title: '模板应用成功',
        description: `已应用「${template.name}」模板`,
        variant: 'success',
      });
    } catch (error) {
      console.error('Apply template failed:', error);
      pushToast({
        title: '模板应用失败',
        description: error instanceof Error ? error.message : '未知错误',
        variant: 'error',
      });
    } finally {
      setIsApplying(false);
    }
  }, [onApplyTemplate]);

  const handleSaveTemplate = useCallback(async () => {
    if (!templateName.trim()) {
      pushToast({
        title: '请输入模板名称',
        variant: 'error',
      });
      return;
    }

    setIsSaving(true);
    try {
      await onSaveAsTemplate(templateName.trim(), templateDescription.trim());
      setSaveDialogOpen(false);
      setTemplateName('');
      setTemplateDescription('');
      
      pushToast({
        title: '模板保存成功',
        description: `已保存为「${templateName}」模板`,
        variant: 'success',
      });
    } catch (error) {
      console.error('Save template failed:', error);
      pushToast({
        title: '模板保存失败',
        description: error instanceof Error ? error.message : '未知错误',
        variant: 'error',
      });
    } finally {
      setIsSaving(false);
    }
  }, [templateName, templateDescription, onSaveAsTemplate]);

  return (
    <Card className={cn('border-l-4 border-l-purple-500', className)}>
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <div className="flex items-center gap-2">
          <Layers className="h-4 w-4 text-purple-600" />
          <span className="font-medium">项目模板</span>
        </div>
        
        <div className="flex items-center gap-2">
          <Dialog open={saveDialogOpen} onOpenChange={setSaveDialogOpen}>
            <DialogTrigger asChild>
              <Button size="sm" variant="outline" className="h-8">
                <Save className="h-3 w-3 mr-1" />
                保存模板
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>保存为模板</DialogTitle>
                <DialogDescription>
                  将当前的任务和依赖关系保存为可重用的项目模板
                </DialogDescription>
              </DialogHeader>
              
              <div className="space-y-4">
                <div>
                  <label className="text-sm font-medium mb-1 block">模板名称</label>
                  <Input
                    value={templateName}
                    onChange={(e) => setTemplateName(e.target.value)}
                    placeholder="输入模板名称..."
                  />
                </div>
                
                <div>
                  <label className="text-sm font-medium mb-1 block">模板描述</label>
                  <Textarea
                    value={templateDescription}
                    onChange={(e) => setTemplateDescription(e.target.value)}
                    placeholder="描述模板的用途和特点..."
                    rows={3}
                  />
                </div>
              </div>
              
              <DialogFooter>
                <Button
                  variant="outline"
                  onClick={() => setSaveDialogOpen(false)}
                >
                  取消
                </Button>
                <Button
                  onClick={handleSaveTemplate}
                  disabled={isSaving || !templateName.trim()}
                >
                  {isSaving ? '保存中...' : '保存模板'}
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
      </div>

      {/* Category Filter */}
      <div className="p-4 border-b">
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant={selectedCategory === 'all' ? 'default' : 'outline'}
            onClick={() => setSelectedCategory('all')}
            className="h-8"
          >
            全部
          </Button>
          {Object.entries(CATEGORY_LABELS).map(([category, label]) => (
            <Button
              key={category}
              size="sm"
              variant={selectedCategory === category ? 'default' : 'outline'}
              onClick={() => setSelectedCategory(category)}
              className="h-8"
            >
              {label}
            </Button>
          ))}
        </div>
      </div>

      {/* Template List */}
      <div className="p-4 space-y-3 max-h-96 overflow-y-auto">
        {filteredTemplates.map((template) => {
          const Icon = template.icon;
          return (
            <Card key={template.id} className="p-4 hover:shadow-md transition-shadow">
              <div className="flex items-start justify-between">
                <div className="flex items-start gap-3 flex-1">
                  <div className="p-2 bg-purple-100 rounded-lg">
                    <Icon className="h-5 w-5 text-purple-600" />
                  </div>
                  
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <h4 className="font-medium">{template.name}</h4>
                      <Badge variant="outline" className="text-xs">
                        {CATEGORY_LABELS[template.category]}
                      </Badge>
                      <Badge 
                        variant={template.complexity === 'simple' ? 'secondary' : 
                               template.complexity === 'medium' ? 'default' : 'destructive'}
                        className="text-xs"
                      >
                        {COMPLEXITY_LABELS[template.complexity]}
                      </Badge>
                    </div>
                    
                    <p className="text-sm text-muted-foreground mb-2">
                      {template.description}
                    </p>
                    
                    <div className="flex items-center gap-4 text-xs text-muted-foreground">
                      <span className="flex items-center gap-1">
                        <Target className="h-3 w-3" />
                        {template.tasks.length} 任务
                      </span>
                      <span className="flex items-center gap-1">
                        <GitBranch className="h-3 w-3" />
                        {template.dependencies.length} 依赖
                      </span>
                      <span className="flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        {template.estimatedDuration}
                      </span>
                    </div>
                  </div>
                </div>
                
                <Button
                  size="sm"
                  onClick={() => handleApplyTemplate(template)}
                  disabled={isApplying}
                  className="ml-4"
                >
                  <Plus className="h-3 w-3 mr-1" />
                  应用
                </Button>
              </div>
            </Card>
          );
        })}
        
        {filteredTemplates.length === 0 && (
          <div className="text-center py-8 text-muted-foreground">
            <Layers className="h-8 w-8 mx-auto mb-2 opacity-50" />
            <p>该分类下暂无模板</p>
          </div>
        )}
      </div>
    </Card>
  );
};