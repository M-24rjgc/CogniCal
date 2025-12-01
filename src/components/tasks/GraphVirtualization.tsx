import React, { useMemo } from 'react';
import { Node, Edge, useReactFlow, Rect } from '@xyflow/react';

interface VirtualizationConfig {
  enabled: boolean;
  threshold: number; // Number of nodes before virtualization kicks in
  viewportPadding: number; // Extra padding around viewport for smooth scrolling
}

interface UseGraphVirtualizationProps {
  nodes: Node[];
  edges: Edge[];
  config?: Partial<VirtualizationConfig>;
}

interface VirtualizedGraph {
  visibleNodes: Node[];
  visibleEdges: Edge[];
  hiddenNodeCount: number;
  isVirtualized: boolean;
}

const DEFAULT_CONFIG: VirtualizationConfig = {
  enabled: true,
  threshold: 500,
  viewportPadding: 200,
};

/**
 * Hook for virtualizing large dependency graphs
 * Only renders nodes and edges that are visible in the current viewport
 */
export function useGraphVirtualization({
  nodes,
  edges,
  config = {},
}: UseGraphVirtualizationProps): VirtualizedGraph {
  const { getViewport } = useReactFlow();
  const finalConfig = { ...DEFAULT_CONFIG, ...config };

  const virtualizedGraph = useMemo(() => {
    // Skip virtualization if disabled or below threshold
    if (!finalConfig.enabled || nodes.length < finalConfig.threshold) {
      return {
        visibleNodes: nodes,
        visibleEdges: edges,
        hiddenNodeCount: 0,
        isVirtualized: false,
      };
    }

    const viewport = getViewport();
    const { x, y, zoom } = viewport;

    // Calculate visible viewport bounds with padding
    const viewportBounds: Rect = {
      x: -x / zoom - finalConfig.viewportPadding,
      y: -y / zoom - finalConfig.viewportPadding,
      width: window.innerWidth / zoom + finalConfig.viewportPadding * 2,
      height: window.innerHeight / zoom + finalConfig.viewportPadding * 2,
    };

    // Filter visible nodes
    const visibleNodes = nodes.filter((node) => {
      const nodeWidth = 280; // Default node width
      const nodeHeight = 120; // Default node height

      return isNodeInViewport(
        node.position.x,
        node.position.y,
        nodeWidth,
        nodeHeight,
        viewportBounds
      );
    });

    const visibleNodeIds = new Set(visibleNodes.map((n) => n.id));

    // Filter visible edges (only edges where both nodes are visible)
    const visibleEdges = edges.filter(
      (edge) =>
        visibleNodeIds.has(edge.source) && visibleNodeIds.has(edge.target)
    );

    return {
      visibleNodes,
      visibleEdges,
      hiddenNodeCount: nodes.length - visibleNodes.length,
      isVirtualized: true,
    };
  }, [nodes, edges, getViewport, finalConfig]);

  return virtualizedGraph;
}

/**
 * Check if a node intersects with the viewport bounds
 */
function isNodeInViewport(
  nodeX: number,
  nodeY: number,
  nodeWidth: number,
  nodeHeight: number,
  viewport: Rect
): boolean {
  return !(
    nodeX + nodeWidth < viewport.x ||
    nodeX > viewport.x + viewport.width ||
    nodeY + nodeHeight < viewport.y ||
    nodeY > viewport.y + viewport.height
  );
}

/**
 * Performance monitoring component for graph rendering
 */
interface GraphPerformanceMonitorProps {
  nodeCount: number;
  edgeCount: number;
  visibleNodeCount: number;
  visibleEdgeCount: number;
  isVirtualized: boolean;
}

export const GraphPerformanceMonitor: React.FC<GraphPerformanceMonitorProps> = ({
  nodeCount,
  edgeCount,
  visibleNodeCount,
  visibleEdgeCount,
  isVirtualized,
}) => {
  const renderPercentage = nodeCount > 0 
    ? ((visibleNodeCount / nodeCount) * 100).toFixed(1)
    : '0';

  if (!isVirtualized) {
    return null;
  }

  return (
    <div className="absolute bottom-4 left-4 bg-white border rounded-lg p-3 shadow-lg text-xs space-y-1 z-10">
      <div className="font-semibold text-gray-700">Performance Stats</div>
      <div className="text-gray-600">
        Nodes: {visibleNodeCount} / {nodeCount} ({renderPercentage}%)
      </div>
      <div className="text-gray-600">
        Edges: {visibleEdgeCount} / {edgeCount}
      </div>
      <div className="text-green-600 font-medium">
        ✓ Virtualization Active
      </div>
    </div>
  );
};

/**
 * Progressive rendering hook for large graphs
 * Renders nodes in batches to avoid blocking the UI
 */
interface UseProgressiveRenderingProps {
  nodes: Node[];
  edges: Edge[];
  batchSize?: number;
  enabled?: boolean;
}

interface ProgressiveRenderState {
  renderedNodes: Node[];
  renderedEdges: Edge[];
  isComplete: boolean;
  progress: number;
}

export function useProgressiveRendering({
  nodes,
  edges,
  batchSize = 50,
  enabled = true,
}: UseProgressiveRenderingProps): ProgressiveRenderState {
  const [renderedCount, setRenderedCount] = React.useState(0);

  React.useEffect(() => {
    if (!enabled || nodes.length <= batchSize) {
      setRenderedCount(nodes.length);
      return;
    }

    // Reset when nodes change
    setRenderedCount(0);

    // Progressively render nodes in batches
    const totalBatches = Math.ceil(nodes.length / batchSize);
    let currentBatch = 0;

    const renderNextBatch = () => {
      currentBatch++;
      setRenderedCount(Math.min(currentBatch * batchSize, nodes.length));

      if (currentBatch < totalBatches) {
        requestAnimationFrame(renderNextBatch);
      }
    };

    requestAnimationFrame(renderNextBatch);
  }, [nodes, batchSize, enabled]);

  const renderedNodes = enabled ? nodes.slice(0, renderedCount) : nodes;
  const renderedNodeIds = new Set(renderedNodes.map((n) => n.id));
  const renderedEdges = edges.filter(
    (edge) =>
      renderedNodeIds.has(edge.source) && renderedNodeIds.has(edge.target)
  );

  return {
    renderedNodes,
    renderedEdges,
    isComplete: renderedCount >= nodes.length,
    progress: nodes.length > 0 ? (renderedCount / nodes.length) * 100 : 100,
  };
}

/**
 * Lazy loading hook for graph data
 * Loads graph data on demand based on viewport
 */
interface UseLazyGraphLoadingProps {
  allNodes: Node[];
  allEdges: Edge[];
  initialLoadCount?: number;
}

export function useLazyGraphLoading({
  allNodes,
  allEdges,
  initialLoadCount = 100,
}: UseLazyGraphLoadingProps) {
  const [loadedNodeCount, setLoadedNodeCount] = React.useState(initialLoadCount);

  const loadMore = React.useCallback(() => {
    setLoadedNodeCount((prev) => Math.min(prev + 50, allNodes.length));
  }, [allNodes.length]);

  const loadedNodes = allNodes.slice(0, loadedNodeCount);
  const loadedNodeIds = new Set(loadedNodes.map((n) => n.id));
  const loadedEdges = allEdges.filter(
    (edge) =>
      loadedNodeIds.has(edge.source) && loadedNodeIds.has(edge.target)
  );

  return {
    nodes: loadedNodes,
    edges: loadedEdges,
    hasMore: loadedNodeCount < allNodes.length,
    loadMore,
    progress: (loadedNodeCount / allNodes.length) * 100,
  };
}
