import { afterEach, expect, it, vi } from "vitest";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { LibraryPage } from "./LibraryPage";
import { LibraryHome } from "./LibraryHome";
import { ImportPaperDialog } from "./ImportPaperDialog";
import type { LibraryApi, ReaderApi } from "../../shared/contracts/ports";
import type {
  IpcResult,
  ImportPreviewDto,
  PaperDto,
  OpenPaperDto,
} from "../../shared/contracts/generated/contracts";

afterEach(cleanup);
const id = "00000000-0000-4000-8000-000000000001";
const paper = (patch: Partial<PaperDto> = {}): PaperDto => ({
  id,
  documentId: "00000000-0000-4000-8000-000000000002",
  title: "Agua industrial y recuperación",
  authors: ["Ana Ruiz"],
  year: 2024,
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
  ...patch,
});
const ok = <T,>(data: T): IpcResult<T> => ({
  contractVersion: 1,
  requestId: id,
  ok: true,
  data,
});
const failure = (code: string): IpcResult<never> => ({
  contractVersion: 1,
  requestId: id,
  ok: false,
  error: {
    code: code as never,
    message: "No se pudo guardar. Revisa los datos.",
    retryable: false,
  },
});
function makeApi(overrides: Partial<LibraryApi> = {}): LibraryApi {
  return {
    selectPdf: vi.fn().mockResolvedValue(ok(null)),
    confirmImport: vi.fn().mockResolvedValue(ok(paper())),
    cancelImport: vi.fn().mockResolvedValue(ok(null)),
    listPapers: vi
      .fn()
      .mockResolvedValue(ok({ items: [], nextCursor: null, total: 0 })),
    getPaper: vi.fn().mockResolvedValue(ok(paper())),
    updateMetadata: vi.fn().mockResolvedValue(ok(paper())),
    archivePaper: vi
      .fn()
      .mockResolvedValue(
        ok(paper({ lifecycle: "ARCHIVED", archivedFromLifecycle: "NEW" })),
      ),
    restorePaper: vi.fn().mockResolvedValue(ok(paper())),
    ...overrides,
  } as LibraryApi;
}
const opened: OpenPaperDto = {
  paper: paper(),
  document: {
    id: "00000000-0000-4000-8000-000000000002",
    paperId: id,
    originalFilename: "synthetic.pdf",
    sha256: "a".repeat(64),
    importedAt: "2026-10-01T00:00:00Z",
    status: "ACTIVE",
  },
  readingPosition: {
    documentId: "00000000-0000-4000-8000-000000000002",
    pageIndex: 1,
    zoom: 1,
    revision: 0,
    updatedAt: "2026-10-01T00:00:00Z",
  },
  documentUrl: "http://research.localhost/00000000-0000-4000-8000-000000000002",
};

