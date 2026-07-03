import { useCallback, useEffect, useState } from 'react';
import { useSearchParams } from 'react-router-dom';

import { tauriClient } from '@/lib/tauriClient';
import { CodeViewer } from '@/components/CodeViewer';
import { EmptyState } from '@/components/EmptyState';
import type { ImportEntry } from '@/types';

export function Viewer(): JSX.Element {
  const [searchParams] = useSearchParams();
  const wsId = searchParams.get('workspaceId');
  const path = searchParams.get('path');
  const lineParam = searchParams.get('line');
  const colParam = searchParams.get('col');

  const workspaceId = wsId !== null ? Number(wsId) : null;
  const filePath = path ?? '';
  const focusLine = lineParam !== null ? Number(lineParam) : undefined;
  const focusColumn = colParam !== null ? Number(colParam) : undefined;

  const [fileInfo, setFileInfo] = useState<{
    language: string;
    totalLines: number;
  } | null>(null);
  const [references, setReferences] = useState<ImportEntry[]>([]);
  const [imports, setImports] = useState<ImportEntry[]>([]);

  const loadImports = useCallback(async () => {
    if (workspaceId === null) return;
    // Find the file ID first, then load imports and references in parallel.
    try {
      const files = await tauriClient.listWorkspaceFiles(workspaceId);
      const match = files.find((f) => f.relativePath === filePath);
      if (match !== undefined) {
        const [imp, refs] = await Promise.all([
          tauriClient.getFileImports(match.id),
          tauriClient.getReferencesToFile(match.id),
        ]);
        setImports(imp);
        setReferences(refs);
      }
    } catch {
      // Non-critical
    }
  }, [workspaceId, filePath]);

  useEffect(() => {
    void loadImports();
  }, [loadImports]);

  if (workspaceId === null || filePath === '') {
    return (
      <EmptyState
        title="No file selected"
        description="Click a file in the tree, graph, or symbol search to view its source."
      />
    );
  }

  return (
    <div className="viewer-page">
      <div className="viewer-toolbar">
        <div className="viewer-file-info">
          <span className="viewer-path">{filePath}</span>
          {fileInfo !== null && (
            <span className="viewer-meta">
              {fileInfo.language} · {fileInfo.totalLines} lines
            </span>
          )}
        </div>
      </div>

      <div className="viewer-layout">
        <div className="viewer-main">
          <CodeViewer
            workspaceId={workspaceId}
            filePath={filePath}
            focusLine={focusLine}
            focusColumn={focusColumn}
            onFileLoaded={setFileInfo}
          />
        </div>

        <div className="viewer-side">
          {imports.length > 0 && (
            <div className="card">
              <div className="panel-title">Imports ({imports.length})</div>
              <ul className="ref-list">
                {imports.map((imp) => (
                  <li key={imp.id}>
                    {imp.isExternal ? '📦' : '📄'} {imp.targetSpecifier}
                    {imp.resolvedTargetFileId !== null && ' ✓'}
                  </li>
                ))}
              </ul>
            </div>
          )}

          {references.length > 0 && (
            <div className="card">
              <div className="panel-title">
                Referenced by ({references.length})
              </div>
              <ul className="ref-list">
                {references.map((ref) => (
                  <li key={ref.id}>📄 {ref.targetSpecifier}</li>
                ))}
              </ul>
            </div>
          )}

          {imports.length === 0 && references.length === 0 && (
            <div className="card">
              <div className="panel-title">References</div>
              <div className="muted">No import data available.</div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
