import type { KeyboardShortcutContextValue } from '../hooks/useKeyboardShortcuts';

export const ONBOARDING_REPLAY_EVENT = 'onboarding:replay';

export type OnboardingStepId =
  | 'dashboard-overview'
  | 'task-quick-create'
  | 'ai-parse-panel'
  | 'planning-center'
  | 'settings-api-key';

export interface OnboardingStepDefinition {
  id: OnboardingStepId;
  selector: string;
  title: string;
  description: string;
  placement?: 'top' | 'bottom' | 'left' | 'right';
}

export const ONBOARDING_TOUR_STEPS: readonly OnboardingStepDefinition[] = [
  {
    id: 'dashboard-overview',
    selector: '[data-onboarding="dashboard-overview"]',
    title: '掌握今日工作全貌',
    description: '首页仪表盘汇总今日任务、生产力评分和预警信息，是开启工作流程的第一站。',
    placement: 'bottom',
  },
  {
    id: 'task-quick-create',
    selector: '[data-onboarding="task-quick-create"], [data-action-id="create-task"]',
    title: '快速创建并规划任务',
    description: '点击“新建任务”或使用 AI 解析输入，即可把灵感转成结构化任务并进入规划流程。',
    placement: 'left',
  },
  {
    id: 'ai-parse-panel',
    selector: '[data-onboarding="ai-parse-panel"]',
    title: 'AI 助你拆解复杂需求',
    description: '自然语言描述交给 DeepSeek，系统会自动给出字段建议、复杂度评估和执行提示。',
    placement: 'left',
  },
  {
    id: 'planning-center',
    selector: '[data-onboarding="planning-center"]',
    title: '智能规划中心',
    description: '在此查看 AI 生成的执行方案、冲突解决建议和可行的时间块安排。',
    placement: 'top',
  },
  {
    id: 'settings-api-key',
    selector: '[data-onboarding="settings-api-key"]',
    title: '完成 DeepSeek 配置',
    description: '前往设置页配置 DeepSeek API Key，才能启用解析、规划与分析等核心 AI 能力。',
    placement: 'left',
  },
] as const;

export const ONBOARDING_STEP_IDS = ONBOARDING_TOUR_STEPS.map((step) => step.id);
const onboardingStepIdSet = new Set<OnboardingStepId>(ONBOARDING_STEP_IDS);

export const isOnboardingStepId = (value: string): value is OnboardingStepId =>
  onboardingStepIdSet.has(value as OnboardingStepId);

export const getOnboardingStepById = (id: OnboardingStepId): OnboardingStepDefinition | undefined =>
  ONBOARDING_TOUR_STEPS.find((step) => step.id === id);

export const getFirstOnboardingStepId = (): OnboardingStepId => ONBOARDING_TOUR_STEPS[0]!.id;

export interface HelpResourceLink {
  label: string;
  href: string;
  external?: boolean;
}

export interface ContextualHelpEntry {
  id: string;
  title: string;
  description: string;
  links?: HelpResourceLink[];
  relatedStepId?: OnboardingStepId;
  relatedShortcutId?: string;
}

export const CONTEXTUAL_HELP_ENTRIES: Record<string, ContextualHelpEntry> = {
  'dashboard-overview': {
    id: 'dashboard-overview',
    title: '仪表盘概览',
    description: '仪表盘展示今日任务、生产力评分与风险预警。可在设置-仪表盘显示中自定义模块。',
    relatedStepId: 'dashboard-overview',
    links: [
      { label: '了解仪表盘模块', href: '#help/dashboard-modules' },
      { label: '自定义显示内容', href: '/settings?tab=dashboard' },
    ],
  },
  'tasks-ai-panel': {
    id: 'tasks-ai-panel',
    title: 'AI 任务解析',
    description: '使用自然语言描述任务，AI 会补全字段并给出复杂度、时间建议。',
    relatedStepId: 'ai-parse-panel',
    links: [{ label: 'AI 解析使用技巧', href: '#help/ai-parsing' }],
  },
  'planning-center': {
    id: 'planning-center',
    title: '智能规划中心',
    description: '对比多套计划方案，查看风险评估，并一键应用推荐排程。',
    relatedStepId: 'planning-center',
    links: [{ label: '规划中心常见问题', href: '#help/planning-faq' }],
  },
  'settings-api-key': {
    id: 'settings-api-key',
    title: '配置 DeepSeek API Key',
    description: '没有 API Key 时，AI 功能会受限。请使用个人密钥并妥善保管。',
    relatedStepId: 'settings-api-key',
    links: [{ label: '前往设置', href: '/settings?tab=ai' }],
  },
};

export const HELP_ENTRY_IDS = Object.keys(CONTEXTUAL_HELP_ENTRIES);

export const dispatchOnboardingReplayEvent = () => {
  if (typeof window === 'undefined') return;
  window.dispatchEvent(new CustomEvent(ONBOARDING_REPLAY_EVENT));
};

export type OverlayAwareShortcuts = Pick<
  KeyboardShortcutContextValue,
  'isCommandPaletteOpen' | 'isShortcutHelpOpen' | 'isHelpCenterOpen'
>;

export const isOverlayActive = (context: OverlayAwareShortcuts): boolean =>
  context.isCommandPaletteOpen || context.isShortcutHelpOpen || context.isHelpCenterOpen;
