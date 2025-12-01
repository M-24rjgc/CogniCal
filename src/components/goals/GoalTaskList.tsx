import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Plus, CheckCircle2, Circle, Clock, AlertCircle, Trash2 } from 'lucide-react';
import { Button } from '../ui/button';
import { Input } from '../ui/input';

interface Task {
  id: string;
  title: string;
  status: string;
  priority: string;
}

interface GoalTaskListProps {
  goalId: string;
  onTasksChange: () => void;
}

export function GoalTaskList({ goalId, onTasksChange }: GoalTaskListProps) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);
  const [newTaskTitle, setNewTaskTitle] = useState('');
  const [addingTask, setAddingTask] = useState(false);

  const loadTasks = async () => {
    try {
      setLoading(true);
      const taskIds = await invoke<string[]>('get_goal_tasks', { goalId });

      const taskPromises = taskIds.map((id) => invoke<Task>('tasks_get', { id }));

      const loadedTasks = await Promise.all(taskPromises);
      setTasks(loadedTasks);
    } catch (error) {
      console.error('加载任务失败:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    const loadTasksData = async () => {
      try {
        setLoading(true);
        const taskIds = await invoke<string[]>('get_goal_tasks', { goalId });

        const taskPromises = taskIds.map((id) => invoke<Task>('tasks_get', { id }));

        const loadedTasks = await Promise.all(taskPromises);
        setTasks(loadedTasks);
      } catch (error) {
        console.error('加载任务失败:', error);
      } finally {
        setLoading(false);
      }
    };

    loadTasksData();
  }, [goalId]);

  const handleAddTask = async () => {
    if (!newTaskTitle.trim()) return;

    try {
      setAddingTask(true);
      const task = await invoke<Task>('tasks_create', {
        task: {
          title: newTaskTitle,
          description: '',
          status: 'todo',
          priority: 'medium',
        },
      });

      await invoke('associate_task_with_goal', {
        goalId,
        taskId: task.id,
      });

      setNewTaskTitle('');
      await loadTasks();
      onTasksChange();
    } catch (error) {
      console.error('添加任务失败:', error);
    } finally {
      setAddingTask(false);
    }
  };

  const handleToggleTask = async (taskId: string, currentStatus: string) => {
    try {
      const newStatus = currentStatus === 'completed' ? 'todo' : 'completed';
      await invoke('tasks_update', {
        id: taskId,
        task: { status: newStatus },
      });
      await loadTasks();
      onTasksChange();
    } catch (error) {
      console.error('更新任务失败:', error);
    }
  };

  const handleRemoveTask = async (taskId: string) => {
    try {
      await invoke('dissociate_task_from_goal', { goalId, taskId });
      await loadTasks();
      onTasksChange();
    } catch (error) {
      console.error('删除任务失败:', error);
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'completed':
        return <CheckCircle2 className="w-5 h-5 text-green-500" />;
      case 'in_progress':
        return <Clock className="w-5 h-5 text-blue-500" />;
      case 'blocked':
        return <AlertCircle className="w-5 h-5 text-red-500" />;
      default:
        return <Circle className="w-5 h-5 text-gray-400" />;
    }
  };

  if (loading) {
    return <div className="text-muted-foreground">加载任务中...</div>;
  }

  return (
    <div className="space-y-3">
      {tasks.map((task) => (
        <div
          key={task.id}
          className="flex items-center gap-3 p-3 border rounded-lg hover:bg-accent transition-colors"
        >
          <button onClick={() => handleToggleTask(task.id, task.status)} className="flex-shrink-0">
            {getStatusIcon(task.status)}
          </button>
          <div className="flex-1">
            <div
              className={`font-medium ${task.status === 'completed' ? 'line-through text-muted-foreground' : ''}`}
            >
              {task.title}
            </div>
          </div>
          <button
            onClick={() => handleRemoveTask(task.id)}
            className="flex-shrink-0 text-muted-foreground hover:text-red-500 transition-colors"
          >
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      ))}

      <div className="flex gap-2">
        <Input
          value={newTaskTitle}
          onChange={(e) => setNewTaskTitle(e.target.value)}
          onKeyPress={(e) => e.key === 'Enter' && handleAddTask()}
          placeholder="添加新任务..."
          disabled={addingTask}
        />
        <Button onClick={handleAddTask} disabled={!newTaskTitle.trim() || addingTask}>
          <Plus className="w-4 h-4" />
        </Button>
      </div>
    </div>
  );
}
