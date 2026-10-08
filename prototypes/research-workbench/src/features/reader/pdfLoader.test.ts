import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { cleanup } from "@testing-library/react";
import type { PDFDocumentLoadingTask, PDFDocumentProxy } from "pdfjs-dist";
import { loadPdfFromTask } from "./pdfLoader";

afterEach(cleanup);
beforeEach(() => vi.clearAllMocks());

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}

function fakeTask(
  promise: Promise<PDFDocumentProxy>,
  destroy = vi.fn().mockResolvedValue(undefined),
) {
  return { promise, destroy } as unknown as PDFDocumentLoadingTask;
}

it("load_failure_destroys_task_and_preserves_original_error_if_teardown_fails", async () => {
  const failure = new Error("PDF protegido");
  const destroy = vi
    .fn()
    .mockRejectedValue(new Error("Worker shutdown failed"));
  await expect(
    loadPdfFromTask(
      fakeTask(Promise.reject(failure), destroy),
      new AbortController().signal,
    ),
  ).rejects.toBe(failure);
  expect(destroy).toHaveBeenCalledTimes(1);
});

it("load_abort_releases_task_even_when_the_pdf_promise_rejects_late", async () => {
  const loading = deferred<PDFDocumentProxy>();
  const destroy = vi.fn().mockResolvedValue(undefined);
  const controller = new AbortController();
  const result = loadPdfFromTask(
    fakeTask(loading.promise, destroy),
    controller.signal,
  );
  controller.abort();
  const failure = new DOMException("aborted", "AbortError");
  loading.reject(failure);
  await expect(result).rejects.toBe(failure);
  await vi.waitFor(() => expect(destroy).toHaveBeenCalledTimes(1));
});

it("successful_task_is_destroyed_and_pdf_cleanup_runs_when_teardown_rejects", async () => {
  const pdf = {
    numPages: 2,
    cleanup: vi.fn(),
    getPage: vi.fn(),
  } as unknown as PDFDocumentProxy;
  const destroy = vi.fn().mockRejectedValue(new Error("teardown"));
  const loaded = await loadPdfFromTask(
    fakeTask(Promise.resolve(pdf), destroy),
    new AbortController().signal,
  );
  await expect(loaded.destroy()).rejects.toThrow("teardown");
  expect(destroy).toHaveBeenCalledTimes(1);
  expect(pdf.cleanup).toHaveBeenCalledTimes(1);
});

it("render_abort_cancels_the_actual_render_task_and_cleans_the_page", async () => {
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(
    {} as CanvasRenderingContext2D,
  );
  const renderPromise = deferred<void>();
  const cancel = vi.fn(() =>
    renderPromise.reject(new DOMException("cancelled", "AbortError")),
  );
  const page = {
    getViewport: () => ({ width: 100, height: 200 }),
    render: () => ({ promise: renderPromise.promise, cancel }),
    cleanup: vi.fn(),
  };
  const pdf = {
    numPages: 1,
    cleanup: vi.fn(),
    getPage: vi.fn().mockResolvedValue(page),
  } as unknown as PDFDocumentProxy;
  const loaded = await loadPdfFromTask(
    fakeTask(Promise.resolve(pdf)),
    new AbortController().signal,
  );
  const controller = new AbortController();
  const canvas = document.createElement("canvas");
  const rendering = loaded.renderPage(1, 1, canvas, controller.signal);
  await vi.waitFor(() => expect(pdf.getPage).toHaveBeenCalledWith(1));
  controller.abort();
  await expect(rendering).rejects.toMatchObject({ name: "AbortError" });
  expect(cancel).toHaveBeenCalledTimes(1);
  expect(page.cleanup).toHaveBeenCalledTimes(1);
});
