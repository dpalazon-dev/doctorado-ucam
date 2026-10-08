import { useCallback, useEffect, useRef, useState } from "react";
import type { OpenPaperDto } from "../../shared/contracts/generated/contracts";
import type { ReaderApi } from "../../shared/contracts/ports";
import { loadPdf } from "./pdfLoader";
import type { LoadedPdf } from "./pdfLoader";
import { useReadingPosition } from "./useReadingPosition";

export function PaperReader({
  opened,
  api,
  onClose,
}: {
  opened: OpenPaperDto;
  api: ReaderApi;
  onClose: () => void;
}) {
  const [pdf, setPdf] = useState<LoadedPdf | null>(null);
  const [pageCount, setPageCount] = useState<number | null>(null);
  const [pageIndex, setPageIndex] = useState(opened.readingPosition.pageIndex);
  const [zoom, setZoom] = useState(opened.readingPosition.zoom);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [renderError, setRenderError] = useState<string | null>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const loadController = useRef<AbortController | null>(null);
  const renderController = useRef<AbortController | null>(null);
  const renderId = useRef(0);
  const pdfRef = useRef<LoadedPdf | null>(null);
  const {
    save,
    status,
    error,
    recovery,
    retryUncertain,
    refreshConflict,
    resolveConflict,
  } = useReadingPosition(api, opened.readingPosition);
  useEffect(() => {
    const controller = new AbortController();
    loadController.current = controller;
    let active = true;
    setLoading(true);
    setLoadError(null);
    setPdf(null);
    setPageCount(null);
    setPageIndex(opened.readingPosition.pageIndex);
    setZoom(opened.readingPosition.zoom);
    void loadPdf(opened.documentUrl, controller.signal)
      .then((document) => {
        if (!active) {
          void document.destroy();
          return;
        }
        pdfRef.current = document;
        setPdf(document);
        setPageCount(document.numPages);
        setPageIndex(
          Math.min(opened.readingPosition.pageIndex, document.numPages),
        );
      })
      .catch(() => {
        if (active && !controller.signal.aborted)
          setLoadError(
            "No se pudo mostrar este PDF. Puede estar dañado, protegido o no estar disponible. Tu ficha sigue en la biblioteca.",
          );
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
      controller.abort();
      renderController.current?.abort();
      const old = pdfRef.current;
      pdfRef.current = null;
      if (old) void old.destroy();
    };
  }, [
    opened.document.id,
    opened.documentUrl,
    opened.readingPosition.pageIndex,
    opened.readingPosition.zoom,
  ]);
  useEffect(() => {
    if (!pdf || !canvas.current) return;
    const controller = new AbortController();
    renderController.current?.abort();
    renderController.current = controller;
    const current = ++renderId.current;
    setRenderError(null);
    void pdf
      .renderPage(pageIndex, zoom, canvas.current, controller.signal)
      .catch(() => {
        if (!controller.signal.aborted && current === renderId.current)
          setRenderError(
            "No se pudo dibujar esta página. Puedes cambiar de página o cerrar el lector.",
          );
      });
    return () => controller.abort();
  }, [pdf, pageIndex, zoom]);
  const close = useCallback(() => {
    loadController.current?.abort();
    renderController.current?.abort();
    const loaded = pdfRef.current;
    pdfRef.current = null;
    if (loaded) void loaded.destroy();
    onClose();
  }, [onClose]);
  function keepSavedPosition() {
    void resolveConflict("keep-saved")
      .then((position) => {
        setPageIndex(
          Math.max(
            1,
            Math.min(pageCount ?? position.pageIndex, position.pageIndex),
          ),
        );
        setZoom(position.zoom);
      })
      .catch(() => {});
  }
  function changePage(next: number) {
    if (pageCount === null || !Number.isInteger(next)) return;
    const bounded = Math.max(1, Math.min(pageCount, next));
    if (bounded === pageIndex) return;
    setPageIndex(bounded);
    void save(bounded, zoom);
  }
  function changeZoom(next: number) {
    const bounded = Math.max(0.25, Math.min(5, Math.round(next * 100) / 100));
    if (bounded === zoom) return;
    setZoom(bounded);
    void save(pageIndex, bounded);
  }
  return (
    <section className="reader-shell" aria-label="Lector PDF">
      <div className="reader-toolbar">
        <button type="button" aria-label="Cerrar lector" onClick={close}>
          Cerrar
        </button>
        <div className="reader-title">
          <span className="eyebrow">Lector PDF</span>
          <h1>{opened.paper.title}</h1>
          <span>{opened.document.originalFilename}</span>
        </div>
        <div className="reader-controls">
          <button
            type="button"
            aria-label="Reducir zoom"
            disabled={zoom <= 0.25}
            onClick={() => changeZoom(zoom - 0.1)}
          >
            −
          </button>
          <span>{Math.round(zoom * 100)}%</span>
          <button
            type="button"
            aria-label="Aumentar zoom"
            disabled={zoom >= 5}
            onClick={() => changeZoom(zoom + 0.1)}
          >
            +
          </button>
        </div>
      </div>
      <div className="reader-pagination">
        <button
          type="button"
          aria-label="Página anterior"
          disabled={pageIndex <= 1}
          onClick={() => changePage(pageIndex - 1)}
        >
          ‹
        </button>
        <label>
          Página{" "}
          <input
            aria-label="Ir a página"
            type="number"
            step={1}
            min={1}
            max={pageCount ?? undefined}
            value={pageIndex}
            onChange={(e) => {
              const value = Number(e.target.value);
              if (Number.isInteger(value)) changePage(value);
              else e.currentTarget.value = String(pageIndex);
            }}
          />
          {pageCount === null ? "" : ` de ${pageCount}`}
        </label>
        <button
          type="button"
          aria-label="Página siguiente"
          disabled={pageCount === null || pageIndex >= pageCount}
          onClick={() => changePage(pageIndex + 1)}
        >
          ›
        </button>
        <span aria-live="polite">
          {loading
            ? "Cargando PDF…"
            : status === "saving"
              ? "Guardando posición…"
              : status === "saved"
                ? "Guardado"
                : status === "error"
                  ? "Cambios pendientes"
                  : ""}
        </span>
      </div>
      {recovery && (
        <section
          className="reader-recovery"
          role="group"
          aria-label="Resolver guardado de posición"
        >
          <h2>
            {recovery.kind === "conflict"
              ? "La posición cambió en otra operación"
              : "No se pudo confirmar el guardado"}
          </h2>
          <p>{recovery.message}</p>
          {recovery.kind === "uncertain" ? (
            <button
              type="button"
              disabled={status === "saving"}
              onClick={() => void retryUncertain().catch(() => {})}
            >
              Reintentar guardado de posición
            </button>
          ) : (
            <>
              <p>
                {recovery.durable
                  ? `Posición guardada: página ${recovery.durable.pageIndex}, ${Math.round(recovery.durable.zoom * 100)}%.`
                  : "No se pudo consultar la posición guardada."}
              </p>
              {recovery.durable ? (
                <>
                  <button
                    type="button"
                    disabled={status === "saving"}
                    onClick={keepSavedPosition}
                  >
                    Conservar posición guardada
                  </button>
                  <button
                    type="button"
                    disabled={status === "saving"}
                    onClick={() =>
                      void resolveConflict("save-local").catch(() => {})
                    }
                  >
                    Guardar mi posición
                  </button>
                </>
              ) : (
                <button
                  type="button"
                  disabled={status === "saving"}
                  onClick={() => void refreshConflict()}
                >
                  Consultar posición guardada
                </button>
              )}
            </>
          )}
        </section>
      )}
      {(loadError || renderError || error) && (
        <p role="alert">{loadError ?? renderError ?? error}</p>
      )}
      {loading && <p role="status">Preparando el documento…</p>}
      <div className="pdf-page-wrap">
        {!loadError && (
          <canvas
            ref={canvas}
            role="img"
            aria-label={`Página ${pageIndex} del documento`}
          />
        )}
      </div>
    </section>
  );
}
