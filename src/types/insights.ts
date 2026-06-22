export interface EntryPoint {
  fileId: number;
  relativePath: string;
  name: string;
  confidence: number;
  reasons: string[];
}

export interface ReadingPathItem {
  order: number;
  fileId: number;
  relativePath: string;
  name: string;
  depth: number;
  reason: string;
}

export interface StructuralFinding {
  category: string;
  severity: string;
  title: string;
  description: string;
  evidence: string[];
  limitation: string;
  investigation: string;
}

export interface WorkspaceInsights {
  entryPoints: EntryPoint[];
  readingPath: ReadingPathItem[];
  findings: StructuralFinding[];
}