it("empty_library_shows_import", async () => {
  const api = makeApi();
  render(
    <LibraryPage
      api={api}
      readerApi={
        {
          openPaper: vi.fn().mockResolvedValue(ok(opened)),
        } as unknown as ReaderApi
      }
      onOpenPaper={vi.fn()}
    />,
  );
  expect(
    await screen.findByRole("heading", { name: "Tu biblioteca" }),
  ).toBeTruthy();
  expect(
    (await screen.findAllByRole("button", { name: "Importar PDF" })).length,
  ).toBeGreaterThan(0);
});
it("picker_cancel_no_form", async () => {
  const api = makeApi();
  render(
    <ImportPaperDialog
      api={api}
      onClose={vi.fn()}
      onImported={vi.fn()}
      readerApi={{ openPaper: vi.fn() } as unknown as ReaderApi}
      onOpenPaper={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Seleccionar PDF" }));
  await waitFor(() => expect(api.selectPdf).toHaveBeenCalled());
  expect(screen.queryByLabelText("Título")).toBeNull();
});
it("metadata_error_accessible", async () => {
  const api = makeApi({
    selectPdf: vi.fn().mockResolvedValue(
      ok({
        importToken: id,
        originalFilename: "sample.pdf",
        sizeBytes: 100,
        sha256: "a".repeat(64),
        candidates: [],
        expiresAt: "2026-10-03T00:00:00Z",
      }),
    ),
  });
  render(
    <ImportPaperDialog
      api={api}
      onClose={vi.fn()}
      onImported={vi.fn()}
      readerApi={{ openPaper: vi.fn() } as unknown as ReaderApi}
      onOpenPaper={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Seleccionar PDF" }));
  fireEvent.click(
    await screen.findByRole("button", { name: "Guardar en biblioteca" }),
  );
  expect(
    await screen.findByText("Escribe un título para identificar el documento."),
  ).toBeTruthy();
});
it("duplicate_opens_existing", async () => {
  const candidate = paper({ title: "Documento existente" });
  const api = makeApi({
    selectPdf: vi.fn().mockResolvedValue(
      ok({
        importToken: id,
        originalFilename: "sample.pdf",
        sizeBytes: 100,
        sha256: "a".repeat(64),
        candidates: [
          {
            paperId: candidate.id,
            title: candidate.title,
            reasons: ["sha256"],
          },
        ],
        expiresAt: "2026-10-03T00:00:00Z",
      }),
    ),
  });
  const readerApi = {
    openPaper: vi.fn().mockResolvedValue(ok(opened)),
  } as unknown as ReaderApi;
  const onOpenPaper = vi.fn();
  render(
    <ImportPaperDialog
      api={api}
      onClose={vi.fn()}
      onImported={vi.fn()}
      readerApi={readerApi}
      onOpenPaper={onOpenPaper}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Seleccionar PDF" }));
  fireEvent.click(
    await screen.findByRole("button", { name: "Abrir existente" }),
  );
  await waitFor(() => expect(api.cancelImport).toHaveBeenCalled());
  await waitFor(() =>
    expect(readerApi.openPaper).toHaveBeenCalledWith(
      expect.objectContaining({ paperId: candidate.id }),
    ),
  );
});
it("archive_filter_and_restore", async () => {
  const api = makeApi({
    listPapers: vi.fn().mockResolvedValue(
      ok({
        items: [paper({ lifecycle: "ARCHIVED", archivedFromLifecycle: "NEW" })],
        nextCursor: null,
        total: 1,
      }),
    ),
  });
  render(
    <LibraryPage
      api={api}
      readerApi={{ openPaper: vi.fn() } as unknown as ReaderApi}
      onOpenPaper={vi.fn()}
    />,
  );
  fireEvent.change(await screen.findByLabelText("Estado"), {
    target: { value: "ARCHIVED" },
  });
  await waitFor(() =>
    expect(api.listPapers).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filter: expect.objectContaining({ lifecycle: "ARCHIVED" }),
      }),
    ),
  );
  fireEvent.click(await screen.findByRole("button", { name: "Restaurar" }));
  await waitFor(() => expect(api.restorePaper).toHaveBeenCalled());
});
it("conflict_preserves_draft", async () => {
  const api = makeApi({
    listPapers: vi
      .fn()
      .mockResolvedValue(ok({ items: [paper()], nextCursor: null, total: 1 })),
    updateMetadata: vi.fn().mockResolvedValue(failure("Conflict")),
  });
  render(
    <LibraryPage
      api={api}
      readerApi={{ openPaper: vi.fn() } as unknown as ReaderApi}
      onOpenPaper={vi.fn()}
    />,
  );
  fireEvent.click(await screen.findByRole("button", { name: "Editar" }));
  const input = await screen.findByLabelText("Título");
  fireEvent.change(input, { target: { value: "Mi borrador pendiente" } });
  fireEvent.click(screen.getByRole("button", { name: "Guardar cambios" }));
  expect(await screen.findByRole("alert")).toBeTruthy();
  expect(screen.getByLabelText("Título")).toHaveProperty(
    "value",
    "Mi borrador pendiente",
  );
});

it("metadata_draft_never_moves_to_another_paper", async () => {
  const second = paper({
    id: "00000000-0000-4000-8000-000000000004",
    documentId: "00000000-0000-4000-8000-000000000005",
    title: "Paper B",
  });
  const api = makeApi({
    listPapers: vi
      .fn()
      .mockResolvedValue(
        ok({ items: [paper(), second], nextCursor: null, total: 2 }),
      ),
  });
  render(
    <LibraryPage
      api={api}
      readerApi={{ openPaper: vi.fn() } as unknown as ReaderApi}
      onOpenPaper={vi.fn()}
    />,
  );
  fireEvent.click(
    (await screen.findAllByRole("button", { name: "Editar" }))[0],
  );
  fireEvent.change(await screen.findByLabelText("Título"), {
    target: { value: "Borrador de A" },
  });
  fireEvent.click(screen.getAllByRole("button", { name: "Editar" })[1]);
  fireEvent.click(
    await screen.findByRole("button", { name: "Guardar cambios" }),
  );
  await waitFor(() => expect(api.updateMetadata).toHaveBeenCalled());
  expect(api.updateMetadata).toHaveBeenLastCalledWith(
    expect.objectContaining({
      paperId: second.id,
      metadata: expect.objectContaining({ title: "Paper B" }),
    }),
  );
});

it("library_pages_forward_cursor_and_reset_after_filter_change", async () => {
  const next = paper({
    id: "00000000-0000-4000-8000-000000000004",
    title: "Página dos",
  });
  const list = vi
    .fn()
    .mockImplementation(({ filter }) =>
      Promise.resolve(
        ok(
          filter.cursor
            ? { items: [next], nextCursor: null, total: 101 }
            : { items: [paper()], nextCursor: "cursor-2", total: 101 },
        ),
      ),
    );
  const api = makeApi({ listPapers: list });
  render(
    <LibraryPage
      api={api}
      readerApi={{ openPaper: vi.fn() } as unknown as ReaderApi}
      onOpenPaper={vi.fn()}
    />,
  );
  fireEvent.click(
    await screen.findByRole("button", { name: "Siguiente página" }),
  );
  await waitFor(() => expect(screen.getByText("Página dos")).toBeTruthy());
  expect(list).toHaveBeenLastCalledWith(
    expect.objectContaining({
      filter: expect.objectContaining({ cursor: "cursor-2" }),
    }),
  );
  fireEvent.change(screen.getByLabelText("Estado"), {
    target: { value: "ARCHIVED" },
  });
  await waitFor(() =>
    expect(list).toHaveBeenLastCalledWith(
      expect.objectContaining({
        filter: expect.objectContaining({
          cursor: null,
          lifecycle: "ARCHIVED",
        }),
      }),
    ),
  );
});

it("duplicate_open_failure_remains_visible_and_retries_reader_only", async () => {
  const candidate = paper({ title: "Paper ya existente" });
  const api = makeApi({
    selectPdf: vi.fn().mockResolvedValue(
      ok({
        importToken: id,
        originalFilename: "sample.pdf",
        sizeBytes: 100,
        sha256: "a".repeat(64),
        candidates: [
          {
            paperId: candidate.id,
            title: candidate.title,
            reasons: ["sha256"],
          },
        ],
        expiresAt: "2026-10-03T00:00:00Z",
      }),
    ),
  });
  const readerApi = {
    openPaper: vi
      .fn()
      .mockResolvedValueOnce(failure("StorageUnavailable"))
      .mockResolvedValueOnce(ok(opened)),
  } as unknown as ReaderApi;
  render(
    <ImportPaperDialog
      api={api}
      onClose={vi.fn()}
      onImported={vi.fn()}
      readerApi={readerApi}
      onOpenPaper={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Seleccionar PDF" }));
  fireEvent.click(
    await screen.findByRole("button", { name: "Abrir existente" }),
  );
  fireEvent.click(
    await screen.findByRole("button", { name: "Reintentar abrir existente" }),
  );
  await waitFor(() => expect(readerApi.openPaper).toHaveBeenCalledTimes(2));
  expect(api.cancelImport).toHaveBeenCalledTimes(1);
  expect(api.selectPdf).toHaveBeenCalledTimes(1);
});

it("failed_duplicate_cancel_is_visible_and_never_opens_reader", async () => {
  const candidate = paper({ title: "Paper ya existente" });
  const api = makeApi({
    selectPdf: vi.fn().mockResolvedValue(
      ok({
        importToken: id,
        originalFilename: "sample.pdf",
        sizeBytes: 100,
        sha256: "a".repeat(64),
        candidates: [
          {
            paperId: candidate.id,
            title: candidate.title,
            reasons: ["sha256"],
          },
        ],
        expiresAt: "2026-10-03T00:00:00Z",
      }),
    ),
    cancelImport: vi.fn().mockResolvedValue(failure("StorageUnavailable")),
  });
  const readerApi = { openPaper: vi.fn() } as unknown as ReaderApi;
  render(
    <ImportPaperDialog
      api={api}
      onClose={vi.fn()}
      onImported={vi.fn()}
      readerApi={readerApi}
      onOpenPaper={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Seleccionar PDF" }));
  fireEvent.click(
    await screen.findByRole("button", { name: "Abrir existente" }),
  );
  expect(await screen.findByRole("alert")).toBeTruthy();
  expect(readerApi.openPaper).not.toHaveBeenCalled();
});

it("out_of_order_open_responses_follow_the_latest_library_intent", async () => {
  const first = paper();
  const second = paper({
    id: "00000000-0000-4000-8000-000000000004",
    title: "Paper B",
  });
  const api = makeApi({
    listPapers: vi
      .fn()
      .mockResolvedValue(
        ok({ items: [first, second], nextCursor: null, total: 2 }),
      ),
  });
  let resolveA!: (value: IpcResult<OpenPaperDto>) => void;
  let resolveB!: (value: IpcResult<OpenPaperDto>) => void;
  const readerApi = {
    openPaper: vi.fn(
      ({ paperId }: { paperId: string }) =>
        new Promise<IpcResult<OpenPaperDto>>((resolve) => {
          if (paperId === first.id) resolveA = resolve;
          else resolveB = resolve;
        }),
    ),
  } as unknown as ReaderApi;
  const onOpenPaper = vi.fn();
  render(
    <LibraryPage api={api} readerApi={readerApi} onOpenPaper={onOpenPaper} />,
  );
  const buttons = await screen.findAllByRole("button", { name: "Leer" });
  fireEvent.click(buttons[0]);
  fireEvent.click(buttons[1]);
  await act(async () => {
    resolveB(ok({ ...opened, paper: second }));
  });
  await act(async () => {
    resolveA(ok(opened));
  });
  expect(onOpenPaper).toHaveBeenCalledTimes(1);
  expect(onOpenPaper).toHaveBeenCalledWith(
    expect.objectContaining({
      paper: expect.objectContaining({ id: second.id }),
    }),
  );
});

it("dialog_close_uses_confirmed_cancel_and_preserves_preview_when_cancel_fails", async () => {
  const preview = {
    importToken: id,
    originalFilename: "sample.pdf",
    sizeBytes: 100,
    sha256: "a".repeat(64),
    candidates: [],
    expiresAt: "2026-10-03T00:00:00Z",
  };
  const api = makeApi({
    selectPdf: vi.fn().mockResolvedValue(ok(preview)),
    cancelImport: vi.fn().mockResolvedValue(failure("StorageUnavailable")),
  });
  const onClose = vi.fn();
  render(
    <ImportPaperDialog
      api={api}
      onClose={onClose}
      onImported={vi.fn()}
      readerApi={{ openPaper: vi.fn() } as unknown as ReaderApi}
      onOpenPaper={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Seleccionar PDF" }));
  await screen.findByLabelText("Título");
  fireEvent.click(screen.getByRole("button", { name: "Cerrar" }));
  expect(await screen.findByRole("alert")).toBeTruthy();
  expect(onClose).not.toHaveBeenCalled();
  expect(screen.getByLabelText("Título")).toBeTruthy();
  expect(api.cancelImport).toHaveBeenCalledTimes(1);
});

it("late_metadata_save_does_not_reselect_a_paper_the_user_left", async () => {
  const first = paper();
  const second = paper({
    id: "00000000-0000-4000-8000-000000000004",
    title: "Paper B",
  });
  let resolveSave!: (value: IpcResult<PaperDto>) => void;
  const api = makeApi({
    listPapers: vi
      .fn()
      .mockResolvedValue(
        ok({ items: [first, second], nextCursor: null, total: 2 }),
      ),
    updateMetadata: vi.fn(
      () =>
        new Promise<IpcResult<PaperDto>>((resolve) => {
          resolveSave = resolve;
        }),
    ),
  });
  render(
    <LibraryPage
      api={api}
      readerApi={{ openPaper: vi.fn() } as unknown as ReaderApi}
      onOpenPaper={vi.fn()}
    />,
  );
  const editButtons = await screen.findAllByRole("button", { name: "Editar" });
  fireEvent.click(editButtons[0]);
  fireEvent.change(await screen.findByLabelText("Título"), {
    target: { value: "Cambio A" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Guardar cambios" }));
  fireEvent.click(screen.getAllByRole("button", { name: "Editar" })[1]);
  expect(screen.getByLabelText("Título")).toHaveProperty("value", "Paper B");
  await act(async () => {
    resolveSave(ok(first));
  });
  expect(screen.getByLabelText("Título")).toHaveProperty("value", "Paper B");
  expect(screen.getAllByRole("heading", { name: "Paper B" })).toHaveLength(2);
});

it("home_discards_open_response_after_leaving_home", async () => {
  let resolveOpen!: (value: IpcResult<OpenPaperDto>) => void;
  const readerApi = {
    getLastOpenedPaper: vi.fn().mockResolvedValue(ok(paper())),
    openPaper: vi.fn(
      () =>
        new Promise<IpcResult<OpenPaperDto>>((resolve) => {
          resolveOpen = resolve;
        }),
    ),
  } as unknown as ReaderApi;
  const libraryApi = makeApi();
  const onOpenPaper = vi.fn();
  const view = render(
    <LibraryHome
      libraryApi={libraryApi}
      readerApi={readerApi}
      onOpenPaper={onOpenPaper}
      onShowLibrary={vi.fn()}
    />,
  );
  fireEvent.click(
    await screen.findByRole("button", { name: "Reanudar lectura" }),
  );
  view.unmount();
  await act(async () => {
    resolveOpen(ok(opened));
  });
  expect(onOpenPaper).not.toHaveBeenCalled();
});

it.each(["selection pending", "preview visible"])(
  "old_library_open_does_not_abandon_import_when_%s",
  async (importState) => {
    let resolveOpen!: (value: IpcResult<OpenPaperDto>) => void;
    let resolveSelection!: (value: IpcResult<ImportPreviewDto | null>) => void;
    const preview = {
      importToken: id,
      originalFilename: "new-paper.pdf",
      sizeBytes: 100,
      sha256: "c".repeat(64),
      candidates: [],
      expiresAt: "2026-10-03T00:00:00Z",
    };
    const api = makeApi({
      listPapers: vi
        .fn()
        .mockResolvedValue(
          ok({ items: [paper()], nextCursor: null, total: 1 }),
        ),
      selectPdf:
        importState === "selection pending"
          ? vi.fn(
              () =>
                new Promise((resolve) => {
                  resolveSelection = resolve;
                }),
            )
          : vi.fn().mockResolvedValue(ok(preview)),
    });
    const readerApi = {
      openPaper: vi.fn(
        () =>
          new Promise<IpcResult<OpenPaperDto>>((resolve) => {
            resolveOpen = resolve;
          }),
      ),
    } as unknown as ReaderApi;
    const onOpenPaper = vi.fn();
    render(
      <LibraryPage api={api} readerApi={readerApi} onOpenPaper={onOpenPaper} />,
    );
    fireEvent.click(
      (await screen.findAllByRole("button", { name: "Leer" }))[0],
    );
    fireEvent.click(
      (await screen.findAllByRole("button", { name: "Importar PDF" }))[0],
    );
    fireEvent.click(
      await screen.findByRole("button", { name: "Seleccionar PDF" }),
    );
    if (importState === "preview visible")
      await screen.findByLabelText("Título");
    await act(async () => {
      resolveOpen(ok(opened));
    });
    expect(onOpenPaper).not.toHaveBeenCalled();
    expect(api.cancelImport).not.toHaveBeenCalled();
    if (importState === "preview visible")
      expect(screen.getByLabelText("Título")).toBeTruthy();
    resolveSelection?.(ok(preview));
  },
);

it("late_duplicate_decision_after_doi_shows_all_validated_candidates", async () => {
  const first = paper({ title: "Candidato A" });
  const second = paper({
    id: "00000000-0000-4000-8000-000000000004",
    documentId: "00000000-0000-4000-8000-000000000005",
    title: "Candidato B",
  });
  const api = makeApi({
    selectPdf: vi.fn().mockResolvedValue(
      ok({
        importToken: id,
        originalFilename: "new.pdf",
        sizeBytes: 100,
        sha256: "d".repeat(64),
        candidates: [],
        expiresAt: "2026-10-03T00:00:00Z",
      }),
    ),
    confirmImport: vi.fn().mockResolvedValue({
      contractVersion: 1,
      requestId: id,
      ok: false,
      error: {
        code: "DuplicateDecisionRequired",
        message: "El DOI coincide con dos fichas.",
        retryable: false,
        details: {
          candidates: [
            { paperId: first.id, title: first.title, reasons: ["doi"] },
            { paperId: second.id, title: second.title, reasons: ["doi"] },
          ],
        },
      },
    }),
  });
  render(
    <ImportPaperDialog
      api={api}
      onClose={vi.fn()}
      onImported={vi.fn()}
      readerApi={{ openPaper: vi.fn() } as unknown as ReaderApi}
      onOpenPaper={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Seleccionar PDF" }));
  fireEvent.change(await screen.findByLabelText("Título"), {
    target: { value: "Paper con DOI" },
  });
  fireEvent.change(screen.getByLabelText("DOI"), {
    target: { value: "10.1234/paper" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Guardar en biblioteca" }),
  );
  expect(
    await screen.findAllByRole("button", { name: "Abrir existente" }),
  ).toHaveLength(2);
  expect(screen.getByRole("heading", { name: "Candidato B" })).toBeTruthy();
  expect(api.confirmImport).toHaveBeenCalledWith(
    expect.objectContaining({
      metadata: expect.objectContaining({ doi: "10.1234/paper" }),
    }),
  );
});

it("candidate_choice_waits_for_confirmed_cancel_and_escape_cannot_change_candidate", async () => {
  const first = paper({ title: "Candidato A" });
  const second = paper({
    id: "00000000-0000-4000-8000-000000000004",
    documentId: "00000000-0000-4000-8000-000000000005",
    title: "Candidato B",
  });
  let resolveCancel!: (value: IpcResult<null>) => void;
  const api = makeApi({
    selectPdf: vi.fn().mockResolvedValue(
      ok({
        importToken: id,
        originalFilename: "new.pdf",
        sizeBytes: 100,
        sha256: "e".repeat(64),
        candidates: [
          { paperId: first.id, title: first.title, reasons: ["doi"] },
          { paperId: second.id, title: second.title, reasons: ["doi"] },
        ],
        expiresAt: "2026-10-03T00:00:00Z",
      }),
    ),
    cancelImport: vi.fn(
      () =>
        new Promise<IpcResult<null>>((resolve) => (resolveCancel = resolve)),
    ),
  });
  const readerApi = {
    openPaper: vi.fn().mockResolvedValue(ok({ ...opened, paper: second })),
  } as unknown as ReaderApi;
  const onClose = vi.fn();
  render(
    <ImportPaperDialog
      api={api}
      onClose={onClose}
      onImported={vi.fn()}
      readerApi={readerApi}
      onOpenPaper={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Seleccionar PDF" }));
  const choices = await screen.findAllByRole("button", {
    name: "Abrir existente",
  });
  fireEvent.click(choices[1]);
  await waitFor(() => expect(api.cancelImport).toHaveBeenCalledTimes(1));
  expect(readerApi.openPaper).not.toHaveBeenCalled();
  expect(screen.queryByRole("button", { name: "Cerrar" })).toBeNull();
  fireEvent.keyDown(document, { key: "Escape" });
  expect(onClose).not.toHaveBeenCalled();
  await act(async () => resolveCancel(ok(null)));
  await waitFor(() =>
    expect(readerApi.openPaper).toHaveBeenCalledWith(
      expect.objectContaining({ paperId: second.id }),
    ),
  );
  expect(onClose).toHaveBeenCalledTimes(1);
});

it("invalid_import_details_keep_the_draft_and_do_not_confirm", async () => {
  const api = makeApi({
    selectPdf: vi.fn().mockResolvedValue(
      ok({
        importToken: id,
        originalFilename: "new.pdf",
        sizeBytes: 100,
        sha256: "f".repeat(64),
        candidates: [],
        expiresAt: "2026-10-03T00:00:00Z",
      }),
    ),
  });
  render(
    <ImportPaperDialog
      api={api}
      onClose={vi.fn()}
      onImported={vi.fn()}
      readerApi={{ openPaper: vi.fn() } as unknown as ReaderApi}
      onOpenPaper={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Seleccionar PDF" }));
  fireEvent.change(await screen.findByLabelText("Título"), {
    target: { value: "Borrador válido" },
  });
  fireEvent.change(screen.getByLabelText("Año"), {
    target: { value: "20x4" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Guardar en biblioteca" }),
  );
  expect(await screen.findByRole("alert")).toBeTruthy();
  expect(screen.getByLabelText("Título")).toHaveProperty(
    "value",
    "Borrador válido",
  );
  expect(screen.getByLabelText("Año")).toHaveProperty("value", "20x4");
  expect(api.confirmImport).not.toHaveBeenCalled();
});

it("malformed_duplicate_candidate_keeps_draft_and_allows_cancel_without_opening", async () => {
  const api = makeApi({
    selectPdf: vi.fn().mockResolvedValue(
      ok({
        importToken: id,
        originalFilename: "new.pdf",
        sizeBytes: 100,
        sha256: "1".repeat(64),
        candidates: [],
        expiresAt: "2026-10-03T00:00:00Z",
      }),
    ),
    confirmImport: vi.fn().mockResolvedValue({
      contractVersion: 1,
      requestId: id,
      ok: false,
      error: {
        code: "DuplicateDecisionRequired",
        message: "Hay una ficha duplicada.",
        retryable: false,
        details: {
          candidates: [
            { paperId: "not-a-uuid", title: "Candidato inválido", reasons: ["doi"] },
          ],
        },
      },
    } as IpcResult<PaperDto>),
  });
  const openPaper = vi.fn();
  const onOpenPaper = vi.fn();
  render(
    <ImportPaperDialog
      api={api}
      onClose={vi.fn()}
      onImported={vi.fn()}
      readerApi={{ openPaper } as unknown as ReaderApi}
      onOpenPaper={onOpenPaper}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Seleccionar PDF" }));
  fireEvent.change(await screen.findByLabelText("Título"), {
    target: { value: "Borrador conservado" },
  });
  fireEvent.change(screen.getByLabelText("Año"), {
    target: { value: "2024" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Guardar en biblioteca" }),
  );
  expect(
    await screen.findByText(
      "Se detectó un duplicado, pero no se pudo validar su ficha. Puedes cancelar la importación.",
    ),
  ).toBeTruthy();
  expect(screen.getByLabelText("Título")).toHaveProperty(
    "value",
    "Borrador conservado",
  );
  expect(screen.getByLabelText("Año")).toHaveProperty("value", "2024");
  expect(screen.getByRole("button", { name: "Cancelar" })).toHaveProperty(
    "disabled",
    false,
  );
  expect(api.cancelImport).not.toHaveBeenCalled();
  expect(openPaper).not.toHaveBeenCalled();
  expect(onOpenPaper).not.toHaveBeenCalled();
});

it.each(["selection", "confirmation"])(
  "escape_does_not_close_while_%s_is_pending",
  async (stage) => {
    let resolveSelection!: (value: IpcResult<ImportPreviewDto | null>) => void;
    let resolveConfirmation!: (value: IpcResult<PaperDto>) => void;
    const preview = {
      importToken: id,
      originalFilename: "pending.pdf",
      sizeBytes: 100,
      sha256: "b".repeat(64),
      candidates: [],
      expiresAt: "2026-10-03T00:00:00Z",
    };
    const api = makeApi({
      selectPdf:
        stage === "selection"
          ? vi.fn(
              () =>
                new Promise<IpcResult<ImportPreviewDto | null>>(
                  (resolve) => (resolveSelection = resolve),
                ),
            )
          : vi.fn().mockResolvedValue(ok(preview)),
      confirmImport: vi.fn(
        () =>
          new Promise<IpcResult<PaperDto>>(
            (resolve) => (resolveConfirmation = resolve),
          ),
      ),
    });
    const onClose = vi.fn();
    render(
      <ImportPaperDialog
        api={api}
        onClose={onClose}
        onImported={vi.fn()}
        readerApi={{ openPaper: vi.fn() } as unknown as ReaderApi}
        onOpenPaper={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Seleccionar PDF" }));
    if (stage === "confirmation") {
      fireEvent.change(await screen.findByLabelText("Título"), {
        target: { value: "Paper pendiente" },
      });
      fireEvent.click(
        screen.getByRole("button", { name: "Guardar en biblioteca" }),
      );
    }
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog")).toBeTruthy();
    if (stage === "selection") {
      await act(async () => resolveSelection(ok(preview)));
      expect(await screen.findByLabelText("Título")).toBeTruthy();
    } else {
      await act(async () => resolveConfirmation(failure("StorageUnavailable")));
      expect(screen.getByLabelText("Título")).toHaveProperty(
        "value",
        "Paper pendiente",
      );
    }
  },
);
