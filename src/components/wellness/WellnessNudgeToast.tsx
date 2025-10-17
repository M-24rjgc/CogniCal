import { AlertCircle, Clock, Coffee } from 'lucide-react';
import { useRespondToNudge, type WellnessEventRecord } from '@/hooks/useWellness';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardFooter } from '@/components/ui/card';
import { useToast } from '@/providers/toast-provider';

interface WellnessNudgeToastProps {
  nudge: WellnessEventRecord | null;
  onDismiss?: () => void;
}

export function WellnessNudgeToast({ nudge, onDismiss }: WellnessNudgeToastProps) {
  const { notify } = useToast();
  const respondMutation = useRespondToNudge();

  // Don't render if no nudge or already responded
  if (!nudge || nudge.response) {
    return null;
  }

  const handleRespond = async (response: 'Completed' | 'Snoozed' | 'Ignored') => {
    try {
      await respondMutation.mutateAsync({ id: nudge.id, response });

      if (response === 'Completed') {
        notify({
          title: '休息完成',
          description: '很好!适当的休息能提高工作效率。',
        });
      } else if (response === 'Snoozed') {
        notify({
          title: '稍后提醒',
          description: '好的,我们稍后再提醒您休息。',
        });
      }

      onDismiss?.();
    } catch {
      notify({
        title: '操作失败',
        description: '无法记录您的响应,请稍后重试。',
        variant: 'error',
      });
    }
  };

  const getMessage = () => {
    if (nudge.trigger_reason === 'WorkStreak') {
      return '您已经连续工作较长时间,建议休息 10 分钟,放松一下。';
    } else {
      return '检测到长时间专注,建议短暂休息,保护视力和健康。';
    }
  };

  const getIcon = () => {
    if (nudge.trigger_reason === 'WorkStreak') {
      return <Coffee className="h-5 w-5 text-orange-500" />;
    } else {
      return <Clock className="h-5 w-5 text-blue-500" />;
    }
  };

  return (
    <div className="fixed bottom-6 right-6 z-50 max-w-md animate-in slide-in-from-bottom-5">
      <Card className="border-2 shadow-lg">
        <CardContent className="pt-6">
          <div className="flex items-start gap-4">
            <div className="mt-0.5">{getIcon()}</div>
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-2">
                <AlertCircle className="h-4 w-4 text-muted-foreground" />
                <h3 className="font-semibold text-sm">健康提醒</h3>
              </div>
              <p className="text-sm text-muted-foreground">{getMessage()}</p>
              {nudge.deferral_count > 0 && (
                <p className="text-xs text-amber-600 mt-2">
                  您已延迟 {nudge.deferral_count} 次,建议尽快休息
                </p>
              )}
            </div>
          </div>
        </CardContent>
        <CardFooter className="flex flex-wrap gap-2 pt-2">
          <Button
            size="sm"
            variant="default"
            onClick={() => handleRespond('Completed')}
            disabled={respondMutation.isPending}
            className="flex-1"
          >
            <Coffee className="h-4 w-4 mr-2" />
            立即休息
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => handleRespond('Snoozed')}
            disabled={respondMutation.isPending || nudge.deferral_count >= 3}
            className="flex-1"
          >
            <Clock className="h-4 w-4 mr-2" />
            {nudge.deferral_count >= 3 ? '已达上限' : '稍后提醒'}
          </Button>
          <Button
            size="sm"
            variant="ghost"
            onClick={() => handleRespond('Ignored')}
            disabled={respondMutation.isPending}
            className="w-full"
          >
            忽略
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}
