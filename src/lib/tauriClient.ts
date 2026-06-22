import { invoke, type InvokeArgs } from '@tauri-apps/api/core';

import type {
  AddFolderResult,
  AnalysisDiagnostic,
  ApplicationInfo,
  DatabaseStatus,
  DependencyGraph,
  FileEntry,
  ImportEntry,
  IndexedFolder,
  ScanRun,
  ScanStatus,
} from '@/types';

/**
 * Typed wrapper around Tauri's `invoke`.
 *
 * All calls to the Rust backend go through this module so that:
 *  - the frontend never imports raw `invoke` with untyped args;
 *  - every command name and return type is declared in one place;
 *  - tests can mock a single module instead of patching globally.
 */
async function call<T>(command: string, args?: InvokeArgs): Promise<T> {
  return invoke<T>(command, args);
}

export const tauriClient = {
  getApplicationInfo(): Promise<ApplicationInfo> {
    return call<ApplicationInfo>('get_application_info');
  },

  getDatabaseStatus(): Promise<DatabaseStatus> {
    return call<DatabaseStatus>('get_database_status');
  },

  pickFolder(): Promise<string | null> {
    return call<string | null>('pick_folder');
  },

  addFolder(path: string): Promise<AddFolderResult> {
    return call<AddFolderResult>('add_folder', { path });
  },

  listIndexedFolders(): Promise<IndexedFolder[]> {
    return call<IndexedFolder[]>('list_indexed_folders_command');
  },

  removeIndexedFolder(id: number): Promise<void> {
    return call<void>('remove_indexed_folder_command', { id });
  },

  startScan(id: number): Promise<ScanRun> {
    return call<ScanRun>('start_scan', { id });
  },

  cancelScan(runId: number): Promise<boolean> {
    return call<boolean>('cancel_scan', { runId });
  },

  getScanStatus(id: number): Promise<ScanStatus | null> {
    return call<ScanStatus | null>('get_scan_status', { id });
  },

  listWorkspaceFiles(id: number): Promise<FileEntry[]> {
    return call<FileEntry[]>('list_workspace_files_command', { id });
  },

  getFileDetails(id: number): Promise<FileEntry | null> {
    return call<FileEntry | null>('get_file_details_command', { id });
  },

  listScanRuns(id: number): Promise<ScanRun[]> {
    return call<ScanRun[]>('list_scan_runs_command', { id });
  },

  revealFolder(path: string): Promise<void> {
    return call<void>('reveal_folder', { path });
  },

  startAnalysis(workspaceId: number): Promise<void> {
    return call<void>('start_analysis', { workspaceId });
  },

  cancelAnalysis(workspaceId: number): Promise<boolean> {
    return call<boolean>('cancel_analysis', { workspaceId });
  },

  getFileImports(fileId: number): Promise<ImportEntry[]> {
    return call<ImportEntry[]>('get_file_imports', { fileId });
  },

  getAnalysisDiagnostics(
    workspaceId: number,
    severity?: string,
  ): Promise<AnalysisDiagnostic[]> {
    return call<AnalysisDiagnostic[]>('get_analysis_diagnostics', {
      workspaceId,
      severity: severity ?? null,
    });
  },

  getDependencyGraph(workspaceId: number): Promise<DependencyGraph> {
    return call<DependencyGraph>('get_dependency_graph', { workspaceId });
  },
} as const;
