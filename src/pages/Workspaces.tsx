import { useEffect, useRef, useState } from 'react';
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

function formatTimestamp(epochSeconds: number | null | undefined): string {
  if (epochSeconds === null || epochSeconds === undefined) {
    return 'Never';
  }
  return new Date(epochSeconds * 1000).toLocaleString();
}

function availabilityLabel(
  availability: IndexedFolder['availability'],
): string {
  switch (availability) {
    case 'available':
      return 'Available';
    case 'missing':
      return 'Missing';
    case 'inaccessible':
      return 'Inaccessible';
    case 'permission_denied':
      return 'Permission denied';
    case 'not_a_directory':
      return 'Not a directory';
    default:
      return 'Unknown';
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
    return <LoadingState label="Loading indexed folders\u2026" />;
  }

  if (foldersState.status === 'error') {
    return (
      <ErrorState
        title="Failed to load indexed folders"
        description={foldersState.message}
        onRetry={reloadFolders}
      />
    );
  }

  const folders = foldersState.data;
  const tree = files !== null ? buildTree(files) : null;

  return (
    <>
      <h1 className="page-title">Indexed Folders</h1>
      <p className="page-subtitle">
        Folders you have registered for local metadata scanning.
      </p>

      <div className="toolbar">
        <button
          className="btn btn-primary"
          onClick={handleAddFolder}
          type="button"
        >
          Add folder
        </button>
      </div>

      {warning !== null && (
        <div className="banner banner-warning" role="alert">
          {warning}
        </div>
      )}

      {completedScan !== null && (
        <div className="banner banner-success" role="status">
          Scan complete: {completedScan.run.filesProcessed} processed,{' '}
          {completedScan.run.filesIndexed} indexed,{' '}
          {completedScan.run.warningCount} warnings,{' '}
          {completedScan.run.errorCount} errors.
        </div>
      )}

      {folders.length === 0 ? (
        <EmptyState
          title="No indexed folders yet"
          description="Add a local folder to start scanning its file structure. Original files are never modified."
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
                    <div className="folder-meta-label">Availability</div>
                    <div className="folder-meta-value">
                      {availabilityLabel(folder.availability)}
                    </div>
                  </div>
                  <div>
                    <div className="folder-meta-label">Monitoring</div>
                    <div className="folder-meta-value">
                      {folder.monitoringEnabled ? 'Enabled' : 'Disabled'}
                    </div>
                  </div>
                  <div>
                    <div className="folder-meta-label">Scan status</div>
                    <div className="folder-meta-value">{folder.scanStatus}</div>
                  </div>
                  <div>
                    <div className="folder-meta-label">Files indexed</div>
                    <div className="folder-meta-value">
                      {status?.fileCount ?? 0}
                    </div>
                  </div>
                  <div>
                    <div className="folder-meta-label">
                      Last successful scan
                    </div>
                    <div className="folder-meta-value">
                      {formatTimestamp(folder.lastSuccessfulScanAt)}
                    </div>
                  </div>
                  <div>
                    <div className="folder-meta-label">Added</div>
                    <div className="folder-meta-value">
                      {formatTimestamp(folder.addedAt)}
                    </div>
                  </div>
                </div>

                {isScanning && status !== undefined && (
                  <div className="scan-progress">
                    <div className="scan-phase">
                      Phase: {status.run.phase ?? 'scanning'}
                    </div>
                    <div className="scan-counters">
                      Processed {status.run.filesProcessed} · Indexed{' '}
                      {status.run.filesIndexed} · Warnings{' '}
                      {status.run.warningCount} · Errors {status.run.errorCount}
                    </div>
                  </div>
                )}

                {analyzingFolderId === folder.id &&
                  (() => {
                    const ap = analysisProgress[folder.id];
                    if (ap === undefined) return null;
                    return (
                      <div className="scan-progress">
                        <div className="scan-phase">Analyzing…</div>
                        <div className="scan-counters">
                          Files {ap.filesProcessed} / {ap.filesTotal} · Parsed{' '}
                          {ap.filesParsed}
                        </div>
                      </div>
                    );
                  })()}

                <div className="folder-actions">
                  {isScanning ? (
                    <button
                      className="btn btn-danger"
                      onClick={handleCancelScan}
                      type="button"
                    >
                      Cancel scan
                    </button>
                  ) : (
                    <>
                      <button
                        className="btn btn-primary"
                        disabled={folder.availability !== 'available'}
                        onClick={() => handleScan(folder.id)}
                        type="button"
                      >
                        Scan folder
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
                          ? 'Analyzing…'
                          : 'Analyze'}
                      </button>
                    </>
                  )}
                  {analyzingFolderId === folder.id && (
                    <button
                      className="btn btn-danger"
                      onClick={handleCancelAnalysis}
                      type="button"
                    >
                      Cancel
                    </button>
                  )}
                  <button
                    className={`btn btn-secondary ${isSelected ? 'active' : ''}`}
                    onClick={() => handleSelectFolder(folder.id)}
                    type="button"
                  >
                    View files
                  </button>
                  <button
                    className="btn btn-secondary"
                    onClick={() => tauriClient.revealFolder(folder.path)}
                    type="button"
                  >
                    Reveal
                  </button>
                  <button
                    className="btn btn-danger"
                    onClick={() => handleRemove(folder.id)}
                    type="button"
                  >
                    Remove
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {selectedFolderId !== null && (
        <div className="workspace-detail">
          <h2 className="section-title">Files &amp; History</h2>
          <div className="detail-grid">
            <div className="detail-panel">
              <div className="panel-title">Indexed file tree</div>
              {filesLoading ? (
                <LoadingState label="Loading files\u2026" />
              ) : tree === null || tree.children.length === 0 ? (
                <EmptyState
                  title="No indexed files"
                  description="Run a successful scan to populate the file tree."
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
                      <span>Relative path</span>
                      <span>{selectedFile.relativePath}</span>
                    </div>
                    <div className="file-detail-row">
                      <span>Extension</span>
                      <span>{selectedFile.extension ?? '-'}</span>
                    </div>
                    <div className="file-detail-row">
                      <span>Size</span>
                      <span>{selectedFile.sizeBytes} bytes</span>
                    </div>
                    <div className="file-detail-row">
                      <span>Modified</span>
                      <span>{formatTimestamp(selectedFile.modifiedAt)}</span>
                    </div>
                    <div className="file-detail-row">
                      <span>Created</span>
                      <span>{formatTimestamp(selectedFile.createdAt)}</span>
                    </div>
                    <div className="file-detail-row">
                      <span>Fingerprint</span>
                      <span className="mono">
                        {selectedFile.fingerprint ?? '-'}
                      </span>
                    </div>
                    <div className="file-detail-row">
                      <span>Status</span>
                      <span
                        className={changeStatusClass(selectedFile.changeStatus)}
                      >
                        {selectedFile.changeStatus}
                      </span>
                    </div>
                  </div>

                  {fileImports !== null && (
                    <div className="card file-detail">
                      <div className="panel-title">
                        Imports ({fileImports.length})
                      </div>
                      {fileImports.length === 0 ? (
                        <div className="muted">No imports found.</div>
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
                <div className="panel-title">Scan history</div>
                {scanHistory === null || scanHistory.length === 0 ? (
                  <div className="muted">No scans yet.</div>
                ) : (
                  <ul className="history-list">
                    {scanHistory.map((run) => (
                      <li key={run.id} className="history-item">
                        <div className="history-status">{run.status}</div>
                        <div className="history-meta">
                          {formatTimestamp(run.startedAt)} · processed{' '}
                          {run.filesProcessed}, indexed {run.filesIndexed}
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
                    Analysis Diagnostics ({diagnostics.length})
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
            </div>
          </div>
        </div>
      )}

      {removingId !== null && (
        <div className="modal-backdrop" role="presentation">
          <div className="modal" role="dialog" aria-modal="true">
            <div className="modal-title">Remove indexed folder?</div>
            <p className="modal-body">
              This will remove Chronicle&apos;s index for this folder and stop
              future scans. Your original files will not be deleted, moved, or
              changed.
            </p>
            <div className="modal-actions">
              <button
                className="btn btn-secondary"
                onClick={() => setRemovingId(null)}
                type="button"
              >
                Cancel
              </button>
              <button
                className="btn btn-danger"
                onClick={confirmRemove}
                type="button"
              >
                Remove
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
