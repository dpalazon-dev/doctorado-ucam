import { useCallback, useEffect, useRef, useState } from "react";
import type {
  AppInfoDto,
  LibraryInfoDto,
  OpenPaperDto,
} from "../shared/contracts/generated/contracts";
import { api } from "../shared/adapters/tauri";
import type { ShellView } from "./ShellView";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from "../shared/ui/dialog";
import { LibraryHome } from "../features/library/LibraryHome";
import { LibraryPage } from "../features/library/LibraryPage";
import { PaperReader } from "../features/reader/PaperReader";

export function App() {
  const [view, setView] = useState<ShellView>({ kind: "Home" });
  const [opened, setOpened] = useState<OpenPaperDto | null>(null);
  const [info, setInfo] = useState<AppInfoDto | null>(null);
  const [library, setLibrary] = useState<LibraryInfoDto | null>(null);
  const [libraryStatus, setLibraryStatus] = useState<{
    writable: boolean;
    activeOperations: number;
    recoveryRequired: boolean;
  } | null>(null);
  const [statusError, setStatusError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [confirmed, setConfirmed] = useState(false);
  const statusRequest = useRef(0);
  const refreshLibraryStatus = useCallback(async () => {
    const request = ++statusRequest.current;
    try {
      const result = await api.settings.getLibraryStatus({
        requestId: crypto.randomUUID(),
      });
      if (request !== statusRequest.current) return;
      if (result.ok) {
        setLibraryStatus(result.data);
        setStatusError(null);
      } else setStatusError(result.error.message);
    } catch {
      if (request !== statusRequest.current) return;
      setStatusError(
        "No se pudo comprobar el estado de recuperación de la biblioteca.",
      );
    }
  }, []);
  useEffect(() => {
    let active = true;
    async function load() {
      const [app, lib] = await Promise.all([
        api.settings.getAppInfo({ requestId: crypto.randomUUID() }),
        api.settings.getLibraryInfo({ requestId: crypto.randomUUID() }),
      ]);
      if (!active) return;
      if (app.ok && lib.ok) {
        setInfo(app.data);
        setLibrary(lib.data);
      } else
        setError(
          !app.ok
            ? app.error.message
            : !lib.ok
              ? lib.error.message
              : "No se pudo abrir la biblioteca.",
        );
      void refreshLibraryStatus();
    }
    void load();
    return () => {
      active = false;
    };
  }, [refreshLibraryStatus]);
  function showSettings() {
    setView({ kind: "Settings" });
    void refreshLibraryStatus();
  }
  function showReader(document: OpenPaperDto) {
    setOpened(document);
    setView({ kind: "PaperWorkspace", paperId: document.paper.id });
    void refreshLibraryStatus();
  }
  function closeReader() {
    setOpened(null);
    setView({ kind: "Library" });
  }
  const activeView =
    view.kind === "PaperWorkspace"
      ? "Lector"
      : view.kind === "Library"
        ? "Biblioteca"
        : view.kind === "Settings"
          ? "Configuración"
          : "Inicio";
  return (
    <div className="app-shell">
      <aside aria-label="Navegación">
        <div className="brand">
          Research
          <br />
          <strong>Workbench</strong>
        </div>
        <nav>
          <button
            type="button"
            aria-current={view.kind === "Home" ? "page" : undefined}
            onClick={() => setView({ kind: "Home" })}
          >
            Inicio
          </button>
          <button
            type="button"
            aria-current={view.kind === "Library" ? "page" : undefined}
            onClick={() => setView({ kind: "Library" })}
          >
            Biblioteca
          </button>
          <button
            type="button"
            disabled
            aria-label="Conocimiento, próximamente"
          >
            Conocimiento · próximo
          </button>
          <button
            type="button"
            aria-current={view.kind === "Settings" ? "page" : undefined}
            onClick={showSettings}
          >
            Configuración
          </button>
        </nav>
        <small>Local · sin conexión</small>
      </aside>
      <main>
        <header>
          <span>Tu espacio de investigación</span>
          <span>{info ? "v" + info.appVersion : "Iniciando…"}</span>
        </header>
        {libraryStatus?.recoveryRequired && (
          <section className="recovery-notice" role="alert">
            <h2>La biblioteca necesita recuperación</h2>
            <p>
              Hay una operación de importación pendiente o que requiere
              atención. Algunas acciones pueden estar bloqueadas hasta
              resolverla.
            </p>
            <button type="button" onClick={() => void refreshLibraryStatus()}>
              Comprobar estado de recuperación
            </button>
          </section>
        )}
        {statusError && (
          <section className="recovery-notice" role="alert">
            <p>{statusError}</p>
            <button type="button" onClick={() => void refreshLibraryStatus()}>
              Comprobar de nuevo
            </button>
          </section>
        )}
        {error ? (
          <section role="alert">
            <h1>No se pudo abrir la biblioteca</h1>
            <p>{error}</p>
            <p>Vuelve a abrir la aplicación para reintentar.</p>
          </section>
        ) : !info || !library ? (
          <section role="status">
            <h1>Preparando tu biblioteca…</h1>
          </section>
        ) : view.kind === "Library" ? (
          <LibraryPage
            api={api.library}
            readerApi={api.reader}
            onOpenPaper={showReader}
            onLibraryChanged={() => void refreshLibraryStatus()}
          />
        ) : view.kind === "PaperWorkspace" ? (
          opened ? (
            <PaperReader
              key={opened.document.id}
              opened={opened}
              api={api.reader}
              onClose={closeReader}
            />
          ) : (
            <section role="status">
              <p>Preparando el lector…</p>
            </section>
          )
        ) : view.kind === "Home" ? (
          <LibraryHome
            libraryApi={api.library}
            readerApi={api.reader}
            onOpenPaper={showReader}
            onShowLibrary={() => setView({ kind: "Library" })}
          />
        ) : (
          <section>
            <p className="eyebrow">{activeView}</p>
            <h1>
              {info.state === "readOnlyDiagnostic"
                ? "Biblioteca en modo diagnóstico"
                : "Configuración"}
            </h1>
            <article className="card">
              <h2>{library.displayName}</h2>
              <dl>
                <dt>Ubicación</dt>
                <dd>{library.rootLabel}</dd>
                <dt>Identidad</dt>
                <dd>{library.libraryId}</dd>
                <dt>Esquema</dt>
                <dd>{library.schemaVersion}</dd>
                <dt>Estado</dt>
                <dd>
                  {statusError
                    ? "No verificado"
                    : libraryStatus?.recoveryRequired
                      ? "Recuperación necesaria"
                      : libraryStatus?.writable
                        ? "Disponible"
                        : library.writable
                          ? "Estado de recuperación desconocido"
                          : "Solo diagnóstico"}
                </dd>
                <dt>Operaciones</dt>
                <dd>{libraryStatus?.activeOperations ?? "No disponible"}</dd>
              </dl>
              {statusError && <p role="alert">{statusError}</p>}
            </article>
          </section>
        )}
      </main>
      <Dialog
        open={Boolean(library) && !confirmed}
        onOpenChange={(open) => {
          if (!open) setConfirmed(true);
        }}
      >
        <DialogContent>
          <DialogTitle>Esta es tu biblioteca local</DialogTitle>
          <DialogDescription>
            Research Workbench guardará una copia administrada de los PDFs que
            importes. Podrás mover el archivo original sin perder la copia. Los
            datos viven separados de la aplicación.
          </DialogDescription>
          <p>{library?.rootLabel}</p>
          <button className="primary" onClick={() => setConfirmed(true)}>
            Entendido
          </button>
        </DialogContent>
      </Dialog>
    </div>
  );
}
