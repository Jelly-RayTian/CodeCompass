export interface ApplicationInfo {
  name: string;
  version: string;
  buildTimestamp: string;
}

export interface DatabaseStatus {
  connected: boolean;
  databasePath: string;
  migrationVersion: number;
}

export interface IndexedFolder {
  id: number;
  name: string;
  path: string;
  addedAt: number;
  lastSuccessfulScanAt: number | null;
  availability:
    | 'available'
    | 'missing'
    | 'inaccessible'
    | 'permission_denied'
    | 'not_a_directory'
    | 'unknown';
  monitoringEnabled: boolean;
  scanStatus:
    | 'idle'
    | 'running'
    | 'completed'
    | 'completed_with_warnings'
    | 'completed_with_errors'
    | 'cancelled'
    | 'failed';
}

export interface AddFolderResult {
  folder: IndexedFolder;
  warning: string | null;
}

export interface ScanRun {
  id: number;
  workspaceId: number;
  status: string;
  startedAt: number;
  completedAt: number | null;
  filesProcessed: number;
  filesIndexed: number;
  warningCount: number;
  errorCount: number;
  errorMessage: string | null;
  phase: string | null;
}

export interface ScanStatus {
  run: ScanRun;
  fileCount: number;
}

export interface FileEntry {
  id: number;
  workspaceId: number;
  relativePath: string;
  name: string;
  parentPath: string;
  extension: string | null;
  sizeBytes: number;
  createdAt: number | null;
  modifiedAt: number | null;
  indexedAt: number | null;
  lastSeenAt: number | null;
  fingerprint: string | null;
  previousFingerprint: string | null;
  isPresent: boolean;
  changeStatus: 'new' | 'changed' | 'unchanged' | 'removed';
}

export interface ScanProgressEvent {
  runId: number;
  workspaceId: number;
  status: string;
  filesProcessed: number;
  filesIndexed: number;
  warningCount: number;
  errorCount: number;
  phase: string | null;
}

export interface CommandResult<T> {
  ok: boolean;
  data: T | null;
  error: string | null;
}

export type { CycleInfo, DependencyGraph, GraphEdge, GraphNode } from './graph';
export type {
  GitFileInfo,
  GitInfo,
  CoChangePair,
  WorkspaceSettings,
} from './git';
export type {
  AffectedItem,
  CallGraph,
  CallGraphEdge,
  CallGraphNode,
  ChangeRisk,
} from './impact';
export type {
  EntryPoint,
  ReadingPathItem,
  StructuralFinding,
  WorkspaceInsights,
} from './insights';
export type { SourceFile } from './source';
export type { SymbolEntry, SymbolSearchResult } from './symbols';

export interface ImportEntry {
  id: number;
  sourceFileId: number;
  targetSpecifier: string;
  resolvedTargetFileId: number | null;
  importType: string;
  isExternal: boolean;
  startLine: number | null;
  startColumn: number | null;
}

export interface AnalysisDiagnostic {
  id: number;
  fileId: number;
  workspaceId: number;
  severity: string;
  message: string;
  line: number | null;
  column: number | null;
  createdAt: number;
}

export interface AnalysisProgressEvent {
  workspaceId: number;
  status: string;
  filesProcessed: number;
  filesTotal: number;
  filesParsed: number;
  errorCount: number;
}
