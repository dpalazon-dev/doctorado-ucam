# Native WebDriver smoke — 2026-10-02

**Outcome: PARTIAL — native WebDriver smoke completed against the frozen T02 copy, with closure and WebView2 profile isolation unverified.** No install, build, product change, VM, or personal-library file was intentionally used.

## Approved artifact gate

Requested artifact: `work/cargo-target/debug/research-workbench.exe`, expected SHA-256 `C3FB989E17B20640FB4C530CC08959CDC839A72E5DF54E5442827F20837924D8` (T02 integration `c4436bd`). A prior same-day preparation copied that matching binary to `work/qa/t02-native-smoke/run-20261002-c5161f4397484521931bbdfd297979ad/research-workbench.exe`; its source and copy both matched the expected hash at that time.

On the next preflight, before creating a new copy or launching anything, the shared executable hash had changed to `F8AF44947DC56FBA3995D8B47A00FC9B7FD1FE46243758C33CD474E44A17FFF4`. Per task gate, did not execute or compile the changed artifact. This historical blocker was resolved when the parent authorized a separately reviewed run from the immutable, hash-verified copy below.

## Earlier attempt evidence (historical, not a passing result)

- First smoke attempt used only the copied executable whose hash matched the approved value. It created synthetic root `work/qa/t02-native-smoke/run-20261002-c5161f4397484521931bbdfd297979ad/synthetic-library`; setup produced the expected synthetic library files. It then failed with EdgeDriver error: `invalid argument: 'args' must be a list (Session info: MicrosoftEdge=154.0.4258.48)`.
- The request builder had sent `arguments: []` to W3C `/execute/sync`; parent review identified that the required key is `args`. The attempt did not save request/response or session-created stage, so session creation versus subsequent command failure cannot be distinguished conclusively.
- A follow-on script copy was prepared with `args: []`, but its stale run-directory guard stopped before starting the driver or app. The next fresh attempt was held at the shared-binary gate; parent then authorized using the already hash-verified immutable copy for a separately reviewed run.
- At the final process check, no `tauri-driver`, `msedgedriver`, or `research-workbench` process was present. No screenshot or successful UI assertion was captured. No close/lock-release result is claimed.

## Environment and limits

Local tools used from ignored `work/tools/`: tauri-driver 2.1.0 (SHA-256 `2684A7B9E1E667D6689BEA8EAFF2AEFE093C5B1F5CA329B3156B384A9CAB9286`) and Microsoft EdgeDriver 154.0.4258.48 (SHA-256 `4032E9D74DA42B30805EC5125EEAF9CBD4F678C2A41104EE90471BE16CAAD997`). In the final session, Edge and WebView2 Runtime were both `154.0.4258.48`.

The final approved run and its limits are documented below. The WebDriver request/response log is retained in the ignored `smoke-result.json` for audit; its screenshot response is large because the driver returned base64 image data.

## Final approved frozen-copy attempt

- Parent authorized using the immutable first-run copy after review of the script. Script `native-smoke.ps1` SHA-256 `7DE976F3B00CCA80F14721EE9F97DAA27073004D4CDB756A3DE4F0943E8A2B6A`. Source and run-copy SHA-256 both matched `C3FB989E17B20640FB4C530CC08959CDC839A72E5DF54E5442827F20837924D8`; the changed shared cache was not used.
- Command: `pwsh -NoProfile -File .\work\qa\t02-native-smoke\reviewed-attempt-20261002-057cfd3016ea4a41b5343cfda4877436\native-smoke.ps1`. Script exit code: 0 (it deliberately returns success for a completed *partial* run). Tauri driver 2.1.0 and EdgeDriver 154.0.4258.48 started; Edge and WebView2 Runtime were both `154.0.4258.48`. W3C session creation and deletion both returned HTTP 200. Eleven request/response records are in `work/qa/t02-native-smoke/reviewed-attempt-20261002-057cfd3016ea4a41b5343cfda4877436/smoke-result.json`.
- Actual native WebView2 document rendered title `Research Workbench` and first-run dialog, then accepted `Entendido` and opened Settings. The Settings content showed `Estado` / `Disponible`. Screenshot: [settings-t02-smoke.png](settings-t02-smoke.png), 116,016 bytes, SHA-256 `CD811AE45FC65F4D16DFB817B87161079FF24FDAE639B31B086503FD8FFD26EE`.
- The synthetic library root and configured WebView2 profile path were under that unique run folder; actual use of that profile path remains unverified. The root contained generated `library.json`, SQLite database and expected synthetic directories. Manifest UUID `6a5cb533-a6ef-44fb-bc6c-aa2d3fed0082` matched the ID displayed in the native UI. The `.writer.lock` file remained; its OS advisory lock release was **not verified**.
- `DELETE /window` returned HTTP 200, but the app process did not yield an observed exit code 0 before `DELETE /session`. The app process was absent afterward; therefore normal window-close behavior is **not confirmed**. Do not count session cleanup as normal app close.
- The run set `WEBVIEW2_USER_DATA_FOLDER` to a new run-local directory, but the directory remained empty and returned WebDriver capabilities showed `msedge.userDataDir` as an empty string. Effective WebView2 user-data isolation is **unverified**. The app-specific candidate `%LOCALAPPDATA%\local.researchworkbench.desktop` existed; its contents were not inspected or cleaned.
- Post-run CIM (`-ErrorAction Stop`) and listener queries (`-ErrorAction Stop`) completed successfully and found no process for the exact copied app, tauri-driver, or EdgeDriver paths and no listeners on the two dynamically selected ports (63242, 63243). Driver cleanup reported its own PID exited and its owned EdgeDriver child absent. No process in the synthetic run remained.

No library-page/picker, installed-app, installer, clean-machine, or PDF-rendering result is claimed.
