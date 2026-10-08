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
          "No se pudo abrir el documento. Puedes intentarlo de nuevo.",
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
      <p className="eyebrow">Biblioteca local</p>
      <div className="library-heading">
        <div>
          <h1>Tu biblioteca</h1>
          <p className="lead">
            Tus papers y sus copias PDF, reunidos en este equipo.
          </p>
        </div>
        <button type="button" className="primary" onClick={beginImport}>
          Importar PDF
        </button>
      </div>
      <div className="library-filters">
        <label>
          Buscar por título, autores, DOI o revista
          <input
            aria-label="Buscar en la biblioteca"
            value={filter.query}
            onChange={(e) => setFilter({ ...filter, query: e.target.value })}
          />
        </label>
        <label>
          Estado
          <select
            aria-label="Estado"
            value={filter.lifecycle}
            onChange={(e) =>
              setFilter({
                ...filter,
                lifecycle: e.target.value as typeof filter.lifecycle,
              })
            }
          >
            <option value="ACTIVE">Activos</option>
            <option value="ARCHIVED">Archivados</option>
            <option value="ALL">Todos</option>
          </select>
        </label>
      </div>
      {actionError && <p role="alert">{actionError}</p>}
      {error && (
        <div role="alert">
          <p>{error}</p>
          <button type="button" onClick={reload}>
            Reintentar
          </button>
        </div>
      )}
      {loading ? (
        <p role="status">Cargando documentos…</p>
      ) : !error && papers.length === 0 ? (
        <div className="empty-state">
          <h2>
            {filter.query ? "No hay resultados" : "Aún no hay documentos"}
          </h2>
          <p>
            {filter.query
              ? "Prueba otra búsqueda o cambia el filtro."
              : "Importa tu primer PDF para comenzar a leer y organizar tu investigación."}
          </p>
          {!filter.query && (
            <button type="button" className="primary" onClick={beginImport}>
              Importar PDF
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
                    ? "Archivado"
                    : paper.lifecycle === "COMPLETED"
                      ? "Completado"
                      : "Activo"}{" "}
                  · {paper.year ?? "Año sin indicar"}
                </p>
                <h2>{paper.title}</h2>
                <p>
                  {paper.authors.join(", ") || "Autores sin indicar"}
                  {paper.venue ? ` · ${paper.venue}` : ""}
                </p>
                {paper.doi && <p className="muted">DOI: {paper.doi}</p>}
              </div>
              <div className="actions">
                <button type="button" onClick={() => void open(paper.id)}>
                  Leer
                </button>
                <button type="button" onClick={() => selectPaper(paper)}>
                  Editar
                </button>
                {paper.lifecycle === "ARCHIVED" ? (
                  <button type="button" onClick={() => void restore(paper)}>
                    Restaurar
                  </button>
                ) : (
                  <button type="button" onClick={() => void archive(paper)}>
                    Archivar
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
          aria-label="Páginas de la biblioteca"
        >
          <button
            type="button"
            disabled={!hasPreviousPage || loading}
            onClick={previousPage}
          >
            Página anterior
          </button>
          <button
            type="button"
            disabled={!nextCursor || loading}
            onClick={nextPage}
          >
            Siguiente página
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
