interface LoadingStateProps {
  label?: string;
}

export function LoadingState({
  label = 'Loading\u2026',
}: LoadingStateProps): JSX.Element {
  return (
    <div className="loading-state" role="status" aria-live="polite">
      <div className="loading-spinner" aria-hidden="true" />
      <span>{label}</span>
    </div>
  );
}
