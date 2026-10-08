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

## Project execution notes from the orchestrator
Rust 1.99 MSVC is installed and verified by compilation; its absence from PATH is intentional (`--no-modify-path`), not an absent prerequisite. See ENVIRONMENT.md for absolute paths and the inherited MSVC environment. Driver commands above are upstream setup examples, not commands already run. This project will use project-local QA tools and an explicitly assigned task; no global driver/tool install is implied by this research report. A clean Windows VM has not been demonstrated available. Installer QA remains a separate acceptance gate, and no VM/security feature has been changed.
