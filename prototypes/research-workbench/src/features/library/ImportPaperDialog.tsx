import { useRef, useState } from "react";
import type { LibraryApi, ReaderApi } from "../../shared/contracts/ports";
import type {
  DuplicateCandidateDto,
  ImportPreviewDto,
  OpenPaperDto,
  PaperMetadataInput,
  ReviewType,
} from "../../shared/contracts/generated/contracts";
import { duplicateCandidateDtoSchema } from "../../shared/contracts/wire";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "../../shared/ui/dialog";

type Props = {
  api: LibraryApi;
  readerApi: ReaderApi;
  onClose: () => void;
  onImported: () => void;
  onOpenPaper: (opened: OpenPaperDto) => void;
  onOperationFinished?: () => void;
};

export function ImportPaperDialog({
  api,
  readerApi,
  onClose,
  onImported,
  onOpenPaper,
  onOperationFinished = () => {},
}: Props) {
  const [preview, setPreview] = useState<ImportPreviewDto | null>(null);
  const [title, setTitle] = useState("");
  const [authors, setAuthors] = useState("");
  const [year, setYear] = useState("");
  const [doi, setDoi] = useState("");
  const [venue, setVenue] = useState("");
  const [domain, setDomain] = useState("");
  const [reviewType, setReviewType] = useState<ReviewType>("unknown");
  const [candidates, setCandidates] = useState<DuplicateCandidateDto[]>([]);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [openFailed, setOpenFailed] = useState<DuplicateCandidateDto | null>(
    null,
  );
  const cancelIntent = useRef<{ token: string; requestId: string } | null>(
    null,
  );
  const metadata: PaperMetadataInput = {
    title: title.trim(),
    authors: authors
      .split(",")
      .map((value) => value.trim())
      .filter(Boolean),
    year: year.trim() ? Number(year) : null,
    doi: doi.trim() || null,
    venue: venue.trim() || null,
    reviewType,
    domain: domain.trim() || null,
  };

  function resetDraft() {
    setTitle("");
    setAuthors("");
    setYear("");
    setDoi("");
    setVenue("");
    setDomain("");
    setReviewType("unknown");
    setCandidates([]);
    setOpenFailed(null);
    setError(null);
  }
  async function closeThroughTransition() {
    if (pending) return;
    if (!preview) {
      onClose();
      return;
    }
    await cancel();
  }
  async function select() {
    setPending(true);
    setError(null);
    try {
      const result = await api.selectPdf(crypto.randomUUID());
      if (!result.ok) {
        setError(result.error.message);
        return;
      }
      if (!result.data) return;
      resetDraft();
      cancelIntent.current = null;
      setPreview(result.data);
      setCandidates(result.data.candidates);
    } catch {
      setError("No se pudo seleccionar el PDF.");
    } finally {
      setPending(false);
      onOperationFinished();
    }
  }
  async function cancel(openCandidate?: DuplicateCandidateDto) {
    if (!preview || pending) return;
    let intent = cancelIntent.current;
    if (!intent || intent.token !== preview.importToken) {
      intent = { token: preview.importToken, requestId: crypto.randomUUID() };
      cancelIntent.current = intent;
    }
    setPending(true);
    setError(null);
    try {
      const result = await api.cancelImport({
        requestId: intent.requestId,
        importToken: preview.importToken,
      });
      if (!result.ok) {
        setError(result.error.message);
        return;
      }
      setPreview(null);
      if (openCandidate) {
        await openExisting(openCandidate);
        return;
      }
      onClose();
    } catch {
      setError(
        "No se pudo cancelar la importación. El borrador sigue disponible.",
      );
    } finally {
      setPending(false);
      onOperationFinished();
    }
  }
  async function openExisting(candidate: DuplicateCandidateDto) {
    setPending(true);
    setError(null);
    try {
      const result = await readerApi.openPaper({
        requestId: crypto.randomUUID(),
        paperId: candidate.paperId,
      });
      if (result.ok) {
        onOpenPaper(result.data);
        onClose();
      } else {
        setOpenFailed(candidate);
        setError(
          "La importación se canceló. No se pudo abrir el documento existente; puedes reintentarlo.",
        );
      }
    } catch {
      setOpenFailed(candidate);
      setError(
        "La importación se canceló. No se pudo abrir el documento existente; puedes reintentarlo.",
      );
    } finally {
      setPending(false);
    }
  }
  async function submit() {
    if (!preview) return;
    if (!metadata.title) {
      setError("Escribe un título para identificar el documento.");
      return;
    }
    if (
      metadata.year !== null &&
      (!Number.isInteger(metadata.year) ||
        metadata.year < 1000 ||
        metadata.year > 9999)
    ) {
      setError("El año debe tener cuatro cifras.");
      return;
    }
    setPending(true);
    setError(null);
    try {
      const result = await api.confirmImport({
        requestId: crypto.randomUUID(),
        importToken: preview.importToken,
        metadata,
      });
      if (result.ok) {
        onImported();
        setPreview(null);
        onClose();
        return;
      }
      if (result.error.code === "DuplicateDecisionRequired") {
        const raw = result.error.details?.candidates;
        const parsed = Array.isArray(raw)
          ? raw.map((value) => duplicateCandidateDtoSchema.safeParse(value))
          : [];
        const valid = parsed
          .filter((item) => item.success)
          .map((item) => item.data);
        if (valid.length) setCandidates(valid);
        else
          setError(
            "Se detectó un duplicado, pero no se pudo validar su ficha. Puedes cancelar la importación.",
          );
      } else setError(result.error.message);
    } catch {
      setError(
        "No se pudo guardar el PDF. Conservamos la importación para que puedas intentarlo de nuevo.",
      );
    } finally {
      setPending(false);
      onOperationFinished();
    }
  }

  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open) void closeThroughTransition();
      }}
    >
      <DialogContent showClose={!pending} aria-describedby="import-description">
        <DialogTitle>Importar PDF</DialogTitle>
        <DialogDescription id="import-description">
          Se guardará una copia local administrada. El archivo original podrá
          moverse después.
        </DialogDescription>
        {error && (
          <p id="import-error" role="alert">
            {error}
          </p>
        )}
        {openFailed ? (
          <div>
            <button
              type="button"
              disabled={pending}
              onClick={() => void openExisting(openFailed)}
            >
              Reintentar abrir existente
            </button>
          </div>
        ) : !preview ? (
          <div>
            <button
              type="button"
              className="primary"
              disabled={pending}
              onClick={() => void select()}
            >
              Seleccionar PDF
            </button>
          </div>
        ) : candidates.length > 0 ? (
          <div className="duplicate-choice">
            <p>Este documento puede estar ya en tu biblioteca:</p>
            {candidates.map((candidate) => (
              <article key={candidate.paperId}>
                <h3>{candidate.title}</h3>
                <p>
                  Coincidencia:{" "}
                  {candidate.reasons
                    .map((reason) => (reason === "doi" ? "DOI" : "archivo"))
                    .join(" y ")}
                </p>
                <button
                  type="button"
                  disabled={pending}
                  onClick={() => void cancel(candidate)}
                >
                  Abrir existente
                </button>
              </article>
            ))}
            <button
              type="button"
              disabled={pending}
              onClick={() => void cancel()}
            >
              Cancelar importación
            </button>
          </div>
        ) : (
          <form
            onSubmit={(event) => {
              event.preventDefault();
              void submit();
            }}
          >
            <p>
              {preview.originalFilename} ·{" "}
              {(preview.sizeBytes / 1024 / 1024).toFixed(1)} MB
            </p>
            <div className="metadata-form">
              <label>
                Título
                <input
                  autoFocus
                  value={title}
                  onChange={(event) => setTitle(event.target.value)}
                  maxLength={1000}
                  aria-describedby={error ? "import-error" : undefined}
                />
              </label>
              <label>
                Autores
                <input
                  value={authors}
                  onChange={(event) => setAuthors(event.target.value)}
                  placeholder="Opcional; separados por coma"
                />
              </label>
              <label>
                Año
                <input
                  inputMode="numeric"
                  value={year}
                  onChange={(event) => setYear(event.target.value)}
                  placeholder="Opcional"
                />
              </label>
              <label>
                DOI
                <input
                  value={doi}
                  onChange={(event) => setDoi(event.target.value)}
                />
              </label>
              <label>
                Revista o editorial
                <input
                  value={venue}
                  onChange={(event) => setVenue(event.target.value)}
                />
              </label>
              <label>
                Tipo de revisión
                <select
                  value={reviewType}
                  onChange={(event) =>
                    setReviewType(event.target.value as ReviewType)
                  }
                >
                  {(
                    [
                      "unknown",
                      "survey",
                      "topical_review",
                      "slr",
                      "mapping_study",
                      "tutorial",
                      "other",
                    ] as const
                  ).map((value) => (
                    <option key={value} value={value}>
                      {
                        {
                          unknown: "Desconocido",
                          survey: "Encuesta",
                          topical_review: "Revisión temática",
                          slr: "Revisión sistemática",
                          mapping_study: "Mapeo",
                          tutorial: "Tutorial",
                          other: "Otro",
                        }[value]
                      }
                    </option>
                  ))}
                </select>
              </label>
              <label>
                Dominio
                <input
                  value={domain}
                  onChange={(event) => setDomain(event.target.value)}
                />
              </label>
            </div>
            <div className="actions">
              <button type="submit" className="primary" disabled={pending}>
                Guardar en biblioteca
              </button>
              <button
                type="button"
                disabled={pending}
                onClick={() => void cancel()}
              >
                Cancelar
              </button>
            </div>
          </form>
        )}
      </DialogContent>
    </Dialog>
  );
}
