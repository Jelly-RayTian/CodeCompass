interface EmptyStateProps {
  icon?: string;
  title: string;
  description: string;
}

export function EmptyState({
  icon = '\u{1F4C1}',
  title,
  description,
}: EmptyStateProps): JSX.Element {
  return (
    <div className="empty-state">
      <div className="empty-state-icon" aria-hidden="true">
        {icon}
      </div>
      <div className="empty-state-title">{title}</div>
      <div className="empty-state-desc">{description}</div>
    </div>
  );
}
