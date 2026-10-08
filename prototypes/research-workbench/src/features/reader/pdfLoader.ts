import { getDocument, GlobalWorkerOptions } from "pdfjs-dist";
import type {
  PDFDocumentLoadingTask,
  PDFDocumentProxy,
  RenderTask,
} from "pdfjs-dist";
import {
  pdfResourceUrls,
  pdfWorkerUrl,
} from "../../shared/adapters/tauri/pdf-assets";

export interface LoadedPdf {
  numPages: number;
  renderPage(
    pageIndex: number,
    zoom: number,
    canvas: HTMLCanvasElement,
    signal: AbortSignal,
  ): Promise<void>;
  destroy(): Promise<void>;
}

export async function loadPdf(
  url: string,
  signal: AbortSignal,
): Promise<LoadedPdf> {
  GlobalWorkerOptions.workerSrc = pdfWorkerUrl;
  const task = getDocument({
    url,
    ...pdfResourceUrls,
    disableAutoFetch: false,
    enableXfa: false,
  });
  return loadPdfFromTask(task, signal);
}

export async function loadPdfFromTask(
  task: PDFDocumentLoadingTask,
  signal: AbortSignal,
): Promise<LoadedPdf> {
  let destroyPromise: Promise<void> | null = null;
  const destroyTask = () => {
    if (!destroyPromise)
      destroyPromise = Promise.resolve()
        .then(() => task.destroy())
        .then(() => undefined);
    return destroyPromise;
  };
  const onAbort = () => {
    void destroyTask().catch(() => undefined);
  };
  signal.addEventListener("abort", onAbort, { once: true });
  try {
    const pdf = await task.promise;
    if (signal.aborted) {
      await destroyTask().catch(() => undefined);
      throw new DOMException("Reading was cancelled.", "AbortError");
    }
    return createDocument(pdf, destroyTask, signal, () =>
      signal.removeEventListener("abort", onAbort),
    );
  } catch (error) {
    signal.removeEventListener("abort", onAbort);
    await destroyTask().catch(() => undefined);
    throw error;
  }
}

function createDocument(
  pdf: PDFDocumentProxy,
  destroyTask: () => Promise<void>,
  loadSignal: AbortSignal,
  releaseLoadSignal: () => void,
): LoadedPdf {
  let destroyed = false;
  const active = new Set<RenderTask>();
  return {
    numPages: pdf.numPages,
    async renderPage(pageIndex, zoom, canvas, signal) {
      if (destroyed || loadSignal.aborted || signal.aborted)
        throw new DOMException("Reading was cancelled.", "AbortError");
      const page = await pdf.getPage(pageIndex);
      if (destroyed || loadSignal.aborted || signal.aborted) {
        page.cleanup();
        throw new DOMException("Reading was cancelled.", "AbortError");
      }
      let render: RenderTask | undefined;
      let cancel: (() => void) | undefined;
      try {
        const viewport = page.getViewport({ scale: zoom });
        canvas.width = Math.ceil(viewport.width);
        canvas.height = Math.ceil(viewport.height);
        const context = canvas.getContext("2d");
        if (!context)
          throw new Error("Could not prepare the reading canvas.");
        render = page.render({ canvas, canvasContext: context, viewport });
        active.add(render);
        cancel = () => render?.cancel();
        signal.addEventListener("abort", cancel, { once: true });
        await render.promise;
      } finally {
        if (cancel) signal.removeEventListener("abort", cancel);
        if (render) active.delete(render);
        page.cleanup();
      }
    },
    async destroy() {
      if (destroyed) return;
      destroyed = true;
      releaseLoadSignal();
      for (const render of active) render.cancel();
      try {
        await destroyTask();
      } finally {
        pdf.cleanup();
      }
    },
  };
}
