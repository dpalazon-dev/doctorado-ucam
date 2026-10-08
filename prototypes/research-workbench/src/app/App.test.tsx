import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import type {
  IpcResult,
  LibraryInfoDto,
  AppInfoDto,
} from "../shared/contracts/generated/contracts";

const { api } = vi.hoisted(() => ({
  api: {
    settings: {
      getAppInfo: vi.fn(),
      getLibraryInfo: vi.fn(),
      getLibraryStatus: vi.fn(),
    },
    library: {
      selectPdf: vi.fn(),
      listPapers: vi.fn(),
      cancelImport: vi.fn(),
    },
    reader: {
      getLastOpenedPaper: vi.fn(),
    },
  },
}));

vi.mock("../shared/adapters/tauri", () => ({ api }));

import { App } from "./App";

afterEach(cleanup);

const id = "00000000-0000-4000-8000-000000000101";
const success = <T,>(data: T): IpcResult<T> => ({
  contractVersion: 1,
  requestId: id,
  ok: true,
  data,
});
const failure: IpcResult<never> = {
  contractVersion: 1,
  requestId: id,
  ok: false,
  error: {
    code: "ImportRecoveryRequired",
    message: "La importación requiere recuperación.",
    retryable: true,
  },
};
const preview = {
  importToken: id,
  originalFilename: "synthetic.pdf",
  sizeBytes: 100,
  sha256: "a".repeat(64),
  candidates: [],
  expiresAt: "2026-10-03T00:00:00Z",
};
const appInfo: AppInfoDto = {
  appVersion: "0.0.1",
  state: "ready",
  contractVersion: 1,
  schemaVersion: 1,
  libraryId: id,
  libraryRootLabel: "Biblioteca sintética",
  capabilities: { reader: true, workflow: false, knowledge: false },
};
const libraryInfo: LibraryInfoDto = {
  libraryId: id,
  displayName: "Biblioteca local",
  rootLabel: "Biblioteca sintética",
  schemaVersion: 1,
  writable: true,
};
const status = (recoveryRequired: boolean) =>
  success({ writable: true, activeOperations: 0, recoveryRequired });

beforeEach(() => {
  vi.clearAllMocks();
  api.settings.getAppInfo.mockResolvedValue(success(appInfo));
  api.settings.getLibraryInfo.mockResolvedValue(success(libraryInfo));
  api.settings.getLibraryStatus.mockResolvedValue(status(false));
  api.library.listPapers.mockResolvedValue(
    success({ items: [], nextCursor: null, total: 0 }),
  );
  api.library.selectPdf.mockResolvedValue(failure);
  api.library.cancelImport.mockResolvedValue(success(null));
  api.reader.getLastOpenedPaper.mockResolvedValue(success(null));
});

async function enterLibrary() {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Entendido" }));
  fireEvent.click(
    await screen.findByRole("button", { name: "Abrir biblioteca" }),
  );
  await screen.findByRole("heading", { name: "Tu biblioteca" });
}

it("refreshes_recovery_after_a_failed_pdf_selection_and_when_entering_settings", async () => {
  api.settings.getLibraryStatus
    .mockResolvedValueOnce(status(false))
    .mockResolvedValue(status(true));
  await enterLibrary();
  fireEvent.click(
    (await screen.findAllByRole("button", { name: "Importar PDF" }))[0],
  );
  fireEvent.click(
    await screen.findByRole("button", { name: "Seleccionar PDF" }),
  );
  await screen.findByRole("alert");
  await waitFor(() =>
    expect(api.settings.getLibraryStatus).toHaveBeenCalledTimes(2),
  );
  fireEvent.click(screen.getByRole("button", { name: "Cerrar" }));
  fireEvent.click(screen.getByRole("button", { name: "Configuración" }));
  expect(await screen.findByText("Recuperación necesaria")).toBeTruthy();
  expect(api.settings.getLibraryStatus).toHaveBeenCalledTimes(3);
});

it("allows_rechecking_a_recovery_notice_and_discards_an_older_status_result", async () => {
  let resolveOld!: (value: ReturnType<typeof status>) => void;
  api.settings.getLibraryStatus
    .mockResolvedValueOnce(status(false))
    .mockImplementationOnce(
      () => new Promise((resolve) => (resolveOld = resolve)),
    )
    .mockResolvedValueOnce(status(false));
  await enterLibrary();
  fireEvent.click(
    (await screen.findAllByRole("button", { name: "Importar PDF" }))[0],
  );
  fireEvent.click(
    await screen.findByRole("button", { name: "Seleccionar PDF" }),
  );
  await screen.findByRole("alert");
  await waitFor(() =>
    expect(api.settings.getLibraryStatus).toHaveBeenCalledTimes(2),
  );
  fireEvent.click(screen.getByRole("button", { name: "Cerrar" }));
  fireEvent.click(screen.getByRole("button", { name: "Configuración" }));
  await waitFor(() =>
    expect(api.settings.getLibraryStatus).toHaveBeenCalledTimes(3),
  );
  expect(await screen.findByText("Disponible")).toBeTruthy();
  await act(async () => resolveOld(status(true)));
  expect(screen.getByText("Disponible")).toBeTruthy();
  expect(screen.queryByText("Recuperación necesaria")).toBeNull();
});

it("clears_the_recovery_notice_after_cancel_finishes_the_last_pending_import", async () => {
  api.settings.getLibraryStatus
    .mockResolvedValueOnce(status(true))
    .mockResolvedValueOnce(status(true))
    .mockResolvedValueOnce(status(false));
  api.library.selectPdf.mockResolvedValue(success(preview));
  await enterLibrary();
  fireEvent.click(
    (await screen.findAllByRole("button", { name: "Importar PDF" }))[0],
  );
  fireEvent.click(
    await screen.findByRole("button", { name: "Seleccionar PDF" }),
  );
  await screen.findByLabelText("Título");
  await waitFor(() =>
    expect(api.settings.getLibraryStatus).toHaveBeenCalledTimes(2),
  );
  fireEvent.click(screen.getByRole("button", { name: "Cancelar" }));
  await waitFor(() =>
    expect(api.settings.getLibraryStatus).toHaveBeenCalledTimes(3),
  );
  await waitFor(() =>
    expect(
      screen.queryByRole("heading", {
        name: "La biblioteca necesita recuperación",
      }),
    ).toBeNull(),
  );
  expect(api.library.cancelImport).toHaveBeenCalledWith(
    expect.objectContaining({ importToken: id }),
  );
});

it("shows_a_manual_recheck_when_recovery_is_reported_and_updates_settings_on_entry", async () => {
  api.settings.getLibraryStatus
    .mockResolvedValueOnce(status(true))
    .mockResolvedValueOnce(status(false))
    .mockResolvedValueOnce(status(true));
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Entendido" }));
  expect(
    await screen.findByRole("heading", {
      name: "La biblioteca necesita recuperación",
    }),
  ).toBeTruthy();
  fireEvent.click(
    screen.getByRole("button", { name: "Comprobar estado de recuperación" }),
  );
  await waitFor(() =>
    expect(
      screen.queryByRole("heading", {
        name: "La biblioteca necesita recuperación",
      }),
    ).toBeNull(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Configuración" }));
  await waitFor(() =>
    expect(screen.getByText("Recuperación necesaria")).toBeTruthy(),
  );
  expect(api.settings.getLibraryStatus).toHaveBeenCalledTimes(3);
});
