# Research Workbench — Architecture Baseline Before Implementation

Design version 0.2 · 1 October 2026 · **Execution baseline accepted by the user's instruction of 1 October 2026.**

This package defines the architecture of the installable Windows application and the functional scope of v0.1, before creation of the product repository. It does not certify that code, performance, installation, or recovery have been implemented or tested. The agreed development method remains Sol as coordinator and Luna agents with bounded responsibilities.

## Intended Outcome

A local application for one researcher: install it, open it, import a paper, read its PDF, process it from PRE through P2, and retain typed knowledge with global concepts, provenance, search, export, and restorable backups. Agents are development tools; the initial application does not run agents or LLMs.

The 0.0.1 pilot delivers a library and reader **with an installer**. v0.1.0 adds the complete PRE → P2 workflow and operational safety capabilities. P3/P4, AI, synchronization, concept merging, and permanent deletion are reserved for new SPECs before implementation.

## Normative Documents and Reading Order

| Document | Question it answers |
|---|---|
| [ARCHITECTURE.md](ARCHITECTURE.md) | How is the program organized, and which frameworks, dependencies, patterns, and execution model does it use? |
| [DOMAIN.md](DOMAIN.md) | What do the entities, states, boundaries, and knowledge invariants mean? |
| [CONTRACTS.md](CONTRACTS.md) | How do capabilities and services communicate, including inputs, outputs, errors, and consistency? |
| [DATA.md](DATA.md) | What persistent model, constraints, transactions, files, migrations, and recovery mechanisms are used? |
| [SPECS.md](SPECS.md) | What behavior and acceptance criteria apply to each capability? |
| [ADRS.md](ADRS.md) | What decisions were made, what alternatives and consequences exist, and why might a decision be revisited? |
| [QUALITY.md](QUALITY.md) | What non-functional requirements, verification, traceability, and pre-implementation conditions apply? |

The original product specification is preserved as a [DOCX](../session/outputs/Research_Workbench_Arquitectura_y_Especificacion_v0.1.docx). The [historical multi-agent plan](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/plans/HISTORICAL_MULTIAGENT_PLAN.md) organizes execution; its abbreviated signatures are historical context, not a second authority for the contracts.

The three `phase-definitions/*.v1.json` files are immutable, versioned source payloads. Their Spanish strings and definition hashes are preserved as historical data for compatibility, not as current documentation or UI copy. Current authored documentation and UI use English. The translation policy and compatibility boundary are described in [WORKFLOW_GATES.md](WORKFLOW_GATES.md).

## Authority and Changes from the Earlier Design

Each topic has one authoritative document: semantics in DOMAIN, interfaces in CONTRACTS, storage in DATA, observable behavior in SPECS, and decisions in ADRS. ARCHITECTURE explains how they fit together. When documents disagree, none is silently preferred; the affected documents are corrected before implementation is delegated.

This package resolves and supersedes the following ambiguities in the earlier design:

- An offline NSIS installer is required from the first pilot, with stable application identity and data stored outside the binaries.
- The application is a modular monolith, using ports and adapters at infrastructure boundaries; local IPC contracts are used, with no HTTP server.
- SQLite is accessed through rusqlite using one connection owned by a dedicated thread; application transactions and optimistic concurrency are used.
- Plain text is the initial body format; a rich-text editor and advanced caching are deferred to later decisions.
- Opening a paper is an explicit operation that updates the last-opened timestamp; reading a record does not.
- Initial capture and provenance are committed atomically. Evaluating a gate for display does not authorize advancing without reevaluating it in the backend.
- Archive and restore are reversible; merge and permanent deletion are outside v0.1. The original concept catalog retains those possibilities for future evolution.

Enums include states reserved for future evolution, but the existence of a column or value does not enable a button or command outside the approved scope. Acceptance of this baseline authorizes execution; it does not certify an implemented product. The originals retain their historical status.

## Conditions for Starting Implementation

1. Acceptance is recorded: the user's instruction of 1 October 2026 authorizes environment preparation and complete phased implementation on this baseline.
2. The coordinator verifies that the SPECs, contracts, domain, and data model have no critical contradictions, then updates the detailed plan with the normative signatures.
3. The exact dependencies, toolchain, and licenses are fixed through a compatibility review and recorded before feature work. Framework selection is already defined; patch versions are pinned in lockfiles and are not invented in these documents.
4. The missing Rust requirement is authorized and installed, and the build environment is verified. A computer used only to run the program does not need development tools.
5. Each task receives its SPEC, ADRs, contract, owned files, dependencies, and acceptance tests. No worker may redefine those boundaries unilaterally.

Review of this package is a design milestone. The internal behavior of P3/P4 or AI does not need to be settled now because they will not be implemented under these contracts.

## Transition to the Future Repository

When the project starts at `C:\Users\david\Projects\Research-Workbench`, copy this package to `docs/architecture/`, the original vision to `docs/PRODUCT.md`, and the plan to `docs/plans/`. The coordinator assigns a commit for the accepted baseline. ADRs move from Proposed to Accepted only when acceptance is recorded; a superseded decision retains its history and links to its successor ADR.

During execution, `docs/STATUS.md` records a single task list and its evidence. Code must generate or verify contract artifacts to prevent Rust and TypeScript types from diverging. Every public contract change updates its version, affected SPECs, required migrations, and consumer tests.

## Documentation Review Performed

Domain/contracts, SPECs/quality, and ADRs were drafted with separate responsibilities. The coordinator integrated architecture and data, and an independent review compared the boundaries between documents. Five material ambiguities were resolved: the P1 archive branch, persistent representation of P3 candidates, provenance review, selectors for external backups, and deterministic phase invalidation.

The final review found no further critical blockers at those boundaries. This is evidence that the proposal is coherent; tests, performance measurements, and checks of the installed product remain requirements for future execution. The eight documents and their local links are checked before the package is delivered.
