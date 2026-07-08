# Roadmap

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 4 — Navigation, Delivery and Process |
| **Normative status** | Plan of record |
| **Authoritative for** | Project phases, ordering, milestones, current status and completion criteria. |
| **Not authoritative for** | Domain structure, architecture, runtime behavior, technology choices or experiment results. |
| **Required reading** | `VISION.md`, `DOCUMENTATION_ARCHITECTURE.md`, current canonical architecture documents and `DECISIONS.md`. |
| **Downstream documents** | `README.md`, `CLAUDE.md`, `IMPLEMENTATION_PLAN.md`, development specifications, implementation planning and release milestones. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

> **Status: Living.** Updated as phases progress. This document owns what gets built, in what order, and why.

## Purpose

ResearchOS has completed its conceptual and architectural foundation. The plan of record now moves from architecture closure to implementable specifications, an executable desktop foundation and one evidence-producing vertical slice.

The current objective is not to add more general models. It is to convert the existing contracts into a local-first product that can be installed, run and evaluated on real data.

---

# Where We Stand

| Layer | Principal documents | State |
|---|---|---|
| Conceptual foundation | Vision, Principles, System Model, Core Operational Model, Responsibilities, Domain Map, Domain Model, Knowledge Model, Capabilities, Use Cases | Mature |
| Cognitive architecture | AI Architecture, Memory Model, Context Model, Event Model | Complete |
| Logical domain | Logical Domain Model | Stable · v1.1 |
| Domain breadth | Derived Types and Use Cases across six operational verticals | Closed |
| Interaction model | Interaction Model | Complete |
| System architecture | System Architecture | Complete · v1.0 |
| Software architecture | Software Architecture | Complete · v1.0 |
| Component architecture | Component Model | Complete · v1.0 |
| Data architecture | Canonical Data Model + Data Architecture | Complete · v1.0 |
| Agent execution | Agent Runtime | Complete · v1.1 |
| Technical baseline | Technical Architecture | Provisional canonical baseline · v0.3 |
| Documentation governance | Documentation Architecture + Implementation Contexts + document contracts | Complete · v1.0 |
| Significant decisions | ADR-0001 through ADR-0011 | Closed for the current baseline |
| Development specifications | `SPEC_CATALOG.md` active; bounded implementation contracts | **In progress · SPEC-001 Ready** |
| Executable product | Desktop foundation and proving slice | Pending |

The architecture is closed far enough to begin development specifications. The baseline is intentionally provisional where experiments must still produce evidence, but no additional cross-cutting architecture document is required before specification work begins.

The product target is now explicit:

```text
single-user
+
local-first
+
cross-platform desktop application
+
Tauri and React Workspace
+
authoritative Rust runtime
+
subordinate Python cognitive sidecar
+
embedded local persistence
```

---

# Delivery Strategy

```text
Architecture Closure
        ↓
Development Specifications
        ↓
Executable Desktop Foundation
        ↓
Document-to-Knowledge Proving Slice
        ↓
First Vertical
        ↓
Product Hardening
        ↓
Additional Verticals
```

The architecture documents define constraints. Specs define implementable contracts. Code begins only against an approved bounded Spec or an explicitly labelled experiment.

---

# Phase A — Domain Breadth

**Status:** Complete

## Objective

Represent the user's operational reality without expanding the root ontology unnecessarily.

## Result

The six operational verticals are represented:

1. Personal;
2. Daily Work;
3. Administration;
4. Teaching;
5. Research;
6. Organization.

The seven Core Entities remain the routine canonical roots. Vertical breadth is expressed through Derived Types, relationships, invariants, lifecycles and use cases.

## Completion criterion

Every operational level has representative Derived Types and Use Cases.

**Criterion met.**

---

# Phase B — Reference Architecture

**Status:** Complete

## Objective

Define authority, semantics, components, data ownership, runtime behavior and interaction boundaries without prematurely binding the system to distributed infrastructure.

## Delivered

- Interaction Model;
- System Architecture;
- Software Architecture;
- Component Model;
- Data Architecture;
- Agent Runtime;
- Technical Architecture baseline;
- documentation authority and bounded implementation contexts.

## Completion criterion

Every major architectural concern has one canonical owner and no unresolved contradiction blocks specification work.

**Criterion met for the current baseline.**

---

# Phase C — Architecture Closure

**Status:** Complete

## Objective

Close the decisions needed to move from architecture into development specifications.

## Delivered

- `CANONICAL_DATA_MODEL.md` as the shared structural contract beneath vertical extensions;
- ADRs for local-first product topology;
- ADR for Tauri and React as the desktop shell;
- ADR for Rust authority and the Python cognitive sidecar;
- ADR for embedded local persistence;
- ADR for canonical governance of vertical data extensions;
- reconciliation of the Technical Architecture with the actual desktop product target.

