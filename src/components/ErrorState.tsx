interface ErrorStateProps {
  title?: string;
  description: string;
  onRetry?: () => void;
}

export function ErrorState({
  title = 'Something went wrong',
  description,
  onRetry,
}: ErrorStateProps): JSX.Element {
  return (
    <div className="error-state" role="alert">
      <div className="error-state-title">{title}</div>
      <div className="error-state-desc">{description}</div>
      {onRetry !== undefined && (
        <button className="error-state-retry" onClick={onRetry}>
          Retry
        </button>
      )}
    </div>
  );
}
