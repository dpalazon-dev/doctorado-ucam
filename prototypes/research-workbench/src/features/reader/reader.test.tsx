import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { PaperReader } from "./PaperReader";
import { loadPdf } from "./pdfLoader";
import type {
  OpenPaperDto,
  IpcResult,
  ReadingPositionDto,
} from "../../shared/contracts/generated/contracts";
import type { ReaderApi } from "../../shared/contracts/ports";
vi.mock("./pdfLoader", () => ({ loadPdf: vi.fn() }));
afterEach(cleanup);
beforeEach(() => {
  load.mockReset();
  pdf.renderPage.mockReset().mockResolvedValue(undefined);
  pdf.destroy.mockReset().mockResolvedValue(undefined);
  vi.mocked(api.getReadingPosition).mockReset().mockResolvedValue(ok(position));
  vi.mocked(api.saveReadingPosition)
    .mockReset()
    .mockResolvedValue(ok(position));
});
const docId = "00000000-0000-4000-8000-000000000002";
const opened: OpenPaperDto = {
  paper: {
    id: "00000000-0000-4000-8000-000000000001",
    documentId: docId,
    title: "Paper sintético",
    authors: [],
    year: null,
    doi: null,
    venue: null,
    reviewType: "unknown",
    domain: null,
    lifecycle: "NEW",
    archivedFromLifecycle: null,
    activePhaseCode: null,
    processingInitialized: false,
    revision: 0,
    createdAt: "2026-10-01T00:00:00Z",
    updatedAt: "2026-10-01T00:00:00Z",
    lastOpenedAt: null,
  },
  document: {
    id: docId,
    paperId: "00000000-0000-4000-8000-000000000001",
    originalFilename: "synthetic-scan.pdf",
    sha256: "b".repeat(64),
    importedAt: "2026-10-01T00:00:00Z",
    status: "ACTIVE",
  },
  readingPosition: {
    documentId: docId,
    pageIndex: 2,
    zoom: 1.25,
    revision: 1,
    updatedAt: "2026-10-01T00:00:00Z",
  },
  documentUrl: `http://research.localhost/${docId}`,
};
const ok = <T,>(data: T): IpcResult<T> => ({
  contractVersion: 1,
  requestId: "00000000-0000-4000-8000-000000000003",
  ok: true,
  data,
});
const position: ReadingPositionDto = {
  documentId: docId,
  pageIndex: 2,
  zoom: 1.25,
  revision: 2,
  updatedAt: "2026-10-02T00:00:00Z",
};
const api = {
  getReadingPosition: vi.fn().mockResolvedValue(ok(position)),
  saveReadingPosition: vi.fn().mockResolvedValue(ok(position)),
} as unknown as ReaderApi;
const load = vi.mocked(loadPdf);
const pdf = {
  numPages: 8,
  renderPage: vi.fn().mockResolvedValue(undefined),
  destroy: vi.fn().mockResolvedValue(undefined),
};
it("reader_saved_page_restored", async () => {
  load.mockResolvedValue(pdf);
  render(<PaperReader opened={opened} api={api} onClose={vi.fn()} />);
  await waitFor(() =>
    expect(pdf.renderPage).toHaveBeenCalledWith(
      2,
      1.25,
      expect.any(HTMLCanvasElement),
      expect.any(AbortSignal),
    ),
  );
});
it("reader_fractional_page_input_is_ignored", async () => {
  load.mockResolvedValue(pdf);
  render(<PaperReader opened={opened} api={api} onClose={vi.fn()} />);
  const input = await screen.findByLabelText("Ir a página");
  fireEvent.change(input, { target: { value: "2.5" } });
  expect(input).toHaveProperty("value", "2");
  expect(api.saveReadingPosition).not.toHaveBeenCalled();
});
it("scanned_pdf_renders", async () => {
  load.mockResolvedValue(pdf);
  render(<PaperReader opened={opened} api={api} onClose={vi.fn()} />);
  expect(
    await screen.findByRole("img", { name: "Página 2 del documento" }),
  ).toBeTruthy();
  await waitFor(() => expect(pdf.renderPage).toHaveBeenCalled());
});
it("corrupt_protected_pdf_recoverable", async () => {
  load.mockRejectedValue(new Error("encrypted"));
  render(<PaperReader opened={opened} api={api} onClose={vi.fn()} />);
  expect(await screen.findByRole("alert")).toBeTruthy();
  expect(screen.getByRole("button", { name: "Cerrar lector" })).toBeTruthy();
});
it("render_old_result_discarded", async () => {
  let rejectOld!: (reason: unknown) => void;
  let firstSignal: AbortSignal | undefined;
  pdf.renderPage
    .mockImplementationOnce((_page, _zoom, _canvas, signal) => {
      firstSignal = signal;
      return new Promise<void>((_resolve, reject) => {
        rejectOld = reject;
      });
    })
    .mockResolvedValue(undefined);
  load.mockResolvedValue(pdf);
  render(<PaperReader opened={opened} api={api} onClose={vi.fn()} />);
  await waitFor(() => expect(pdf.renderPage).toHaveBeenCalled());
  fireEvent.click(screen.getByRole("button", { name: "Página siguiente" }));
  expect(firstSignal?.aborted).toBe(true);
  rejectOld(new Error("late render failure"));
  await waitFor(() =>
    expect(screen.getByLabelText("Ir a página")).toHaveProperty("value", "3"),
  );
  expect(screen.queryByRole("alert")).toBeNull();
});
it("close_cancels_render", async () => {
  load.mockResolvedValue(pdf);
  const onClose = vi.fn();
  render(<PaperReader opened={opened} api={api} onClose={onClose} />);
  fireEvent.click(await screen.findByRole("button", { name: "Cerrar lector" }));
  expect(onClose).toHaveBeenCalled();
  await waitFor(() => expect(pdf.destroy).toHaveBeenCalled());
});
it("save_error_never_shows_saved", async () => {
  load.mockResolvedValue(pdf);
  const failingApi = {
    ...api,
    saveReadingPosition: vi.fn().mockResolvedValue({
      contractVersion: 1,
      requestId: "00000000-0000-4000-8000-000000000003",
      ok: false,
      error: {
        code: "StorageUnavailable",
        message: "No se pudo guardar.",
        retryable: true,
      },
    }),
  } as unknown as ReaderApi;
  render(<PaperReader opened={opened} api={failingApi} onClose={vi.fn()} />);
  fireEvent.click(
    await screen.findByRole("button", { name: "Página siguiente" }),
  );
  expect(await screen.findByRole("alert")).toBeTruthy();
  expect(screen.queryByText("Guardado")).toBeNull();
});

