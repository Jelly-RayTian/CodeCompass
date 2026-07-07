import { useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Background,
  Controls,
  type Edge,
  type FitViewOptions,
  type Node,
  ReactFlow,
  useEdgesState,
  useNodesState,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import { tauriClient } from '@/lib/tauriClient';
import { useAsyncData } from '@/lib/useAsyncData';
import type {
  DependencyGraph,
  GraphEdge,
  GraphNode,
  IndexedFolder,
} from '@/types';
import { EmptyState } from '@/components/EmptyState';
import { ErrorState } from '@/components/ErrorState';
import { LoadingState } from '@/components/LoadingState';
import { useT } from '@/i18n/useT';

const fitViewOptions: FitViewOptions = { padding: 0.2, maxZoom: 1.5 };

function extensionColor(ext: string | null): string {
  switch (ext) {
    case 'ts':
      return '#3178c6';
    case 'tsx':
      return '#5ba0f5';
    case 'js':
      return '#f7df1e';
    case 'jsx':
      return '#61dafb';
    default:
      return '#888';
  }
}

function toFlowNodes(nodes: GraphNode[]): Node[] {
  return nodes.map((n) => ({
    id: String(n.fileId),
    type: 'default',
    data: {
      label: n.name,
      fileId: n.fileId,
      path: n.relativePath,
      ext: n.extension,
      incoming: n.incomingCount,
      outgoing: n.outgoingCount,
    },
    position: { x: 0, y: 0 },
    style: {
      background: '#fff',
      border: `2px solid ${extensionColor(n.extension)}`,
      borderRadius: '6px',
      padding: '8px 14px',
      fontSize: 12,
      width: Math.max(120, n.name.length * 7 + 40),
    },
  }));
}

function toFlowEdges(edges: GraphEdge[]): Edge[] {
  return edges.map((e, i) => ({
    id: `${e.sourceFileId}-${e.targetFileId}-${i}`,
    source: String(e.sourceFileId),
    target: String(e.targetFileId),
    animated: false,
    style: { stroke: '#94a3b8', strokeWidth: 1.5 },
    markerEnd: { type: 'arrowclosed' as const },
  }));
}

function simpleLayout(nodes: Node[], _edges: Edge[]): { nodes: Node[] } {
  const cols = Math.ceil(Math.sqrt(nodes.length));
  const spacing = 180;
  return {
    nodes: nodes.map((n, i) => ({
      ...n,
      position: {
        x: (i % cols) * spacing + 50,
        y: Math.floor(i / cols) * 80 + 50,
      },
    })),
  };
}

interface SelectedFileDetail {
  fileId: number;
  name: string;
  path: string;
  imports: { specifier: string; importType: string }[];
  importedBy: { path: string; specifier: string }[];
}

export function Graph(): JSX.Element {
  const navigate = useNavigate();
  const { t } = useT();
  const [foldersState] = useAsyncData<IndexedFolder[]>(() =>
    tauriClient.listIndexedFolders(),
  );

  const [selectedFolderId, setSelectedFolderId] = useState<number | null>(null);
  const [graphData, setGraphData] = useState<DependencyGraph | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState('');
  const [folderFilter, setFolderFilter] = useState('');
  const [selectedDetail, setSelectedDetail] =
    useState<SelectedFileDetail | null>(null);

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  const loadGraph = useCallback(
    async (folderId: number) => {
      setLoading(true);
      setError(null);
      setSelectedDetail(null);
      try {
        const data = await tauriClient.getDependencyGraph(folderId);
        setGraphData(data);
        const flowNodes = toFlowNodes(data.nodes);
        const flowEdges = toFlowEdges(data.edges);
        const laidOut = simpleLayout(flowNodes, flowEdges);
        setNodes(laidOut.nodes);
        setEdges(flowEdges);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    },
    [setNodes, setEdges],
  );

  useEffect(() => {
    if (selectedFolderId !== null) {
      loadGraph(selectedFolderId);
    }
  }, [selectedFolderId, loadGraph]);

  const filteredNodes = useMemo(
    () =>
      nodes.filter((n) => {
        const data = n.data as {
          label: string;
          path: string;
        };
        if (filter && !data.path.toLowerCase().includes(filter.toLowerCase()))
          return false;
        if (
          folderFilter &&
          !data.path
            .toLowerCase()
            .includes('/' + folderFilter.toLowerCase() + '/') &&
          !data.path.toLowerCase().startsWith(folderFilter.toLowerCase() + '/')
        )
          return false;
        return true;
      }),
    [nodes, filter, folderFilter],
  );

  const filteredEdges = useMemo(() => {
    const nodeIds = new Set(filteredNodes.map((n) => n.id));
    return edges.filter((e) => nodeIds.has(e.source) && nodeIds.has(e.target));
  }, [edges, filteredNodes]);

  const handleNodeClick = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      const data = node.data as {
        fileId: number;
        label: string;
        path: string;
        incoming: number;
        outgoing: number;
      };
      if (graphData === null) return;

      const imports = graphData.edges
        .filter((e) => e.sourceFileId === data.fileId)
        .map((e) => ({
          specifier:
            graphData.nodes.find((n) => n.fileId === e.targetFileId)
              ?.relativePath ?? `#${e.targetFileId}`,
          importType: e.importType,
        }));

      const importedBy = graphData.edges
        .filter((e) => e.targetFileId === data.fileId)
        .map((e) => ({
          path:
            graphData.nodes.find((n) => n.fileId === e.sourceFileId)
              ?.relativePath ?? `#${e.sourceFileId}`,
          specifier: graphData.edges
            .filter(
              (x) =>
                x.sourceFileId === e.sourceFileId &&
                x.targetFileId === e.targetFileId,
            )
            .map((x) => x.importType)
            .join(', '),
        }));

      setSelectedDetail({
        fileId: data.fileId,
        name: data.label,
        path: data.path,
        imports,
        importedBy,
      });
    },
    [graphData],
  );

  if (foldersState.status === 'loading') {
    return <LoadingState label={t.general.loading} />;
  }
  if (foldersState.status === 'error') {
    return (
      <ErrorState
        title={t.workspaces.loadError}
        description={foldersState.message}
        onRetry={() => window.location.reload()}
      />
    );
  }

  const folders = foldersState.data;

  return (
    <div className="graph-page">
      <h1 className="page-title">{t.graph.title}</h1>
      <p className="page-subtitle">{t.graph.subtitle}</p>

      <div className="toolbar">
        <select
          className="select"
          value={selectedFolderId ?? ''}
          onChange={(e) => {
            const id = Number(e.target.value);
            setSelectedFolderId(id || null);
          }}
        >
          <option value="">{t.graph.selectFolder}</option>
          {folders.map((f) => (
            <option key={f.id} value={f.id}>
              {f.name}
            </option>
          ))}
        </select>

        {selectedFolderId !== null && (
          <>
            <input
              className="input"
              placeholder={t.graph.filterByPath}
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
            />
            <input
              className="input"
              placeholder={t.graph.filterByDir}
              value={folderFilter}
              onChange={(e) => setFolderFilter(e.target.value)}
            />
          </>
        )}
      </div>

      {error !== null && (
        <div className="banner banner-warning" role="alert">
          {error}
        </div>
      )}

      {loading && <LoadingState label={t.graph.building} />}

      {graphData !== null && !loading && (
        <>
          <div className="graph-summary">
            <span>
              {graphData.totalFiles} {t.graph.filesIndexed}
            </span>
            <span>
              {graphData.totalImports} {t.graph.internalImports}
            </span>
            <span>
              {graphData.nodes.length} {t.graph.graphNodes}
            </span>
            <span>
              {graphData.edges.length} {t.graph.graphEdges}
            </span>
            {graphData.cycles.length > 0 && (
              <span className="graph-cycle-warn">
                {graphData.cycles.length} {t.graph.cycleDetected}
              </span>
            )}
          </div>

          {graphData.truncated && (
            <div className="banner banner-warning" role="status">
              {t.graph.truncatedWarning
                .replace('{total}', String(graphData.totalGraphNodes))
                .replace('{shown}', String(graphData.nodes.length))}
            </div>
          )}

          {graphData.cycles.length > 0 && (
            <div className="card" style={{ marginBottom: 16 }}>
              <h3 className="section-title">
                {t.graph.circularDependencies} ({graphData.cycles.length})
              </h3>
              {graphData.cycles.slice(0, 5).map((cycle, i) => (
                <div key={i} className="cycle-path">
                  {cycle.filePaths.join(' → ')}
                </div>
              ))}
            </div>
          )}

          <div className="graph-layout">
            <div className="graph-canvas">
              <ReactFlow
                nodes={filteredNodes}
                edges={filteredEdges}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                onNodeClick={handleNodeClick}
                fitView
                fitViewOptions={fitViewOptions}
                nodesDraggable
                nodesConnectable={false}
                elementsSelectable
                minZoom={0.1}
                maxZoom={2}
              >
                <Background />
                <Controls />
              </ReactFlow>
            </div>

            {selectedDetail !== null && (
              <div className="graph-detail">
                <h3 className="panel-title">{selectedDetail.name}</h3>
                <div className="file-detail-row">
                  <span>{t.graph.path}</span>
                  <span>{selectedDetail.path}</span>
                </div>

                <button
                  className="btn btn-secondary"
                  style={{ marginTop: 8, width: '100%' }}
                  onClick={() =>
                    navigate(
                      `/viewer?workspaceId=${selectedFolderId}&path=${encodeURIComponent(
                        selectedDetail.path,
                      )}`,
                    )
                  }
                  type="button"
                >
                  {t.graph.viewSource}
                </button>

                <h4 className="detail-label">
                  {t.graph.imports} ({selectedDetail.imports.length})
                </h4>
                {selectedDetail.imports.length === 0 ? (
                  <div className="muted">{t.graph.noResolved}</div>
                ) : (
                  <ul className="history-list">
                    {selectedDetail.imports.map((imp, i) => (
                      <li key={i} className="history-item">
                        <span>{imp.specifier}</span>
                        <span className="muted"> {imp.importType}</span>
                      </li>
                    ))}
                  </ul>
                )}

                <h4 className="detail-label">
                  {t.graph.importedBy} ({selectedDetail.importedBy.length})
                </h4>
                {selectedDetail.importedBy.length === 0 ? (
                  <div className="muted">{t.graph.notImported}</div>
                ) : (
                  <ul className="history-list">
                    {selectedDetail.importedBy.map((imp, i) => (
                      <li key={i} className="history-item">
                        <span>{imp.path}</span>
                        <span className="muted"> ({imp.specifier})</span>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            )}

            {selectedDetail === null && nodes.length > 0 && (
              <div className="graph-detail">
                <div className="panel-title">{t.graph.fileDetails}</div>
                <div className="muted">{t.graph.clickNode}</div>
              </div>
            )}
          </div>

          {graphData.nodes.length === 0 && graphData.totalFiles > 0 && (
            <EmptyState
              title={t.graph.noInternalDeps}
              description={t.graph.noInternalDepsDesc}
            />
          )}
        </>
      )}
    </div>
  );
}
