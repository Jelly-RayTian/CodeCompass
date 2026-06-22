import { useCallback, useEffect, useRef, useState } from 'react';

export type AsyncState<T> =
  | { status: 'loading' }
  | { status: 'error'; message: string }
  | { status: 'ready'; data: T };

/**
 * Loads async data from a `loader` function (typically one that calls
 * `tauriClient`). Returns the current state and a `reload` callback for
 * retry-on-error.  The loader is stored in a ref so callers can pass an
 * inline arrow function without causing re-fetches.
 */
export function useAsyncData<T>(
  loader: () => Promise<T>,
): [AsyncState<T>, () => void] {
  const [state, setState] = useState<AsyncState<T>>({ status: 'loading' });
  const loaderRef = useRef(loader);
  loaderRef.current = loader;

  const reload = useCallback(() => {
    setState({ status: 'loading' });
    loaderRef
      .current()
      .then((data): void => {
        setState({ status: 'ready', data });
      })
      .catch((err: unknown): void => {
        const message = err instanceof Error ? err.message : String(err);
        setState({ status: 'error', message });
      });
  }, []);

  useEffect(() => {
    reload();
  }, [reload]);

  return [state, reload];
}
