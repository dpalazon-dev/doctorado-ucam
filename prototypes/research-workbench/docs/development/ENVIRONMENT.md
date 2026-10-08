# Windows developer environment

Prerequisites checked on 2026-10-01 on this Windows workstation. Foundation integration checked on 2026-10-02; the commands and evidence below distinguish native debug construction from installer validation.

## Installed and verified

The v0.1 library data directory must be on local NTFS (Windows x64); remote/UNC and other filesystems fail closed under [ADR-016 directory guards](WINDOWS_DIRECTORY_GUARDS.md). This is a product storage constraint, not a restriction on selecting an original PDF. The experimental guard probe ran on Windows10.0.26200 / C:NTFS; Rust implementation and installed-app verification remain separate gates.

| Tool or prerequisite | Observed version / evidence |
| --- | --- |
| Git for Windows | `git version 2.43.0.windows.1` |
| Node.js | `v24.14.1` |
| npm | `11.11.0` |
| Codex CLI | `codex-cli 0.159.2` |
| Visual Studio Community 2022 | `17.9.2` at `C:\Program Files\Microsoft Visual Studio\2022\Community` |
| MSVC x64 compiler and linker | MSVC toolset `14.39.33519`; `cl.exe` and `link.exe` exist under `VC\Tools\MSVC\14.39.33519\bin\Hostx64\x64` |
| Windows SDK | `10.0.22621.0` installed under `C:\Program Files (x86)\Windows Kits\10` |
| Microsoft Edge WebView2 Runtime | `154.0.4258.48` registered and installed under `C:\Program Files (x86)\Microsoft\EdgeWebView\Application` |
| Rust stable MSVC toolchain | `rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo 1.99.0 (5f94df478 2026-08-27)`, host `x86_64-pc-windows-msvc` |
| Rust components | `rustfmt 1.10.0-stable`, `clippy 0.1.99` |

Rust was installed using the official x64 MSVC rustup executable from `https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe`. Its `.sha256` companion was downloaded from the same URL with `.sha256` appended. SHA-256 verified before execution: `6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e`. The installer selected the minimal stable profile with `rustfmt` and `clippy`, and `--no-modify-path`. It uses the existing user rustup/cargo directories (`C:\Users\david\.rustup` and `C:\Users\david\.cargo`) and did not add Cargo to PATH.

A throwaway Rust binary was compiled and run successfully using Cargo and Visual Studio's x64 compiler/linker. Its source, manifest, and build artifacts are in ignored `work/toolchain/smoke/`. The downloaded installer and checksum are in ignored `work/toolchain/` as well.

## Reproducible developer shell

Open a PowerShell child that inherits the x64 Visual Studio developer environment with:

```powershell
cmd.exe /c 'call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" && powershell.exe -NoExit'
```

Run these PowerShell commands in that child shell using absolute paths (so the user PATH stays untouched):

```powershell
& 'C:\Users\david\.cargo\bin\rustc.exe' --version
& 'C:\Users\david\.cargo\bin\cargo.exe' --version
& 'C:\Users\david\.cargo\bin\cargo.exe' fmt --all -- --check
& 'C:\Users\david\.cargo\bin\cargo.exe' clippy --workspace --all-targets -- -D warnings
```

The final two commands are project checks to run when the Rust workspace is ready; they were not run as part of this environment setup. `vcvars64.bat` supplies MSVC and Windows SDK environment variables to that developer shell.

## Project commands verified after scaffold integration

The product is currently developed in linked worktrees; main holds integrated milestones and documentation. The following commands passed on the prepared T01 integration that became commit58585fe. They use lockfiles and the process-local helper, without changing global PATH or Rust defaults:

```powershell
Set-Location 'C:\Users\david\Projects\Research-Workbench\.worktrees\integration'
npm.cmd ci
. .\scripts\development-env.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
npm.cmd run tauri:build -- --debug --no-bundle
```

The helper supplies Cargo/MSVC/SDK to the current process and uses the repository cache at `C:\Users\david\Projects\Research-Workbench\work\cargo-target`. The project pins Rust1.99.0; package-lock.json and Cargo.lock fix the actual dependency graph. The full gate runs TypeScript checking, frontend tests/build, Rust formatting/Clippy/tests and generated-contract drift checking, preserving failure exit codes. Evidence and counts are in [T01 integration](../reviews/task-01/INTEGRATION.md).

For interactive development, load that same helper and set a separate synthetic library before `npm.cmd run tauri:dev`:

```powershell
$env:RESEARCH_WORKBENCH_TEST_ROOT = Join-Path (Get-Location) 'work/manual-smoke/library'
npm.cmd run tauri:dev
```

This override is debug-only; it does not redirect a release build. The verified native debug binary is produced at the shared cache's `debug\research-workbench.exe`. The native smoke used its own synthetic library; no personal library was used for validation.

## Packaging command and remaining validation

T10-pilot uses `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/build-release.ps1` from a clean checkout. It loads the process-local developer environment and invokes `npm.cmd run tauri:build -- --target x86_64-pc-windows-msvc --bundles nsis --ci` into a unique Cargo target directory. It freezes the setup and manifest under `dist-release/0.0.1/`, refuses an existing release directory, and records the source commit, locks and toolchain. The application PE is x64; the NSIS stub can be i386. The debug no-bundle command does not exercise this packaging path.

On 2026-10-03, the build from `510edb1` generated a 222,023,458-byte installer after a first attempt failed downloading NSIS with HTTP503. The failed attempt wrote no success manifest. The successful setup SHA256 is `03682f9d0a63b70d525c61ff2f21eb67c2cca1981bf79a9648f8bd099dc50bb9`. Static inspection and installed acceptance are recorded separately. Tauri patches the installed executable's bundle marker; its extracted payload hash is recorded separately from the restored build-output executable hash.

T02 dependencies and later feature checks are reported separately. Native picker behavior, offline installed operation on a clean Windows system, upgrade, uninstall/reinstall and final release remain pending. A development machine with WebView2/SDKs installed does not establish those results.

## Local native QA tools prepared on 2026-10-02

`work/tools/tauri-driver/bin/tauri-driver.exe` is pinned to tauri-driver2.1.0 using a project-local Cargo install root/home/build cache. `work/tools/msedgedriver/msedgedriver.exe` is the official Microsoft-signed x64 driver154.0.4258.48, matching the observed Edge/WebView2 runtime. Recheck that match before a later native run because Evergreen can update. No global PATH or configuration was changed. tauri-driver supports `--help`, not `--version`; its pinned Cargo installation record supplies version evidence.

Commands, hashes and primary sources are in [the setup report](../reviews/task-10/native-webdriver-setup.md). The executables are available; no driver server, native WebDriver session, app journey, picker or installer was exercised by that preparation. Use a fresh synthetic debug library for later app QA; this does not replace clean-machine installation acceptance.

## Official Rust references

- [Install Rust](https://rust-lang.org/tools/install/)
- [rustup installation on Windows](https://rust-lang.github.io/rustup/installation/windows.html)

