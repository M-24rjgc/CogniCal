import {
  Brain,
  CalendarDays,
  CheckSquare,
  HelpCircle,
  KeyRound,
  LayoutDashboard,
  Loader2,
  MessageSquare,
  Settings2,
  Target,
} from 'lucide-react';
import {
  Link,
  Navigate,
  Outlet,
  createHashRouter,
  useLocation,
  useNavigate,
} from 'react-router-dom';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useHotkeys } from 'react-hotkeys-hook';
import CommandPalette, { type CommandPaletteItem } from '../components/keyboard/CommandPalette';
import KeyboardShortcutsHelp, {
  type ShortcutGroup,
} from '../components/keyboard/KeyboardShortcutsHelp';
import { AppShell } from '../components/layout/AppShell';
import type { SidebarItem } from '../components/layout/Sidebar';
import { Badge } from '../components/ui/badge';
import { Button } from '../components/ui/button';
import { FOCUS_SEARCH_EVENT_NAME, KeyboardShortcutContext } from '../hooks/useKeyboardShortcuts';
import { useSettingsStore } from '../stores/settingsStore';
import { useAnalyticsStore } from '../stores/analyticsStore';
import { useTheme, type ThemeMode } from '../providers/theme-provider';
import CalendarPage from '../pages/Calendar';
import ChatPage from '../pages/Chat';
import DashboardPage from '../pages/Dashboard';
import GoalsPage from '../pages/Goals';
import MemoryManagementPage from '../pages/MemoryManagement';
import SettingsPage from '../pages/Settings';
import TasksPage from '../pages/Tasks';
import { HelpCenterDialog } from '../components/help/HelpCenterDialog';
import OnboardingOrchestrator from '../components/onboarding/OnboardingOrchestrator';

