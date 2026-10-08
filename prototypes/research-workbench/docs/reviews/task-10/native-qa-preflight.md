# Native Windows QA path (research only)

## Recommendation

Use two clearly labeled layers:

1. Frontend/React tests with mocked `invoke()` for fast renderer coverage. These do **not** prove Rust IPC, WebView2, application packaging, or offline behavior.
2. A small native WebDriver smoke suite against the built Tauri release executable, then one install/launch smoke against the produced NSIS installer on a clean Windows 11 x64 machine. Use the app's synthetic local library and keep network unavailable during the workflow. Native WebDriver controls the actual desktop WebView; it does not by itself prove that installed-app data paths or installer behavior are correct, so retain a separate installer launch check.

Tauri's current WebDriver overview recommends WebdriverIO with `@wdio/tauri-service`. Its `external` driver provider uses `tauri-driver` and the platform native WebDriver; Windows Edge WebDriver must match the Edge/WebView2 runtime used by the app. The service can synchronize EdgeDriver. The embedded provider avoids an external driver but requires its optional Tauri embedded-WebDriver plugin; external provider avoids that plugin. For this Windows-specific QA target, external provider is suitable and keeps the test pointed at the release binary. The service also offers a browser-only mode with mocked `invoke()`; classify that as renderer testing only.

## Suggested small v0.1 native journey

Start from an empty, disposable app data directory and a checked-in, synthetic fixture PDF/library (no private research documents). Build the release executable; run the WebDriver suite on that executable with networking disconnected. Through the UI, import/open the fixture PDF, move through the supported workflow, create a typed knowledge item with a page locator and explicit relation, restart the app, and confirm those records remain. Export/backup and restore into a second empty disposable library; verify the restored record and locator. Capture a screenshot on failure. Assert persisted state through visible UI and a separate supported export, not by reaching inside controls. Run this journey again against an installation created from the actual NSIS artifact, validating first-run empty state, launch, persistence, and no network dependency. Keep the test account/data isolated and discard it after the run.

If no offline PDF fixture is checked in, use a small synthetic PDF created in-repo with stable text and page count. Do not depend on downloaded papers, remote DOI resolution, OCR, or LLM services. Preserve the fixture's source and checksum so expected locator assertions remain reproducible.

## Setup/commands (not run)

Official direct-driver setup uses:

```powershell
cargo install tauri-driver --locked
cargo install --git https://github.com/chippers/msedgedriver-tool
& "$HOME/.cargo/bin/msedgedriver-tool.exe"
```

The resulting `msedgedriver.exe` must be on `PATH`, or pass the `tauri-driver --native-driver` option. Exact invocation may be wrapped by `@wdio/tauri-service`; configure `appBinaryPath` to the release executable and `driverProvider: 'external'`. For installer validation, build the configured NSIS target, install into an isolated clean test account/VM, and point the same suite at the installed executable. Verify actual artifact paths from `tauri.conf.json` before scripting.

Native Windows build prerequisites per Tauri: Microsoft C++ Build Tools with Desktop development with C++, Rust stable MSVC, and WebView2. Node is needed for the React/TypeScript and WebdriverIO toolchain. For MSI only, Tauri separately calls out the Windows VBScript optional feature; the intended NSIS route does not need that MSI-only prerequisite. Tauri docs say EdgeDriver must match the Edge version; mismatch can hang WebDriver startup. They do not prescribe a fixed EdgeDriver version in the test guide.

## Verified host snapshot (read only)

- `node` is present and reports `v24.14.1`; `git` is present.
- `rustc` and `msedgedriver` were not found on `PATH`.
- `WindowsSandbox.exe` was not found; `virtmgmt.msc` exists. `Get-VM` could not enumerate VMs because the Hyper-V cmdlet/module was unavailable in this shell. No VM availability claim is made.
- No installation, VM, feature, security, or product/test/config changes were made for this report. No native UI test was run.

## Current official sources

