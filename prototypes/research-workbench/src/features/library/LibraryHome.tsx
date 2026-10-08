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
          setError("Could not retrieve the last opened document.");
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
        setError("Could not resume reading.");
    }
  }
  return (
    <section className="home-page">
      <p className="eyebrow">Your research workspace</p>
      <h1>Pick up where you left off</h1>
      <p className="lead">
        Your library keeps local copies of your papers and your confirmed
        reading position.
      </p>
      {loading ? (
        <p role="status">Looking for the last document…</p>
      ) : paper ? (
        <article className="card">
          <p className="paper-state">Last opened document</p>
          <h2>{paper.title}</h2>
          <p>{paper.authors.join(", ") || "Authors not specified"}</p>
          <div className="actions">
            <button
              type="button"
              className="primary"
              onClick={() => void resume()}
            >
              Resume reading
            </button>
            <button type="button" onClick={onShowLibrary}>
              View library
            </button>
          </div>
        </article>
      ) : (
        <article className="card">
          <h2>Start with a paper</h2>
          <p>{error ?? "You have not opened a document yet."}</p>
          <div className="actions">
            <button type="button" className="primary" onClick={onShowLibrary}>
              Open library
            </button>
          </div>
        </article>
      )}
      {error && paper && <p role="alert">{error}</p>}
    </section>
  );
}
