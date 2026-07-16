import { useState } from 'react';

import { tauriClient } from '@/lib/tauriClient';
import { useAsyncData } from '@/lib/useAsyncData';
import { LoadingState } from '@/components/LoadingState';
import { useT } from '@/i18n/useT';
import type {
  FileChurn,
  IndexedFolder,
  RepositoryEvolution,
  TimelinePoint,
} from '@/types';
import type { CoChangePair } from '@/types/git';

function formatTs(epoch: number): string {
  if (epoch <= 0) return '\u2014';
  return new Date(epoch * 1000).toLocaleDateString();
}

function timelineChart(points: TimelinePoint[]): JSX.Element {
  if (points.length === 0) return <span>{'\u2014'}</span>;
  const maxCommits = Math.max(...points.map((p) => p.commitCount), 1);
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'flex-end',
        gap: 2,
        height: 120,
        padding: '4px 0',
      }}
    >
      {points.map((p) => (
        <div
          key={p.month}
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            flex: 1,
            minWidth: 0,
          }}
          title={`${p.month}: ${p.commitCount} commits, ${p.fileChanges} files`}
        >
          <div
            style={{
              width: '100%',
              maxWidth: 32,
              height: `${Math.max((p.commitCount / maxCommits) * 100, 4)}%`,
              backgroundColor: '#3b82f6',
              borderRadius: '2px 2px 0 0',
              minHeight: 2,
            }}
          />
          <span style={{ fontSize: 8, transform: 'rotate(-60deg)', marginTop: 4, whiteSpace: 'nowrap' }}>
            {p.month.slice(2)}
          </span>
        </div>
      ))}
    </div>
  );
}

export function GitEvolution(): JSX.Element {
  const { t } = useT();
  const [foldersState] = useAsyncData<IndexedFolder[]>(() =>
    tauriClient.listIndexedFolders(),
  );
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [evolution, setEvolution] = useState<RepositoryEvolution | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchEvolution = (id: number) => {
    setLoading(true);
    setError(null);
    setEvolution(null);
    tauriClient
      .getRepositoryEvolution(id)
      .then(setEvolution)
      .catch((err: unknown) =>
        setError(err instanceof Error ? err.message : String(err)),
      )
      .finally(() => setLoading(false));
  };

  if (foldersState.status === 'loading') {
    return <LoadingState label={t.general.loading} />;
  }
  if (foldersState.status === 'error') {
    return <div className="banner banner-warning">{foldersState.message}</div>;
  }
  const folders = foldersState.data;

  return (
    <div className="evolution-page">
      <h1 className="page-title">{t.evolution.title}</h1>
      <p className="page-subtitle">{t.evolution.subtitle}</p>

      <div className="toolbar">
        <select
          className="select"
          value={selectedId ?? ''}
          onChange={(e) => {
            const id = Number(e.target.value) || null;
            setSelectedId(id);
            if (id !== null) fetchEvolution(id);
          }}
        >
          <option value="">{t.evolution.selectFolder}</option>
          {folders.map((f: IndexedFolder) => (
            <option key={f.id} value={f.id}>
              {f.name}
            </option>
          ))}
        </select>
      </div>

      {error !== null && <div className="banner banner-warning">{error}</div>}
      {loading && <LoadingState label={t.general.loading} />}

      {evolution !== null && !loading && (
        <>
          {/* Summary Cards */}
          <section>
            <h2 className="section-title">{t.evolution.summary}</h2>
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))',
                gap: 12,
                marginBottom: 24,
              }}
            >
              <Card
                label={t.evolution.totalCommits}
                value={String(evolution.summary.totalCommits)}
              />
              <Card
                label={t.evolution.filesChanged}
                value={String(evolution.summary.totalFilesChanged)}
              />
              <Card
                label={t.evolution.fileChanges}
                value={String(evolution.summary.totalFileChanges)}
              />
              <Card
                label={t.evolution.mostActiveMonth}
                value={evolution.summary.mostActiveMonth}
              />
              <Card
                label={t.evolution.dateRange}
                value={`${formatTs(evolution.summary.oldestCommitTs)} \u2013 ${formatTs(evolution.summary.newestCommitTs)}`}
              />
            </div>
          </section>

          {/* Commit Timeline */}
          <section>
            <h2 className="section-title">
              {t.evolution.timeline} ({evolution.timeline.length} months)
            </h2>
            {evolution.timeline.length === 0 ? (
              <div className="card">
                <div className="muted">{t.evolution.noTimeline}</div>
              </div>
            ) : (
              <div className="card">{timelineChart(evolution.timeline)}</div>
            )}
          </section>

          {/* File Churn */}
          <section>
            <h2 className="section-title">
              {t.evolution.fileChurn} ({evolution.topChurnFiles.length})
            </h2>
            {evolution.topChurnFiles.length === 0 ? (
              <div className="card">
                <div className="muted">{t.evolution.noChurn}</div>
              </div>
            ) : (
              <div style={{ overflowX: 'auto' }}>
                <table className="file-table">
                  <thead>
                    <tr>
                      <th>File</th>
                      <th>{t.evolution.changes}</th>
                      <th>{t.evolution.churnBar}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {evolution.topChurnFiles.map((f: FileChurn) => (
                      <tr key={f.relativePath}>
                        <td style={{ fontSize: 13, wordBreak: 'break-all' }}>
                          {f.relativePath}
                        </td>
                        <td style={{ textAlign: 'right', fontSize: 13 }}>
                          {f.changeCount}
                        </td>
                        <td>
                          <ChurnBar
                            count={f.changeCount}
                            max={evolution.topChurnFiles[0]?.changeCount ?? 1}
                          />
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </section>

          {/* Hotspots */}
          <section>
            <h2 className="section-title">
              {t.evolution.hotspots} ({evolution.topHotspots.length})
            </h2>
            {evolution.topHotspots.length === 0 ? (
              <div className="card">
                <div className="muted">{t.evolution.noHotspots}</div>
              </div>
            ) : (
              <div className="finding-list">
                {evolution.topHotspots.map((h: CoChangePair, i: number) => (
                  <div key={i} className="card" style={{ padding: '8px 12px' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                      <span style={{ fontSize: 13 }}>
                        {h.fileA} {'\u2194'} {h.fileB}
                      </span>
                      <span className="muted" style={{ fontSize: 12 }}>
                        {h.togetherCount}x
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </section>

          <div
            className="card"
            style={{ marginTop: 24, fontSize: 12, color: '#6b7280' }}
          >
            {t.evolution.limitation}
          </div>
        </>
      )}

      {evolution === null && !loading && !error && selectedId === null && (
        <div className="card">
          <div className="muted">{t.evolution.selectFolder}</div>
        </div>
      )}
    </div>
  );
}

function Card({ label, value }: { label: string; value: string }) {
  return (
    <div className="card" style={{ textAlign: 'center' }}>
      <div style={{ fontSize: 24, fontWeight: 700 }}>{value}</div>
      <div className="muted" style={{ fontSize: 12 }}>
        {label}
      </div>
    </div>
  );
}

function ChurnBar({ count, max }: { count: number; max: number }) {
  const pct = Math.max((count / max) * 100, 2);
  return (
    <div
      style={{
        height: 12,
        width: `${pct}%`,
        minWidth: 4,
        backgroundColor: '#f97316',
        borderRadius: 2,
      }}
    />
  );
}
