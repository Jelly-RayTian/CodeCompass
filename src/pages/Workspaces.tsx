import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { listen } from '@tauri-apps/api/event';

import { tauriClient } from '@/lib/tauriClient';
import { useAsyncData } from '@/lib/useAsyncData';
import type {
  AnalysisDiagnostic,
  AnalysisProgressEvent,
  FileEntry,
  ImportEntry,
  IndexedFolder,
  ScanProgressEvent,
  ScanRun,
  ScanStatus,
} from '@/types';
import { EmptyState } from '@/components/EmptyState';
import { ErrorState } from '@/components/ErrorState';
import { LoadingState } from '@/components/LoadingState';
import { SymbolSearch } from '@/components/SymbolSearch';
import { useT } from '@/i18n/useT';

function formatTimestamp(
  epochSeconds: number | null | undefined,
  neverLabel: string,
): string {
  if (epochSeconds === null || epochSeconds === undefined) {
    return neverLabel;
  }
  return new Date(epochSeconds * 1000).toLocaleString();
}

function availabilityLabel(
  availability: IndexedFolder['availability'],
  t: ReturnType<typeof useT>['t'],
): string {
  switch (availability) {
    case 'available':
      return t.general.available;
    case 'missing':
      return t.general.missing;
    case 'inaccessible':
      return t.general.inaccessible;
    case 'permission_denied':
      return t.general.permissionDenied;
    case 'not_a_directory':
      return t.general.notADirectory;
    default:
      return t.general.unknown;
  }
}

function changeStatusClass(status: FileEntry['changeStatus']): string {
  switch (status) {
    case 'new':
      return 'status-new';
    case 'changed':
      return 'status-changed';
    case 'removed':
      return 'status-removed';
    default:
      return 'status-unchanged';
  }
}

interface TreeNode {
  name: string;
  relativePath: string;
  file?: FileEntry;
  children: TreeNode[];
}

function buildTree(files: FileEntry[]): TreeNode {
  const root: TreeNode = { name: '', relativePath: '', children: [] };
  for (const file of files) {
    const parts = file.relativePath.split(/[\\/]/);
    let current = root;
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      if (part === undefined) {
        continue;
      }
      const isFile = i === parts.length - 1;
      const relativePath = parts.slice(0, i + 1).join('/');
      const existing = current.children.find((c) => c.name === part);
      let child: TreeNode;
      if (existing === undefined) {
        child = { name: part, relativePath, children: [] };
        current.children.push(child);
      } else {
        child = existing;
      }
      if (isFile) {
        child.file = file;
      }
      current = child;
    }
  }
  sortTree(root);
  return root;
}

function sortTree(node: TreeNode): void {
  node.children.sort((a, b) => {
    const aIsDir = a.children.length > 0;
    const bIsDir = b.children.length > 0;
    if (aIsDir !== bIsDir) {
      return aIsDir ? -1 : 1;
    }
    return a.name.localeCompare(b.name);
  });
  node.children.forEach(sortTree);
}

interface TreeNodeViewProps {
  node: TreeNode;
  selectedId: number | null;
  onSelect: (file: FileEntry) => void;
}

function TreeNodeView({
  node,
  selectedId,
  onSelect,
}: TreeNodeViewProps): JSX.Element {
  if (node.children.length === 0) {
    return <></>;
  }
  return (
    <ul className="file-tree-list">
      {node.children.map((child) => {
        const isSelected = child.file?.id === selectedId;
        if (child.file !== undefined) {
          const file = child.file;
          return (
            <li key={child.relativePath}>
              <button
                className={`file-tree-file ${isSelected ? 'selected' : ''} ${changeStatusClass(file.changeStatus)}`}
                onClick={() => onSelect(file)}
                type="button"
              >
                {child.name}
              </button>
            </li>
          );
        }
        return (
          <li key={child.relativePath}>
            <div className="file-tree-dir">{child.name}</div>
            <TreeNodeView
              node={child}
              selectedId={selectedId}
              onSelect={onSelect}
            />
          </li>
        );
      })}
    </ul>
  );
}

