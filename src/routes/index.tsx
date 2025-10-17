import {
  CalendarDays,
  CheckSquare,
  KeyRound,
  LayoutDashboard,
  Loader2,
  Settings2,
} from 'lucide-react';
import { Outlet, createHashRouter, Navigate, Link } from 'react-router-dom';
import { useEffect, useMemo, useRef } from 'react';
import { AppShell } from '../components/layout/AppShell';
import type { SidebarItem } from '../components/layout/Sidebar';
import { Badge } from '../components/ui/badge';
import { Button } from '../components/ui/button';
import { useSettingsStore } from '../stores/settingsStore';
import { useAnalyticsStore } from '../stores/analyticsStore';
import { useTheme, type ThemeMode } from '../providers/theme-provider';
import CalendarPage from '../pages/Calendar';
import DashboardPage from '../pages/Dashboard';
import SettingsPage from '../pages/Settings';
import TasksPage from '../pages/Tasks';

function RootLayout() {
  const navItems: SidebarItem[] = useMemo(
    () => [
      { key: 'dashboard', label: '仪表盘', to: '/', icon: LayoutDashboard },
      { key: 'tasks', label: '任务', to: '/tasks', icon: CheckSquare },
      { key: 'calendar', label: '日历', to: '/calendar', icon: CalendarDays },
      { key: 'settings', label: '设置', to: '/settings', icon: Settings2 },
    ],
    [],
  );

  const settings = useSettingsStore((state) => state.settings);
  const isSettingsLoading = useSettingsStore((state) => state.isLoading);
  const loadSettings = useSettingsStore((state) => state.loadSettings);
  const { setTheme } = useTheme();

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

  return (
    <AppShell
      sidebarItems={navItems}
      sidebarFooter={sidebarFooter}
      phaseLabel="智能任务与时间管理"
      phaseStatus={phaseStatus}
    >
      <Outlet />
    </AppShell>
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
      { path: 'settings', element: <SettingsPage /> },
      { path: '*', element: <Navigate to="/" replace /> },
    ],
  },
]);

export type AppRoute = 'dashboard' | 'tasks' | 'calendar' | 'settings';
