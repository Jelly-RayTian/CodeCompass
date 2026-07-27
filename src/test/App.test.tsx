import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, beforeEach } from 'vitest';

import { App } from '@/app/App';
import { formatDatabasePath } from '@/lib/formatPath';
import { mockTauriCommand } from '@/test/setup';

const mockAppInfo = {
  name: 'CodeCompass',
  version: '0.1.1',
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

  it('renders the application shell with brand text', async () => {
    render(<App />);
    expect(await screen.findByText('CodeCompass')).toBeInTheDocument();
    expect(screen.getByText('Understand any codebase')).toBeInTheDocument();
  });

  it('shows the home page with application version on initial load', async () => {
    render(<App />);
    expect(await screen.findByText('0.1.1')).toBeInTheDocument();
  });

  it('shows database status on the home page', async () => {
    render(<App />);
    expect(await screen.findByText('Connected')).toBeInTheDocument();
    expect(screen.getByText('/tmp/test.db')).toBeInTheDocument();
  });

  it('redacts the Windows user profile from the displayed database path', () => {
    expect(
      formatDatabasePath(
        'C:\\Users\\Developer\\AppData\\Roaming\\io.github.jellyraytian.codecompass\\codecompass.db',
      ),
    ).toBe('%APPDATA%\\io.github.jellyraytian.codecompass\\codecompass.db');
  });

  it('navigates to the Workspaces page and shows empty state', async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole('link', { name: /workspaces/i }));
    expect(
      await screen.findByText('No indexed folders yet'),
    ).toBeInTheDocument();
  });

  it('loads the persisted file count when the Workspaces page opens', async () => {
    mockTauriCommand('list_indexed_folders_command', async () => [
      {
        id: 1,
        displayName: 'CodeCompass',
        absolutePath: 'D:\\CodeCompass',
        addedAt: 1,
        lastSuccessfulScanAt: 2,
        scanStatus: 'idle',
        availability: 'available',
        monitoringEnabled: false,
      },
    ]);
    mockTauriCommand('get_scan_status', async () => ({
      run: {
        id: 2,
        workspaceId: 1,
        status: 'completed',
        startedAt: 1,
        completedAt: 2,
        filesProcessed: 206,
        filesIndexed: 44,
        warningCount: 0,
        errorCount: 0,
        phase: 'finished',
        errorMessage: null,
      },
      fileCount: 44,
    }));

    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole('link', { name: /workspaces/i }));

    expect(await screen.findByText('44')).toBeInTheDocument();
  });

  it('navigates to the Settings page', async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole('link', { name: /settings/i }));
    expect(await screen.findByText('Database Status')).toBeInTheDocument();
  });

  it('navigates to the Insights page and shows folder selector', async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole('link', { name: /insights/i }));
    expect(
      await screen.findByRole('heading', { name: 'Insights' }),
    ).toBeInTheDocument();
    expect(screen.getByText(/Select a folder/)).toBeInTheDocument();
  });

  it('loads the source viewer on demand', async () => {
    render(<App initialEntries={['/viewer']} />);
    expect(await screen.findByText('No file selected')).toBeInTheDocument();
  });

  it('shows error state when workspace list fails', async () => {
    mockTauriCommand('list_indexed_folders_command', async () => {
      throw new Error('database locked');
    });
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole('link', { name: /workspaces/i }));
    expect(
      await screen.findByText('Failed to load indexed folders'),
    ).toBeInTheDocument();
  });

  it('retry button appears on error state', async () => {
    mockTauriCommand('list_indexed_folders_command', async () => {
      throw new Error('transient');
    });
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole('link', { name: /workspaces/i }));
    const retry = await screen.findByRole('button', { name: /retry/i });
    expect(retry).toBeInTheDocument();
  });
});
