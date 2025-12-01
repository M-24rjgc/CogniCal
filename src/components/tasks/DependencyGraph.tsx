import React, { useCallback, useEffect, useMemo, useState, useRef } from 'react';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  Edge,
  Node,
  ConnectionMode,
  Panel,
  useReactFlow,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import dagre from 'dagre';
import ELK from 'elkjs/lib/elk.bundled.js';
import { TaskNode } from './TaskNode';
import { DependencyEdge } from './DependencyEdge';
import { Button } from '../ui/button';
import { Badge } from '../ui/badge';
import { Card } from '../ui/card';
import { 
  LayoutGrid, 
  ZoomIn, 
  ZoomOut, 
  Maximize, 
  RefreshCw,
  AlertTriangle,
  CheckCircle2,
  Search,
  ArrowDown,
  ArrowRight,
  Shuffle,
  Target,
  Route
} from 'lucide-react';
import { Input } from '../ui/input';
import { cn } from '../../lib/utils';
import { useDependencies } from '../../hooks/useDependencies';
import { Task } from '../../types/task';
import { 
  TaskDependency, 
  DependencyCreateInput, 
  DependencyType,
  GraphNode, 
  GraphEdge,
  DependencyFilter,
  DependencyValidation
} from '../../types/dependency';
import { pushToast } from '../../stores/uiStore';
import { ConnectionValidationFeedback } from './ConnectionValidationFeedback';
import { DragConnectionOverlay } from './DragConnectionOverlay';
import { BulkDependencyOperations } from './BulkDependencyOperations';
import { 
  useGraphVirtualization, 
  GraphPerformanceMonitor,
  useProgressiveRendering 
} from './GraphVirtualization';

// Custom node types
const nodeTypes: any = {
  taskNode: TaskNode,
};

// Custom edge types  
const edgeTypes: any = {
  dependencyEdge: DependencyEdge,
};

interface DependencyGraphProps {
  tasks: Task[];
  dependencies: TaskDependency[];
  onAddDependency?: (from: string, to: string) => void;
  onRemoveDependency?: (dependencyId: string) => void;
  onTaskClick?: (taskId: string) => void;
  className?: string;
}

interface ConnectionValidationState {
  isValidating: boolean;
  validation: DependencyValidation | null;
  pendingConnection: Connection | null;
}

// Layout algorithms
type LayoutAlgorithm = 'dagre' | 'elk';
type LayoutDirection = 'TB' | 'BT' | 'LR' | 'RL';

// Dagre layout algorithm
const getDagreLayoutedElements = (
  nodes: Node[],
  edges: Edge[],
  direction: LayoutDirection = 'TB'
): { nodes: Node[]; edges: Edge[] } => {
  const dagreGraph = new dagre.graphlib.Graph();
  dagreGraph.setDefaultEdgeLabel(() => ({}));
  dagreGraph.setGraph({ 
    rankdir: direction,
    nodesep: 100,
    ranksep: 150,
    marginx: 50,
    marginy: 50,
  });

  nodes.forEach((node) => {
    dagreGraph.setNode(node.id, { width: 280, height: 120 });
  });

  edges.forEach((edge) => {
    dagreGraph.setEdge(edge.source, edge.target);
  });

  dagre.layout(dagreGraph);

  const layoutedNodes = nodes.map((node) => {
    const nodeWithPosition = dagreGraph.node(node.id);
    return {
      ...node,
      position: {
        x: nodeWithPosition.x - 140, // Center the node
        y: nodeWithPosition.y - 60,
      },
    };
  });

  return { nodes: layoutedNodes, edges };
};

