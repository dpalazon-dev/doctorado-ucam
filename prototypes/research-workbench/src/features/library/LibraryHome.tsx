import { useEffect, useRef, useState } from "react";
import type { LibraryApi, ReaderApi } from "../../shared/contracts/ports";
import type {
  OpenPaperDto,
  PaperDto,
} from "../../shared/contracts/generated/contracts";
export function LibraryHome({
  libraryApi,
  readerApi,
  onOpenPaper,
  onShowLibrary,
}: {
  libraryApi: LibraryApi;
  readerApi: ReaderApi;
  onOpenPaper: (opened: OpenPaperDto) => void;
  onShowLibrary: () => void;
}) {
  const [paper, setPaper] = useState<PaperDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const active = useRef(true);
  const openIntent = useRef(0);
  useEffect(() => {
    active.current = true;
    return () => {
      active.current = false;
      openIntent.current++;
    };
  }, []);
  useEffect(() => {
    let active = true;
    void readerApi
      .getLastOpenedPaper({ requestId: crypto.randomUUID() })
      .then(async (result) => {
        if (!active) return;
        if (!result.ok) {
          setError(result.error.message);
          return;
        }
        if (!result.data) {
          setPaper(null);
          return;
        }
        const current = await libraryApi.getPaper({
          requestId: crypto.randomUUID(),
          paperId: result.data.id,
        });
        if (active) {
          if (current.ok) setPaper(current.data);
          else setError(current.error.message);
        }
      })
      .catch(() => {
        if (active)
          setError("No se pudo recuperar el último documento abierto.");
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [libraryApi, readerApi]);
  async function resume() {
    if (!paper) return;
    const intent = ++openIntent.current;
    try {
      const result = await readerApi.openPaper({
        requestId: crypto.randomUUID(),
        paperId: paper.id,
      });
      if (!active.current || intent !== openIntent.current) return;
      if (result.ok) onOpenPaper(result.data);
      else setError(result.error.message);
    } catch {
      if (active.current && intent === openIntent.current)
        setError("No se pudo reanudar la lectura.");
    }
  }
  return (
    <section className="home-page">
      <p className="eyebrow">Tu espacio de investigación</p>
      <h1>Continúa donde lo dejaste</h1>
      <p className="lead">
        Tu biblioteca conserva las copias locales de tus papers y la página de
        lectura confirmada.
      </p>
      {loading ? (
        <p role="status">Buscando el último documento…</p>
      ) : paper ? (
        <article className="card">
          <p className="paper-state">Último documento abierto</p>
          <h2>{paper.title}</h2>
          <p>{paper.authors.join(", ") || "Autores sin indicar"}</p>
          <div className="actions">
            <button
              type="button"
              className="primary"
              onClick={() => void resume()}
            >
              Reanudar lectura
            </button>
            <button type="button" onClick={onShowLibrary}>
              Ver biblioteca
            </button>
          </div>
        </article>
      ) : (
        <article className="card">
          <h2>Empieza con un paper</h2>
          <p>{error ?? "Todavía no has abierto ningún documento."}</p>
          <div className="actions">
            <button type="button" className="primary" onClick={onShowLibrary}>
              Abrir biblioteca
            </button>
          </div>
        </article>
      )}
      {error && paper && <p role="alert">{error}</p>}
    </section>
  );
}
