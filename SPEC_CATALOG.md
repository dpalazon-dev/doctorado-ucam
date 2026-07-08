# Development Specification Catalog

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 4 — Navigation, Delivery and Process |
| **Normative status** | Plan of record for development specifications |
| **Authoritative for** | Development Spec identifiers, titles, category, lifecycle status, dependency order, current delivery wave, entry gates and completion reporting. |
| **Not authoritative for** | Product vision, domain meaning, architecture, technology selection, behavior inside a Spec, implementation details or experiment results. |
| **Required reading** | `DOCUMENTATION_ARCHITECTURE.md`, `ROADMAP.md`, `DECISIONS.md`, `IMPLEMENTATION_CONTEXTS.md`. |
| **Downstream documents** | Every development Spec, coding-agent prompt, implementation branch, pull request and milestone report. |

> This catalog sequences and tracks Specs. The authoritative rule implemented by a Spec remains in the upstream concern owner named by that Spec.

---

> **Status: Active · v1.0.** Architecture Closure is complete. `SPEC-001` is the next specification to author.

## Purpose

ResearchOS has reached the point where architecture must be converted into bounded implementation contracts. This catalog prevents three failure modes:

- implementing directly from broad architecture documents;
- writing Specs in an order that assumes unavailable contracts;
- allowing code, status tables and agent prompts to disagree about what is ready.

A development Spec is the smallest independently reviewable contract that can authorize production implementation for one coherent concern.

---

# Position in the Documentation

```text
Canonical and specialized architecture owners
                    ↓ constrain
SPEC_CATALOG.md
identity · status · dependency order
                    ↓ governs
specs/**/*.md
exact bounded implementation contracts
                    ↓ authorize
source code · schemas · tests · packaging
                    ↓ produce evidence for
IMPLEMENTATION_PLAN.md · ADRs · architecture stabilization
```

The catalog does not summarize the content of every Spec. It records enough metadata to determine what may be authored or implemented next.

---

# Specification Authority Rules

1. A Spec MUST name every upstream document whose authority it translates.
2. A Spec MUST NOT change an upstream invariant, technical baseline or domain meaning by implication.
3. A conflict discovered while authoring a Spec is resolved in the upstream owner first.
4. A significant owner change requires an ADR before the Spec can be approved.
5. A Spec owns its exact bounded implementation contract: schemas, interfaces, state transitions, failure semantics, tests and completion evidence.
6. Source code MUST NOT become the only place where a contract exists.
7. A dependency Spec is consumed through its approved public contract, not through private implementation assumptions.
8. Proving or spike code outside an approved Spec MUST be explicitly labelled experimental, isolated from production authority and tied to `IMPLEMENTATION_PLAN.md`.

---

# Identifier and Path Rules

## Identifier

```text
SPEC-NNN
```

- Identifiers are sequential and immutable.
- A retired identifier is never reused.
- Renaming a Spec does not change its identifier.
- A substantial replacement receives a new identifier and marks the old Spec `Superseded`.

## Paths

Approved category layout:

```text
specs/
├── foundation/
├── platform/
├── cognitive/
├── workspace/
└── verticals/
```

File naming convention:

```text
specs/<category>/SPEC-NNN_<UPPER_SNAKE_TITLE>.md
```

Example:

```text
specs/foundation/SPEC-001_REPOSITORY_AND_BUILD_SYSTEM.md
```

`SPEC_CATALOG.md` remains at the repository root because it is a navigation and delivery owner rather than an implementation contract.

---

# Lifecycle

| Status | Meaning | Code allowed? |
|---|---|---:|
| **Planned** | Identified and ordered, but prerequisites or authoring inputs are incomplete. | No |
| **Ready** | Prerequisites are satisfied and the Spec may be authored. | No |
| **Draft** | Contract is being written; unresolved decisions are visible. | Experimental fixtures only |
| **Review** | Content is complete enough for architecture, data, security and test review. | No production implementation |
| **Approved** | Contract is accepted and may authorize implementation. | Yes |
| **Implementing** | Production implementation is active. | Yes |
| **Implemented** | Code and required tests exist; validation evidence is still being collected. | Maintenance only within scope |
| **Validated** | Completion criteria and proving evidence are accepted. | Yes |
| **Deferred** | Intentionally postponed with rationale and re-entry trigger. | No |
| **Superseded** | Replaced by another Spec; retained for history. | No new work |
| **Rejected** | Considered and explicitly declined. | No |

Status transitions:

```text
Planned → Ready → Draft → Review → Approved → Implementing → Implemented → Validated
                         ↘ Deferred
Draft/Review/Approved ──→ Superseded
Draft/Review ───────────→ Rejected
```

A status is changed only when its gate is met. Percentage-complete reporting is non-normative and does not replace lifecycle status.

---

# Gates

## Ready gate

A Spec may move to `Ready` when:

- all catalogued prerequisite Specs are `Approved` or later, unless the Spec only depends on their reviewed public contract;
- upstream concern owners exist and are internally consistent;
- the owning component and state owner are known;
- the relevant use case or proving objective is identified;
- no unresolved ADR blocks the boundary.

