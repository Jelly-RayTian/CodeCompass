import { tauriClient } from '@/lib/tauriClient';
import { useAsyncData } from '@/lib/useAsyncData';
import type { DatabaseStatus } from '@/types';
import { ErrorState } from '@/components/ErrorState';
import { LoadingState } from '@/components/LoadingState';
import { useT } from '@/i18n/useT';

export function Settings(): JSX.Element {
  const { t } = useT();
  const [state, reload] = useAsyncData<DatabaseStatus>(() =>
    tauriClient.getDatabaseStatus(),
  );

  if (state.status === 'loading') {
    return <LoadingState label={t.general.loading} />;
  }

  if (state.status === 'error') {
    return (
      <ErrorState
        title={t.settings.title}
        description={state.message}
        onRetry={reload}
      />
    );
  }

  const db = state.data;

  return (
    <>
      <h1 className="page-title">{t.settings.title}</h1>
      <p className="page-subtitle">{t.settings.subtitle}</p>

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
        <div className="card-value">v{db.migrationVersion}</div>
      </div>
    </>
  );
}
