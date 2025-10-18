import '@testing-library/jest-dom/vitest';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import type { DashboardConfig } from '../types/dashboard';
import DashboardPage from '../pages/Dashboard';

const { useDashboardConfigMock, usePendingNudgeMock } = vi.hoisted(() => ({
  useDashboardConfigMock: vi.fn(),
  usePendingNudgeMock: vi.fn(),
}));

vi.mock('../hooks/useDashboardConfig', () => ({
  useDashboardConfig: useDashboardConfigMock,
}));

vi.mock('../hooks/useWellness', () => ({
  usePendingNudge: usePendingNudgeMock,
}));

vi.mock('../components/wellness/WellnessNudgeToast', () => ({
  WellnessNudgeToast: () => null,
}));

vi.mock('../components/dashboard/moduleRegistry', () => {
  const createModule = (id: string) => ({
    component: () => <div data-testid={`module-${id}`}>{id}</div>,
  });

  return {
    dashboardModuleRegistry: {
      'today-tasks': createModule('today-tasks'),
      'analytics-overview': createModule('analytics-overview'),
      'wellness-summary': createModule('wellness-summary'),
      'workload-forecast': createModule('workload-forecast'),
    },
  };
});

const renderDashboard = (config: DashboardConfig) => {
  useDashboardConfigMock.mockReturnValue({
    config,
  });
  usePendingNudgeMock.mockReturnValue({ data: null });

  render(
    <MemoryRouter>
      <DashboardPage />
    </MemoryRouter>,
  );
};

describe('DashboardPage integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders quick actions and enabled modules according to configuration', () => {
    renderDashboard({
      modules: {
        'quick-actions': true,
        'today-tasks': true,
        'analytics-overview': true,
        'wellness-summary': false,
        'workload-forecast': false,
        'productivity-lite': false,
        'upcoming-alerts': false,
      },
      lastUpdatedAt: null,
    });

    expect(screen.getByRole('heading', { name: '快捷操作' })).toBeInTheDocument();
    expect(screen.getByTestId('module-today-tasks')).toBeInTheDocument();
    expect(screen.getByTestId('module-analytics-overview')).toBeInTheDocument();
    expect(screen.queryByTestId('module-workload-forecast')).not.toBeInTheDocument();
  });

  it('shows empty state when all modules are disabled', () => {
    renderDashboard({
      modules: {
        'quick-actions': false,
        'today-tasks': false,
        'analytics-overview': false,
        'wellness-summary': false,
        'workload-forecast': false,
        'productivity-lite': false,
        'upcoming-alerts': false,
      },
      lastUpdatedAt: null,
    });

    expect(screen.queryByRole('heading', { name: '快捷操作' })).not.toBeInTheDocument();
    expect(
      screen.getByText('当前未启用任何仪表盘模块，可在设置 > 仪表盘配置中重新选择需要展示的内容。'),
    ).toBeInTheDocument();
  });
});
