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
        "Could not check the library recovery status.",
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
              : "Could not open the library.",
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
      ? "Reader"
      : view.kind === "Library"
        ? "Library"
        : view.kind === "Settings"
          ? "Settings"
          : "Home";
  return (
    <div className="app-shell">
      <aside aria-label="Navigation">
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
            Home
          </button>
          <button
            type="button"
            aria-current={view.kind === "Library" ? "page" : undefined}
            onClick={() => setView({ kind: "Library" })}
          >
            Library
          </button>
          <button
            type="button"
            disabled
            aria-label="Knowledge, coming soon"
          >
            Knowledge · coming soon
          </button>
          <button
            type="button"
            aria-current={view.kind === "Settings" ? "page" : undefined}
            onClick={showSettings}
          >
            Settings
          </button>
        </nav>
        <small>Local · offline</small>
      </aside>
      <main>
        <header>
          <span>Your research workspace</span>
          <span>{info ? "v" + info.appVersion : "Starting…"}</span>
        </header>
        {libraryStatus?.recoveryRequired && (
          <section className="recovery-notice" role="alert">
            <h2>The library needs recovery</h2>
            <p>
              An import operation is pending or needs attention. Some actions
              may be blocked until it is resolved.
            </p>
            <button type="button" onClick={() => void refreshLibraryStatus()}>
              Check recovery status
            </button>
          </section>
        )}
        {statusError && (
          <section className="recovery-notice" role="alert">
            <p>{statusError}</p>
            <button type="button" onClick={() => void refreshLibraryStatus()}>
              Check again
            </button>
          </section>
        )}
        {error ? (
          <section role="alert">
            <h1>Could not open the library</h1>
            <p>{error}</p>
            <p>Reopen the app to try again.</p>
          </section>
        ) : !info || !library ? (
          <section role="status">
            <h1>Preparing your library…</h1>
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
              <p>Preparing the reader…</p>
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
                ? "Library in diagnostic mode"
                : "Settings"}
            </h1>
            <article className="card">
              <h2>{library.displayName}</h2>
              <dl>
                <dt>Location</dt>
                <dd>{library.rootLabel}</dd>
                <dt>Identity</dt>
                <dd>{library.libraryId}</dd>
                <dt>Schema</dt>
                <dd>{library.schemaVersion}</dd>
                <dt>Status</dt>
                <dd>
                  {statusError
                    ? "Unverified"
                    : libraryStatus?.recoveryRequired
                      ? "Recovery required"
                      : libraryStatus?.writable
                        ? "Available"
                        : library.writable
                          ? "Recovery status unknown"
                          : "Diagnostic only"}
                </dd>
                <dt>Operations</dt>
                <dd>{libraryStatus?.activeOperations ?? "Unavailable"}</dd>
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
          <DialogTitle>This is your local library</DialogTitle>
          <DialogDescription>
            Research Workbench will keep a managed copy of every PDF you
            import. You can move the original file without losing the copy. Your
            data is stored separately from the app.
          </DialogDescription>
          <p>{library?.rootLabel}</p>
          <button className="primary" onClick={() => setConfirmed(true)}>
            Got it
          </button>
        </DialogContent>
      </Dialog>
    </div>
  );
}
