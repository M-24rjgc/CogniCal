import { useMemo } from 'react';
import { Cell, Legend, Pie, PieChart, ResponsiveContainer, Tooltip } from 'recharts';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Skeleton } from '../ui/skeleton';
import { type TimeAllocationBreakdown } from '../../types/analytics';
import { type TaskPriority, type TaskType } from '../../types/task';

interface TimeAllocationChartProps {
  allocation: TimeAllocationBreakdown | null;
  isLoading: boolean;
}

const COLORS = ['#6366f1', '#22c55e', '#f97316', '#0ea5e9', '#a855f7', '#ef4444', '#14b8a6'];

export function TimeAllocationChart({ allocation, isLoading }: TimeAllocationChartProps) {
  const typeData = useMemo(
    () =>
      allocation?.byType.map((item) => ({
        name: formatType(item.type),
        value: item.percentage,
      })) ?? [],
    [allocation],
  );
  const priorityData = useMemo(
    () =>
      allocation?.byPriority.map((item) => ({
        name: formatPriority(item.priority),
        value: item.percentage,
      })) ?? [],
    [allocation],
  );

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle className="text-lg">时间分配</CardTitle>
        <p className="text-sm text-muted-foreground">查看任务类型与优先级的时间占比。</p>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <LoadingState />
        ) : typeData.length === 0 && priorityData.length === 0 ? (
          <EmptyState />
        ) : (
          <div className="grid gap-8 md:grid-cols-2">
            <div className="flex flex-col items-center">
              <SectionTitle>按任务类型</SectionTitle>
              <div className="h-[320px] w-full">
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={typeData}
                      dataKey="value"
                      nameKey="name"
                      innerRadius={70}
                      outerRadius={110}
                      paddingAngle={4}
                    >
                      {typeData.map((entry, index) => (
                        <Cell key={`type-${entry.name}`} fill={COLORS[index % COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip formatter={(value: number) => `${value.toFixed(1)}%`} />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
              </div>
            </div>
            <div className="flex flex-col items-center">
              <SectionTitle>按优先级</SectionTitle>
              <div className="h-[320px] w-full">
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={priorityData}
                      dataKey="value"
                      nameKey="name"
                      innerRadius={70}
                      outerRadius={110}
                      paddingAngle={4}
                    >
                      {priorityData.map((entry, index) => (
                        <Cell
                          key={`priority-${entry.name}`}
                          fill={COLORS[(index + 3) % COLORS.length]}
                        />
                      ))}
                    </Pie>
                    <Tooltip formatter={(value: number) => `${value.toFixed(1)}%`} />
                    <Legend />
                  </PieChart>
                </ResponsiveContainer>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function SectionTitle({ children }: { children: string }) {
  return <h4 className="mb-4 text-center text-sm font-medium text-muted-foreground">{children}</h4>;
}

function LoadingState() {
  return (
    <div className="flex h-[320px] flex-col justify-center gap-4">
      <Skeleton className="mx-auto h-6 w-1/3" />
      <Skeleton className="mx-auto h-6 w-1/2" />
      <Skeleton className="mx-auto h-6 w-2/5" />
    </div>
  );
}

function EmptyState() {
  return (
    <div className="flex h-[320px] flex-col items-center justify-center rounded-md border border-dashed text-center text-sm text-muted-foreground">
      <p>等待更多已分类的任务，即可生成时间分布图。</p>
    </div>
  );
}

function formatType(type: TaskType) {
  switch (type) {
    case 'work':
      return '工作';
    case 'study':
      return '学习';
    case 'life':
      return '生活';
    default:
      return '其他';
  }
}

function formatPriority(priority: TaskPriority) {
  switch (priority) {
    case 'urgent':
      return '紧急';
    case 'high':
      return '高';
    case 'medium':
      return '中';
    default:
      return '低';
  }
}
