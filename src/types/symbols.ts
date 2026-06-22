export interface SymbolEntry {
  id: number;
  workspaceId: number;
  fileId: number;
  name: string;
  kind: string;
  parentSymbolId: number | null;
  sourceLine: number;
  sourceColumn: number;
  sourceEndLine: number;
  sourceEndColumn: number;
  signature: string | null;
  visibility: string;
  isExported: boolean;
  relativePath: string | null;
}

export interface SymbolSearchResult {
  symbols: SymbolEntry[];
  total: number;
  page: number;
  pageSize: number;
}
