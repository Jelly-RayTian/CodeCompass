import { useNavigate } from 'react-router-dom';

import { tauriClient } from '@/lib/tauriClient';
import { useAsyncData } from '@/lib/useAsyncData';
import type { ApplicationInfo, DatabaseStatus, IndexedFolder } from '@/types';
import { ErrorState } from '@/components/ErrorState';
import { LoadingState } from '@/components/LoadingState';
import { useT } from '@/i18n/LanguageContext';

type HomeData = { info: ApplicationInfo; db: DatabaseStatus; folders: IndexedFolder[] };

export function Home(): JSX.Element {
  const { t } = useT();
  const navigate = useNavigate();
  const [state, reload] = useAsyncData<HomeData>(async () => {
    const [info, db, folders] = await Promise.all([
      tauriClient.getApplicationInfo(),
      tauriClient.getDatabaseStatus(),
      tauriClient.listIndexedFolders(),
    ]);
    return { info, db, folders };
  });

  if (state.status === 'loading') {
    return <LoadingState label={t.general.loading} />;
  }

  if (state.status === 'error') {
    return (
      <ErrorState title="Error" description={state.message} onRetry={reload} />
    );
  }

  const { info, db, folders } = state.data;
  const hasFolders = folders.length > 0;

  return (
    <>
      <h1 className="page-title">{t.home.title}</h1>
      <p className="page-subtitle">{t.home.subtitle}</p>

      <div className="home-quick-actions">
        <button
          className="btn btn-primary home-cta"
          onClick={() => navigate('/workspaces')}
          type="button"
        >
          {t.home.openWorkspaces}
        </button>
        {!hasFolders && (
          <p className="home-hint">{t.home.scanFirst}</p>
        )}
      </div>

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
        <div className="card-label">{t.settings.databaseStatus}</div>
        <div className="card-value">
          <span
            className={`status-dot ${db.connected ? 'connected' : 'disconnected'}`}
            aria-hidden="true"
          />
          {db.connected ? t.settings.connected : t.settings.notConnected}
        </div>
      </div>

      <div className="card">
        <div className="card-label">{t.settings.path}</div>
        <div className="card-value">{db.databasePath}</div>
      </div>

      <div className="card">
        <div className="card-label">{t.settings.migrationVersion}</div>
        <div className="card-value">
          {db.migrationVersion >= 0
            ? `v${db.migrationVersion}`
            : 'No migrations'}
        </div>
      </div>
    </>
  );
}
