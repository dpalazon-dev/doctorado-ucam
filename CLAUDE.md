# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repository is

ResearchOS (formerly Doctorado_UCAM) is a from-scratch design of a research operating system for a PhD candidate: knowledge management, research workflow and AI assistance unified around one domain model. The repository currently contains the conceptual, logical, system and software reference architecture — Markdown documents, no code. Phase A (design the domain) is complete. Phase B has completed `SYSTEM_ARCHITECTURE.md` and `SOFTWARE_ARCHITECTURE.md`; implementation views, ADRs, Technical Architecture and the proving slice are next. There is no package.json, build system, linter or test suite yet — do not invent one outside the roadmap.

`ROADMAP.md` is the plan of record. Read it first to see what phase the project is in before making structural changes.

## Document architecture

The documents form a dependency chain — each depends conceptually on the ones before it. `README.md` keeps the authoritative numbered list; the shape is:

```
Vision → Principles → System Model → Operational Model → Responsibilities
  → Domain Map → Domain Model → Knowledge Model → System Capabilities → Use Cases
  → AI Architecture (+ Memory / Context / Event Model) → Logical Domain Model → Interaction Model
  → System Architecture → Software Architecture
```

Four documents now anchor the transition from domain definition to implementation:

- **`LOGICAL_DOMAIN_MODEL.md`** — frozen at v1.0. The canonical, technology-neutral specification from which every future schema, diagram and type is derived. Its own evolution rule: additions (new Derived Types under existing Core Entities) are welcome; structural churn (touching the seven Core Entities or their relationships) is not.
- **`USE_CASES.md`** — the single behavioral specification of the platform (52 use cases across 12 groups as of this writing). Every use case follows the same 8-part structure (Intention, Context, Operational Flow, Expected Outcome, State Changes, Capabilities, Domain Entities, Related Use Cases). New concepts enter the Domain Model only after recurring across multiple use cases (the "Domain Consistency" principle, documented in that file's own Evolution section) — never speculatively.
- **`SYSTEM_ARCHITECTURE.md`** — the canonical authority model: Domain State as the single source of truth, Domain Kernel mutation authority, system planes, trust boundaries and runtime invariants.
- **`SOFTWARE_ARCHITECTURE.md`** — the implementable reference architecture: modular monolith, ports and adapters, commands/queries/proposals, transactional outbox, durable processes, Cognitive Runtime and conformance tests.

## Core invariant: seven entities, no new roots

The entire domain — every operational reality of the researcher — is described with exactly seven Core Entities: **Project, Document, Knowledge, Task, Activity, Person, Resource**. Breadth comes exclusively from *Derived Types* (specializations of those seven) and *Use Cases* (behavioral compositions), never from adding an eighth root entity. This rule is stated repeatedly across `ROADMAP.md`, `DOMAIN_VERTICALS.md` and the Logical Domain Model — treat it as load-bearing when proposing any new concept.

## The vertical layer

`DOMAIN_VERTICALS.md` is the index of how the seven entities specialize across six operational verticals (Personal, Daily Work, Administration, Research, Teaching, Organization). Four verticals have enough real content to carry their own document (`RESEARCH_VERTICAL.md`, `TEACHING_VERTICAL.md`, `ADMINISTRATION_VERTICAL.md`, `ORGANIZATION_VERTICAL.md`); Personal and Daily Work stay inline in the index because they're intentionally thin, not because they're incomplete. Each `*_VERTICAL.md` file holds Purpose, Scope, Derived Types and a table of its Use Cases **referenced by ID** — it never copies use case prose out of `USE_CASES.md`. Follow that pattern for any new vertical: it graduates to its own file only once it has real use cases, and the file references rather than duplicates them.

A vertical named "Doctorate" appears in old commit messages and in `ROADMAP.md`'s history notes but no longer exists: it never gained its own use cases, and dissolved into Research (thesis chapters are just Writing, UC-W01) and Administration (progress reports, committee decisions, deadlines). If you're tracing an old reference to "Doctorate," that's where it went.

## Keeping documents in sync

`ROADMAP.md`'s status tables and `DOMAIN_VERTICALS.md`'s vertical descriptions describe `USE_CASES.md` — they are not independent sources of truth. These have drifted out of sync before (an entire vertical, Administration, existed fully-built in `USE_CASES.md` while `ROADMAP.md` still called it "Absent" and didn't mention it at all). When you add, move or retire use cases in `USE_CASES.md`, update the corresponding vertical file and `DOMAIN_VERTICALS.md`/`ROADMAP.md` status in the same change — don't let the catalogue outrun the documents that describe it.

## Conventions

- The repository remains intentionally flat while it contains only architecture documents. The package topology in `SOFTWARE_ARCHITECTURE.md` is normative in responsibility but does not justify creating source directories before implementation begins.
- Naming follows the suffix of what a document is: `*_MODEL.md` / `*_MAP.md` for foundational and cognitive documents, `*_VERTICAL.md` for vertical specializations, plain names (`VISION.md`, `ROADMAP.md`, `USE_CASES.md`) for singular documents.
- Documents are written in English; conversation with the maintainer happens in Spanish. `README.md` is the one document written in Spanish, as the repo's front door.
