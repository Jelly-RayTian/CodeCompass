import { useEffect, useState } from 'react';

import { tauriClient } from '@/lib/tauriClient';
import type { CoChangePair, GitInfo, WorkspaceSettings } from '@/types';

interface GitPanelProps {
  workspaceId: number;
}

function formatTs(epoch: number | null): string {
  if (epoch === null) return '—';
  return new Date(epoch * 1000).toLocaleString();
}

export function GitPanel({ workspaceId }: GitPanelProps): JSX.Element {
  const [gitInfo, setGitInfo] = useState<GitInfo | null>(null);
  const [settings, setSettings] = useState<WorkspaceSettings | null>(null);
  const [hotspots, setHotspots] = useState<CoChangePair[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const load = async (): Promise<void> => {
      setLoading(true);
      try {
        const [info, s, h] = await Promise.all([
          tauriClient.getGitInfo(workspaceId),
          tauriClient.getWorkspaceSettings(workspaceId),
          tauriClient.getCoChangeHotspots(workspaceId),
        ]);
        if (cancelled) return;
        setGitInfo(info);
        setSettings(s);
        setHotspots(h);
      } catch {
        // non-critical
      } finally {
        if (!cancelled) setLoading(false);
      }
    };
    load();
    return () => {
      cancelled = true;
    };
  }, [workspaceId]);

  const toggleSetting = async (
    key: 'gitAnalysisEnabled' | 'autoReanalyzeEnabled',
  ): Promise<void> => {
    if (settings === null) return;
    const current =
      key === 'gitAnalysisEnabled'
        ? settings.gitAnalysisEnabled
        : settings.autoReanalyzeEnabled;
    try {
      await tauriClient.updateWorkspaceSettings(
        workspaceId,
        key === 'gitAnalysisEnabled' ? !current : undefined,
        key === 'autoReanalyzeEnabled' ? !current : undefined,
      );
      const updated = await tauriClient.getWorkspaceSettings(workspaceId);
      setSettings(updated);
    } catch {
      // ignore
    }
  };

  if (loading) {
    return (
      <div className="card">
        <div className="muted">Loading Git info…</div>
      </div>
    );
  }

  return (
    <div className="card git-panel">
      <div className="panel-title">Git Repository</div>

      {gitInfo !== null && !gitInfo.isRepo && (
        <div className="muted">Not a Git repository.</div>
      )}

      {gitInfo !== null && gitInfo.isRepo && (
        <>
          <div className="git-info-grid">
            <div>
              <span className="git-label">Branch</span>
              <span className="git-value">{gitInfo.branch ?? '—'}</span>
            </div>
            <div>
              <span className="git-label">Status</span>
              <span
                className={`git-value ${gitInfo.status === 'dirty' ? 'status-changed' : ''}`}
              >
                {gitInfo.status ?? '—'}
              </span>
            </div>
            <div>
              <span className="git-label">Commits</span>
              <span className="git-value">{gitInfo.commitCount ?? '—'}</span>
            </div>
            <div>
              <span className="git-label">Last commit</span>
              <span className="git-value">
                {gitInfo.lastCommitShort ?? '—'}
              </span>
            </div>
            <div>
              <span className="git-label">Date</span>
              <span className="git-value">
                {formatTs(gitInfo.lastCommitTimestamp)}
              </span>
            </div>
            <div>
              <span className="git-label">Message</span>
              <span className="git-value">
                {gitInfo.lastCommitMessage ?? '—'}
              </span>
            </div>
          </div>

          {settings !== null && (
            <div className="git-settings">
              <label className="git-toggle">
                <input
                  type="checkbox"
                  checked={settings.gitAnalysisEnabled}
                  onChange={() => toggleSetting('gitAnalysisEnabled')}
                />
                <span>Git analysis (commit history, hotspots)</span>
              </label>
              <label className="git-toggle">
                <input
                  type="checkbox"
                  checked={settings.autoReanalyzeEnabled}
                  onChange={() => toggleSetting('autoReanalyzeEnabled')}
                />
                <span>Auto re-analyze on file changes</span>
              </label>
            </div>
          )}
        </>
      )}

      {hotspots.length > 0 && (
        <div className="git-hotspots">
          <div className="detail-label">
            Frequently Changed Together ({hotspots.length})
          </div>
          <ul className="history-list">
            {hotspots.slice(0, 5).map((h, i) => (
              <li key={i} className="history-item">
                <span className="muted">
                  {h.fileA} ↔ {h.fileB}
                </span>
                <span className="muted"> ({h.togetherCount}x)</span>
              </li>
            ))}
            {hotspots.length === 0 && (
              <li className="muted">No co-change data available.</li>
            )}
          </ul>
        </div>
      )}
    </div>
  );
}
