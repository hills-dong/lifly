import { useEffect, useState, useCallback, useRef } from 'react';

export interface UseFetchDataResult<T> {
  data: T | undefined;
  loading: boolean;
  error: string;
  refetch: () => void;
}

/**
 * Generic data-fetching hook that encapsulates the loading / error / refetch
 * pattern shared across most pages.
 *
 * @param fetcher  Async function that returns the data.
 * @param deps     Dependency array — the fetcher is re-invoked whenever deps change.
 */
export function useFetchData<T>(
  fetcher: () => Promise<T>,
  deps: unknown[] = [],
): UseFetchDataResult<T> {
  const [data, setData] = useState<T | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  // Keep a stable reference to the latest fetcher so callers don't need to
  // memoize it themselves.
  const fetcherRef = useRef(fetcher);
  fetcherRef.current = fetcher;

  const execute = useCallback(() => {
    setLoading(true);
    setError('');
    fetcherRef
      .current()
      .then(setData)
      .catch((err: unknown) =>
        setError(err instanceof Error ? err.message : String(err)),
      )
      .finally(() => setLoading(false));
  }, []);

  // Re-run whenever external deps change.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => { execute(); }, deps);

  return { data, loading, error, refetch: execute };
}
