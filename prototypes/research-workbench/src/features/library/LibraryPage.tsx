import { useEffect, useRef, useState } from "react";
import type { LibraryApi, ReaderApi } from "../../shared/contracts/ports";
import type {
  OpenPaperDto,
  PaperDto,
} from "../../shared/contracts/generated/contracts";
import { useLibrary } from "./useLibrary";
import { ImportPaperDialog } from "./ImportPaperDialog";
import { PaperDetails } from "./PaperDetails";

export function LibraryPage({
  api,
  readerApi,
  onOpenPaper,
  onLibraryChanged = () => {},
}: {
  api: LibraryApi;
  readerApi: ReaderApi;
  onOpenPaper: (opened: OpenPaperDto) => void;
  onLibraryChanged?: () => void;
}) {
  const {
    papers,
    filter,
    setFilter,
    loading,
    error,
    reload,
    nextCursor,
    hasPreviousPage,
    nextPage,
    previousPage,
  } = useLibrary(api);
  const [importOpen, setImportOpen] = useState(false);
  const [selected, setSelected] = useState<PaperDto | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const selectedId = useRef<string | null>(null);
  const openIntent = useRef(0);
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      openIntent.current++;
    };
  }, []);
  function selectPaper(paper: PaperDto | null) {
    selectedId.current = paper?.id ?? null;
    setSelected(paper);
  }
  function beginImport() {
    openIntent.current++;
    setActionError(null);
    setImportOpen(true);
  }
  async function open(paperId: string) {
    const intent = ++openIntent.current;
    setActionError(null);
    try {
      const result = await readerApi.openPaper({
        requestId: crypto.randomUUID(),
        paperId,
      });
      if (!mounted.current || intent !== openIntent.current) return;
      if (result.ok) onOpenPaper(result.data);
      else setActionError(result.error.message);
    } catch {
      if (mounted.current && intent === openIntent.current)
        setActionError(
          "Could not open the document. You can try again.",
        );
    }
  }
  async function archive(paper: PaperDto) {
    setActionError(null);
    const result = await api.archivePaper({
      requestId: crypto.randomUUID(),
      paperId: paper.id,
      expectedRevision: paper.revision,
    });
    if (result.ok) {
      reload();
      onLibraryChanged();
    } else setActionError(result.error.message);
  }
  async function restore(paper: PaperDto) {
    setActionError(null);
    const result = await api.restorePaper({
      requestId: crypto.randomUUID(),
      paperId: paper.id,
      expectedRevision: paper.revision,
    });
    if (result.ok) {
      reload();
      onLibraryChanged();
    } else setActionError(result.error.message);
  }
  return (
    <section className="library-page">
      <p className="eyebrow">Local library</p>
      <div className="library-heading">
        <div>
          <h1>Your library</h1>
          <p className="lead">
            Your papers and their PDF copies, together on this computer.
          </p>
        </div>
        <button type="button" className="primary" onClick={beginImport}>
          Import PDF
        </button>
      </div>
      <div className="library-filters">
        <label>
          Search by title, authors, DOI, or journal
          <input
            aria-label="Search the library"
            value={filter.query}
            onChange={(e) => setFilter({ ...filter, query: e.target.value })}
          />
        </label>
        <label>
          Status
          <select
            aria-label="Status"
            value={filter.lifecycle}
            onChange={(e) =>
              setFilter({
                ...filter,
                lifecycle: e.target.value as typeof filter.lifecycle,
              })
            }
          >
            <option value="ACTIVE">Active</option>
            <option value="ARCHIVED">Archived</option>
            <option value="ALL">All</option>
          </select>
        </label>
      </div>
      {actionError && <p role="alert">{actionError}</p>}
      {error && (
        <div role="alert">
          <p>{error}</p>
          <button type="button" onClick={reload}>
            Try again
          </button>
        </div>
      )}
      {loading ? (
        <p role="status">Loading documents…</p>
      ) : !error && papers.length === 0 ? (
        <div className="empty-state">
          <h2>
            {filter.query ? "No results" : "No documents yet"}
          </h2>
          <p>
            {filter.query
              ? "Try another search or change the filter."
              : "Import your first PDF to start reading and organizing your research."}
          </p>
          {!filter.query && (
            <button type="button" className="primary" onClick={beginImport}>
              Import PDF
            </button>
          )}
        </div>
      ) : (
        <div className="paper-list">
          {papers.map((paper) => (
            <article className="paper-card" key={paper.id}>
              <div>
                <p className="paper-state">
                  {paper.lifecycle === "ARCHIVED"
                    ? "Archived"
                    : paper.lifecycle === "COMPLETED"
                      ? "Completed"
                      : "Active"}{" "}
                  · {paper.year ?? "Year not specified"}
                </p>
                <h2>{paper.title}</h2>
                <p>
                  {paper.authors.join(", ") || "Authors not specified"}
                  {paper.venue ? ` · ${paper.venue}` : ""}
                </p>
                {paper.doi && <p className="muted">DOI: {paper.doi}</p>}
              </div>
              <div className="actions">
                <button type="button" onClick={() => void open(paper.id)}>
                  Read
                </button>
                <button type="button" onClick={() => selectPaper(paper)}>
                  Edit
                </button>
                {paper.lifecycle === "ARCHIVED" ? (
                  <button type="button" onClick={() => void restore(paper)}>
                    Restore
                  </button>
                ) : (
                  <button type="button" onClick={() => void archive(paper)}>
                    Archive
                  </button>
                )}
              </div>
            </article>
          ))}
        </div>
      )}
      {(nextCursor || hasPreviousPage) && (
        <nav
          className="library-pagination"
          aria-label="Library pages"
        >
          <button
            type="button"
            disabled={!hasPreviousPage || loading}
            onClick={previousPage}
          >
            Previous page
          </button>
          <button
            type="button"
            disabled={!nextCursor || loading}
            onClick={nextPage}
          >
            Next page
          </button>
        </nav>
      )}
      {selected && (
        <PaperDetails
          key={selected.id}
          paper={selected}
          api={api}
          onClose={() => selectPaper(null)}
          onOpen={(id) => void open(id)}
          onChanged={(paper) => {
            if (selectedId.current === paper.id) selectPaper(paper);
            reload();
            onLibraryChanged();
          }}
        />
      )}
      {importOpen && (
        <ImportPaperDialog
          api={api}
          readerApi={readerApi}
          onClose={() => setImportOpen(false)}
          onImported={() => {
            reload();
          }}
          onOpenPaper={onOpenPaper}
          onOperationFinished={onLibraryChanged}
        />
      )}
    </section>
  );
}
