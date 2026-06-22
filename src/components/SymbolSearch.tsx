import { useEffect, useState } from 'react';

import { tauriClient } from '@/lib/tauriClient';
import type { SymbolEntry } from '@/types';

interface SymbolSearchProps {
  workspaceId: number;
}

const KIND_OPTIONS = [
  { value: '', label: 'All' },
  { value: 'function', label: 'Function' },
  { value: 'class', label: 'Class' },
  { value: 'interface', label: 'Interface' },
  { value: 'type', label: 'Type' },
  { value: 'enum', label: 'Enum' },
  { value: 'variable', label: 'Variable' },
  { value: 'react_component', label: 'React' },
];

export function SymbolSearch({ workspaceId }: SymbolSearchProps): JSX.Element {
  const [query, setQuery] = useState('');
  const [kind, setKind] = useState('');
  const [results, setResults] = useState<SymbolEntry[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(false);
  const pageSize = 15;

  useEffect(() => {
    let cancelled = false;
    const search = async (): Promise<void> => {
      setLoading(true);
      try {
        const res = await tauriClient.searchSymbols(
          workspaceId,
          query || undefined,
          kind || undefined,
          page,
          pageSize,
        );
        if (cancelled) return;
        setResults(res.symbols);
        setTotal(res.total);
      } catch {
        if (!cancelled) setResults([]);
      } finally {
        if (!cancelled) setLoading(false);
      }
    };
    search();
    return () => {
      cancelled = true;
    };
  }, [workspaceId, query, kind, page]);

  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  return (
    <div className="card symbol-search">
      <div className="panel-title">Symbol Search</div>

      <div style={{ display: 'flex', gap: 8, marginBottom: 12 }}>
        <input
          className="input"
          placeholder="Search symbols…"
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setPage(1);
          }}
          style={{ flex: 1 }}
        />
        <select
          className="select"
          value={kind}
          onChange={(e) => {
            setKind(e.target.value);
            setPage(1);
          }}
        >
          {KIND_OPTIONS.map((o) => (
            <option key={o.value} value={o.value}>
              {o.label}
            </option>
          ))}
        </select>
      </div>

      {loading && <div className="muted">Searching…</div>}

      {!loading && results.length === 0 && query.length > 0 && (
        <div className="muted">No symbols found.</div>
      )}

      {results.length > 0 && (
        <>
          <div className="symbol-result-count">
            {total} results (page {page}/{totalPages})
          </div>
          <ul className="symbol-list">
            {results.map((sym) => (
              <li key={sym.id} className="symbol-item">
                <span className="symbol-kind-badge">{sym.kind}</span>
                <span className="symbol-name">{sym.name}</span>
                {sym.isExported && (
                  <span className="symbol-exported">exported</span>
                )}
                <span className="symbol-location">
                  {sym.relativePath ?? '—'}:{sym.sourceLine}
                </span>
              </li>
            ))}
          </ul>

          {totalPages > 1 && (
            <div className="symbol-pagination">
              <button
                className="btn btn-secondary"
                disabled={page <= 1}
                onClick={() => setPage((p) => p - 1)}
                type="button"
              >
                Prev
              </button>
              <span className="muted">
                Page {page} of {totalPages}
              </span>
              <button
                className="btn btn-secondary"
                disabled={page >= totalPages}
                onClick={() => setPage((p) => p + 1)}
                type="button"
              >
                Next
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