## Completion criterion

The product topology, runtime ownership, local data baseline and vertical-extension mechanism are explicit and recorded.

**Criterion met.**

---

# Phase D — Development Specifications

**Status:** In progress · `SPEC_CATALOG.md` active · `SPEC-001` Ready

## Objective

Translate the canonical architecture into bounded, testable and implementable contracts.

## Specification structure

```text
SPEC_CATALOG.md
specs/
├── foundation/
├── platform/
├── cognitive/
├── workspace/
└── verticals/
```

## Initial specification order

| Order | Specification | Primary outcome |
|---:|---|---|
| 1 | `SPEC-001 Repository and Build System` | Cross-platform workspace, toolchains and architecture enforcement |
| 2 | `SPEC-002 Desktop Application Lifecycle` | Startup, single-instance behavior, shutdown, paths and recovery |
| 3 | `SPEC-003 Rust Module Boundaries` | Authoritative crates, dependency rules and ports |
| 4 | `SPEC-004 SQLite Canonical Persistence` | Connections, transactions, migrations, concurrency and backup |
| 5 | `SPEC-005 Canonical Entity and Relationship Storage` | Shared records, extensions, provenance and repositories |
| 6 | `SPEC-006 Content-Addressed Document Store` | Immutable content objects, manifests and safe deletion |
| 7 | `SPEC-007 Durable Jobs and Event Runtime` | Jobs, outbox, inbox, scheduling, retries and recovery |
| 8 | `SPEC-008 Rust–Python IPC Protocol` | Versioned messages, capability negotiation, progress and cancellation |
| 9 | `SPEC-009 Python Cognitive Sidecar` | Process lifecycle, cognitive capability adapters and packaging |
| 10 | `SPEC-010 Document Ingestion Pipeline` | Import, extraction, versions, segments and provenance |
| 11 | `SPEC-011 Retrieval and Projection System` | FTS, vector projection, relationship traversal and context inputs |
| 12 | `SPEC-012 Proposal and Approval Flow` | Human-gated cognitive results and canonical commitment |
| 13 | `SPEC-013 Workspace Shell` | Initial navigation, health, document and operation surfaces |

Vertical Specs begin only after the shared foundation has executable evidence.

## Required content of every Spec

1. scope and non-goals;
2. concern owner and component boundary;
3. use cases or proving experiment;
4. entities, invariants and state transitions;
5. commands, queries, events, jobs, processes, Proposals and effects;
6. ports and adapters;
7. logical and physical schema plus migration implications;
8. Workspace, IPC, runtime-role and external-interface contracts;
9. authorization, approval and classification rules;
10. failure, idempotency, retry and recovery behavior;
11. observability and audit evidence;
12. tests and acceptance fixtures;
13. implementation tasks and dependency order;
14. completion criteria and explicit deferrals.

## Completion criterion

The executable foundation and proving slice can be implemented without inventing architecture inside code.

---

# Phase E — Executable Desktop Foundation

**Status:** Pending

## Objective

Produce the smallest installable application that proves the process topology and local operational baseline.

## Minimum executable cycle

```text
Tauri application starts
        ↓
React Workspace loads
        ↓
Rust initializes application paths and SQLite
        ↓
Schema migrations run
        ↓
Python cognitive sidecar starts
        ↓
IPC handshake and capability negotiation succeed
        ↓
Health and diagnostics are visible
        ↓
Application shuts down and recovers cleanly
```

## Required evidence

- Windows, macOS and Linux builds execute in CI or documented target environments;
- local paths and credentials are not hard-coded;
- migrations are repeatable and failure-safe;
- the sidecar can be supervised, cancelled and restarted;
- logs correlate Workspace, Rust and Python operations;
- the application can create and restore a minimal backup;
- no external server is required to start the product.

## Completion criterion

A clean machine can install, open, close and reopen ResearchOS while preserving valid local state.

---

# Phase F — Document-to-Knowledge Proving Slice

**Status:** Pending

## Objective

Validate the center of ResearchOS through one complete, evidence-producing workflow rather than broad CRUD coverage.

## Proving flow

```text
Import document
        ↓
Store immutable source content
        ↓
Extract normalized text
        ↓
Create document version and segments
        ↓
Build lexical and vector projections
        ↓
Extract Knowledge candidates
        ↓
User reviews Proposals
        ↓
Commit accepted Knowledge and relationships
        ↓
Ask a question
        ↓
Retrieve hybrid context
        ↓
Answer with source evidence
```

## What this slice must validate

- Document and Knowledge roots;
- canonical data envelopes and vertical-neutral extension rules;
- content-addressed storage;
- provenance and source spans;
- jobs, events, checkpoints and recovery;
- Rust–Python IPC;
- document processing and embeddings;
- lexical, vector and relationship retrieval;
- Context Manifest construction;
- cognitive operation budgets and verification;
- Proposal and approval boundaries;
- citations and traceability in the Workspace.