## Approval gate

A Spec may move to `Approved` when it contains:

1. scope and non-goals;
2. concern owner and implementing component;
3. relevant use cases or experiment hypothesis;
4. entities, invariants and state transitions;
5. commands, queries, events, jobs, processes, Proposals and effects;
6. public and driven ports;
7. logical and physical data schema plus migration behavior;
8. Workspace, IPC, runtime-role or external-interface contracts;
9. authorization, approval and data-classification rules;
10. failure, idempotency, retry and recovery behavior;
11. observability and audit evidence;
12. unit, property, architecture, integration and acceptance tests;
13. implementation tasks and dependency order;
14. completion criteria and explicit deferrals.

It must also pass:

- authority review;
- dependency review;
- data and migration review when applicable;
- security and approval review when applicable;
- testability review;
- local-first and packaging impact review.

## Implemented gate

A Spec may move to `Implemented` when:

- all required code, schemas and migrations exist;
- architecture and dependency tests pass;
- public contracts are versioned as specified;
- failure and recovery paths have executable tests;
- documentation generated or affected by the implementation is synchronized;
- known deferrals are recorded rather than silently omitted.

## Validated gate

A Spec may move to `Validated` when:

- acceptance criteria pass on the declared target platforms;
- proving data or user evidence is recorded;
- performance, durability and recovery budgets are met;
- no open issue invalidates the Spec's main hypothesis;
- `ROADMAP.md` milestone consequences are updated.

---

# Dependency Semantics

| Relationship | Meaning |
|---|---|
| **Prerequisite** | The upstream Spec must expose an approved contract before this Spec can be approved. |
| **Implementation dependency** | Implementation consumes the upstream public interface or artifact. |
| **Evidence dependency** | Validation requires output or measurements produced by another Spec or experiment. |
| **Follow-up** | Work intentionally excluded and delegated to a later Spec. |

Dependencies do not permit private coupling. If a downstream Spec needs an undocumented internal detail, the upstream contract is incomplete.

---

# Current Delivery Waves

## Wave 1 — Desktop and authoritative foundation

Establish the repository, desktop lifecycle, Rust boundaries, embedded persistence, canonical storage, durable execution and Rust–Python process contract.

## Wave 2 — Cognitive and document proving foundation

Establish the Python sidecar, document ingestion, retrieval projections, Proposal/approval flow and initial Workspace surfaces.

## Wave 3 — First end-to-end proving slice

Integrate Wave 1 and Wave 2 into the Document-to-Knowledge cycle defined by `ROADMAP.md` and evaluated by `IMPLEMENTATION_PLAN.md`.

## Wave 4 — First vertical and product hardening

Author vertical Specs only after the shared extension mechanism and proving slice have executable evidence.

---

# Specification Registry

| ID | Title | Category | Wave | Prerequisites | Primary owners | Status |
|---|---|---|---:|---|---|---|
| **SPEC-001** | Repository and Build System | foundation | 1 | — | `TECHNICAL_ARCHITECTURE.md`, `SOFTWARE_ARCHITECTURE.md` | **Ready** |
| **SPEC-002** | Desktop Application Lifecycle | foundation | 1 | SPEC-001 | `TECHNICAL_ARCHITECTURE.md`, `COMPONENT_MODEL.md` | Planned |
| **SPEC-003** | Rust Module Boundaries | foundation | 1 | SPEC-001 | `SOFTWARE_ARCHITECTURE.md`, `COMPONENT_MODEL.md`, ADR-0009 | Planned |
| **SPEC-004** | SQLite Canonical Persistence | platform | 1 | SPEC-001, SPEC-003 | `DATA_ARCHITECTURE.md`, `TECHNICAL_ARCHITECTURE.md`, ADR-0010 | Planned |
| **SPEC-005** | Canonical Entity and Relationship Storage | platform | 1 | SPEC-003, SPEC-004 | `CANONICAL_DATA_MODEL.md`, `LOGICAL_DOMAIN_MODEL.md`, `DATA_ARCHITECTURE.md` | Planned |
| **SPEC-006** | Content-Addressed Document Store | platform | 1 | SPEC-003, SPEC-004 | `CANONICAL_DATA_MODEL.md`, `DATA_ARCHITECTURE.md`, `TECHNICAL_ARCHITECTURE.md` | Planned |
| **SPEC-007** | Durable Jobs and Event Runtime | platform | 1 | SPEC-003, SPEC-004, SPEC-005 | `SOFTWARE_ARCHITECTURE.md`, `EVENT_MODEL.md`, `DATA_ARCHITECTURE.md` | Planned |
| **SPEC-008** | Rust–Python IPC Protocol | cognitive | 1 | SPEC-001, SPEC-002, SPEC-003 | `TECHNICAL_ARCHITECTURE.md`, `AGENT_RUNTIME.md`, ADR-0009 | Planned |
| **SPEC-009** | Python Cognitive Sidecar | cognitive | 2 | SPEC-001, SPEC-008 | `AGENT_RUNTIME.md`, `TECHNICAL_ARCHITECTURE.md`, `COMPONENT_MODEL.md` | Planned |
| **SPEC-010** | Document Ingestion Pipeline | platform | 2 | SPEC-005, SPEC-006, SPEC-007, SPEC-008, SPEC-009 | `CANONICAL_DATA_MODEL.md`, `DATA_ARCHITECTURE.md`, `AGENT_RUNTIME.md`, `TECHNICAL_ARCHITECTURE.md` | Planned |
| **SPEC-011** | Retrieval and Projection System | cognitive | 2 | SPEC-004, SPEC-005, SPEC-010 | `DATA_ARCHITECTURE.md`, `CONTEXT_MODEL.md`, `TECHNICAL_ARCHITECTURE.md` | Planned |
| **SPEC-012** | Proposal and Approval Flow | platform | 2 | SPEC-005, SPEC-007, SPEC-008, SPEC-009, SPEC-010 | `SOFTWARE_ARCHITECTURE.md`, `INTERACTION_MODEL.md`, `AGENT_RUNTIME.md` | Planned |
| **SPEC-013** | Workspace Shell | workspace | 2 | SPEC-002, SPEC-003, SPEC-005, SPEC-007, SPEC-012 | `INTERACTION_MODEL.md`, `COMPONENT_MODEL.md`, `TECHNICAL_ARCHITECTURE.md` | Planned |

