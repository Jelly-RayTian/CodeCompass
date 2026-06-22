export interface GitInfo {
  isRepo: boolean;
  branch: string | null;
  status: string | null;
  commitCount: number | null;
  lastCommitShort: string | null;
  lastCommitTimestamp: number | null;
  lastCommitMessage: string | null;
}

export interface GitFileInfo {
  lastCommit: string | null;
  changeFrequency: number;
}

export interface WorkspaceSettings {
  workspaceId: number;
  gitAnalysisEnabled: boolean;
  autoReanalyzeEnabled: boolean;
}

export interface CoChangePair {
  fileA: string;
  fileB: string;
  togetherCount: number;
}
