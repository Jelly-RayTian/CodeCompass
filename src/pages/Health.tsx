import { useState } from 'react';

import { tauriClient } from '@/lib/tauriClient';
import { useAsyncData } from '@/lib/useAsyncData';
import { LoadingState } from '@/components/LoadingState';
import { useT } from '@/i18n/useT';
import type { FileHealth, IndexedFolder, RepositoryHealth } from '@/types';

function riskBar(score: number): string {
  const p = Math.round(score);
  return (
    '\u2588'.repeat(Math.round(p / 10)) +
    '\u2591'.repeat(10 - Math.round(p / 10)) +
    ` ${p}`
  );
}

function riskColor(category: string): string {
  switch (category) {
    case 'critical':
      return '#dc2626';
    case 'high':
      return '#ea580c';
    case 'medium':
      return '#ca8a04';
    case 'low':
      return '#16a34a';
    default:
      return '#6b7280';
  }
}

export function Health(): JSX.Element {
  const { t } = useT();
  const [foldersState] = useAsyncData<IndexedFolder[]>(() =>
    tauriClient.listIndexedFolders(),
  );
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [health, setHealth] = useState<RepositoryHealth | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showAll, setShowAll] = useState(false);

  const fetchHealth = (id: number) => {
    setLoading(true);
    setError(null);
    setHealth(null);
    tauriClient
      .getRepositoryHealth(id)
      .then(setHealth)
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
    <div className="health-page">
      <h1 className="page-title">{t.health.title}</h1>
      <p className="page-subtitle">{t.health.subtitle}</p>

      <div className="toolbar">
        <select
          className="select"
          value={selectedId ?? ''}
          onChange={(e) => {
            const id = Number(e.target.value) || null;
            setSelectedId(id);
            setShowAll(false);
            if (id !== null) fetchHealth(id);
          }}
        >
          <option value="">{t.health.selectFolder}</option>
          {folders.map((f: IndexedFolder) => (
            <option key={f.id} value={f.id}>
              {f.name}
            </option>
          ))}
        </select>
      </div>

      {error !== null && <div className="banner banner-warning">{error}</div>}

      {loading && <LoadingState label={t.general.loading} />}

      {health !== null && !loading && (
        <>
          {/* Summary Cards */}
          <section>
            <h2 className="section-title">{t.health.summary}</h2>
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))',
                gap: 12,
                marginBottom: 24,
              }}
            >
              <Card
                label={t.health.totalFiles}
                value={String(health.summary.totalFiles)}
              />
              <Card
                label={t.health.filesAnalyzed}
                value={String(health.summary.filesAnalyzed)}
              />
              <Card
                label={t.health.totalImports}
                value={String(health.summary.totalImports)}
              />
              <Card
                label={t.health.totalSymbols}
                value={String(health.summary.totalSymbols)}
              />
              <Card
                label={t.health.cyclesDetected}
                value={String(health.summary.cycleCount)}
                highlight={health.summary.cycleCount > 0}
              />
              <Card
                label={t.health.avgRiskScore}
                value={`${health.summary.avgRiskScore.toFixed(1)}`}
                highlight={health.summary.avgRiskScore >= 25}
              />
            </div>
          </section>

          {/* Risk Distribution */}
          <section>
            <h2 className="section-title">{t.health.riskDistribution}</h2>
            <div style={{ display: 'flex', gap: 12, marginBottom: 24, flexWrap: 'wrap' }}>
              <RiskBadge label={t.health.lowRisk} count={health.summary.filesLowRisk} color="#16a34a" />
              <RiskBadge label={t.health.mediumRisk} count={health.summary.filesMediumRisk} color="#ca8a04" />
              <RiskBadge label={t.health.highRisk} count={health.summary.filesHighRisk} color="#ea580c" />
              <RiskBadge label={t.health.criticalRisk} count={health.summary.filesCriticalRisk} color="#dc2626" />
            </div>
          </section>

          {/* File Table */}
          <section>
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
              }}
            >
              <h2 className="section-title">
                {showAll ? t.health.allFiles : t.health.topRiskFiles}
              </h2>
              <button
                className="btn"
                type="button"
                onClick={() => setShowAll(!showAll)}
              >
                {showAll
                  ? t.health.topRiskFiles
                  : `${t.health.allFiles} (${health.allFiles.length})`}
              </button>
            </div>

            {health.allFiles.length === 0 ? (
              <div className="card">
                <div className="muted">{t.health.noData}</div>
              </div>
            ) : (
              <FileTable files={showAll ? health.allFiles : health.topRiskFiles} t={t} />
            )}
          </section>

          {/* Limitation Note */}
          <div
            className="card"
            style={{ marginTop: 24, fontSize: 12, color: '#6b7280' }}
          >
            {t.health.limitation}
          </div>
        </>
      )}

      {health === null && !loading && !error && selectedId === null && (
        <div className="card">
          <div className="muted">{t.health.selectFolder}</div>
        </div>
      )}
    </div>
  );
}

