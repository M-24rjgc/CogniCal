import { useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '../ui/dialog';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Textarea } from '../ui/textarea';
import { Label } from '../ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { GOAL_TEMPLATES } from './GoalTemplates';
import { GoalTemplate, CreateGoalRequest, Goal } from '../../types/goal';
import type { Task } from '../../types/task';
import { invoke } from '@tauri-apps/api/core';

interface GoalCreationWizardProps {
  open: boolean;
  onClose: () => void;
  onGoalCreated: () => void;
  parentGoalId?: string;
}

export function GoalCreationWizard({
  open,
  onClose,
  onGoalCreated,
  parentGoalId,
}: GoalCreationWizardProps) {
  const [step, setStep] = useState(1);
  const [selectedTemplate, setSelectedTemplate] = useState<GoalTemplate | null>(null);
  const [goalData, setGoalData] = useState<CreateGoalRequest>({
    title: '',
    description: '',
    priority: 'medium',
    parentGoalId,
  });
  const [targetDate, setTargetDate] = useState('');
  const [loading, setLoading] = useState(false);

  const handleTemplateSelect = (templateId: string) => {
    const template = GOAL_TEMPLATES.find((t) => t.id === templateId);
    setSelectedTemplate(template || null);
    setStep(2);
  };

  const handleSkipTemplate = () => {
    setSelectedTemplate(null);
    setStep(2);
  };

  const handleCreateGoal = async () => {
    try {
      setLoading(true);

      const request: CreateGoalRequest = {
        ...goalData,
        targetDate: targetDate || undefined,
      };

      const goal = await invoke<Goal>('create_goal', { request });

      // If a template was selected, create tasks based on the template
      if (selectedTemplate && goal) {
        const goalId = goal.id;
        const createdTasks: Task[] = [];

        for (const taskTemplate of selectedTemplate.taskStructure) {
          const task = await invoke<Task>('tasks_create', {
            task: {
              title: taskTemplate.title,
              description: taskTemplate.description || '',
              status: 'todo',
              priority: goalData.priority,
              estimatedMinutes: taskTemplate.estimatedMinutes,
            },
          });

          createdTasks.push(task);

          // Associate task with goal
          await invoke('associate_task_with_goal', {
            goalId,
            taskId: task.id,
          });

          // Create dependencies if specified
          if (taskTemplate.dependencies) {
            for (const depIndex of taskTemplate.dependencies) {
              if (createdTasks[depIndex]) {
                await invoke('add_dependency', {
                  predecessorId: createdTasks[depIndex].id,
                  successorId: task.id,
                  dependencyType: 'finish_to_start',
                });
              }
            }
          }
        }
      }

      onGoalCreated();
      handleClose();
    } catch (error) {
      console.error('创建目标失败:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    setStep(1);
    setSelectedTemplate(null);
    setGoalData({
      title: '',
      description: '',
      priority: 'medium',
      parentGoalId,
    });
    setTargetDate('');
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{step === 1 ? '选择模板' : '创建目标'}</DialogTitle>
        </DialogHeader>

        {step === 1 && (
          <div className="space-y-4">
            <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
              <h3 className="font-semibold text-blue-900 mb-2">💡 目标分解指导</h3>
              <p className="text-sm text-blue-800">
                将大目标分解为较小的任务，使其更易于管理和跟踪。
                选择与您的项目结构匹配的模板，或创建自定义目标。
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {GOAL_TEMPLATES.map((template) => (
                <button
                  key={template.id}
                  onClick={() => handleTemplateSelect(template.id)}
                  className="p-4 border rounded-lg hover:border-primary hover:bg-accent text-left transition-colors group"
                >
                  <div className="flex items-start justify-between mb-2">
                    <h3 className="font-semibold group-hover:text-primary transition-colors">
                      {template.name}
                    </h3>
                    <span className="text-xs px-2 py-1 bg-primary/10 text-primary rounded">
                      {template.type}
                    </span>
                  </div>
                  <p className="text-sm text-muted-foreground mb-3">{template.description}</p>
                  <div className="flex items-center gap-2 text-xs text-muted-foreground">
                    <span className="font-medium">{template.taskStructure.length} 个任务</span>
                    <span>•</span>
                    <span>
                      {
                        template.taskStructure.filter(
                          (t) => t.dependencies && t.dependencies.length > 0,
                        ).length
                      }{' '}
                      个依赖
                    </span>
                  </div>
                </button>
              ))}
            </div>

            <div className="flex justify-between pt-4">
              <Button variant="outline" onClick={handleClose}>
                取消
              </Button>
              <Button variant="secondary" onClick={handleSkipTemplate}>
                跳过模板
              </Button>
            </div>
          </div>
        )}

        {step === 2 && (
          <div className="space-y-4">
            {selectedTemplate && (
              <div className="p-4 bg-accent rounded-lg space-y-3">
                <div>
                  <p className="text-sm font-medium">使用模板：{selectedTemplate.name}</p>
                  <p className="text-xs text-muted-foreground">
                    将创建 {selectedTemplate.taskStructure.length} 个任务及其依赖关系
                  </p>
                </div>
                <div className="space-y-2 max-h-40 overflow-y-auto">
                  {selectedTemplate.taskStructure.map((task, index) => (
                    <div key={index} className="text-xs p-2 bg-background rounded border">
                      <div className="font-medium">
                        {index + 1}. {task.title}
                      </div>
                      {task.dependencies && task.dependencies.length > 0 && (
                        <div className="text-muted-foreground mt-1">
                          依赖于：{task.dependencies.map((d) => `任务 ${d + 1}`).join('、')}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div className="space-y-2">
              <Label htmlFor="title">目标标题 *</Label>
              <Input
                id="title"
                value={goalData.title}
                onChange={(e) => setGoalData({ ...goalData, title: e.target.value })}
                placeholder="输入目标标题"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="description">描述</Label>
              <Textarea
                id="description"
                value={goalData.description}
                onChange={(e) => setGoalData({ ...goalData, description: e.target.value })}
                placeholder="描述您的目标"
                rows={3}
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="priority">优先级</Label>
                <Select
                  value={goalData.priority}
                  onValueChange={(value) => setGoalData({ ...goalData, priority: value })}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="low">低</SelectItem>
                    <SelectItem value="medium">中</SelectItem>
                    <SelectItem value="high">高</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <Label htmlFor="targetDate">目标日期</Label>
                <Input
                  id="targetDate"
                  type="date"
                  value={targetDate}
                  onChange={(e) => setTargetDate(e.target.value)}
                />
              </div>
            </div>

            <div className="flex justify-between pt-4">
              <Button variant="outline" onClick={() => setStep(1)}>
                返回
              </Button>
              <Button onClick={handleCreateGoal} disabled={!goalData.title || loading}>
                {loading ? '创建中...' : '创建目标'}
              </Button>
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
