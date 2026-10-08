# Research Workbench

> **Partial, paused prototype.** This project is published as a reusable reference and starting point for forks. It has no active maintenance because the author currently has no time to continue it. It is an independent Windows 11 x64 prototype, not an implementation of the ResearchOS specifications. See [snapshot provenance and limitations](../../docs/publication/README.md).

A local desktop application for reading papers and turning them into structured knowledge with explicit provenance. Windows 11 x64 · Tauri 2 · React/TypeScript · Rust · SQLite.

The execution status preserves evidence from local development. See [status](docs/STATUS.md), [intent](INTENT.md), [architecture](docs/architecture/README.md) and [implementation plan](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/plans/IMPLEMENTATION.md).

## Development

This directory contains the prototype project for inspection and forks. Development on Windows x64 requires Node.js/npm, Rust with the MSVC target, Visual Studio C++ Build Tools and the Windows SDK, and the WebView2 Runtime. From PowerShell in this directory:

```powershell
npm.cmd ci
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
. .\scripts\development-env.ps1
npm.cmd run tauri:dev
```

Use synthetic libraries for checks and debug development. [The environment guide](docs/development/ENVIRONMENT.md) records toolchain details and validation limits.

## Snapshot sources

The `docs/session/` directory and auxiliary working materials were excluded from publication. The originals remain in the local source repository; they are not included in this snapshot. The original editable product specification is available in [docs/PRODUCT.md](docs/PRODUCT.md).

## Language

Current documentation, source comments and interface copy use English. Immutable versioned definition snapshots and original source quotations retain their original data values so their hashes, saved answers and historical evidence remain valid. Presentation text is separate from those identities.
