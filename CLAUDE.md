# CLAUDE.md

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 4 — Navigation, Delivery and Process |
| **Normative status** | Coding-agent operating instructions |
| **Authoritative for** | How coding agents should orient themselves, select context, preserve repository conventions and report work. |
| **Not authoritative for** | System behavior, architecture, domain meaning, roadmap status beyond what it references or technology selection. |
| **Required reading** | `DOCUMENTATION_ARCHITECTURE.md`, `IMPLEMENTATION_CONTEXTS.md`, `ROADMAP.md`. |
| **Downstream documents** | Every coding-agent session, implementation prompt and automated repository workflow. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repository is

ResearchOS (formerly Doctorado_UCAM) is a from-scratch design of a research operating system for a PhD candidate: knowledge management, research workflow and AI assistance unified around one domain model. The repository currently contains the conceptual, logical, system and software reference architecture — Markdown documents, no code. Phase A (design the domain) is complete. Phase B has completed `SYSTEM_ARCHITECTURE.md` and `SOFTWARE_ARCHITECTURE.md`; implementation views, ADRs, Technical Architecture and the proving slice are next. There is no package.json, build system, linter or test suite yet — do not invent one outside the roadmap.

`ROADMAP.md` is the plan of record. Read it before structural changes. For implementation work, select the appropriate bounded reading bundle in `IMPLEMENTATION_CONTEXTS.md` before loading any architectural documents.

## Documentation and context protocol

The repository uses a governed four-level documentation architecture. `DOCUMENTATION_ARCHITECTURE.md` owns document scope, authority, precedence and downstream-impact rules. `IMPLEMENTATION_CONTEXTS.md` owns the task-specific reading bundles used by coding agents.

Do **not** load every Markdown file by default. For every task:

1. Read this file.
2. Check the current phase in `ROADMAP.md`.
3. Select one primary bundle in `IMPLEMENTATION_CONTEXTS.md`.
4. Load the concern-owner documents in full.
5. Load only the named sections of supporting documents.
6. State the invariants to preserve before editing code or architecture.
7. Review the downstream documents declared by any changed owner.
8. End with the completion report required by `IMPLEMENTATION_CONTEXTS.md`.

The four documentation levels are:

- **Level 1 — Canonical Foundations:** `VISION.md`, `SYSTEM_PRINCIPLES.md`, `SYSTEM_ARCHITECTURE.md`, `LOGICAL_DOMAIN_MODEL.md`, `SOFTWARE_ARCHITECTURE.md`.
- **Level 2 — Specialized Models:** System, operational, responsibility, domain, knowledge, AI, memory, context, event and interaction models.
- **Level 3 — Functional and Vertical Specifications:** capabilities, use cases, domain vertical index and vertical files.
- **Level 4 — Navigation, Delivery and Process:** README, roadmap, implementation plan, agent guidance and documentation/context governance.

A document is authoritative only for the concerns declared in its Document Contract. Repeated statements elsewhere are summaries or local consequences, not competing definitions.

## Core invariant: seven entities, no new roots

The entire domain — every operational reality of the researcher — is described with exactly seven Core Entities: **Project, Document, Knowledge, Task, Activity, Person, Resource**. Breadth comes exclusively from *Derived Types* (specializations of those seven) and *Use Cases* (behavioral compositions), never from adding an eighth root entity. The authoritative definition lives in `LOGICAL_DOMAIN_MODEL.md`; `DOMAIN_VERTICALS.md` applies it and `ROADMAP.md` reports it. Treat it as load-bearing when proposing any new concept.

## The vertical layer

`DOMAIN_VERTICALS.md` is the index of how the seven entities specialize across six operational verticals (Personal, Daily Work, Administration, Research, Teaching, Organization). Four verticals have enough real content to carry their own document (`RESEARCH_VERTICAL.md`, `TEACHING_VERTICAL.md`, `ADMINISTRATION_VERTICAL.md`, `ORGANIZATION_VERTICAL.md`); Personal and Daily Work stay inline in the index because they're intentionally thin, not because they're incomplete. Each `*_VERTICAL.md` file holds Purpose, Scope, Derived Types and a table of its Use Cases **referenced by ID** — it never copies use case prose out of `USE_CASES.md`. Follow that pattern for any new vertical: it graduates to its own file only once it has real use cases, and the file references rather than duplicates them.

A vertical named "Doctorate" appears in old commit messages and in `ROADMAP.md`'s history notes but no longer exists: it never gained its own use cases, and dissolved into Research (thesis chapters are just Writing, UC-W01) and Administration (progress reports, committee decisions, deadlines). If you're tracing an old reference to "Doctorate," that's where it went.

## Keeping documents in sync

Every document declares its authority, required reading and downstream documents in its Document Contract. Follow the change-impact protocol in `DOCUMENTATION_ARCHITECTURE.md` rather than updating files by intuition.

`USE_CASES.md` remains the owner of full use-case prose. Vertical files and roadmap tables reference or summarize it; they never become competing catalogues. When a canonical use case changes, review the affected vertical mapping and status documents in the same change.

Do not copy global invariants into specialized or process documents. Import them through a short attributed statement and preserve the authoritative definition in its owner.

## Conventions

- The repository remains intentionally flat while it contains only architecture documents. The package topology in `SOFTWARE_ARCHITECTURE.md` is normative in responsibility but does not justify creating source directories before implementation begins.
- Naming follows the suffix of what a document is: `*_MODEL.md` / `*_MAP.md` for foundational and cognitive documents, `*_VERTICAL.md` for vertical specializations, plain names (`VISION.md`, `ROADMAP.md`, `USE_CASES.md`) for singular documents.
- Documents are written in English; conversation with the maintainer happens in Spanish. `README.md` is the one document written in Spanish, as the repo's front door.
- Every Markdown file must keep an accurate Document Contract.
- Coding agents must use `IMPLEMENTATION_CONTEXTS.md`; repository-wide prompt loading is an exception that requires explicit justification.