## Completion criterion

The cycle runs over real documents with reproducible traceability, survives an interrupted long-running operation and never allows the cognitive sidecar to mutate canonical state directly.

---

# Phase G — First Vertical

**Status:** Pending

## Objective

Build the first domain extension on top of the proven shared foundation.

## Initial vertical

Research is the preferred first vertical because it exercises the strongest combination of:

- Documents;
- Knowledge;
- evidence;
- provenance;
- relationships;
- retrieval;
- reasoning;
- citation;
- projects and activities.

The first bounded type set should remain small:

```text
ResearchProject
Hypothesis
Evidence
Finding
Paper
LiteratureReview
```

## Completion criterion

The vertical demonstrates that Derived Types, typed extension profiles, vertical invariants, relationships and projections can be added without creating a separate persistence authority or runtime.

---

# Phase H — Product Hardening

**Status:** Pending

## Objective

Make the proving product safe to retain and evolve before broadening vertical coverage.

## Required work

- native signing, notarization and update flow;
- backup, restoration and migration rollback;
- credential storage and provider configuration;
- crash recovery and interrupted-job reconciliation;
- complete deletion and projection invalidation;
- disk-space, cache and rebuild management;
- diagnostic export with sensitive-data redaction;
- cognitive evaluation and regression suites;
- security tests for prompt injection, retrieval poisoning and unsafe tools;
- performance budgets for startup, retrieval and document processing.

## Completion criterion

The application can be upgraded, diagnosed, backed up, restored and recovered without hidden infrastructure or loss of canonical authority.

---

# Additional Verticals

Additional verticals follow only after the shared foundation and first vertical demonstrate stable extension mechanics.

Recommended order is determined by real user value rather than conceptual completeness. Each vertical Spec must extend `CANONICAL_DATA_MODEL.md` and declare:

- base Core Entities used;
- Derived Types introduced;
- typed extension profiles;
- relationships and cardinalities;
- invariants and state machines;
- commands, events and workflows;
- projections and interaction surfaces;
- explicit non-goals.

No vertical introduces an independent database, graph, memory system, event mechanism or AI runtime.

---

# Milestones

| Milestone | Done when | Status |
|---|---|---|
| M1 · Domain breadth | All six operational levels are represented | Achieved |
| M2 · Interaction model | Manual, assisted, conversational and autonomous interaction modes are defined | Achieved |
| M3 · Reference architecture | System and Software Architecture are complete | Achieved |
| M3.5 · Documentation operationalization | Ownership, precedence and bounded context bundles are defined | Achieved |
| M4 · Implementation views | Structural and runtime views are derived | Achieved |
| M5 · Component model | Component responsibilities, dependencies and runtime placement are fixed | Achieved |
| M6 · Data architecture | Data ownership, lifecycle, projections and persistence classes are fixed | Achieved |
| M7 · Agent runtime | Cognitive lifecycle, verification, budgets and recovery are fixed | Achieved |
| M8 · Technical baseline | Local-first desktop technologies and process topology are selected provisionally | Achieved |
| M8.5 · Canonical data model | Shared structural contracts and vertical extension mechanism are canonical | Achieved |
| M9 · Architecture decision closure | ADR-0001 through ADR-0011 record the current baseline | Achieved |
| M10 · Development specification set | Catalogue and foundation Specs are approved | **In progress** |
| M11 · Executable desktop foundation | Installable shell, Rust core, SQLite and Python sidecar operate together | Pending |
| M12 · Proving slice | Document-to-Knowledge cycle executes with evidence and recovery | Pending |
| M13 · First vertical | Research extension operates on the shared model | Pending |
| M14 · Product hardening | Upgrade, backup, recovery, diagnostics and security gates are proven | Pending |

---

# Working Principles

- **Architecture is closed enough to build.** New general architecture documents require evidence of a real unresolved concern.
- **Specs before implementation.** Production code implements bounded contracts rather than inventing them.
- **Local-first is the product baseline.** No hidden server or managed infrastructure is required to start ResearchOS.
- **Rust owns authority.** Python and models perform bounded cognitive work and return typed results or Proposals.
- **One canonical state.** Vertical extensions never create parallel data authority.
- **Derived Types before new roots.** ADR-0004 remains the burden-of-proof rule.
- **Projections are rebuildable.** Search, vectors, graph views and dashboards do not become truth.
- **Human authority is explicit.** Sensitive or epistemically meaningful mutations pass through policy and approval gates.
- **Thin slice before breadth.** Prove one complete operating cycle before implementing full vertical coverage.
- **Experiments may challenge the baseline.** Measured evidence can trigger a new ADR; convenience cannot silently rewrite architecture.
- **The objective is execution.** Value now comes from installable software, real data and reproducible evidence.