export function Workspaces(): JSX.Element {
  const navigate = useNavigate();
  const { t } = useT();
  const [foldersState, reloadFolders] = useAsyncData<IndexedFolder[]>(() =>
    tauriClient.listIndexedFolders(),
  );
  const [warning, setWarning] = useState<string | null>(null);
  const [removingId, setRemovingId] = useState<number | null>(null);
  const [scanningFolderId, setScanningFolderId] = useState<number | null>(null);
  const [scanStatus, setScanStatus] = useState<Record<number, ScanStatus>>({});
  const [completedScan, setCompletedScan] = useState<ScanStatus | null>(null);
  const pollingRef = useRef<number | null>(null);

  const [selectedFolderId, setSelectedFolderId] = useState<number | null>(null);
  const [files, setFiles] = useState<FileEntry[] | null>(null);
  const [filesLoading, setFilesLoading] = useState(false);
  const [selectedFile, setSelectedFile] = useState<FileEntry | null>(null);
  const [scanHistory, setScanHistory] = useState<ScanRun[] | null>(null);
  const [analyzingFolderId, setAnalyzingFolderId] = useState<number | null>(
    null,
  );
  const [fileTreeFilter, setFileTreeFilter] = useState('');
  const [analysisProgress, setAnalysisProgress] = useState<
    Record<number, AnalysisProgressEvent>
  >({});
  const [fileImports, setFileImports] = useState<ImportEntry[] | null>(null);
  const [diagnostics, setDiagnostics] = useState<AnalysisDiagnostic[] | null>(
    null,
  );

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setup = async (): Promise<void> => {
      unlisten = await listen<ScanProgressEvent>('scan:progress', (event) => {
        const e = event.payload;
        setScanStatus((prev) => {
          const existing = prev[e.workspaceId];
          return {
            ...prev,
            [e.workspaceId]: {
              run: {
                ...(existing?.run ?? {
                  id: e.runId,
                  workspaceId: e.workspaceId,
                  startedAt: 0,
                  completedAt: null,
                  errorMessage: null,
                }),
                id: e.runId,
                workspaceId: e.workspaceId,
                status: e.status,
                filesProcessed: e.filesProcessed,
                filesIndexed: e.filesIndexed,
                warningCount: e.warningCount,
                errorCount: e.errorCount,
                phase: e.phase,
              },
              fileCount: existing?.fileCount ?? 0,
            },
          };
        });
        if (
          e.status !== 'running' &&
          e.status !== 'queued' &&
          e.status !== 'pending'
        ) {
          setScanningFolderId((current) =>
            current === e.workspaceId ? null : current,
          );
          reloadFolders();
          if (selectedFolderId === e.workspaceId) {
            loadFiles(e.workspaceId);
            loadHistory(e.workspaceId);
          }
        }
      });
    };

    setup().catch((err: unknown) => {
      console.error('Failed to listen to scan progress', err);
    });

    return () => {
      if (unlisten !== undefined) {
        unlisten();
      }
    };
  }, [selectedFolderId, reloadFolders]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const setup = async (): Promise<void> => {
      unlisten = await listen<AnalysisProgressEvent>(
        'analysis:progress',
        (event) => {
          const e = event.payload;
          setAnalysisProgress((prev) => ({ ...prev, [e.workspaceId]: e }));
          if (e.status !== 'running') {
            setAnalyzingFolderId((current) =>
              current === e.workspaceId ? null : current,
            );
            reloadFolders();
          }
        },
      );
    };
    setup().catch((err: unknown) => {
      console.error('Failed to listen to analysis progress', err);
    });
    return () => {
      if (unlisten !== undefined) {
        unlisten();
      }
    };
  }, [reloadFolders]);

  useEffect(() => {
    if (scanningFolderId === null) {
      return undefined;
    }

    const poll = async (): Promise<void> => {
      try {
        const status = await tauriClient.getScanStatus(scanningFolderId);
        if (status === null) {
          return;
        }
        setScanStatus((prev) => ({ ...prev, [scanningFolderId]: status }));
        if (
          status.run.status !== 'running' &&
          status.run.status !== 'queued' &&
          status.run.status !== 'pending'
        ) {
          setScanningFolderId(null);
          setCompletedScan(status);
          reloadFolders();
          if (selectedFolderId === scanningFolderId) {
            loadFiles(scanningFolderId);
            loadHistory(scanningFolderId);
          }
        }
      } catch (err) {
        console.error('Failed to fetch scan status', err);
      }
    };

    poll();
    const id = window.setInterval(poll, 300);
    pollingRef.current = id;
    return () => {
      window.clearInterval(id);
    };
  }, [scanningFolderId, reloadFolders, selectedFolderId]);

  const loadFiles = async (folderId: number): Promise<void> => {
    setFilesLoading(true);
    try {
      const data = await tauriClient.listWorkspaceFiles(folderId);
      setFiles(data);
      setSelectedFile(null);
    } catch (err) {
      setWarning(err instanceof Error ? err.message : String(err));
    } finally {
      setFilesLoading(false);
    }
  };

  const loadHistory = async (folderId: number): Promise<void> => {
    try {
      const data = await tauriClient.listScanRuns(folderId);
      setScanHistory(data);
    } catch (err) {
      setWarning(err instanceof Error ? err.message : String(err));
    }
  };

  const handleSelectFolder = (folderId: number): void => {
    setSelectedFolderId(folderId);
    setSelectedFile(null);
    setFileImports(null);
    loadFiles(folderId);
    loadHistory(folderId);
    loadDiagnostics(folderId);
  };

  const handleAddFolder = async (): Promise<void> => {
    setWarning(null);
    try {
      const picked = await tauriClient.pickFolder();
      if (picked === null) {
        return;
      }
      const result = await tauriClient.addFolder(picked);
      if (result.warning !== null) {
        setWarning(result.warning);
      }
      reloadFolders();
    } catch (err) {
      setWarning(err instanceof Error ? err.message : String(err));
    }
  };

  const handleScan = async (folderId: number): Promise<void> => {
    setCompletedScan(null);
    try {
      const run = await tauriClient.startScan(folderId);
      setScanningFolderId(folderId);
      setScanStatus((prev) => ({
        ...prev,
        [folderId]: {
          run,
          fileCount: prev[folderId]?.fileCount ?? 0,
        },
      }));
    } catch (err) {
      setWarning(err instanceof Error ? err.message : String(err));
    }
  };

  const handleCancelScan = async (): Promise<void> => {
    if (scanningFolderId === null) {
      return;
    }
    const status = scanStatus[scanningFolderId];
    if (status === undefined) {
      return;
    }
    try {
      await tauriClient.cancelScan(status.run.id);
    } catch (err) {
      setWarning(err instanceof Error ? err.message : String(err));
    }
  };

  const handleAnalyze = async (folderId: number): Promise<void> => {
    try {
      await tauriClient.startAnalysis(folderId);
      setAnalyzingFolderId(folderId);
    } catch (err) {
      setWarning(err instanceof Error ? err.message : String(err));
    }
  };

  const handleCancelAnalysis = async (): Promise<void> => {
    if (analyzingFolderId === null) return;
    try {
      await tauriClient.cancelAnalysis(analyzingFolderId);
    } catch (err) {
      setWarning(err instanceof Error ? err.message : String(err));
    }
  };

  const loadDiagnostics = async (folderId: number): Promise<void> => {
    try {
      const data = await tauriClient.getAnalysisDiagnostics(folderId);
      setDiagnostics(data);
    } catch (err) {
      setWarning(err instanceof Error ? err.message : String(err));
    }
  };

  const handleFileSelect = async (file: FileEntry): Promise<void> => {
    setSelectedFile(file);
    try {
      const imports = await tauriClient.getFileImports(file.id);
      setFileImports(imports);
    } catch (err) {
      setWarning(err instanceof Error ? err.message : String(err));
    }
  };

  const handleRemove = async (id: number): Promise<void> => {
    setRemovingId(id);
  };

  const confirmRemove = async (): Promise<void> => {
    if (removingId === null) {
      return;
    }
    try {
      await tauriClient.removeIndexedFolder(removingId);
      setRemovingId(null);
      setScanStatus((prev) => {
        const next = { ...prev };
        delete next[removingId];
        return next;
      });
      if (selectedFolderId === removingId) {
        setSelectedFolderId(null);
        setFiles(null);
        setScanHistory(null);
        setSelectedFile(null);
      }
      reloadFolders();
    } catch (err) {
      setWarning(err instanceof Error ? err.message : String(err));
      setRemovingId(null);
    }
  };

  if (foldersState.status === 'loading') {
    return <LoadingState label={t.general.loading} />;
  }

  if (foldersState.status === 'error') {
    return (
      <ErrorState
        title={t.workspaces.loadError}
        description={foldersState.message}
        onRetry={reloadFolders}
      />
    );
  }

  const folders = foldersState.data;
  const tree =
    files !== null
      ? buildTree(
          fileTreeFilter.trim() === ''
            ? files
            : files.filter((f) =>
                f.relativePath
                  .toLowerCase()
                  .includes(fileTreeFilter.toLowerCase()),
              ),
        )
      : null;

  return (
    <>
      <h1 className="page-title">{t.workspaces.title}</h1>
      <p className="page-subtitle">{t.workspaces.subtitle}</p>

      <div className="toolbar">
        <button
          className="btn btn-primary"
          onClick={handleAddFolder}
          type="button"
        >
          {t.workspaces.addFolder}
        </button>
      </div>

      {warning !== null && (
        <div className="banner banner-warning" role="alert">
          {warning}
        </div>
      )}

      {completedScan !== null && (
        <div className="banner banner-success" role="status">
          {t.workspaces.scanComplete}: {completedScan.run.filesProcessed}{' '}
          {t.workspaces.processed}, {completedScan.run.filesIndexed}{' '}
          {t.workspaces.indexed}, {completedScan.run.warningCount}{' '}
          {t.workspaces.warnings}, {completedScan.run.errorCount}{' '}
          {t.workspaces.errors}.
        </div>
      )}

      {folders.length === 0 ? (
        <EmptyState
          title={t.workspaces.noIndexedFolders}
          description={t.workspaces.noIndexedFoldersDesc}
        />
      ) : (
        <div className="folder-list">
          {folders.map((folder) => {
            const status = scanStatus[folder.id];
            const isScanning =
              scanningFolderId === folder.id ||
              status?.run.status === 'running' ||
              status?.run.status === 'queued';
            const isSelected = selectedFolderId === folder.id;
            return (
              <div
                className={`card folder-card ${folder.availability !== 'available' ? 'unavailable' : ''} ${isSelected ? 'selected' : ''}`}
                key={folder.id}
              >
                <div className="folder-header">
                  <div className="folder-name">{folder.name}</div>
                  <div className="folder-path">{folder.path}</div>
                </div>

                <div className="folder-meta-grid">
                  <div>
                    <div className="folder-meta-label">
                      {t.workspaces.availability}
                    </div>
                    <div className="folder-meta-value">
                      {availabilityLabel(folder.availability, t)}
                    </div>
                  </div>
                  <div>
                    <div className="folder-meta-label">
                      {t.workspaces.monitoring}
                    </div>
                    <div className="folder-meta-value">
                      {folder.monitoringEnabled
                        ? t.workspaces.enabled
                        : t.workspaces.disabled}
                    </div>
                  </div>
                  <div>
                    <div className="folder-meta-label">
                      {t.workspaces.scanStatus}
                    </div>
                    <div className="folder-meta-value">{folder.scanStatus}</div>
                  </div>
                  <div>
                    <div className="folder-meta-label">
                      {t.workspaces.filesIndexed}
                    </div>
                    <div className="folder-meta-value">
                      {status?.fileCount ?? 0}
                    </div>
                  </div>
                  <div>
                    <div className="folder-meta-label">
                      {t.workspaces.lastSuccessfulScan}
                    </div>
                    <div className="folder-meta-value">
                      {formatTimestamp(
                        folder.lastSuccessfulScanAt,
                        t.general.never,
                      )}
                    </div>
                  </div>
                  <div>
                    <div className="folder-meta-label">{t.workspaces.added}</div>
                    <div className="folder-meta-value">
                      {formatTimestamp(folder.addedAt, t.general.never)}
                    </div>
                  </div>
                </div>

                {isScanning && status !== undefined && (
                  <div className="scan-progress">
                    <div className="scan-phase">
                      {t.workspaces.phase}: {status.run.phase ?? 'scanning'}
                    </div>
                    <div className="scan-counters">
                      {t.workspaces.processed} {status.run.filesProcessed} ·{' '}
                      {t.workspaces.indexed} {status.run.filesIndexed} ·{' '}
                      {t.workspaces.warnings} {status.run.warningCount} ·{' '}
                      {t.workspaces.errors} {status.run.errorCount}
                    </div>
                  </div>
                )}

                {analyzingFolderId === folder.id &&
                  (() => {
                    const ap = analysisProgress[folder.id];
                    if (ap === undefined) return null;
                    return (
                      <div className="scan-progress">
                        <div className="scan-phase">
                          {t.workspaces.analyzing}
                        </div>
                        <div className="scan-counters">
                          {ap.filesProcessed} / {ap.filesTotal} ·{' '}
                          {ap.filesParsed}
                        </div>
                      </div>
                    );
                  })()}

                <div className="folder-actions">
                  <div className="folder-actions-primary">
                    {isScanning ? (
                      <button
                        className="btn btn-danger"
                        onClick={handleCancelScan}
                        type="button"
                      >
                        {t.workspaces.cancelScan}
                      </button>
                    ) : (
                      <>
                        <button
                          className="btn btn-primary"
                          disabled={folder.availability !== 'available'}
                          onClick={() => handleScan(folder.id)}
                          type="button"
                        >
                          {t.workspaces.scanFolder}
                        </button>
                        <button
                          className="btn btn-primary"
                          disabled={
                            folder.availability !== 'available' ||
                            analyzingFolderId === folder.id
                          }
                          onClick={() => handleAnalyze(folder.id)}
                          type="button"
                        >
                          {analyzingFolderId === folder.id
                            ? t.workspaces.analyzing
                            : t.workspaces.analyze}
                        </button>
                      </>
                    )}
                    {analyzingFolderId === folder.id && (
                      <button
                        className="btn btn-danger"
                        onClick={handleCancelAnalysis}
                        type="button"
                      >
                        {t.workspaces.cancel}
                      </button>
                    )}
                  </div>
                  <div className="folder-actions-secondary">
                    <button
                      className={`btn btn-secondary ${isSelected ? 'active' : ''}`}
                      onClick={() => handleSelectFolder(folder.id)}
                      type="button"
                    >
                      {t.workspaces.viewFiles}
                    </button>
                    <button
                      className="btn btn-secondary"
                      onClick={() => tauriClient.revealFolder(folder.path)}
                      type="button"
                    >
                      {t.workspaces.reveal}
                    </button>
                    <button
                      className="btn btn-danger"
                      onClick={() => handleRemove(folder.id)}
                      type="button"
                    >
                      {t.workspaces.remove}
                    </button>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {selectedFolderId !== null && (
        <div className="workspace-detail">
          <h2 className="section-title">{t.workspaces.filesAndHistory}</h2>
          <div className="detail-grid">
            <div className="detail-panel">
              <div className="panel-title">{t.workspaces.indexedFileTree}</div>
              {files !== null && files.length > 0 && (
                <input
                  className="input file-tree-filter"
                  placeholder={t.general.search}
                  value={fileTreeFilter}
                  onChange={(e) => setFileTreeFilter(e.target.value)}
                  type="search"
                />
              )}
              {filesLoading ? (
                <LoadingState label={t.workspaces.loadingFiles} />
              ) : tree === null || tree.children.length === 0 ? (
                <EmptyState
                  title={t.workspaces.noIndexedFiles}
                  description={t.workspaces.noIndexedFilesDesc}
                />
              ) : (
                <div className="file-tree">
                  <TreeNodeView
                    node={tree}
                    selectedId={selectedFile?.id ?? null}
                    onSelect={handleFileSelect}
                  />
                </div>
              )}
            </div>

            <div className="detail-side">
              {selectedFile !== null && (
                <>
                  <div className="card file-detail">
                    <div className="panel-title">{selectedFile.name}</div>
                    <div className="file-detail-row">
                      <span>{t.graph.path}</span>
                      <span>{selectedFile.relativePath}</span>
                    </div>
                    <div className="file-detail-row">
                      <span>{t.workspaces.availability}</span>
                      <span>{selectedFile.extension ?? '-'}</span>
                    </div>
                    <div className="file-detail-row">
                      <span>{t.workspaces.filesIndexed}</span>
                      <span>{selectedFile.sizeBytes} bytes</span>
                    </div>
                    <div className="file-detail-row">
                      <span>{t.workspaces.lastSuccessfulScan}</span>
                      <span>
                        {formatTimestamp(selectedFile.modifiedAt, t.general.never)}
                      </span>
                    </div>
                    <div className="file-detail-row">
                      <span>{t.workspaces.added}</span>
                      <span>
                        {formatTimestamp(selectedFile.createdAt, t.general.never)}
                      </span>
                    </div>
                    <div className="file-detail-row">
                      <span>{t.workspaces.scanStatus}</span>
                      <span className="mono">
                        {selectedFile.fingerprint ?? '-'}
                      </span>
                    </div>
                    <div className="file-detail-row">
                      <span>{t.workspaces.scanStatus}</span>
                      <span
                        className={changeStatusClass(selectedFile.changeStatus)}
                      >
                        {selectedFile.changeStatus}
                      </span>
                    </div>
                    <button
                      className="btn btn-secondary"
                      style={{ marginTop: 8, width: '100%' }}
                      onClick={() =>
                        navigate(
                          `/viewer?workspaceId=${selectedFolderId}&path=${encodeURIComponent(
                            selectedFile.relativePath,
                          )}`,
                        )
                      }
                      type="button"
                    >
                      {t.workspaces.viewSource}
                    </button>
                  </div>

                  {fileImports !== null && (
                    <div className="card file-detail">
                      <div className="panel-title">
                        {t.workspaces.imports} ({fileImports.length})
                      </div>
                      {fileImports.length === 0 ? (
                        <div className="muted">
                          {t.workspaces.noImportsFound}
                        </div>
                      ) : (
                        <ul className="history-list">
                          {fileImports.map((imp) => (
                            <li key={imp.id} className="history-item">
                              <div className="history-status">
                                {imp.isExternal ? '📦' : '📄'}{' '}
                                {imp.targetSpecifier}
                              </div>
                              <div className="history-meta">
                                {imp.importType.replace(/_/g, ' ')}
                                {imp.resolvedTargetFileId !== null &&
                                  ' · resolved'}
                              </div>
                            </li>
                          ))}
                        </ul>
                      )}
                    </div>
                  )}
                </>
              )}

              <div className="card scan-history">
                <div className="panel-title">{t.workspaces.scanHistory}</div>
                {scanHistory === null || scanHistory.length === 0 ? (
                  <div className="muted">{t.workspaces.noScansYet}</div>
                ) : (
                  <ul className="history-list">
                    {scanHistory.map((run) => (
                      <li key={run.id} className="history-item">
                        <div className="history-status">{run.status}</div>
                        <div className="history-meta">
                          {formatTimestamp(run.startedAt, t.general.never)} ·{' '}
                          {t.workspaces.processed} {run.filesProcessed},{' '}
                          {t.workspaces.indexed} {run.filesIndexed}
                        </div>
                        {run.errorMessage !== null && (
                          <div className="history-error">
                            {run.errorMessage}
                          </div>
                        )}
                      </li>
                    ))}
                  </ul>
                )}
              </div>

              {diagnostics !== null && diagnostics.length > 0 && (
                <div className="card scan-history">
                  <div className="panel-title">
                    {t.workspaces.analysisDiagnostics} ({diagnostics.length})
                  </div>
                  <ul className="history-list">
                    {diagnostics.slice(0, 20).map((d) => (
                      <li key={d.id} className="history-item">
                        <div className="history-status">
                          [{d.severity}] {d.line !== null ? `L${d.line}` : '—'}
                        </div>
                        <div className="history-meta">{d.message}</div>
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              <SymbolSearch workspaceId={selectedFolderId} />
            </div>
          </div>
        </div>
      )}

      {removingId !== null && (
        <div className="modal-backdrop" role="presentation">
          <div className="modal" role="dialog" aria-modal="true">
            <div className="modal-title">
              {t.workspaces.removeConfirmTitle}
            </div>
            <p className="modal-body">{t.workspaces.removeConfirmBody}</p>
            <div className="modal-actions">
              <button
                className="btn btn-secondary"
                onClick={() => setRemovingId(null)}
                type="button"
              >
                {t.workspaces.cancelBtn}
              </button>
              <button
                className="btn btn-danger"
                onClick={confirmRemove}
                type="button"
              >
                {t.workspaces.removeBtn}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
