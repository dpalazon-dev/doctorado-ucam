# Publication and provenance

## Scope

This repository brings together two related but independent scopes: the future ResearchOS architecture at the root, and a snapshot of the local Research Workbench prototype under [`../../prototypes/research-workbench/`](../../prototypes/research-workbench/). The snapshot does not implement ResearchOS specifications, share its canonical model, or include its cognitive sidecar. Repository consolidation does not change product contracts or make the prototype an architectural authority for ResearchOS.

## Workbench provenance

The original product snapshot was imported from integrated milestone `702d08b02f2d2655584a932455ac394fb026f8d2` on `integration/v0.1`, including the reader and PRE/P1 backend. The pending PRE/P1 workflow UI is not part of the active product; its exact original patch remains available through the pinned historical link in [`pending-work/`](pending-work/README.md) and remains unapplied. Added or modified documents came from `6e20428fe8267dddac3f4a8adca41a158314fd05`; design proposals came from `1f598cbe0bb36d5b0fd8f65c530337a7b36378e6`.

The directory was imported as a file snapshot rather than as Workbench Git history. The complete local source history remains in a verified local bundle outside this publication. This repository preserves ResearchOS history. The original local source repository and its pending work remain intact.

The import excluded `.codex`, `docs/session`, installers, releases, personal libraries, caches, `node_modules`, `target` and `work`. [import-manifest.json](import-manifest.json) records the exact original import; it is historical provenance, not a checksum inventory of later English copy changes. [VERIFICATION.md](VERIFICATION.md) records the original consolidation checks.

## Development

From [`../../prototypes/research-workbench/`](../../prototypes/research-workbench/) on Windows 11 x64:

```powershell
npm.cmd ci
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
. .\scripts\development-env.ps1
npm.cmd run tauri:dev
```

Development requires Node.js/npm, Rust MSVC, Visual Studio C++ Build Tools with the Windows SDK, and the WebView2 Runtime. Tests use synthetic data. These development commands do not establish clean-machine installation or completion of v0.1. See the [prototype README](../../prototypes/research-workbench/README.md), [prototype status](../../prototypes/research-workbench/docs/STATUS.md) and [environment guide](../../prototypes/research-workbench/docs/development/ENVIRONMENT.md).

## Repository language and historical data

Current authored documentation, help text, source comments and interface copy use English. Immutable v1 workflow definition snapshots, original bibliographic names, technical identifiers and exact historical observations retain their original data values. Their preservation protects definition hashes, saved answer snapshots and provenance; they are not a second set of current-language documentation. Git history is preserved rather than rewritten.

Historical screenshots of the former Spanish interface are not presented as English UI evidence. Their originals remain recoverable from the previous repository revision and local source backup. The vector design mockup is an English proposal, not evidence of an implemented feature or a native test run.

## Licenses

The root MIT license covers original material. Dependencies and other third-party components retain their own licenses and notices; see [THIRD_PARTY_NOTICES.md](../../prototypes/research-workbench/THIRD_PARTY_NOTICES.md).
