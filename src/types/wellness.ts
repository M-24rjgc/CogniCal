// Wellness Nudge Types
export type WellnessTriggerReason = 'FocusStreak' | 'WorkStreak';
export type WellnessResponse = 'Completed' | 'Snoozed' | 'Ignored';

export interface WellnessEventRecord {
  id: number;
  window_start: string;
  trigger_reason: WellnessTriggerReason;
  recommended_break_minutes: number;
  suggested_micro_task?: string | null;
  response: WellnessResponse | null;
  response_at: string | null;
  deferral_count: number;
}

export interface WeeklySummary {
  week_start: string;
  week_end: string;
  total_nudges: number;
  completed_count: number;
  snoozed_count: number;
  ignored_count: number;
  average_focus_minutes: number;
  max_work_streak_hours: number;
  rest_compliance_rate: number;
  focus_rhythm_score: number;
  peak_hours: number[];
  recommendations: string[];
}

export interface RespondToNudgeInput {
  eventId: number;
  response: WellnessResponse;
}

// Wellness Nudge Display
export interface WellnessNudge {
  id: number;
  trigger_reason: WellnessTriggerReason;
  triggered_at: string;
  recommended_break_minutes: number;
  message: string;
}
