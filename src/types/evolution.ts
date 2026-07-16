export interface TimelinePoint {
  month: string;
  commitCount: number;
  fileChanges: number;
}

export interface FileChurn {
  relativePath: string;
  changeCount: number;
}

export interface EvolutionSummary {
  totalCommits: number;
  totalFilesChanged: number;
  totalFileChanges: number;
  mostActiveMonth: string;
  oldestCommitTs: number;
  newestCommitTs: number;
}

export interface RepositoryEvolution {
  summary: EvolutionSummary;
  timeline: TimelinePoint[];
  topChurnFiles: FileChurn[];
  topHotspots: import('./git').CoChangePair[];
}