// ELK layout algorithm
const getElkLayoutedElements = async (
  nodes: Node[],
  edges: Edge[],
  direction: LayoutDirection = 'TB'
): Promise<{ nodes: Node[]; edges: Edge[] }> => {
  const elk = new ELK();
  
  const elkNodes = nodes.map((node) => ({
    id: node.id,
    width: 280,
    height: 120,
  }));

  const elkEdges = edges.map((edge) => ({
    id: edge.id,
    sources: [edge.source],
    targets: [edge.target],
  }));

  const elkDirection = direction === 'TB' ? 'DOWN' : 
                     direction === 'BT' ? 'UP' :
                     direction === 'LR' ? 'RIGHT' : 'LEFT';

  const elkGraph = {
    id: 'root',
    layoutOptions: {
      'elk.algorithm': 'layered',
      'elk.direction': elkDirection,
      'elk.spacing.nodeNode': '100',
      'elk.layered.spacing.nodeNodeBetweenLayers': '150',
      'elk.padding': '[top=50,left=50,bottom=50,right=50]',
      'elk.layered.crossingMinimization.strategy': 'LAYER_SWEEP',
      'elk.layered.nodePlacement.strategy': 'BRANDES_KOEPF',
    },
    children: elkNodes,
    edges: elkEdges,
  };

  try {
    const layoutedGraph = await elk.layout(elkGraph);
    
    const layoutedNodes = nodes.map((node) => {
      const elkNode = layoutedGraph.children?.find((n) => n.id === node.id);
      return {
        ...node,
        position: {
          x: elkNode?.x ?? node.position.x,
          y: elkNode?.y ?? node.position.y,
        },
      };
    });

    return { nodes: layoutedNodes, edges };
  } catch (error) {
    console.error('ELK layout failed, falling back to dagre:', error);
    return getDagreLayoutedElements(nodes, edges, direction);
  }
};

// Generic layout function
const getLayoutedElements = async (
  nodes: Node[],
  edges: Edge[],
  algorithm: LayoutAlgorithm = 'dagre',
  direction: LayoutDirection = 'TB'
): Promise<{ nodes: Node[]; edges: Edge[] }> => {
  if (algorithm === 'elk') {
    return await getElkLayoutedElements(nodes, edges, direction);
  } else {
    return getDagreLayoutedElements(nodes, edges, direction);
  }
};

