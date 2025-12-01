import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ReactFlowProvider } from '@xyflow/react';
import { DependencyGraph } from '../DependencyGraph';
import { Task } from '../../../types/task';
import { TaskDependency } from '../../../types/dependency';

// Mock ResizeObserver
beforeAll(() => {
  global.ResizeObserver = vi.fn().mockImplementation(() => ({
    observe: vi.fn(),
    unobserve: vi.fn(),
    disconnect: vi.fn(),
  }));
});

// Mock the hooks
vi.mock('../../../hooks/useDependencies', () => ({
  useDependencies: () => ({
    dependencies: [],
    dependencyGraph: null,
    readyTasks: [],
    isLoading: false,
    isMutating: false,
    fetchDependencies: vi.fn(),
    fetchDependencyGraph: vi.fn(),
    fetchReadyTasks: vi.fn(),
    createDependency: vi.fn(),
    deleteDependency: vi.fn(),
    updateDependencyType: vi.fn(),
    validateDependency: vi.fn(),
  }),
}));

// Mock React Flow hooks
vi.mock('@xyflow/react', async () => {
  const actual = await vi.importActual('@xyflow/react');
  return {
    ...actual,
    useReactFlow: () => ({
      fitView: vi.fn(),
      zoomIn: vi.fn(),
      zoomOut: vi.fn(),
    }),
  };
});

// Mock pushToast
vi.mock('../../../stores/uiStore', () => ({
  pushToast: vi.fn(),
}));

const mockTasks: Task[] = [
  {
    id: '1',
    title: 'Task 1',
    description: 'First task',
    status: 'todo',
    priority: 'medium',
    tags: [],
    createdAt: '2025-01-01T00:00:00Z',
    updatedAt: '2025-01-01T00:00:00Z',
    isRecurring: false,
  },
  {
    id: '2',
    title: 'Task 2',
    description: 'Second task',
    status: 'todo',
    priority: 'high',
    tags: [],
    createdAt: '2025-01-01T00:00:00Z',
    updatedAt: '2025-01-01T00:00:00Z',
    isRecurring: false,
  },
];

const mockDependencies: TaskDependency[] = [
  {
    id: 'dep1',
    predecessorId: '1',
    successorId: '2',
    dependencyType: 'finish_to_start',
    createdAt: '2025-01-01T00:00:00Z',
  },
];

describe('DependencyGraph', () => {
  it('renders without crashing', () => {
    render(
      <ReactFlowProvider>
        <DependencyGraph
          tasks={mockTasks}
          dependencies={mockDependencies}
        />
      </ReactFlowProvider>
    );

    expect(screen.getByText('任务依赖图')).toBeDefined();
  });

  it('displays task statistics', () => {
    render(
      <ReactFlowProvider>
        <DependencyGraph
          tasks={mockTasks}
          dependencies={mockDependencies}
        />
      </ReactFlowProvider>
    );

    expect(screen.getByText('2 任务')).toBeDefined();
    expect(screen.getByText('1 依赖')).toBeDefined();
  });

  it('shows search input', () => {
    render(
      <ReactFlowProvider>
        <DependencyGraph
          tasks={mockTasks}
          dependencies={mockDependencies}
        />
      </ReactFlowProvider>
    );

    expect(screen.getByPlaceholderText('搜索任务...')).toBeDefined();
  });

  it('shows filter controls', () => {
    render(
      <ReactFlowProvider>
        <DependencyGraph
          tasks={mockTasks}
          dependencies={mockDependencies}
        />
      </ReactFlowProvider>
    );

    expect(screen.getByText('显示已完成')).toBeDefined();
    expect(screen.getByText('高亮关键路径')).toBeDefined();
    expect(screen.getByText('高亮可执行任务')).toBeDefined();
  });

  it('shows layout algorithm controls', () => {
    render(
      <ReactFlowProvider>
        <DependencyGraph
          tasks={mockTasks}
          dependencies={mockDependencies}
        />
      </ReactFlowProvider>
    );

    expect(screen.getByText('Dagre')).toBeDefined();
    expect(screen.getByText('ELK')).toBeDefined();
  });

  it('shows layout direction controls', () => {
    render(
      <ReactFlowProvider>
        <DependencyGraph
          tasks={mockTasks}
          dependencies={mockDependencies}
        />
      </ReactFlowProvider>
    );

    // Check for layout direction buttons (they have titles)
    expect(screen.getByTitle('Top to Bottom')).toBeDefined();
    expect(screen.getByTitle('Left to Right')).toBeDefined();
    expect(screen.getByTitle('Bottom to Top')).toBeDefined();
    expect(screen.getByTitle('Right to Left')).toBeDefined();
  });
});