The registry status is the only authoritative source for which Spec is next. `ROADMAP.md` owns milestone ordering at a higher level.

## SPEC-010 Deferred Parser Evaluation

SPEC-010 must preserve the simple PyMuPDF extraction baseline and evaluate optional rich structured representations only where the document corpus demonstrates a need.

The evaluation set must include:

```text
Baseline A
PyMuPDF → normalized text

Candidate B
Docling → native structured representation → normalized text

Candidate C
Docling or compatible adapter → DocLang → normalized text + structural anchors
```

Docling and DocLang are candidates, not preselected dependencies. Adoption requires measurable improvement in structural fidelity, evidence localization or retrieval quality that justifies runtime, packaging, security and maintenance cost.

SPEC-010 must not introduce a technology-specific canonical `representation_kind`. When a rich representation is retained, the canonical record uses a semantic kind such as `structured_document` and records the concrete media type, format version, generator and generator version separately.

---

# Critical Path

```text
SPEC-001
   ├── SPEC-002 ──┐
   └── SPEC-003 ──┼── SPEC-008 ── SPEC-009 ──┐
          │       │                           │
          └── SPEC-004 ── SPEC-005 ──┐       │
                    └── SPEC-006      ├── SPEC-010
SPEC-005 ── SPEC-007 ─────────────────┘       │
                                              ├── SPEC-011
                                              └── SPEC-012 ── SPEC-013
```

Parallel authoring is allowed only where the public contracts needed by both branches are explicit. Parallel implementation must not race on the same state owner or undecided schema.

---

# First Proving-Slice Composition

The first complete product slice is not a separate broad Spec. It is the accepted integration of the relevant foundation Specs:

```text
SPEC-002 Desktop lifecycle
SPEC-004 SQLite persistence
SPEC-005 canonical records
SPEC-006 content store
SPEC-007 durable execution
SPEC-008 IPC
SPEC-009 cognitive sidecar
SPEC-010 document ingestion
SPEC-011 retrieval
SPEC-012 proposal and approval
SPEC-013 Workspace
        ↓
Document → segments → projections → Knowledge Proposal
        ↓
human approval → canonical Knowledge → evidence-backed answer
```

Integration acceptance is governed by the proving criteria in `IMPLEMENTATION_PLAN.md` and milestone M12 in `ROADMAP.md`.

---

# Future Vertical Specifications

Vertical Specs are intentionally not assigned identifiers yet. They enter the registry only after:

- SPEC-005 proves the canonical extension mechanism;
- the Document-to-Knowledge slice reaches `Validated` or produces an explicit revision decision;
- the first vertical scope is bounded to real user value;
- its Derived Types, profiles, relationships, invariants and projections can be stated without creating independent authority.

Research is the expected first vertical, but that preference does not authorize premature schema or UI work.

---

# Catalog Update Protocol

For every catalog change:

1. identify the status or dependency being changed;
2. confirm the relevant gate with evidence;
3. update the catalog row;
4. update the Spec's own status and change log;
5. review `ROADMAP.md`, `README.md` and `CLAUDE.md` if the active phase or next work changed;
6. record an ADR if the change alters architecture rather than delivery order.

A pull request changing implementation code and catalog status should state why the transition gate is met.

---

# Immediate Next Action

Author:

```text
specs/foundation/SPEC-001_REPOSITORY_AND_BUILD_SYSTEM.md
```

Its purpose is to establish the reproducible cross-platform repository, toolchains, dependency boundaries, CI matrix and architecture-enforcement foundation required by every later Spec. It must not implement product features that belong to SPEC-002 or later.
