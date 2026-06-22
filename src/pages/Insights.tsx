import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { tauriClient } from '@/lib/tauriClient';
import { useAsyncData } from '@/lib/useAsyncData';
import { LoadingState } from '@/components/LoadingState';
import type {
  EntryPoint,
  IndexedFolder,
  ReadingPathItem,
  StructuralFinding,
  WorkspaceInsights,
} from '@/types';

function confidenceBar(pct: number): string {
  const p = Math.round(pct * 100);
  return (
    '█'.repeat(Math.round(p / 10)) +
    '░'.repeat(10 - Math.round(p / 10)) +
    ` ${p}%`
  );
}

export function Insights(): JSX.Element {
  const navigate = useNavigate();
  const [foldersState] = useAsyncData<IndexedFolder[]>(() =>
    tauriClient.listIndexedFolders(),
  );
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [insights, setInsights] = useState<WorkspaceInsights | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (selectedId === null) return;
    setLoading(true);
    setError(null);
    tauriClient
      .getWorkspaceInsights(selectedId)
      .then(setInsights)
      .catch((err: unknown) =>
        setError(err instanceof Error ? err.message : String(err)),
      )
      .finally(() => setLoading(false));
  }, [selectedId]);

  if (foldersState.status === 'loading') {
    return <LoadingState label="Loading…" />;
  }
  if (foldersState.status === 'error') {
    return <div className="banner banner-warning">{foldersState.message}</div>;
  }
  const folders = foldersState.data;

  return (
    <div className="insights-page">
      <h1 className="page-title">Insights</h1>
      <p className="page-subtitle">
        Entry points, reading paths, and structural findings for understanding a
        new codebase.
      </p>

      <div className="toolbar">
        <select
          className="select"
          value={selectedId ?? ''}
          onChange={(e) => setSelectedId(Number(e.target.value) || null)}
        >
          <option value="">Select a folder…</option>
          {folders.map((f: IndexedFolder) => (
            <option key={f.id} value={f.id}>
              {f.name}
            </option>
          ))}
        </select>
      </div>

      {error !== null && <div className="banner banner-warning">{error}</div>}

      {loading && <LoadingState label="Analyzing…" />}

      {insights !== null && !loading && (
        <>
          {/* Entry Points */}
          <section>
            <h2 className="section-title">
              Entry Points ({insights.entryPoints.length})
            </h2>
            {insights.entryPoints.length === 0 ? (
              <div className="card">
                <div className="muted">No entry points detected.</div>
              </div>
            ) : (
              <div className="finding-list">
                {insights.entryPoints.map((ep: EntryPoint) => (
                  <div key={ep.fileId} className="card">
                    <div
                      className="finding-header"
                      onClick={() =>
                        navigate(
                          `/viewer?workspaceId=${selectedId}&path=${encodeURIComponent(ep.relativePath)}`,
                        )
                      }
                      role="button"
                      tabIndex={0}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter')
                          navigate(
                            `/viewer?workspaceId=${selectedId}&path=${encodeURIComponent(ep.relativePath)}`,
                          );
                      }}
                      style={{ cursor: 'pointer' }}
                    >
                      <span>{ep.name}</span>
                      <span className="finding-confidence">
                        {confidenceBar(ep.confidence)}
                      </span>
                    </div>
                    <ul className="finding-evidence">
                      {ep.reasons.map((r, i) => (
                        <li key={i}>{r}</li>
                      ))}
                    </ul>
                  </div>
                ))}
              </div>
            )}
          </section>

          {/* Reading Path */}
          <section>
            <h2 className="section-title">
              Reading Path ({insights.readingPath.length} files)
            </h2>
            {insights.readingPath.length === 0 ? (
              <div className="card">
                <div className="muted">
                  No reading path available. Run Analyze first.
                </div>
              </div>
            ) : (
              <div className="reading-path">
                {insights.readingPath.map((item: ReadingPathItem) => (
                  <div
                    key={item.fileId}
                    className="reading-item"
                    onClick={() =>
                      navigate(
                        `/viewer?workspaceId=${selectedId}&path=${encodeURIComponent(item.relativePath)}`,
                      )
                    }
                    role="button"
                    tabIndex={0}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter')
                        navigate(
                          `/viewer?workspaceId=${selectedId}&path=${encodeURIComponent(item.relativePath)}`,
                        );
                    }}
                  >
                    <span className="reading-order">{item.order + 1}.</span>
                    <span className="reading-name">{item.name}</span>
                    <span className="reading-depth">depth {item.depth}</span>
                    <span className="reading-reason">{item.reason}</span>
                  </div>
                ))}
              </div>
            )}
          </section>

          {/* Structural Findings */}
          <section>
            <h2 className="section-title">
              Structural Findings ({insights.findings.length})
            </h2>
            {insights.findings.length === 0 ? (
              <div className="card">
                <div className="muted">No findings. Everything looks good.</div>
              </div>
            ) : (
              <div className="finding-list">
                {insights.findings.map((f: StructuralFinding, i: number) => (
                  <div key={i} className="card finding-card">
                    <div className="finding-header">
                      <span className={`finding-severity fsev-${f.severity}`}>
                        {f.severity}
                      </span>
                      <span>{f.title}</span>
                    </div>
                    <p className="muted">{f.description}</p>
                    <div className="finding-evidence">
                      <strong>Evidence:</strong>
                      <ul>
                        {f.evidence.slice(0, 5).map((e, j) => (
                          <li key={j}>{e}</li>
                        ))}
                        {f.evidence.length > 5 && (
                          <li>…and {f.evidence.length - 5} more</li>
                        )}
                      </ul>
                    </div>
                    <p className="muted" style={{ fontSize: 11 }}>
                      <strong>Limitation:</strong> {f.limitation}
                    </p>
                    <p className="muted" style={{ fontSize: 11 }}>
                      <strong>How to investigate:</strong> {f.investigation}
                    </p>
                  </div>
                ))}
              </div>
            )}
          </section>
        </>
      )}
    </div>
  );
}