- [Tauri tests overview](https://v2.tauri.app/develop/tests/) — mock runtime does not execute native webview libraries; distinguishes WebDriver E2E.
- [Tauri WebDriver overview](https://v2.tauri.app/develop/tests/webdriver/) — WDIO service, provider choices, external driver, and browser/mock mode.
- [Tauri manual WebDriver setup](https://v2.tauri.app/develop/tests/webdriver/manual-setup/) — `tauri-driver`, EdgeDriver matching, `msedgedriver-tool`, and PATH/native-driver requirement.
- [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/) — C++ Build Tools, WebView2, Rust MSVC, optional Node frontend tooling.

The docs were checked on 2026-10-01. Versions 154.0.4258.48 (WebView2), Node 24.14.1, and Rust 1.99 were supplied as target expectations in the research request; the official Tauri test pages support the Edge/EdgeDriver matching rule and stable Rust/LTS Node guidance but do not state those exact pinned versions. Recheck runtime and toolchain versions when provisioning the actual clean Win11 x64 test machine.

## Preflight update — 2026-10-02 (read only)

### Current host evidence

- Microsoft Edge executable and the WebView2 Evergreen Runtime registry entry both report **154.0.4258.48**. Microsoft says the first three version components must match. The compatible x64 EdgeDriver line is therefore **154.0.4258.***; choose the x64 download from Microsoft's official Edge WebDriver page and verify its reported version before use. This does not prove a driver is installed or that a test ran.
- `tauri-driver.exe` and `msedgedriver.exe` were not present on PATH or `%USERPROFILE%\.cargo\bin`. Cargo/rustc are present in that Cargo bin but absent from this shell's PATH: `cargo 1.99.0`, `rustc 1.99.0`, stable MSVC toolchain. Node remains `v24.14.1`.
- `WindowsSandbox.exe` is absent; `virtmgmt.msc` exists; the Hyper-V cmdlet exists, but `Get-VM` failed with `Value cannot be null (Parameter 'name')`. Windows optional-feature state queries could not be read in this shell. `Win32_ComputerSystem.HypervisorPresent` is true. VM availability and Sandbox feature state remain **unknown**; nothing was enabled, restarted, installed, or created.
- No native UI test was run. The only UI automation exposed in this task is browser CUA; it cannot operate the Windows desktop picker.

### Concrete native route and picker boundary

For future automated native smoke, build a **debug-assertions** Tauri executable and configure WebdriverIO against that binary with `@wdio/tauri-service`, a single worker, and its current configuration's explicit `driverProvider: 'official'` (cargo-installed `tauri-driver`). Tauri's v2 guide calls the same provider `external`; recent WDIO docs use `official` and mark `external`/`official` naming as having changed, so use the spelling supported by the pinned service version. Enable its EdgeDriver auto-download or supply the Microsoft x64 driver line matching 154.0.4258. Keep `maxInstances: 1` because this application permits only one library writer. The service launches the actual binary and connects to its WebView2 document; its browser mode and invoke mocks are renderer-only. For offline testing, block networking for the app/test run and set Microsoft EdgeDriver's documented process-level telemetry opt-out, `MSEDGEDRIVER_TELEMETRY_OPTOUT=1`.

The installed Tauri dialog plugin presents a **native system file selector**. WebDriver here automates the app's web document; WDIO `uploadFile` sets a web `<input type=file>` and does not prove a native selector choice. Therefore native picker opening, navigation, cancellation, and choosing an allowed synthetic PDF need a separate authorized manual Windows UI check (or a separately provisioned Windows UI Automation driver). Do not mock the picker or count a supplied path/invoke mock as native-picker success. Appium's Windows driver is a possible separate UIA route for Win32 windows, but its current README says Windows 10 host and Developer Mode prerequisites and warns its Microsoft WinAppDriver server is unmaintained; it is not a ready Win11 clean-machine assumption. The picker remains an explicit manual limitation until actually exercised.

### Isolated debug data root

The read-only source inspected in `.worktrees/task-01-scaffold/src-tauri/src/adapters/windows/paths.rs` confirms a debug-only override: `RESEARCH_WORKBENCH_TEST_ROOT`; it must be an absolute path. In debug builds, that path is the library root. Release builds ignore this override and use `%LOCALAPPDATA%\ResearchWorkbench\library`. To launch a future debug binary without touching the personal library, create a new unique empty synthetic root under `%TEMP%`, set this variable only in the PowerShell process that launches the executable or WDIO, and remove the variable in a `finally` block. Do not set it globally, reuse the personal library, or set `LOCALAPPDATA` to redirect the whole user profile. Example shell pattern (not executed):

```powershell
$qaRoot = Join-Path $env:TEMP ('rw-native-' + [guid]::NewGuid().ToString('N'))
if (Test-Path -LiteralPath $qaRoot) { throw 'QA root must be new' }
New-Item -ItemType Directory -Path $qaRoot | Out-Null
$env:RESEARCH_WORKBENCH_TEST_ROOT = $qaRoot
$env:MSEDGEDRIVER_TELEMETRY_OPTOUT = '1'
try {
  & .\src-tauri\target\debug\research_workbench.exe
} finally {
  Remove-Item Env:RESEARCH_WORKBENCH_TEST_ROOT -ErrorAction SilentlyContinue
  Remove-Item Env:MSEDGEDRIVER_TELEMETRY_OPTOUT -ErrorAction SilentlyContinue
}
```

Replace the executable path with the actual artifact name and run from the task worktree. The same process-scoped variables must be set around the WDIO command if the service is spawning the app. The T01 path helper is evidence for the override's existence, not proof that T02's final binary has retained it; recheck its final source/build before relying on this QA isolation. Never point that variable at a personal library.

### Current sources checked 2026-10-02

- [Tauri tests](https://v2.tauri.app/develop/tests/) and [Tauri WebDriver](https://v2.tauri.app/develop/tests/webdriver/) — mock runtime excludes native webview execution; native WebDriver and provider distinctions.
- [Tauri manual WebDriver setup](https://v2.tauri.app/develop/tests/webdriver/manual-setup/) — matching EdgeDriver requirement and direct-driver setup.
- [Microsoft Edge WebDriver guide](https://learn.microsoft.com/en-us/microsoft-edge/webdriver/) — first three version components match; driver download process; EdgeDriver telemetry opt-out.
- [WebdriverIO Tauri service](https://webdriver.io/docs/wdio-tauri-service/) and [configuration](https://webdriver.io/docs/desktop-testing/tauri/configuration/) — Windows provider names, actual executable launch, auto driver management, and service limits.
- [WebdriverIO uploadFile](https://webdriver.io/docs/api/browser/uploadFile/) — file upload is applied to a DOM input; [Tauri dialog plugin](https://v2.tauri.app/plugin/dialog/) documents native system dialogs and returned Windows filesystem paths.
- [Appium Windows Driver](https://github.com/appium/appium-windows-driver) — separate native UIA possibility and documented host/developer-mode/server constraints.

No native picker result, installed-app validation, clean-VM run, or product behavior is claimed.
