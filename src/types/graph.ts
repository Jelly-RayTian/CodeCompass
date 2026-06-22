export interface GraphNode {
  fileId: number;
  name: string;
  relativePath: string;
  extension: string | null;
  incomingCount: number;
  outgoingCount: number;
}

export interface GraphEdge {
  sourceFileId: number;
  targetFileId: number;
  importType: string;
  isExternal: boolean;
}

export interface CycleInfo {
  fileIds: number[];
  filePaths: string[];
}

export interface DependencyGraph {
  nodes: GraphNode[];
  edges: GraphEdge[];
  cycles: CycleInfo[];
  totalFiles: number;
  totalImports: number;
}
