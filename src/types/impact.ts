export interface CallGraphNode {
  symbolId: number;
  name: string;
  kind: string;
  filePath: string;
  callersCount: number;
  calleesCount: number;
  isExported: boolean;
}

export interface CallGraphEdge {
  callerId: number;
  calleeId: number;
  referenceType: string;
  sourceLine: number;
}

export interface CallGraph {
  nodes: CallGraphNode[];
  edges: CallGraphEdge[];
  cycles: number[][];
  depthLimitReached: boolean;
}

export interface AffectedItem {
  kind: string;
  id: number;
  name: string;
  path: string;
  depth: number;
  isExported: boolean;
  hasCycles: boolean;
  reason: string;
}

export interface ChangeRisk {
  symbolId: number;
  name: string;
  riskLevel: string;
  riskScore: number;
  directDependents: number;
  transitiveDependents: number;
  isExported: boolean;
  hasCycles: boolean;
  affectedFiles: AffectedItem[];
  affectedSymbols: AffectedItem[];
  explanation: string;
  limitation: string;
}
