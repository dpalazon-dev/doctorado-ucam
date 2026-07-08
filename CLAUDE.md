# CLAUDE.md

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 4 — Navigation, Delivery and Process |
| **Normative status** | Coding-agent operating instructions |
| **Authoritative for** | How coding agents should orient themselves, select context, preserve repository conventions and report work. |
| **Not authoritative for** | System behavior, architecture, domain meaning, roadmap status beyond what it references or technology selection. |
| **Required reading** | `DOCUMENTATION_ARCHITECTURE.md`, `IMPLEMENTATION_CONTEXTS.md`, `ROADMAP.md`, `SPEC_CATALOG.md`. |
| **Downstream documents** | Every coding-agent session, implementation prompt and automated repository workflow. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

This file provides guidance to coding agents working in the ResearchOS repository.

## What this repository is

ResearchOS is a local-first, single-user desktop operating environment for knowledge work, research and operational continuity. The canonical architecture is complete through `CANONICAL_DATA_MODEL.md`, `AGENT_RUNTIME.md` and the provisional `TECHNICAL_ARCHITECTURE.md` baseline.

The current phase is **Development Specifications**. `SPEC_CATALOG.md` is the registry and dependency order for those Specs. Do not introduce production code, source directories, build systems or dependencies outside an approved Spec. Explicit proving experiments may create disposable code only within the boundary defined by `IMPLEMENTATION_PLAN.md`.

The current product baseline is:

```text
Tauri + React + TypeScript Workspace
        ↓
authoritative Rust runtime
        ↓ controlled, versioned IPC
subordinate Python cognitive sidecar
        ↓
SQLite + content-addressed filesystem + rebuildable projections
```

Rust owns canonical state, commands, policies, durable execution and commitment. Python performs bounded cognitive and scientific operations and must not write canonical state directly.

## Mandatory task protocol

Before any code or structural repository change:

1. Read this file.
2. Read `ROADMAP.md` and identify the active phase.
3. Read `SPEC_CATALOG.md` and identify the governing Spec and its status.
4. Do not implement a Spec whose prerequisites are incomplete or whose status is not `Approved` or `Implementing`.
5. Select one primary context bundle in `IMPLEMENTATION_CONTEXTS.md`.
6. Load the governing Spec and its authoritative owner documents in full.
7. Load supporting sections only when named by the Spec or bundle.
8. State the invariants, authority boundaries and owned state before editing.
9. Implement the smallest coherent change inside the Spec scope.
10. Run the Spec's tests and architectural conformance checks.
11. Update the Spec and catalog status only when their completion criteria are actually met.
12. Report changed owners, downstream documents reviewed, tests run, deferrals and ADR needs.

Do not load every Markdown file by default. Do not use broad repository context as a substitute for identifying concern ownership.

## Documentation levels

- **Level 1 — Canonical Foundations:** vision, principles, system authority, logical domain and implementation-neutral software architecture.
- **Level 2 — Specialized Models:** conceptual models plus Component Model, Canonical Data Model, Data Architecture, Agent Runtime and Technical Architecture.
- **Level 3 — Functional, Vertical and Development Specifications:** capabilities, use cases, vertical definitions and bounded Specs under `specs/`.
- **Level 4 — Navigation, Delivery and Process:** README, roadmap, ADR register, Spec catalog, proving plan and agent/documentation guidance.

A document is authoritative only for the concerns declared in its Document Contract. A Spec translates upstream authority into an exact implementation contract; it does not replace that authority.

## Load-bearing invariants

### One canonical authority

Committed domain state is authoritative. Search indexes, embeddings, graph projections, context bodies, model outputs and external systems are derived or mediated mechanisms.

### Seven Core Entities

The routine roots are **Project, Document, Knowledge, Task, Activity, Person and Resource**. Breadth comes from Derived Types, Value Objects, typed relationships, process state and projections. A new root requires Experiment 0 evidence and an ADR.

### One model, many verticals

Personal, Daily Work, Administration, Teaching, Research and Organization extend `CANONICAL_DATA_MODEL.md`. A vertical does not create an independent database, graph, memory, event runtime or AI authority.

### Rust authority, Python cognition

Python receives typed, budgeted operations and returns Artifacts, observations or Proposals. Rust validates and commits. No model or framework controls permissions, scheduling, budgets or canonical mutation.

### Local-first product

The installed application must not require the user to administer PostgreSQL, Docker, a broker, a reverse proxy, a graph server or a vector database server. Specialized engines enter only through adapters and measured replacement triggers.

## Repository and naming conventions

- Architecture and process documents remain at the repository root.
- Development Specs are registered in `SPEC_CATALOG.md` and will live under `specs/` in the category assigned by the catalog.
- Source topology is introduced by `SPEC-001`; do not invent an alternative tree beforehand.
- Documents are written in English. Maintainer conversation and `README.md` are in Spanish.
- Every Markdown document must keep an accurate Document Contract.
- Vertical documents reference use-case IDs instead of copying use-case prose.
- Significant architecture changes require an ADR and downstream reconciliation.
- Never treat a framework's internal state, agent graph or memory store as ResearchOS authority.
