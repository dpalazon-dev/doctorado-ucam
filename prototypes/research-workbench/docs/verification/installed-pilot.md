# Installed verification — pilot 0.0.1

Preparation date: 2026-10-03. This historical record distinguishes package preparation from QA on an installed application. It is not a new English-interface test run; see the current [status](../STATUS.md).

## Artifact and environment

| Field | State |
| --- | --- |
| Setup commit and SHA-256 | Pending at preparation: Sol would build after review of the clean commit containing the script. |
| Clean Windows 11 x64 / VM available | Pending; VM/Sandbox availability was not established. |
| WebView2 initially absent | Pending. |
| Network disconnected | Pending. |
| Node/npm/Rust/Cargo/Git/Codex absent | Pending. |
| Source checkout absent and execution cwd recorded | Pending. |
| Isolated synthetic library | Pending; the release executable had not been launched or installed in the working profile. |

## Separate results

| Gate | State | Evidence at preparation |
| --- | --- | --- |
| Script behavior checks | Verified: exit 0 on 2026-10-03. | `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/tests/build-release.tests.ps1`; output `build-release behavior checks passed`. Covers LF/CRLF, wrong host, installed `rustc` probe, x64 app PE and NSIS setup with i386 stub. |
| NSIS bundle generated | Pending; reserved for Sol's build after review. | The release command had not run. |
| App executable architecture | Pending inspection of Tauri-produced `research-workbench.exe`. | The script requires PE `x64 (0x8664)` and records SHA-256/bytes separately. |
| NSIS setup PE machine | Pending inspection of the actual artifact. | The script accepts/records the compatible stub's actual machine (`i386` or `x64`); this is not proof of app architecture. |
| Offline WebView2 payload | Pending inspection of generated NSIS and clean disconnected testing. | `offlineInstaller` is configuration, not evidence of payload or offline installation. |
| PDF.js resources inside the product | Pending inspection/installed reading. | T03 configures local worker, cmaps, standard fonts, ICC and WASM in the frontend. |
| Start-menu launch / execution without source or server | Pending. | The release application had not run. |
| Import, move original, page 2, close/reopen, archive/restore | Pending installed QA. | No installer or personal data had been used. |
| Second instance / single writer | Pending installed QA. | Release Busy notification had not been checked. |
| Upgrade with actual migration | Pending; no compatible earlier version was established for this pilot. | Reinstalling 0.0.1 does not count as migration. |
| Uninstall/reinstall preserving library/backups | Pending installed QA. | Neither installer nor uninstaller had run. |

## Prepared commands

Once included in a clean commit, the script fixes the platform explicitly and saves the candidate under `dist-release/0.0.1/`:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/build-release.ps1
```

Focused script behavior test:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/tests/build-release.tests.ps1
```

This report does not accept the pilot installation gate. Only evidence for the exact setup on a clean/offline system can close it.
