# ResearchOS and Research Workbench

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 4 — Navigation, Delivery and Process |
| **Normative status** | Repository front door; non-authoritative for system behavior |
| **Authoritative for** | Concise repository identity, current documentation map, links and current high-level status. |
| **Not authoritative for** | Product requirements, architectural decisions, domain semantics, implementation contracts or phase details. |
| **Required reading** | `VISION.md`, `DOCUMENTATION_ARCHITECTURE.md`, `ROADMAP.md`, `SPEC_CATALOG.md`. |
| **Downstream documents** | New-contributor orientation and repository navigation. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

This repository brings together the **ResearchOS design**, a vision for a personal knowledge-work environment, and a **partial Research Workbench prototype** that can serve as a reference or a starting point for forks. Development of both projects is paused because the author currently has no time to maintain them. Publication does not imply an ongoing support commitment.

## Two scopes

- **ResearchOS**, at the repository root, contains the canonical architecture and specification catalog for a future cross-platform application. There is no production implementation of ResearchOS here.
- **Research Workbench**, under [`prototypes/research-workbench/`](prototypes/research-workbench/), is a local Windows 11 x64 application built with Tauri, React/TypeScript, Rust and SQLite. It is an independent, incomplete implementation: it does not implement ResearchOS specifications, share its canonical model, or include AI integration.

The ResearchOS design includes optional Python cognitive processing. That capability is not part of the Workbench prototype. See [`docs/publication/`](docs/publication/) for the snapshot's sources and limits.

## Workbench prototype

Start at [`prototypes/research-workbench/`](prototypes/research-workbench/) for its README, status, historical intent and architecture. Development requires Windows x64, Node.js/npm, Rust MSVC, Visual Studio C++ Build Tools with the Windows SDK, and the WebView2 Runtime. From PowerShell in the prototype directory:

```powershell
npm.cmd ci
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
. .\scripts\development-env.ps1
npm.cmd run tauri:dev
```

Tests use synthetic data. These development commands do not establish clean-machine installation or completion of v0.1.

## ResearchOS

The architecture and design status are described in [VISION.md](VISION.md), [DOCUMENTATION_ARCHITECTURE.md](DOCUMENTATION_ARCHITECTURE.md), [ROADMAP.md](ROADMAP.md) and [SPEC_CATALOG.md](SPEC_CATALOG.md). This README provides navigation; those documents retain authority over their declared concerns.

## Repository language

Repository prose, source comments, help text and the prototype interface use English. Technical identifiers, paths, original bibliographic names and immutable serialized historical snapshots retain their original values. Translating presentation does not authorize changing persisted identities, definition hashes or recorded evidence.

## License

Original material is available under [MIT](LICENSE). Third-party components retain their own licenses and notices under [`prototypes/research-workbench/`](prototypes/research-workbench/). See [`docs/publication/`](docs/publication/) for snapshot provenance.
