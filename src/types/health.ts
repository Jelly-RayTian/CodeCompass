export interface FileHealth {
  fileId: number;
  relativePath: string;
  name: string;
  sizeBytes: number;
  lineCount: number;
  importOutDegree: number;
  importInDegree: number;
  symbolCount: number;
  diagnosticCount: number;
  changeCount: number;
  isInCycle: boolean;
  riskScore: number;
  riskCategory: 'low' | 'medium' | 'high' | 'critical';
}

export interface HealthSummary {
  totalFiles: number;
  filesAnalyzed: number;
  totalImports: number;
  totalSymbols: number;
  cycleCount: number;
  avgRiskScore: number;
  filesLowRisk: number;
  filesMediumRisk: number;
  filesHighRisk: number;
  filesCriticalRisk: number;
}

export interface RepositoryHealth {
  summary: HealthSummary;
  topRiskFiles: FileHealth[];
  allFiles: FileHealth[];
}
