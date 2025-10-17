import { useState } from 'react';
import { ThumbsUp, ThumbsDown, MessageSquare } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Textarea } from '@/components/ui/textarea';
import { useSubmitFeedback, useCheckOptOut, type FeedbackSubmission } from '@/hooks/useFeedback';
import { useToast } from '@/providers/toast-provider';

export interface FeedbackControlsProps {
  surface: 'score' | 'recommendation' | 'forecast';
  sessionId?: string;
  promptSnapshot: string;
  contextSnapshot: Record<string, unknown>;
  onFeedbackSubmitted?: (sentiment: 'up' | 'down') => void;
}

export function FeedbackControls({
  surface,
  sessionId,
  promptSnapshot,
  contextSnapshot,
  onFeedbackSubmitted,
}: FeedbackControlsProps) {
  const [showNoteDialog, setShowNoteDialog] = useState(false);
  const [selectedSentiment, setSelectedSentiment] = useState<'up' | 'down' | null>(null);
  const [note, setNote] = useState('');

  const { data: optedOut, isLoading: optOutLoading } = useCheckOptOut();
  const submitFeedback = useSubmitFeedback();
  const { notify } = useToast();

  // Don't show controls if user has opted out
  if (optOutLoading) {
    return null;
  }

  if (optedOut) {
    return null;
  }

  const handleQuickFeedback = async (sentiment: 'up' | 'down') => {
    const submission: FeedbackSubmission = {
      surface,
      sessionId,
      sentiment,
      promptSnapshot,
      contextSnapshot,
    };

    try {
      await submitFeedback.mutateAsync(submission);
      notify({
        title: '感谢反馈!',
        description: sentiment === 'up' ? '很高兴帮到你!' : '我们会持续改进',
        variant: 'default',
      });
      onFeedbackSubmitted?.(sentiment);
    } catch (error) {
      notify({
        title: '反馈提交失败',
        description: error instanceof Error ? error.message : '未知错误',
        variant: 'error',
      });
    }
  };

  const handleFeedbackWithNote = (sentiment: 'up' | 'down') => {
    setSelectedSentiment(sentiment);
    setShowNoteDialog(true);
  };

  const handleSubmitNote = async () => {
    if (!selectedSentiment) return;

    const submission: FeedbackSubmission = {
      surface,
      sessionId,
      sentiment: selectedSentiment,
      note: note.trim() || undefined,
      promptSnapshot,
      contextSnapshot,
    };

    try {
      await submitFeedback.mutateAsync(submission);
      notify({
        title: '感谢详细反馈!',
        description: '你的意见将帮助我们改进',
        variant: 'default',
      });
      setShowNoteDialog(false);
      setNote('');
      setSelectedSentiment(null);
      onFeedbackSubmitted?.(selectedSentiment);
    } catch (error) {
      notify({
        title: '反馈提交失败',
        description: error instanceof Error ? error.message : '未知错误',
        variant: 'error',
      });
    }
  };

  return (
    <>
      <div className="inline-flex items-center gap-1 rounded-md border bg-background p-1">
        <Button
          variant="ghost"
          size="sm"
          className="h-7 px-2"
          onClick={() => handleQuickFeedback('up')}
          disabled={submitFeedback.isPending}
        >
          <ThumbsUp className="h-3.5 w-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="h-7 px-2"
          onClick={() => handleQuickFeedback('down')}
          disabled={submitFeedback.isPending}
        >
          <ThumbsDown className="h-3.5 w-3.5" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="h-7 px-2"
          onClick={() => {
            // Default to positive feedback with note
            handleFeedbackWithNote('up');
          }}
          disabled={submitFeedback.isPending}
        >
          <MessageSquare className="h-3.5 w-3.5" />
        </Button>
      </div>

      <Dialog open={showNoteDialog} onOpenChange={setShowNoteDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {selectedSentiment === 'up' ? '分享你的想法' : '告诉我们如何改进'}
            </DialogTitle>
            <DialogDescription>
              {selectedSentiment === 'up'
                ? '感谢支持!可选择性地分享更多细节'
                : '你的反馈将帮助我们改进功能'}
            </DialogDescription>
          </DialogHeader>

          <Textarea
            placeholder="输入你的意见... (可选)"
            value={note}
            onChange={(e) => setNote(e.target.value)}
            rows={4}
            className="resize-none"
          />

          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setShowNoteDialog(false);
                setNote('');
                setSelectedSentiment(null);
              }}
            >
              取消
            </Button>
            <Button onClick={handleSubmitNote} disabled={submitFeedback.isPending}>
              {submitFeedback.isPending ? '提交中...' : '提交反馈'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