function RootLayout() {
  const navigate = useNavigate();
  const location = useLocation();

  const settings = useSettingsStore((state) => state.settings);
  const isSettingsLoading = useSettingsStore((state) => state.isLoading);
  const loadSettings = useSettingsStore((state) => state.loadSettings);
  const { theme, setTheme } = useTheme();

  const [isCommandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [isShortcutHelpOpen, setShortcutHelpOpen] = useState(false);
  const [isHelpCenterOpen, setHelpCenterOpen] = useState(false);

  const goSequenceRef = useRef(false);
  const goTimeoutRef = useRef<number | null>(null);

  const startGoSequence = useCallback(() => {
    goSequenceRef.current = true;
    if (goTimeoutRef.current !== null) {
      window.clearTimeout(goTimeoutRef.current);
    }
    goTimeoutRef.current = window.setTimeout(() => {
      goSequenceRef.current = false;
      goTimeoutRef.current = null;
    }, 1200);
  }, []);

  const consumeGoSequence = useCallback(() => {
    if (!goSequenceRef.current) {
      return false;
    }
    goSequenceRef.current = false;
    if (goTimeoutRef.current !== null) {
      window.clearTimeout(goTimeoutRef.current);
      goTimeoutRef.current = null;
    }
    return true;
  }, []);

  useEffect(() => {
    return () => {
      if (goTimeoutRef.current !== null) {
        window.clearTimeout(goTimeoutRef.current);
      }
    };
  }, []);

  const openCommandPalette = useCallback(() => setCommandPaletteOpen(true), []);
  const closeCommandPalette = useCallback(() => setCommandPaletteOpen(false), []);
  const toggleCommandPalette = useCallback(() => setCommandPaletteOpen((prev) => !prev), []);
  const openShortcutHelp = useCallback(() => setShortcutHelpOpen(true), []);
  const closeShortcutHelp = useCallback(() => setShortcutHelpOpen(false), []);
  const openHelpCenter = useCallback(() => setHelpCenterOpen(true), []);
  const closeHelpCenter = useCallback(() => setHelpCenterOpen(false), []);

  const triggerFocusSearch = useCallback(() => {
    window.dispatchEvent(new Event(FOCUS_SEARCH_EVENT_NAME));
  }, []);

  const shortcutContextValue = useMemo(
    () => ({
      isCommandPaletteOpen,
      openCommandPalette,
      closeCommandPalette,
      toggleCommandPalette,
      isShortcutHelpOpen,
      openShortcutHelp,
      closeShortcutHelp,
      isHelpCenterOpen,
      openHelpCenter,
      closeHelpCenter,
      triggerFocusSearch,
    }),
    [
      closeCommandPalette,
      closeHelpCenter,
      closeShortcutHelp,
      isCommandPaletteOpen,
      isHelpCenterOpen,
      isShortcutHelpOpen,
      openCommandPalette,
      openHelpCenter,
      openShortcutHelp,
      toggleCommandPalette,
      triggerFocusSearch,
    ],
  );

  const navigateWithReplace = useCallback(
    (path: string) => {
      navigate(path, { replace: false });
    },
    [navigate],
  );

  const navigateToSettingsFromHelp = useCallback(() => {
    closeHelpCenter();
    navigateWithReplace('/settings');
  }, [closeHelpCenter, navigateWithReplace]);

  const focusTaskSearch = useCallback(() => {
    if (location.pathname.startsWith('/tasks')) {
      triggerFocusSearch();
    } else {
      navigate('/tasks', { state: { intent: 'focus-search' } });
    }
  }, [location.pathname, navigate, triggerFocusSearch]);

  const handleCreateTask = useCallback(() => {
    navigate('/tasks', { state: { intent: 'create-task' } });
  }, [navigate]);

  const handleOpenPlanning = useCallback(() => {
    navigate('/tasks', { state: { intent: 'open-planning' } });
  }, [navigate]);

  const getNextTheme = useCallback((): ThemeMode => {
    const modes: ThemeMode[] = ['light', 'dark', 'system'];
    const currentIndex = Math.max(0, modes.indexOf(theme));
    return modes[(currentIndex + 1) % modes.length];
  }, [theme]);

  const handleCycleTheme = useCallback(() => {
    setTheme(getNextTheme());
  }, [getNextTheme, setTheme]);

  const isOverlayOpen = isCommandPaletteOpen || isShortcutHelpOpen || isHelpCenterOpen;

  const isEditableElement = (target: EventTarget | null) => {
    if (!(target instanceof HTMLElement)) return false;
    if (target.isContentEditable) return true;
    const tag = target.tagName;
    return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT';
  };

  useHotkeys(
    'ctrl+k,meta+k',
    (event) => {
      event.preventDefault();
      toggleCommandPalette();
    },
    [toggleCommandPalette],
  );

  useHotkeys(
    'ctrl+shift+p,meta+shift+p',
    (event) => {
      event.preventDefault();
      openCommandPalette();
    },
    [openCommandPalette],
  );

  useHotkeys(
    'ctrl+n,meta+n',
    (event) => {
      if (isOverlayOpen) return;
      event.preventDefault();
      handleCreateTask();
    },
    [handleCreateTask, isOverlayOpen],
  );

  useHotkeys(
    'ctrl+comma,meta+comma',
    (event) => {
      if (isOverlayOpen) return;
      event.preventDefault();
      navigateWithReplace('/settings');
    },
    [isOverlayOpen, navigateWithReplace],
  );

  useHotkeys(
    'shift+/',
    (event) => {
      if (isEditableElement(event.target)) return;
      if (isOverlayOpen && !isHelpCenterOpen) {
        event.preventDefault();
        return;
      }
      event.preventDefault();
      if (isHelpCenterOpen) {
        closeHelpCenter();
      } else {
        openHelpCenter();
      }
    },
    [closeHelpCenter, isHelpCenterOpen, isOverlayOpen, openHelpCenter],
  );

  useHotkeys(
    '/',
    (event) => {
      if (isOverlayOpen) return;
      if (isEditableElement(event.target)) return;
      event.preventDefault();
      focusTaskSearch();
    },
    [focusTaskSearch, isOverlayOpen],
  );

  useHotkeys(
    'ctrl+1,meta+1',
    (event) => {
      if (isOverlayOpen) return;
      event.preventDefault();
      navigateWithReplace('/');
    },
    [isOverlayOpen, navigateWithReplace],
  );

  useHotkeys(
    'ctrl+2,meta+2',
    (event) => {
      if (isOverlayOpen) return;
      event.preventDefault();
      navigateWithReplace('/tasks');
    },
    [isOverlayOpen, navigateWithReplace],
  );

  useHotkeys(
    'ctrl+3,meta+3',
    (event) => {
      if (isOverlayOpen) return;
      event.preventDefault();
      navigateWithReplace('/calendar');
    },
    [isOverlayOpen, navigateWithReplace],
  );

  useHotkeys(
    'ctrl+4,meta+4',
    (event) => {
      if (isOverlayOpen) return;
      event.preventDefault();
      navigateWithReplace('/goals');
    },
    [isOverlayOpen, navigateWithReplace],
  );

  useHotkeys(
    'ctrl+5,meta+5',
    (event) => {
      if (isOverlayOpen) return;
      event.preventDefault();
      navigateWithReplace('/chat');
    },
    [isOverlayOpen, navigateWithReplace],
  );

  useHotkeys(
    'ctrl+6,meta+6',
    (event) => {
      if (isOverlayOpen) return;
      event.preventDefault();
      navigateWithReplace('/memory');
    },
    [isOverlayOpen, navigateWithReplace],
  );

  useHotkeys(
    'ctrl+shift+l,meta+shift+l',
    (event) => {
      if (isEditableElement(event.target)) return;
      event.preventDefault();
      handleCycleTheme();
    },
    [handleCycleTheme],
  );

  useHotkeys(
    'esc',
    (event) => {
      if (goSequenceRef.current) {
        consumeGoSequence();
      }
      if (!isOverlayOpen) return;
      event.preventDefault();
      if (isCommandPaletteOpen) {
        closeCommandPalette();
      } else if (isShortcutHelpOpen) {
        closeShortcutHelp();
      } else if (isHelpCenterOpen) {
        closeHelpCenter();
      }
    },
    [
      closeCommandPalette,
      closeHelpCenter,
      closeShortcutHelp,
      consumeGoSequence,
      isCommandPaletteOpen,
      isHelpCenterOpen,
      isOverlayOpen,
      isShortcutHelpOpen,
    ],
  );

  useHotkeys(
    'g',
    (event) => {
      if (isOverlayOpen) return;
      if (isEditableElement(event.target)) return;
      startGoSequence();
      event.preventDefault();
    },
    [isOverlayOpen, startGoSequence],
  );

  useHotkeys(
    'd,t,c,g,a,m,s,p,h',
    (event) => {
      if (isOverlayOpen) return;
      if (!consumeGoSequence()) return;
      event.preventDefault();
      const key = event.key.toLowerCase();
      switch (key) {
        case 'd':
          navigateWithReplace('/');
          break;
        case 't':
          navigateWithReplace('/tasks');
          break;
        case 'c':
          navigateWithReplace('/calendar');
          break;
        case 'g':
          navigateWithReplace('/goals');
          break;
        case 'a':
          navigateWithReplace('/chat');
          break;
        case 'm':
          navigateWithReplace('/memory');
          break;
        case 's':
          navigateWithReplace('/settings');
          break;
        case 'p':
          handleOpenPlanning();
          break;
        case 'h':
          openHelpCenter();
          break;
        default:
          break;
      }
    },
    [consumeGoSequence, handleOpenPlanning, isOverlayOpen, navigateWithReplace, openHelpCenter],
  );

  const requestRef = useRef(false);
  const hasAttemptedRef = useRef(false);

  const hasDeepseekKey = settings?.hasDeepseekKey ?? false;
  const lastSettingsUpdated = settings?.lastUpdatedAt
    ? new Date(settings.lastUpdatedAt).toLocaleString('zh-CN')
    : null;

  useEffect(() => {
    if (settings || requestRef.current || hasAttemptedRef.current) {
      return;
    }

    const loadOnce = async () => {
      if (requestRef.current) {
        return;
      }
      requestRef.current = true;
      hasAttemptedRef.current = true;
      try {
        await loadSettings();
      } finally {
        requestRef.current = false;
      }
    };

    void loadOnce();
  }, [settings, loadSettings]);

  useEffect(() => {
    if (!settings?.themePreference) return;
    setTheme(settings.themePreference as ThemeMode);
  }, [settings?.themePreference, setTheme]);

  const phaseStatus = (
    <Badge
      variant={hasDeepseekKey ? 'secondary' : 'destructive'}
      className="flex items-center gap-1"
    >
      {isSettingsLoading ? (
        <Loader2 className="h-3.5 w-3.5 animate-spin" />
      ) : (
        <KeyRound className="h-3.5 w-3.5" />
      )}
      {isSettingsLoading
        ? '配置信息加载中'
        : hasDeepseekKey
          ? 'DeepSeek 已配置'
          : 'DeepSeek 未配置'}
    </Badge>
  );

  const lastAnalyticsRefreshed = useAnalyticsStore((state) => state.lastRefreshedAt);
  const isDemoData = useAnalyticsStore((state) => state.isDemoData);
  const isOnboardingComplete = useAnalyticsStore((state) => state.isOnboardingComplete);

  const analyticsStatusCopy = hasDeepseekKey
    ? lastAnalyticsRefreshed
      ? `最近刷新：${new Date(lastAnalyticsRefreshed).toLocaleString('zh-CN')}`
      : '智能分析已准备，就绪后将显示最新刷新时间。'
    : '配置 DeepSeek API Key 以启用 AI 洞察与自动化建议。';

  const sidebarFooter = (
    <div className="flex flex-col gap-2 rounded-xl border border-border/60 bg-muted/40 px-3 py-3">
      <div className="flex flex-wrap items-center gap-2 text-xs font-semibold text-foreground/80">
        <span>CogniCal v1.0</span>
        {isDemoData ? (
          <Badge variant="outline" className="text-[11px]">
            示例数据
          </Badge>
        ) : null}
        {lastSettingsUpdated ? (
          <Badge variant="muted" className="text-[11px]">
            设置于 {lastSettingsUpdated}
          </Badge>
        ) : null}
      </div>
      <span className="text-[11px] text-muted-foreground">{analyticsStatusCopy}</span>
      <div className="flex flex-wrap gap-2 pt-1">
        <Button asChild variant="outline" size="sm" className="h-8 px-3 text-[11px]">
          <Link to="/">查看仪表盘</Link>
        </Button>
        <Button asChild variant="ghost" size="sm" className="h-8 px-3 text-[11px]">
          <Link to="/settings">前往设置</Link>
        </Button>
        {isDemoData || !isOnboardingComplete ? (
          <Button asChild variant="secondary" size="sm" className="h-8 px-3 text-[11px]">
            <Link to="/">完成引导</Link>
          </Button>
        ) : null}
      </div>
    </div>
  );

  const commandPaletteItems = useMemo<CommandPaletteItem[]>(
    () => [
      {
        id: 'navigate-dashboard',
        label: '前往仪表盘',
        description: '查看实时效率洞察与任务总览',
        category: '导航',
        shortcut: 'Ctrl + 1',
        keywords: ['dashboard', '首页', '仪表盘'],
        action: () => navigateWithReplace('/'),
      },
      {
        id: 'navigate-tasks',
        label: '前往任务中心',
        description: '管理任务与智能规划',
        category: '导航',
        shortcut: 'Ctrl + 2',
        keywords: ['tasks', '任务'],
        action: () => navigateWithReplace('/tasks'),
      },
      {
        id: 'navigate-calendar',
        label: '前往日历',
        description: '查看任务安排与时间分布',
        category: '导航',
        shortcut: 'Ctrl + 3',
        keywords: ['calendar', '日历', '安排'],
        action: () => navigateWithReplace('/calendar'),
      },
      {
        id: 'navigate-goals',
        label: '前往目标管理',
        description: '管理目标与任务分解',
        category: '导航',
        shortcut: 'Ctrl + 4',
        keywords: ['goals', '目标', '分解'],
        action: () => navigateWithReplace('/goals'),
      },
      {
        id: 'navigate-chat',
        label: '前往 AI 对话',
        description: '与 AI 助手自由对话',
        category: '导航',
        shortcut: 'Ctrl + 5',
        keywords: ['chat', 'ai', '对话', '聊天'],
        action: () => navigateWithReplace('/chat'),
      },
      {
        id: 'navigate-memory',
        label: '前往记忆管理',
        description: '管理 AI 对话记忆与历史',
        category: '导航',
        shortcut: 'Ctrl + 6',
        keywords: ['memory', '记忆', '历史', '管理'],
        action: () => navigateWithReplace('/memory'),
      },
      {
        id: 'navigate-settings',
        label: '打开设置中心',
        description: '调整主题、集成与通知偏好',
        category: '导航',
        shortcut: 'Ctrl + ,',
        keywords: ['settings', '配置', '设置'],
        action: () => navigateWithReplace('/settings'),
      },
      {
        id: 'focus-task-search',
        label: '聚焦任务搜索',
        description: '快速定位任务或过滤条件',
        category: '任务',
        shortcut: '/',
        keywords: ['search', '搜索'],
        action: focusTaskSearch,
      },
      {
        id: 'create-task',
        label: '新建任务',
        description: '打开任务创建表单',
        category: '任务',
        shortcut: 'Ctrl + N',
        keywords: ['新建任务', 'create task'],
        action: handleCreateTask,
      },
      {
        id: 'open-planning',
        label: '打开规划中心',
        description: '定位到智能规划面板',
        category: '任务',
        shortcut: 'G 然后 P',
        keywords: ['规划', 'planning'],
        action: handleOpenPlanning,
      },
      {
        id: 'open-help-center',
        label: '打开帮助中心',
        description: '查看互动引导、快捷键与常见问题',
        category: '帮助',
        shortcut: 'Shift + /',
        keywords: ['帮助', 'help center'],
        action: openHelpCenter,
      },
      {
        id: 'open-shortcuts-help',
        label: '查看快捷键列表',
        description: '显示全局快捷键列表',
        category: '帮助',
        shortcut: 'G 然后 H',
        keywords: ['帮助', '快捷键'],
        action: openShortcutHelp,
      },
      {
        id: 'toggle-command-palette',
        label: '打开指令面板',
        description: '手动打开或关闭指令面板',
        category: '帮助',
        shortcut: 'Ctrl + K',
        keywords: ['命令', 'palette'],
        action: toggleCommandPalette,
      },
      {
        id: 'cycle-theme',
        label: '切换主题模式',
        description: '在亮色、暗色与跟随系统之间切换',
        category: '外观',
        shortcut: 'Ctrl + Shift + L',
        keywords: ['主题', '外观', 'theme'],
        action: handleCycleTheme,
      },
    ],
    [
      focusTaskSearch,
      handleCreateTask,
      handleCycleTheme,
      handleOpenPlanning,
      navigateWithReplace,
      openHelpCenter,
      openShortcutHelp,
      toggleCommandPalette,
    ],
  );

  const shortcutGroups = useMemo<ShortcutGroup[]>(
    () => [
      {
        title: '全局',
        shortcuts: [
          { keys: 'Ctrl / Cmd + K', description: '打开或关闭指令面板' },
          { keys: 'Ctrl / Cmd + Shift + P', description: '强制打开指令面板' },
          { keys: 'Shift + /', description: '打开帮助中心' },
          { keys: 'Esc', description: '退出当前打开的叠加面板' },
        ],
      },
      {
        title: '导航',
        shortcuts: [
          { keys: 'Ctrl / Cmd + 1', description: '前往仪表盘' },
          { keys: 'Ctrl / Cmd + 2', description: '前往任务中心' },
          { keys: 'Ctrl / Cmd + 3', description: '前往日历' },
          { keys: 'Ctrl / Cmd + 4', description: '前往目标管理' },
          { keys: 'Ctrl / Cmd + 5', description: '前往 AI 对话' },
          { keys: 'Ctrl / Cmd + 6', description: '前往记忆管理' },
          { keys: 'Ctrl / Cmd + ,', description: '打开设置中心' },
          { keys: 'G 然后 D / T / C / G / A / M / S', description: '使用 Go 序列跳转目标页面' },
        ],
      },
      {
        title: '任务操作',
        shortcuts: [
          { keys: 'Ctrl / Cmd + N', description: '新建任务' },
          { keys: '/', description: '聚焦任务搜索' },
          { keys: 'G 然后 P', description: '打开智能规划中心' },
        ],
      },
      {
        title: '外观与帮助',
        shortcuts: [
          { keys: 'Ctrl / Cmd + Shift + L', description: '切换主题模式' },
          { keys: 'G 然后 H', description: '打开帮助中心' },
        ],
      },
    ],
    [],
  );

  const navItems = useMemo<SidebarItem[]>(
    () => [
      {
        key: 'dashboard',
        label: '仪表盘',
        icon: LayoutDashboard,
        to: '/',
      },
      {
        key: 'tasks',
        label: '任务中心',
        icon: CheckSquare,
        to: '/tasks',
      },
      {
        key: 'calendar',
        label: '日历视图',
        icon: CalendarDays,
        to: '/calendar',
      },
      {
        key: 'goals',
        label: '目标管理',
        icon: Target,
        to: '/goals',
      },
      {
        key: 'chat',
        label: 'AI 对话',
        icon: MessageSquare,
        to: '/chat',
      },
      {
        key: 'memory',
        label: '记忆管理',
        icon: Brain,
        to: '/memory',
      },
      {
        key: 'settings',
        label: '设置中心',
        icon: Settings2,
        to: '/settings',
      },
      {
        key: 'help-center',
        label: '帮助中心',
        icon: HelpCircle,
        onSelect: openHelpCenter,
        isActive: isHelpCenterOpen,
        description: '查看引导、快捷键与文档',
      },
    ],
    [isHelpCenterOpen, openHelpCenter],
  );

  return (
    <KeyboardShortcutContext.Provider value={shortcutContextValue}>
      <AppShell
        sidebarItems={navItems}
        sidebarFooter={sidebarFooter}
        phaseLabel="智能任务与时间管理"
        phaseStatus={phaseStatus}
      >
        <Outlet />
      </AppShell>

      <CommandPalette
        open={isCommandPaletteOpen}
        onOpenChange={setCommandPaletteOpen}
        commands={commandPaletteItems}
      />

      <KeyboardShortcutsHelp
        open={isShortcutHelpOpen}
        onOpenChange={setShortcutHelpOpen}
        groups={shortcutGroups}
      />

      <HelpCenterDialog
        open={isHelpCenterOpen}
        onOpenChange={setHelpCenterOpen}
        shortcutGroups={shortcutGroups}
        onOpenShortcuts={openShortcutHelp}
        onNavigateToSettings={navigateToSettingsFromHelp}
      />

      <OnboardingOrchestrator />
    </KeyboardShortcutContext.Provider>
  );
}

export const router = createHashRouter([
  {
    path: '/',
    element: <RootLayout />,
    children: [
      { index: true, element: <DashboardPage /> },
      { path: 'tasks', element: <TasksPage /> },
      { path: 'calendar', element: <CalendarPage /> },
      { path: 'goals', element: <GoalsPage /> },
      { path: 'chat', element: <ChatPage /> },
      { path: 'memory', element: <MemoryManagementPage /> },
      { path: 'settings', element: <SettingsPage /> },
      { path: '*', element: <Navigate to="/" replace /> },
    ],
  },
]);

export type AppRoute = 'dashboard' | 'tasks' | 'calendar' | 'chat' | 'memory' | 'settings';
