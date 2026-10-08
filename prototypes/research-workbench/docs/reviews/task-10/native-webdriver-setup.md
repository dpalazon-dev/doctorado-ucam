# Native WebDriver local tools setup

**Date:** 2026-10-02
**Scope:** Local QA tools only for later T03/T10. No product code, harness, tests, global PATH/config, registry, Windows feature, VM, installer, application launch, or personal library was changed or used. No driver service was left running.

## Verified runtime and selected versions

- Edge executable and WebView2 Evergreen Runtime registry `pv`: **154.0.4258.48**.
- Tauri's manual WebDriver page requires a matching EdgeDriver. Microsoft specifies that Edge and EdgeDriver must match in their first three version components. Chose the exact x64 build **154.0.4258.48**, obtained from Microsoft's official distribution endpoint.
- Selected latest stable Tauri driver **2.1.0** from Tauri's official release changelog (2026-09-26), not the prerelease returned by an unqualified `cargo info` query. Rust stable MSVC `1.99.0` meets its documented MSRV 1.90.

## Installed local artifacts

| Tool | Version / status | Location |
|---|---|---|
| tauri-driver | 2.1.0 installed with `--locked`; `--help` passes. This binary does not implement `--version` (that command exits 1 with “unused arguments”). | `work/tools/tauri-driver/bin/tauri-driver.exe` |
| Microsoft Edge WebDriver | 154.0.4258.48 x64; `--version` exits 0. Authenticode status Valid, signer Microsoft Corporation. | `work/tools/msedgedriver/msedgedriver.exe` |
| Cargo home/cache | Dedicated project-local home, used by this install. | `work/tools/cargo-home` |
| Rust compilation target cache | Dedicated project-local `CARGO_TARGET_DIR`. | `work/tools/build-cache` |

The versions above describe tool preparation only. No native app, WebDriver session, E2E journey, file picker, installer, or clean-machine behavior was exercised.

## Commands and outcomes

Read-only version checks before installation:

```powershell
node --version
# v24.14.1

Get-Item 'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe' |
  Select-Object -ExpandProperty VersionInfo | Select-Object ProductVersion
# 154.0.4258.48

# Read-only lookup under HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients
# Matching entry: WebView2 Runtime de Microsoft Edge; pv=154.0.4258.48
```

Pinned cargo installation (exit **0**):

```powershell
$repo = (Get-Location).Path
$env:CARGO_HOME = Join-Path $repo 'work/tools/cargo-home'
$env:CARGO_TARGET_DIR = Join-Path $repo 'work/tools/build-cache'
& "$env:USERPROFILE\.cargo\bin\cargo.exe" install tauri-driver `
  --version 2.1.0 --locked `
  --root (Join-Path $repo 'work/tools/tauri-driver')
```

The command output identified `tauri-driver v2.1.0` and installed the executable under the specified root. These environment variables were process-scoped to the install command; no global Cargo configuration or PATH was edited. The Rust 1.99 toolchain already present under the user's Cargo/rustup location was reused, not installed or modified.

Microsoft EdgeDriver download and extraction (exit **0**):

```powershell
$url = 'https://msedgedriver.microsoft.com/154.0.4258.48/edgedriver_win64.zip'
$dest = Join-Path (Get-Location).Path 'work/tools/msedgedriver'
New-Item -ItemType Directory -Path $dest | Out-Null
$zip = Join-Path $dest 'edgedriver_win64_154.0.4258.48.zip'
Invoke-WebRequest -Uri $url -OutFile $zip
Get-FileHash -LiteralPath $zip -Algorithm SHA256
Expand-Archive -LiteralPath $zip -DestinationPath $dest
```

Endpoint HEAD check returned HTTP **200**, `application/x-zip-compressed`, length **11,652,157** bytes. SHA-256 values:

```text
4796F46BA79F6297E6F43AD7410BBED22139D82D49E4FC607B2F4B8892EB41E5  work/tools/msedgedriver/edgedriver_win64_154.0.4258.48.zip
4032E9D74DA42B30805EC5125EEAF9CBD4F678C2A41104EE90471BE16CAAD997  work/tools/msedgedriver/msedgedriver.exe
2684A7B9E1E667D6689BEA8EAFF2AEFE093C5B1F5CA329B3156B384A9CAB9286  work/tools/tauri-driver/bin/tauri-driver.exe
```

Verification results:

```powershell
& '.\work\tools\tauri-driver\bin\tauri-driver.exe' --help
# exit 0; usage lists --native-driver PATH, --port NUMBER, --native-port NUMBER

& '.\work\tools\tauri-driver\bin\tauri-driver.exe' --version
# exit 1; this CLI does not accept --version. Version evidence is the pinned Cargo install output.

& '.\work\tools\msedgedriver\msedgedriver.exe' --version
# exit 0: Microsoft Edge WebDriver 154.0.4258.48 (80bb122d4912059dc0cf309273bbe3191ae80a62)

Get-AuthenticodeSignature '.\work\tools\msedgedriver\msedgedriver.exe'
# Status Valid; CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US
```

## Future command (not run)

Start the intermediary only when the later native suite is ready; this command runs a persistent WebDriver server until stopped. Do not run it as part of preflight:

```powershell
& 'C:\Users\david\Projects\Research-Workbench\work\tools\tauri-driver\bin\tauri-driver.exe' `
  --port 4444 `
  --native-port 4445 `
  --native-driver 'C:\Users\david\Projects\Research-Workbench\work\tools\msedgedriver\msedgedriver.exe'
```

For WebdriverIO, pass the same local binary paths in the Tauri service configuration, pin its `driverProvider` spelling to the selected service version (`official` in current WDIO docs; Tauri's older v2 guide calls this `external`), and use one worker. A browser-mode run with mocked `invoke()` is not a native result. Keep any future app launch pointed at a fresh synthetic debug root; the T01 debug-only `RESEARCH_WORKBENCH_TEST_ROOT` still requires confirmation in the final T02 source before relying on it.

## Sources

- [Tauri WebDriver guide](https://v2.tauri.app/develop/tests/webdriver/) and [manual setup](https://v2.tauri.app/develop/tests/webdriver/manual-setup/) — `tauri-driver`, Windows native driver and matching EdgeDriver.
- [Tauri driver changelog](https://v2.tauri.app/release/tauri-driver/all-versions/) — stable 2.1.0 release dated 2026-09-26.
- [Microsoft Edge WebDriver official downloads](https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/) and [Microsoft compatibility instructions](https://learn.microsoft.com/en-us/microsoft-edge/webdriver/) — version matching and download channel. The version page lists 154.0.4258.48 and the direct download used above is on Microsoft's `msedgedriver.microsoft.com` domain.
- [WebdriverIO Tauri configuration](https://webdriver.io/docs/desktop-testing/tauri/configuration/) — current provider naming and path options.
