import { tauriClient } from '@/lib/tauriClient';
import { useAsyncData } from '@/lib/useAsyncData';
import type { ApplicationInfo } from '@/types';
import { ErrorState } from '@/components/ErrorState';
import { LoadingState } from '@/components/LoadingState';

export function Settings(): JSX.Element {
  const [state, reload] = useAsyncData<ApplicationInfo>(() =>
    tauriClient.getApplicationInfo(),
  );

  if (state.status === 'loading') {
    return <LoadingState label="Loading settings\u2026" />;
  }

  if (state.status === 'error') {
    return (
      <ErrorState
        title="Failed to load settings"
        description={state.message}
        onRetry={reload}
      />
    );
  }

  const { name, version, buildTimestamp } = state.data;

  return (
    <>
      <h1 className="page-title">Settings</h1>
      <p className="page-subtitle">Application information and preferences.</p>

      <div className="card">
        <div className="card-label">Application name</div>
        <div className="card-value">{name}</div>
      </div>

      <div className="card">
        <div className="card-label">Version</div>
        <div className="card-value">{version}</div>
      </div>

      <div className="card">
        <div className="card-label">Build timestamp</div>
        <div className="card-value">{buildTimestamp}</div>
      </div>

      <div className="card">
        <div className="card-label">Preferences</div>
        <div className="card-value" style={{ color: 'var(--cc-text-muted)' }}>
          Configurable preferences will appear here in a future milestone.
        </div>
      </div>
    </>
  );
}