function Card({
  label,
  value,
  highlight,
}: {
  label: string;
  value: string;
  highlight?: boolean;
}) {
  return (
    <div
      className="card"
      style={{
        textAlign: 'center',
        borderLeft: highlight ? '3px solid #ea580c' : undefined,
      }}
    >
      <div style={{ fontSize: 24, fontWeight: 700 }}>{value}</div>
      <div className="muted" style={{ fontSize: 12 }}>
        {label}
      </div>
    </div>
  );
}

function RiskBadge({
  label,
  count,
  color,
}: {
  label: string;
  count: number;
  color: string;
}) {
  return (
    <div
      className="card"
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        padding: '6px 12px',
        minWidth: 120,
      }}
    >
      <div
        style={{
          width: 12,
          height: 12,
          borderRadius: 3,
          backgroundColor: color,
        }}
      />
      <span style={{ fontWeight: 600 }}>{count}</span>
      <span className="muted" style={{ fontSize: 12 }}>
        {label}
      </span>
    </div>
  );
}

function FileTable({
  files,
  t,
}: {
  files: FileHealth[];
  t: ReturnType<typeof useT>['t'];
}) {
  return (
    <div style={{ overflowX: 'auto' }}>
      <table className="file-table">
        <thead>
          <tr>
            <th>File</th>
            <th>{t.health.riskScore}</th>
            <th>{t.health.lines}</th>
            <th>{t.health.imports}</th>
            <th>{t.health.symbols}</th>
            <th>{t.health.changes}</th>
            <th>{t.health.inCycle}</th>
          </tr>
        </thead>
        <tbody>
          {files.map((f: FileHealth) => (
            <tr key={f.fileId}>
              <td>
                <span
                  style={{ fontSize: 13, wordBreak: 'break-all' }}
                  title={f.relativePath}
                >
                  {f.relativePath}
                </span>
              </td>
              <td>
                <span
                  style={{
                    color: riskColor(f.riskCategory),
                    fontWeight: 600,
                    fontSize: 13,
                    whiteSpace: 'nowrap',
                  }}
                  title={riskBar(f.riskScore)}
                >
                  {riskBar(f.riskScore)}
                </span>
              </td>
              <td style={{ textAlign: 'right', fontSize: 13 }}>
                {f.lineCount}
              </td>
              <td style={{ textAlign: 'right', fontSize: 13 }}>
                {f.importOutDegree + f.importInDegree}
              </td>
              <td style={{ textAlign: 'right', fontSize: 13 }}>
                {f.symbolCount}
              </td>
              <td style={{ textAlign: 'right', fontSize: 13 }}>
                {f.changeCount}
              </td>
              <td style={{ textAlign: 'center', fontSize: 13 }}>
                {f.isInCycle ? '\u26A0' : '-'}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
