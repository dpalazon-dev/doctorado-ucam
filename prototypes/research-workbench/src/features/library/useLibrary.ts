import { useCallback, useEffect, useState } from "react";
import type { LibraryApi } from "../../shared/contracts/ports";
import type {
  PaperDto,
  PaperFilterDto,
} from "../../shared/contracts/generated/contracts";

const initialFilter: PaperFilterDto = {
  lifecycle: "ACTIVE",
  query: "",
  yearFrom: null,
  yearTo: null,
  reviewTypes: [],
  domain: null,
  phase: null,
  cursor: null,
  limit: 100,
};

export function useLibrary(api: LibraryApi) {
  const [papers, setPapers] = useState<PaperDto[]>([]);
  const [filter, setFilterState] = useState(initialFilter);
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [history, setHistory] = useState<Array<string | null>>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [reloadKey, setReloadKey] = useState(0);

  const reload = useCallback(() => setReloadKey((v) => v + 1), []);
  const setFilter = useCallback((next: PaperFilterDto) => {
    setHistory([]);
    setNextCursor(null);
    setFilterState({ ...next, cursor: null });
  }, []);
  const nextPage = useCallback(() => {
    if (!nextCursor || loading) return;
    setHistory((pages) => [...pages, filter.cursor]);
    setFilterState((current) => ({ ...current, cursor: nextCursor }));
  }, [filter.cursor, loading, nextCursor]);
  const previousPage = useCallback(() => {
    if (!history.length || loading) return;
    const pages = history.slice(0, -1);
    setFilterState((current) => ({
      ...current,
      cursor: history[history.length - 1],
    }));
    setHistory(pages);
  }, [history, loading]);

  useEffect(() => {
    let current = true;
    setLoading(true);
    setError(null);
    void api
      .listPapers({ requestId: crypto.randomUUID(), filter })
      .then((result) => {
        if (!current) return;
        if (result.ok) {
          setPapers(result.data.items);
          setNextCursor(result.data.nextCursor);
        } else setError(result.error.message);
      })
      .catch(() => {
        if (current)
          setError(
            "Could not load the library. You can try again.",
          );
      })
      .finally(() => {
        if (current) setLoading(false);
      });
    return () => {
      current = false;
    };
  }, [api, filter, reloadKey]);

  return {
    papers,
    filter,
    setFilter,
    loading,
    error,
    reload,
    nextCursor,
    hasPreviousPage: history.length > 0,
    nextPage,
    previousPage,
  };
}
