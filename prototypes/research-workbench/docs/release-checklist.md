# Release checklist

## Pilot 0.0.1

- [ ] Build from a clean commit with explicit `x86_64-pc-windows-msvc`.
- [ ] `manifest.json` identifies commit, lockfiles, tools, command, setup source path and frozen artifact SHA-256.
- [ ] Target `research-workbench.exe` is x64; record name, architecture and SHA-256. Read/record the NSIS stub's actual PE machine separately (`i386` or `x64` allowed) and the frozen setup SHA-256.
- [ ] Review generated `.nsi`: current-user installation, icon, Start menu, Installed Apps entry and uninstaller; rule out recursive removal of user data or unrelated paths.
- [ ] Confirm installer contains the offline WebView2 payload and PDF.js resources served from `dist` (worker, cmaps, standard fonts, required ICC and WASM).
- [ ] On clean Windows 11 x64 without Node/npm/Rust/Cargo/Git/Codex/WebView2 or network, install and launch from Start. Record source-checkout presence and execution working directory.
- [ ] Import a synthetic PDF; move the original; read page 2; close and confirm process exit; reopen and check position; archive and restore.
- [ ] Open a second instance and check `Busy` notification/state while the first holds the writer.
- [ ] Upgrade from an earlier version: mark only if an actual migration and source fixture exist. Reinstalling 0.0.1 does not prove migration.
- [ ] Uninstall and verify app/shortcuts removed and library/backups retained; reinstall and reopen that library.
- [ ] Record each step as verified, failed or pending with evidence, fixture and environment. Without a clean/offline VM, the result is a candidate with installation pending.

## Pilot limits

- No Authenticode signature or integrated updater.
- Migration, induced rollback, future-schema downgrade and failure recovery remain pending without independent tests on the exact artifact.
- The pilot establishes no planned v0.1/P3/P4, AI, OCR, merging or permanent-deletion capability.