export const DependencyGraph: React.FC<DependencyGraphProps> = ({
  tasks,
  dependencies,
  onAddDependency,
  onRemoveDependency,
  onTaskClick,
  className,
}) => {
  const { 
    dependencyGraph,
    isLoading,
    fetchDependencyGraph,
    createDependency,
    deleteDependency,
    updateDependencyType,
    validateDependency,
  } = useDependencies();

  const { fitView, zoomIn, zoomOut } = useReactFlow();
  
  const [searchTerm, setSearchTerm] = useState('');
  const [showCompleted, setShowCompleted] = useState(false);
  const [highlightCriticalPath, setHighlightCriticalPath] = useState(false);
  const [layoutAlgorithm, setLayoutAlgorithm] = useState<LayoutAlgorithm>('dagre');
  const [layoutDirection, setLayoutDirection] = useState<LayoutDirection>('TB');
  const [showReadyTasks, setShowReadyTasks] = useState(true);
  const [isLayouting, setIsLayouting] = useState(false);
  const [connectionValidation, setConnectionValidation] = useState<ConnectionValidationState>({
    isValidating: false,
    validation: null,
    pendingConnection: null,
  });
  const [isConnecting, setIsConnecting] = useState(false);
  const [connectingFromNode, setConnectingFromNode] = useState<string | null>(null);
  const [draggedConnection, setDraggedConnection] = useState<{
    sourceX: number;
    sourceY: number;
    targetX: number;
    targetY: number;
  } | null>(null);
  const [hoveredNode, setHoveredNode] = useState<string | null>(null);
  const [selectedEdge, setSelectedEdge] = useState<string | null>(null);
  const [selectedEdges, setSelectedEdges] = useState<string[]>([]);
  const [isMultiSelectMode, setIsMultiSelectMode] = useState(false);
  const [enableVirtualization, setEnableVirtualization] = useState(true);
  const [enableProgressiveRendering, setEnableProgressiveRendering] = useState(false);

  // Initialize with empty arrays first
  const [nodes, setNodes, onNodesChange] = useNodesState<GraphNode>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<GraphEdge>([]);

  // Apply virtualization for large graphs
  const virtualizedGraph = useGraphVirtualization({
    nodes,
    edges,
    config: {
      enabled: enableVirtualization,
      threshold: 500,
      viewportPadding: 200,
    },
  });

  // Apply progressive rendering if enabled
  const progressiveRender = useProgressiveRendering({
    nodes: virtualizedGraph.visibleNodes,
    edges: virtualizedGraph.visibleEdges,
    batchSize: 50,
    enabled: enableProgressiveRendering && nodes.length > 200,
  });

  // Use virtualized or progressive nodes/edges
  const displayNodes = (enableProgressiveRendering && nodes.length > 200
    ? progressiveRender.renderedNodes
    : virtualizedGraph.visibleNodes) as GraphNode[];
  
  const displayEdges = (enableProgressiveRendering && nodes.length > 200
    ? progressiveRender.renderedEdges
    : virtualizedGraph.visibleEdges) as GraphEdge[];

  // Handle dependency deletion
  const handleDeleteDependency = useCallback(async (dependencyId: string) => {
    try {
      await deleteDependency(dependencyId);
      setEdges(eds => eds.filter(edge => edge.id !== dependencyId));
      onRemoveDependency?.(dependencyId);
    } catch (error) {
      console.error('Failed to delete dependency:', error);
    }
  }, [deleteDependency, onRemoveDependency, setEdges]);

  // Handle dependency type update
  const handleUpdateDependencyType = useCallback(async (dependencyId: string, newType: DependencyType) => {
    try {
      await updateDependencyType(dependencyId, newType);
      
      // Update the edge in the local state
      setEdges(eds => eds.map(edge => {
        if (edge.id === dependencyId && edge.data) {
          return {
            ...edge,
            data: {
              ...edge.data,
              dependency: {
                ...edge.data.dependency,
                dependencyType: newType,
              },
            },
          };
        }
        return edge;
      }));
    } catch (error) {
      console.error('Failed to update dependency type:', error);
    }
  }, [updateDependencyType, setEdges]);

  // Handle bulk operations
  const handleBulkDelete = useCallback(async (dependencyIds: string[]) => {
    try {
      await Promise.all(dependencyIds.map(id => deleteDependency(id)));
      setEdges(eds => eds.filter(edge => !dependencyIds.includes(edge.id)));
      setSelectedEdges([]);
      setIsMultiSelectMode(false);
    } catch (error) {
      console.error('Failed to delete dependencies:', error);
    }
  }, [deleteDependency, setEdges]);

  const handleBulkTypeUpdate = useCallback(async (dependencyIds: string[], newType: DependencyType) => {
    try {
      await Promise.all(dependencyIds.map(id => updateDependencyType(id, newType)));
      setEdges(eds => eds.map(edge => {
        if (dependencyIds.includes(edge.id) && edge.data) {
          return {
            ...edge,
            data: {
              ...edge.data,
              dependency: {
                ...edge.data.dependency,
                dependencyType: newType,
              },
            },
          };
        }
        return edge;
      }));
      setSelectedEdges([]);
      setIsMultiSelectMode(false);
    } catch (error) {
      console.error('Failed to update dependency types:', error);
    }
  }, [updateDependencyType, setEdges]);

  // Use refs to avoid dependency issues in useEffect
  const deleteHandlerRef = useRef(handleDeleteDependency);
  deleteHandlerRef.current = handleDeleteDependency;
  
  const updateTypeHandlerRef = useRef(handleUpdateDependencyType);
  updateTypeHandlerRef.current = handleUpdateDependencyType;

  // Handle keyboard shortcuts
  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    if (event.key === 'Escape' && isConnecting) {
      setIsConnecting(false);
      setConnectingFromNode(null);
      setDraggedConnection(null);
      setHoveredNode(null);
    }
    
    if ((event.key === 'Delete' || event.key === 'Backspace')) {
      event.preventDefault();
      if (selectedEdges.length > 0) {
        if (window.confirm(`确定要删除选中的 ${selectedEdges.length} 个依赖关系吗？`)) {
          selectedEdges.forEach(edgeId => handleDeleteDependency(edgeId));
          setSelectedEdges([]);
        }
      } else if (selectedEdge) {
        if (window.confirm('确定要删除选中的依赖关系吗？')) {
          handleDeleteDependency(selectedEdge);
          setSelectedEdge(null);
        }
      }
    }
  }, [isConnecting, selectedEdges, selectedEdge, handleDeleteDependency]);

  // Add keyboard event listeners
  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleKeyDown]);

  // Convert tasks and dependencies to React Flow format
  // Process tasks and dependencies into React Flow format
  useEffect(() => {
    const filteredTasks = tasks.filter(task => {
      const matchesSearch = !searchTerm || 
        task.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
        task.description?.toLowerCase().includes(searchTerm.toLowerCase());
      
      const includeCompleted = showCompleted || task.status !== 'done';
      
      return matchesSearch && includeCompleted;
    });

    const taskMap = new Map(filteredTasks.map(task => [task.id, task]));
    
    // Create nodes
    const nodes: GraphNode[] = filteredTasks.map((task, index) => {
      const taskDependencies = dependencies
        .filter(dep => dep.successorId === task.id)
        .map(dep => dep.predecessorId);
      
      const taskDependents = dependencies
        .filter(dep => dep.predecessorId === task.id)
        .map(dep => dep.successorId);

      const isReady = taskDependencies.every(depId => {
        const depTask = taskMap.get(depId);
        return depTask?.status === 'done';
      });

      const isOnCriticalPath = highlightCriticalPath && 
        dependencyGraph?.criticalPath?.includes(task.id) || false;
      
      const isBlocked = !isReady && task.status !== 'done';

      return {
        id: task.id,
        type: 'taskNode',
        position: { x: (index % 4) * 300, y: Math.floor(index / 4) * 150 },
        data: {
          task,
          isReady,
          isOnCriticalPath,
          isBlocked,
          dependencies: taskDependencies,
          dependents: taskDependents,
        },
      };
    });

    // Create edges
    const edges: GraphEdge[] = dependencies
      .filter(dep => 
        taskMap.has(dep.predecessorId) && taskMap.has(dep.successorId)
      )
      .map(dep => ({
        id: dep.id,
        source: dep.predecessorId,
        target: dep.successorId,
        type: 'dependencyEdge',
        selected: selectedEdges.includes(dep.id),
        data: {
          dependency: dep,
          onDelete: deleteHandlerRef.current,
          onUpdateType: updateTypeHandlerRef.current,
        },
      }));

    // Update nodes and edges
    setNodes(nodes);
    setEdges(edges);
  }, [tasks, dependencies, searchTerm, showCompleted, highlightCriticalPath, dependencyGraph, setNodes, setEdges]);



  // Handle connection start
  const onConnectStart = useCallback((_event: any, { nodeId }: { nodeId: string | null }) => {
    if (nodeId) {
      setIsConnecting(true);
      setConnectingFromNode(nodeId);
      setDraggedConnection(null);
    }
  }, []);

  // Handle connection end
  const onConnectEnd = useCallback(() => {
    setIsConnecting(false);
    setConnectingFromNode(null);
    setDraggedConnection(null);
    setHoveredNode(null);
  }, []);

  // Handle mouse move during connection drag
  const onMouseMove = useCallback((event: React.MouseEvent) => {
    if (isConnecting && connectingFromNode) {
      const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
      setDraggedConnection({
        sourceX: 0, // Will be calculated from node position
        sourceY: 0,
        targetX: event.clientX - rect.left,
        targetY: event.clientY - rect.top,
      });
    }
  }, [isConnecting, connectingFromNode]);

  // Handle node hover during connection
  const onNodeMouseEnter = useCallback((_event: React.MouseEvent, node: Node) => {
    if (isConnecting && connectingFromNode && node.id !== connectingFromNode) {
      setHoveredNode(node.id);
    }
  }, [isConnecting, connectingFromNode]);

  const onNodeMouseLeave = useCallback(() => {
    if (isConnecting) {
      setHoveredNode(null);
    }
  }, [isConnecting]);

  // Handle connection validation and creation
  const onConnect = useCallback(async (connection: Connection) => {
    if (!connection.source || !connection.target) return;

    // Prevent self-connections
    if (connection.source === connection.target) {
      pushToast({
        title: '无法创建依赖关系',
        description: '任务不能依赖自己',
        variant: 'error',
      });
      return;
    }

    setConnectionValidation({
      isValidating: true,
      validation: null,
      pendingConnection: connection,
    });

    try {
      const validation = await validateDependency(connection.source, connection.target);
      
      setConnectionValidation({
        isValidating: false,
        validation,
        pendingConnection: connection,
      });

      if (validation.isValid) {
        // Create the dependency
        const input: DependencyCreateInput = {
          predecessorId: connection.source,
          successorId: connection.target,
          dependencyType: 'finish_to_start', // Default type
        };

        const newDependency = await createDependency(input);
        
        if (newDependency) {
          // Add edge to the graph
          const newEdge: GraphEdge = {
            id: newDependency.id,
            source: connection.source,
            target: connection.target,
            type: 'dependencyEdge',
            data: {
              dependency: newDependency,
              onDelete: deleteHandlerRef.current,
              onUpdateType: updateTypeHandlerRef.current,
            },
          };
          
          setEdges(eds => addEdge(newEdge, eds));
          onAddDependency?.(connection.source, connection.target);
          
          pushToast({
            title: '依赖关系创建成功',
            variant: 'success',
          });
        }
      } else {
        // Show validation error
        pushToast({
          title: '无法创建依赖关系',
          description: validation.errorMessage || '未知错误',
          variant: 'error',
        });
        
        if (validation.wouldCreateCycle && validation.cyclePath) {
          pushToast({
            title: '检测到循环依赖',
            description: `路径: ${validation.cyclePath.join(' → ')}`,
            variant: 'error',
          });
        }
      }
    } catch (error) {
      console.error('Failed to validate connection:', error);
      pushToast({
        title: '连接验证失败',
        description: error instanceof Error ? error.message : '未知错误',
        variant: 'error',
      });
    } finally {
      // Clear validation state after a delay
      setTimeout(() => {
        setConnectionValidation({
          isValidating: false,
          validation: null,
          pendingConnection: null,
        });
      }, 3000);
    }
  }, [validateDependency, createDependency, onAddDependency, setEdges]);



  // Handle node clicks
  const onNodeClick = useCallback((_event: React.MouseEvent, node: Node) => {
    if (!isConnecting) {
      onTaskClick?.(node.id);
    }
  }, [onTaskClick, isConnecting]);

  // Handle edge clicks
  const onEdgeClick = useCallback((event: React.MouseEvent, edge: Edge) => {
    if (event.ctrlKey || event.metaKey) {
      // Multi-select mode
      setIsMultiSelectMode(true);
      setSelectedEdges(prev => {
        if (prev.includes(edge.id)) {
          return prev.filter(id => id !== edge.id);
        } else {
          return [...prev, edge.id];
        }
      });
    } else {
      // Single select mode
      setSelectedEdge(edge.id);
      setSelectedEdges([]);
      setIsMultiSelectMode(false);
    }
  }, []);

  // Handle edge double-click for quick deletion
  const onEdgeDoubleClick = useCallback(async (_event: React.MouseEvent, edge: Edge) => {
    if (window.confirm('确定要删除这个依赖关系吗？')) {
      await handleDeleteDependency(edge.id);
    }
  }, [handleDeleteDependency]);

  // Auto-layout function
  const onLayout = useCallback(async (
    algorithm?: LayoutAlgorithm, 
    direction?: LayoutDirection
  ) => {
    setIsLayouting(true);
    
    try {
      const { nodes: layoutedNodes, edges: layoutedEdges } = await getLayoutedElements(
        nodes as Node[],
        edges as Edge[],
        algorithm || layoutAlgorithm,
        direction || layoutDirection
      );
      
      setNodes(layoutedNodes as GraphNode[]);
      setEdges(layoutedEdges as GraphEdge[]);
      
      // Fit view after layout
      setTimeout(() => fitView(), 100);
    } catch (error) {
      console.error('Layout failed:', error);
      pushToast({
        title: '布局失败',
        description: '自动布局时发生错误，请重试',
        variant: 'error',
      });
    } finally {
      setIsLayouting(false);
    }
  }, [nodes, edges, setNodes, setEdges, fitView, layoutAlgorithm, layoutDirection]);

  // Refresh graph data
  const handleRefresh = useCallback(async () => {
    const filter: DependencyFilter = {
      includeCompleted: showCompleted,
    };
    await fetchDependencyGraph(filter);
  }, [fetchDependencyGraph, showCompleted]);

  // Statistics
  const stats = useMemo(() => {
    const totalTasks = nodes.length;
    const totalDependencies = edges.length;
    const readyTasks = nodes.filter(node => node.data.isReady && node.data.task.status !== 'done').length;
    const blockedTasks = nodes.filter(node => !node.data.isReady && node.data.task.status !== 'done').length;
    const completedTasks = nodes.filter(node => node.data.task.status === 'done').length;
    const inProgressTasks = nodes.filter(node => node.data.task.status === 'in_progress').length;
    const criticalPathLength = dependencyGraph?.criticalPath?.length || 0;
    
    return {
      totalTasks,
      totalDependencies,
      readyTasks,
      blockedTasks,
      completedTasks,
      inProgressTasks,
      criticalPathLength,
    };
  }, [nodes, edges, dependencyGraph]);

  return (
    <Card className={cn('h-[600px] w-full', className)}>
      {/* Header Controls */}
      <div className="flex flex-wrap items-center justify-between gap-4 border-b p-4">
        <div className="flex items-center gap-4">
          <h3 className="text-lg font-semibold">任务依赖图</h3>
          <div className="flex items-center gap-2">
            <Badge variant="outline" className="text-xs">
              {stats.totalTasks} 任务
            </Badge>
            <Badge variant="outline" className="text-xs">
              {stats.totalDependencies} 依赖
            </Badge>
            <Badge variant="secondary" className="text-xs">
              <CheckCircle2 className="mr-1 h-3 w-3 text-green-600" />
              {stats.readyTasks} 可执行
            </Badge>
            <Badge variant="destructive" className="text-xs">
              <AlertTriangle className="mr-1 h-3 w-3" />
              {stats.blockedTasks} 受阻
            </Badge>
            {stats.criticalPathLength > 0 && (
              <Badge variant="outline" className="text-xs border-red-200 text-red-700">
                <Route className="mr-1 h-3 w-3" />
                关键路径 {stats.criticalPathLength}
              </Badge>
            )}
            <Badge variant="outline" className="text-xs text-blue-700">
              {stats.inProgressTasks} 进行中
            </Badge>
            <Badge variant="outline" className="text-xs text-green-700">
              {stats.completedTasks} 已完成
            </Badge>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {(selectedEdges.length > 0 || isMultiSelectMode) && (
            <Badge variant="secondary" className="text-xs">
              已选择 {selectedEdges.length} 个依赖关系
            </Badge>
          )}
          
          {/* Layout Algorithm Selector */}
          <div className="flex items-center gap-1 border rounded-md">
            <Button
              size="sm"
              variant={layoutAlgorithm === 'dagre' ? 'default' : 'ghost'}
              onClick={() => setLayoutAlgorithm('dagre')}
              className="h-8 px-2 text-xs"
            >
              Dagre
            </Button>
            <Button
              size="sm"
              variant={layoutAlgorithm === 'elk' ? 'default' : 'ghost'}
              onClick={() => setLayoutAlgorithm('elk')}
              className="h-8 px-2 text-xs"
            >
              ELK
            </Button>
          </div>

          {/* Layout Direction Selector */}
          <div className="flex items-center gap-1 border rounded-md">
            <Button
              size="sm"
              variant={layoutDirection === 'TB' ? 'default' : 'ghost'}
              onClick={() => setLayoutDirection('TB')}
              className="h-8 px-2"
              title="Top to Bottom"
            >
              <ArrowDown className="h-3 w-3" />
            </Button>
            <Button
              size="sm"
              variant={layoutDirection === 'LR' ? 'default' : 'ghost'}
              onClick={() => setLayoutDirection('LR')}
              className="h-8 px-2"
              title="Left to Right"
            >
              <ArrowRight className="h-3 w-3" />
            </Button>
            <Button
              size="sm"
              variant={layoutDirection === 'BT' ? 'default' : 'ghost'}
              onClick={() => setLayoutDirection('BT')}
              className="h-8 px-2"
              title="Bottom to Top"
            >
              <ArrowDown className="h-3 w-3 rotate-180" />
            </Button>
            <Button
              size="sm"
              variant={layoutDirection === 'RL' ? 'default' : 'ghost'}
              onClick={() => setLayoutDirection('RL')}
              className="h-8 px-2"
              title="Right to Left"
            >
              <ArrowRight className="h-3 w-3 rotate-180" />
            </Button>
          </div>
          
          <Button
            size="sm"
            variant="outline"
            onClick={handleRefresh}
            disabled={isLoading}
            title="刷新数据"
          >
            <RefreshCw className={cn('h-4 w-4', isLoading && 'animate-spin')} />
          </Button>
          
          <Button
            size="sm"
            variant="outline"
            onClick={() => onLayout()}
            disabled={isLayouting}
            title="自动布局"
          >
            <LayoutGrid className={cn('h-4 w-4', isLayouting && 'animate-spin')} />
          </Button>
        </div>
      </div>

      {/* Filter Controls */}
      <div className="flex flex-wrap items-center gap-4 border-b p-4">
        <div className="flex items-center gap-2">
          <Search className="h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="搜索任务..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-48"
          />
        </div>

        <div className="flex items-center gap-4">
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={showCompleted}
              onChange={(e) => setShowCompleted(e.target.checked)}
              className="rounded"
            />
            显示已完成
          </label>

          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={showReadyTasks}
              onChange={(e) => setShowReadyTasks(e.target.checked)}
              className="rounded"
            />
            <CheckCircle2 className="h-3 w-3 text-green-600" />
            高亮可执行任务
          </label>

          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={highlightCriticalPath}
              onChange={(e) => setHighlightCriticalPath(e.target.checked)}
              className="rounded"
            />
            <Route className="h-3 w-3 text-red-600" />
            高亮关键路径
          </label>

          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={isMultiSelectMode}
              onChange={(e) => {
                setIsMultiSelectMode(e.target.checked);
                if (!e.target.checked) {
                  setSelectedEdges([]);
                }
              }}
              className="rounded"
            />
            多选模式
          </label>

          {nodes.length > 200 && (
            <>
              <label className="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={enableVirtualization}
                  onChange={(e) => setEnableVirtualization(e.target.checked)}
                  className="rounded"
                />
                虚拟化 ({nodes.length} 节点)
              </label>

              <label className="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={enableProgressiveRendering}
                  onChange={(e) => setEnableProgressiveRendering(e.target.checked)}
                  className="rounded"
                />
                渐进式渲染
              </label>
            </>
          )}
        </div>
      </div>

      {/* Connection Validation Status */}
      {(connectionValidation.isValidating || connectionValidation.validation) && (
        <div className="border-b p-4">
          <ConnectionValidationFeedback
            isValidating={connectionValidation.isValidating}
            validation={connectionValidation.validation}
          />
        </div>
      )}

      {/* React Flow Graph */}
      <div className="relative h-full" onMouseMove={onMouseMove}>
        <ReactFlow
          nodes={displayNodes}
          edges={displayEdges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onConnectStart={onConnectStart}
          onConnectEnd={onConnectEnd}
          onNodeClick={onNodeClick}
          onNodeMouseEnter={onNodeMouseEnter}
          onNodeMouseLeave={onNodeMouseLeave}
          onEdgeClick={onEdgeClick}
          onEdgeDoubleClick={onEdgeDoubleClick}
          nodeTypes={nodeTypes}
          edgeTypes={edgeTypes}
          connectionMode={ConnectionMode.Loose}
          fitView
          attributionPosition="bottom-left"
          className={cn(
            "bg-gray-50 transition-all duration-200",
            isConnecting && "cursor-crosshair"
          )}
          deleteKeyCode={['Delete', 'Backspace']}
          multiSelectionKeyCode={['Control', 'Meta']}
        >
          <Background />
          <Controls />
          <MiniMap 
            nodeColor={(node) => {
              const nodeData = node.data as any;
              const task = nodeData?.task;
              if (!task) return '#e5e7eb';
              
              // Priority: Critical path > Ready tasks > Status
              if (nodeData?.isOnCriticalPath && highlightCriticalPath) {
                return '#dc2626'; // Red for critical path
              }
              
              if (nodeData?.isReady && showReadyTasks && task.status !== 'done') {
                return '#16a34a'; // Green for ready tasks
              }
              
              if (nodeData?.isBlocked) {
                return '#ea580c'; // Orange for blocked tasks
              }
              
              switch (task.status) {
                case 'done': return '#10b981';
                case 'in_progress': return '#3b82f6';
                default: return '#6b7280';
              }
            }}
            className="!bg-white !border !border-gray-200"
          />
          
          {/* Custom Panel for Graph Controls */}
          <Panel position="top-right" className="flex flex-col gap-2">
            <div className="flex gap-2">
              <Button
                size="sm"
                variant="outline"
                onClick={() => zoomIn()}
                className="bg-white"
                title="放大"
              >
                <ZoomIn className="h-4 w-4" />
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => zoomOut()}
                className="bg-white"
                title="缩小"
              >
                <ZoomOut className="h-4 w-4" />
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => fitView({ padding: 0.1, duration: 800 })}
                className="bg-white"
                title="适应屏幕"
              >
                <Maximize className="h-4 w-4" />
              </Button>
            </div>
            
            <div className="flex gap-2">
              <Button
                size="sm"
                variant="outline"
                onClick={() => onLayout('dagre', 'TB')}
                className="bg-white"
                title="垂直布局"
                disabled={isLayouting}
              >
                <ArrowDown className="h-4 w-4" />
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => onLayout('dagre', 'LR')}
                className="bg-white"
                title="水平布局"
                disabled={isLayouting}
              >
                <ArrowRight className="h-4 w-4" />
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => onLayout('elk')}
                className="bg-white"
                title="智能布局"
                disabled={isLayouting}
              >
                <Shuffle className={cn('h-4 w-4', isLayouting && 'animate-spin')} />
              </Button>
            </div>

            {highlightCriticalPath && dependencyGraph?.criticalPath && (
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  // Focus on critical path
                  const criticalNodes = nodes.filter(node => 
                    dependencyGraph.criticalPath?.includes(node.id)
                  );
                  if (criticalNodes.length > 0) {
                    fitView({
                      nodes: criticalNodes,
                      padding: 0.2,
                      duration: 800,
                    });
                  }
                }}
                className="bg-white"
                title="聚焦关键路径"
              >
                <Target className="h-4 w-4 text-red-600" />
              </Button>
            )}
          </Panel>

          {/* Drag Connection Overlay */}
          <DragConnectionOverlay
            isConnecting={isConnecting}
            sourceNodeId={connectingFromNode || undefined}
            hoveredNodeId={hoveredNode || undefined}
            draggedConnection={draggedConnection}
          />
        </ReactFlow>

        {/* Performance Monitor */}
        <GraphPerformanceMonitor
          nodeCount={nodes.length}
          edgeCount={edges.length}
          visibleNodeCount={displayNodes.length}
          visibleEdgeCount={displayEdges.length}
          isVirtualized={virtualizedGraph.isVirtualized}
        />

        {/* Progressive Rendering Progress */}
        {enableProgressiveRendering && !progressiveRender.isComplete && (
          <div className="absolute top-4 left-1/2 transform -translate-x-1/2 bg-white border rounded-lg px-4 py-2 shadow-lg z-10">
            <div className="text-sm text-gray-700">
              Loading graph... {progressiveRender.progress.toFixed(0)}%
            </div>
            <div className="w-48 h-2 bg-gray-200 rounded-full mt-2">
              <div 
                className="h-full bg-blue-500 rounded-full transition-all duration-300"
                style={{ width: `${progressiveRender.progress}%` }}
              />
            </div>
          </div>
        )}
      </div>

      {/* Bulk Operations */}
      <BulkDependencyOperations
        selectedDependencies={selectedEdges.map(edgeId => {
          const edge = edges.find(e => e.id === edgeId);
          return edge?.data?.dependency;
        }).filter(Boolean) as TaskDependency[]}
        onDeleteSelected={handleBulkDelete}
        onUpdateTypeSelected={handleBulkTypeUpdate}
        onClearSelection={() => {
          setSelectedEdges([]);
          setIsMultiSelectMode(false);
        }}
      />
    </Card>
  );
};