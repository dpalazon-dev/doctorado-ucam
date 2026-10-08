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
      setError("Could not select the PDF.");
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
        "Could not cancel the import. The draft is still available.",
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
          "The import was cancelled. Could not open the existing document; you can try again.",
        );
      }
    } catch {
      setOpenFailed(candidate);
      setError(
        "The import was cancelled. Could not open the existing document; you can try again.",
      );
    } finally {
      setPending(false);
    }
  }
  async function submit() {
    if (!preview) return;
    if (!metadata.title) {
      setError("Enter a title to identify the document.");
      return;
    }
    if (
      metadata.year !== null &&
      (!Number.isInteger(metadata.year) ||
        metadata.year < 1000 ||
        metadata.year > 9999)
    ) {
      setError("Year must have four digits.");
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
            "A duplicate was detected, but its record could not be validated. You can cancel the import.",
          );
      } else setError(result.error.message);
    } catch {
      setError(
        "Could not save the PDF. The import has been kept so you can try again.",
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
        <DialogTitle>Import PDF</DialogTitle>
        <DialogDescription id="import-description">
          A managed local copy will be saved. You can move the original file
          later.
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
              Try opening the existing paper again
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
              Select PDF
            </button>
          </div>
        ) : candidates.length > 0 ? (
          <div className="duplicate-choice">
            <p>This document may already be in your library:</p>
            {candidates.map((candidate) => (
              <article key={candidate.paperId}>
                <h3>{candidate.title}</h3>
                <p>
                  Match: {candidate.reasons
                    .map((reason) => (reason === "doi" ? "DOI" : "file"))
                    .join(" and ")}
                </p>
                <button
                  type="button"
                  disabled={pending}
                  onClick={() => void cancel(candidate)}
                >
                  Open existing
                </button>
              </article>
            ))}
            <button
              type="button"
              disabled={pending}
              onClick={() => void cancel()}
            >
              Cancel import
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
                Title
                <input
                  autoFocus
                  value={title}
                  onChange={(event) => setTitle(event.target.value)}
                  maxLength={1000}
                  aria-describedby={error ? "import-error" : undefined}
                />
              </label>
              <label>
                Authors
                <input
                  value={authors}
                  onChange={(event) => setAuthors(event.target.value)}
                  placeholder="Optional; comma-separated"
                />
              </label>
              <label>
                Year
                <input
                  inputMode="numeric"
                  value={year}
                  onChange={(event) => setYear(event.target.value)}
                  placeholder="Optional"
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
                Journal or publisher
                <input
                  value={venue}
                  onChange={(event) => setVenue(event.target.value)}
                />
              </label>
              <label>
                Review type
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
                          unknown: "Unknown",
                          survey: "Survey",
                          topical_review: "Topical review",
                          slr: "Systematic review",
                          mapping_study: "Mapping study",
                          tutorial: "Tutorial",
                          other: "Other",
                        }[value]
                      }
                    </option>
                  ))}
                </select>
              </label>
              <label>
                Domain
                <input
                  value={domain}
                  onChange={(event) => setDomain(event.target.value)}
                />
              </label>
            </div>
            <div className="actions">
              <button type="submit" className="primary" disabled={pending}>
                Save to library
              </button>
              <button
                type="button"
                disabled={pending}
                onClick={() => void cancel()}
              >
                Cancel
              </button>
            </div>
          </form>
        )}
      </DialogContent>
    </Dialog>
  );
}
