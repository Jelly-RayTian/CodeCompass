import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, beforeEach } from 'vitest';

import { App } from '@/app/App';
import { mockTauriCommand } from '@/test/setup';

const mockAppInfo = {
  name: 'CodeCompass',
  version: '0.1.0',
  buildTimestamp: '2026-01-01T00:00:00Z',
};

const mockDbStatus = {
  connected: true,
  databasePath: '/tmp/test.db',
  migrationVersion: 1,
};

describe('App', () => {
  beforeEach(() => {
    mockTauriCommand('get_application_info', async () => mockAppInfo);
    mockTauriCommand('get_database_status', async () => mockDbStatus);
    mockTauriCommand('list_indexed_folders_command', async () => []);
  });

  it('renders the application shell with brand text', () => {
    render(<App />);
    expect(screen.getByText('CodeCompass')).toBeInTheDocument();
    expect(screen.getByText('Understand any codebase')).toBeInTheDocument();
  });

  it('shows the home page with application version on initial load', async () => {
    render(<App />);
    expect(await screen.findByText('0.1.0')).toBeInTheDocument();
  });

  it('shows database status on the home page', async () => {
    render(<App />);
    expect(await screen.findByText('Connected')).toBeInTheDocument();
    expect(screen.getByText('/tmp/test.db')).toBeInTheDocument();
  });

  it('navigates to the Workspaces page and shows empty state', async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole('link', { name: /workspaces/i }));
    expect(
      await screen.findByText('No indexed folders yet'),
    ).toBeInTheDocument();
  });

  it('navigates to the Settings page', async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole('link', { name: /settings/i }));
    expect(await screen.findByText('Database Status')).toBeInTheDocument();
  });
});
