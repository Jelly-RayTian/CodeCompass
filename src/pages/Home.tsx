import { tauriClient } from '@/lib/tauriClient';
import { useAsyncData } from '@/lib/useAsyncData';
import type { ApplicationInfo, DatabaseStatus } from '@/types';
import { ErrorState } from '@/components/ErrorState';
import { LoadingState } from '@/components/LoadingState';

type HomeData = { info: ApplicationInfo; db: DatabaseStatus };

export function Home(): JSX.Element {
  const [state, reload] = useAsyncData<HomeData>(async () => {
    const [info, db] = await Promise.all([
      tauriClient.getApplicationInfo(),
      tauriClient.getDatabaseStatus(),
    ]);
    return { info, db };
  });

  if (state.status === 'loading') {
    return <LoadingState label="Loading application info\u2026" />;
  }

  if (state.status === 'error') {
    return (
      <ErrorState
        title="Failed to load application info"
        description={state.message}
        onRetry={reload}
      />
    );
  }

  const { info, db } = state.data;

  return (
    <>
      <h1 className="page-title">Home</h1>
      <p className="page-subtitle">
        CodeCompass helps you understand unfamiliar codebases by analyzing
        structure locally.
      </p>

      <div className="card-grid">
        <div className="card">
          <div className="card-label">Application</div>
          <div className="card-value">{info.name}</div>
        </div>
        <div className="card">
          <div className="card-label">Version</div>
          <div className="card-value">{info.version}</div>
        </div>
        <div className="card">
          <div className="card-label">Build</div>
          <div className="card-value">{info.buildTimestamp}</div>
        </div>
      </div>

      <div className="card">
        <div className="card-label">Database</div>
        <div className="card-value">
          <span
            className={`status-dot ${db.connected ? 'connected' : 'disconnected'}`}
            aria-hidden="true"
          />
          {db.connected ? 'Connected' : 'Disconnected'}
        </div>
      </div>

      <div className="card">
        <div className="card-label">Database path</div>
        <div className="card-value">{db.databasePath}</div>
      </div>

      <div className="card">
        <div className="card-label">Migration version</div>
        <div className="card-value">
          {db.migrationVersion >= 0
            ? `v${db.migrationVersion}`
            : 'No migrations applied'}
        </div>
      </div>
    </>
  );
}