it("reader_conflict_offers_explicit_saved_or_local_position_resolution", async () => {
  const documentId = "00000000-0000-4000-8000-000000000121";
  const current = {
    ...opened,
    paper: { ...opened.paper, documentId },
    document: { ...opened.document, id: documentId },
    readingPosition: { ...opened.readingPosition, documentId, revision: 0 },
    documentUrl: `http://research.localhost/${documentId}`,
  };
  const save = vi
    .fn()
    .mockResolvedValueOnce({
      contractVersion: 1,
      requestId: "00000000-0000-4000-8000-000000000003",
      ok: false,
      error: {
        code: "Conflict",
        message: "La posición cambió en otra operación.",
        retryable: false,
      },
    })
    .mockResolvedValueOnce(
      ok({ ...position, documentId, pageIndex: 3, revision: 3 }),
    );
  const durable = { ...position, documentId, pageIndex: 9, revision: 2 };
  const conflictApi = {
    getReadingPosition: vi.fn().mockResolvedValue(ok(durable)),
    saveReadingPosition: save,
  } as unknown as ReaderApi;
  load.mockResolvedValue(pdf);
  render(<PaperReader opened={current} api={conflictApi} onClose={vi.fn()} />);
  fireEvent.click(
    await screen.findByRole("button", { name: "Página siguiente" }),
  );
  fireEvent.click(
    await screen.findByRole("button", { name: "Guardar mi posición" }),
  );
  await waitFor(() => expect(save).toHaveBeenCalledTimes(2));
  expect(save.mock.calls[1][0]).toMatchObject({
    documentId,
    expectedRevision: 2,
    pageIndex: 3,
  });
  expect(save.mock.calls[1][0].requestId).not.toBe(
    save.mock.calls[0][0].requestId,
  );
});

it("reader_keep_saved_applies_durable_page_and_zoom_to_the_canvas", async () => {
  const documentId = "00000000-0000-4000-8000-000000000129";
  const current = {
    ...opened,
    paper: { ...opened.paper, documentId },
    document: { ...opened.document, id: documentId },
    readingPosition: { ...opened.readingPosition, documentId, revision: 0 },
    documentUrl: `http://research.localhost/${documentId}`,
  };
  const conflictApi = {
    getReadingPosition: vi
      .fn()
      .mockResolvedValue(
        ok({ ...position, documentId, pageIndex: 5, zoom: 1.75, revision: 3 }),
      ),
    saveReadingPosition: vi.fn().mockResolvedValue({
      contractVersion: 1,
      requestId: "00000000-0000-4000-8000-000000000003",
      ok: false,
      error: {
        code: "Conflict",
        message: "La posición cambió en otra operación.",
        retryable: false,
      },
    }),
  } as unknown as ReaderApi;
  load.mockResolvedValue(pdf);
  render(<PaperReader opened={current} api={conflictApi} onClose={vi.fn()} />);
  fireEvent.click(
    await screen.findByRole("button", { name: "Página siguiente" }),
  );
  fireEvent.click(
    await screen.findByRole("button", { name: "Conservar posición guardada" }),
  );
  await waitFor(() =>
    expect(pdf.renderPage).toHaveBeenLastCalledWith(
      5,
      1.75,
      expect.any(HTMLCanvasElement),
      expect.any(AbortSignal),
    ),
  );
  expect(screen.getByLabelText("Ir a página")).toHaveProperty("value", "5");
  expect(screen.getByText("175%")).toBeTruthy();
});

it("reader_shows_pending_position_when_replay_confirms_an_older_page", async () => {
  const documentId = "00000000-0000-4000-8000-000000000135";
  const current = {
    ...opened,
    paper: { ...opened.paper, documentId },
    document: { ...opened.document, id: documentId },
    readingPosition: { ...opened.readingPosition, documentId, revision: 0 },
    documentUrl: `http://research.localhost/${documentId}`,
  };
  const uncertain: IpcResult<never> = {
    contractVersion: 1,
    requestId: "00000000-0000-4000-8000-000000000003",
    ok: false,
    error: {
      code: "StorageUnavailable",
      message: "No se pudo confirmar.",
      retryable: true,
    },
  };
  const save = vi
    .fn()
    .mockResolvedValueOnce(uncertain)
    .mockResolvedValueOnce(ok({ ...position, documentId, pageIndex: 3, revision: 1 }));
  const readerApi = {
    getReadingPosition: vi
      .fn()
      .mockResolvedValue(ok({ ...position, documentId, pageIndex: 3, revision: 1 })),
    saveReadingPosition: save,
  } as unknown as ReaderApi;
  load.mockResolvedValue(pdf);
  render(
    <PaperReader opened={current} api={readerApi} onClose={vi.fn()} />,
  );
  fireEvent.click(
    await screen.findByRole("button", { name: "Página siguiente" }),
  );
  await screen.findByRole("button", { name: "Reintentar guardado de posición" });
  fireEvent.click(screen.getByRole("button", { name: "Página siguiente" }));
  const retry = screen.getByRole("button", {
    name: "Reintentar guardado de posición",
  });
  await waitFor(() => expect(retry).toHaveProperty("disabled", false));
  fireEvent.click(retry);
  await waitFor(() => expect(save).toHaveBeenCalledTimes(2));
  expect(screen.getByLabelText("Ir a página")).toHaveProperty("value", "4");
  expect(await screen.findByText("Cambios pendientes")).toBeTruthy();
  expect(
    screen.getByRole("button", { name: "Guardar mi posición" }),
  ).toBeTruthy();
  expect(screen.queryByText("Guardado")).toBeNull();
});

it.each([
  ["conflict", "Conservar posición guardada"],
  ["uncertain result", "Reintentar guardado de posición"],
])(
  "reader_reopened_same_document_receives_late_%s_controls",
  async (label, recoveryAction) => {
    const documentId =
      label === "conflict"
        ? "00000000-0000-4000-8000-000000000136"
        : "00000000-0000-4000-8000-000000000137";
    const current = {
      ...opened,
      paper: { ...opened.paper, documentId },
      document: { ...opened.document, id: documentId },
      readingPosition: { ...opened.readingPosition, documentId, revision: 0 },
      documentUrl: `http://research.localhost/${documentId}`,
    };
    let finishSave!: (value: IpcResult<ReadingPositionDto>) => void;
    const save = vi.fn(
      () =>
        new Promise<IpcResult<ReadingPositionDto>>(
          (resolve) => (finishSave = resolve),
        ),
    );
    const readerApi = {
      getReadingPosition: vi
        .fn()
        .mockResolvedValue(ok({ ...position, documentId, pageIndex: 7, revision: 1 })),
      saveReadingPosition: save,
    } as unknown as ReaderApi;
    load.mockResolvedValue(pdf);
    const first = render(
      <PaperReader opened={current} api={readerApi} onClose={vi.fn()} />,
    );
    fireEvent.click(
      await screen.findByRole("button", { name: "Página siguiente" }),
    );
    await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
    first.unmount();
    render(
      <PaperReader
        opened={{
          ...current,
          readingPosition: { ...current.readingPosition, revision: 1 },
        }}
        api={readerApi}
        onClose={vi.fn()}
      />,
    );
    await screen.findByRole("img", { name: /Página/ });
    await act(async () => {
      finishSave(
        label === "conflict"
          ? {
              contractVersion: 1,
              requestId: "00000000-0000-4000-8000-000000000003",
              ok: false,
              error: {
                code: "Conflict",
                message: "La posición cambió en otra operación.",
                retryable: false,
              },
            }
          : {
              contractVersion: 1,
              requestId: "00000000-0000-4000-8000-000000000003",
              ok: false,
              error: {
                code: "StorageUnavailable",
                message: "No se pudo confirmar.",
                retryable: true,
              },
            },
      );
    });
    expect(await screen.findByRole("button", { name: recoveryAction })).toBeTruthy();
  },
);
