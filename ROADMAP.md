# Roadmap

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 4 — Navigation, Delivery and Process |
| **Normative status** | Plan of record |
| **Authoritative for** | Project phases, ordering, milestones, current status and completion criteria. |
| **Not authoritative for** | Domain structure, architecture, runtime behavior, technology choices or experiment results. |
| **Required reading** | `VISION.md`, `DOCUMENTATION_ARCHITECTURE.md`, current canonical architecture documents. |
| **Downstream documents** | `README.md`, `CLAUDE.md`, `IMPLEMENTATION_PLAN.md`, development planning and release milestones. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

> **Status: Living.** Updated as phases progress. This document owns *what gets built, in what order, and why*. It is the plan of record.

## Purpose

The conceptual work is mature. This document turns it into a build plan.

It does not add models. It sequences the work that carries a frozen conceptual foundation into a real system — first by completing the *breadth* of the domain, then by building the *system* that runs it.

---

# Where We Stand

| Layer | Documents | State |
|--------|-----------|-------|
| Conceptual | Vision, Principles, System Model, Operational Model, Responsibilities, Domain Map, Domain Model, Knowledge Model, System Capabilities, Use Cases | Mature |
| Cognitive Architecture | AI Architecture, Memory Model, Context Model, Event Model | Complete |
| Logical | Logical Domain Model | Stable · v1.1 |
| **Domain breadth** | Derived Types + Use Cases across all operational domains | **Closed** |
| **Interaction model** | System interaction modalities | **Complete** |
| **System architecture** | System Architecture | **Complete · v1.0** |
| **Software architecture** | Software Architecture | **Complete · v1.0** |
| **Documentation governance** | Documentation Architecture + Implementation Contexts + per-document contracts | **Complete · v1.0** |
| **Implementation design** | Views, ADRs and Technical Architecture | **Next** |

The reference architecture is now defined. The remaining gap is **implementation evidence**.

Breadth is closed. Every operational level now has Derived Types and worked Use Cases — [USE_CASES.md](USE_CASES.md) carries 52 across twelve groups. [DOMAIN_VERTICALS.md](DOMAIN_VERTICALS.md) itemizes 39 of them by ID across the six verticals (see the Phase A Deliverables table below); the remaining 13 — People, Context and Traceability — are catalogued there as cross-cutting infrastructure, not verticals, since they operate identically regardless of which vertical produced the entity they touch. The [Interaction Model](INTERACTION_MODEL.md) is also complete.

`SYSTEM_ARCHITECTURE.md` now defines authority, planes, trust boundaries and runtime invariants. `SOFTWARE_ARCHITECTURE.md` translates them into modules, ports, adapters, commands, events, processes and executable runtime roles.

`DOCUMENTATION_ARCHITECTURE.md` assigns one owner to every concern, and `IMPLEMENTATION_CONTEXTS.md` packages bounded reading sets for humans and coding agents. Every Markdown file now declares its authority, prerequisites and downstream impact.

What remains is to derive implementation views, record the significant decisions as ADRs, select concrete technologies and execute the proving experiments.

---

# The Breadth Gap — Closed

A researcher is not only a researcher.

The Vision and the Operational Model always said so — the Operational Model names Research, Project Management, Knowledge Management, Communication, Teaching and Institutional Responsibilities as parallel domains. The Derived Types and the Use Cases drifted, for a time, into a research-only view.

That gap is now closed. Teaching, Administration and Organization each carry a full set of Derived Types and worked Use Cases — Teaching alone has more of them (7) than Research's own dedicated group (4). Personal and Daily Work remain intentionally thin: not an oversight but the point, since personal life needs almost nothing the platform doesn't already provide, and daily coordination is largely Planning and Communication wearing no vertical-specific clothing.

One level named in earlier drafts of this document — **Doctorate** — never became a use-case group of its own. Producing a thesis chapter is Research (Writing); everything institutional around it — progress reports, procedures, training credits, committee decisions, deadlines — is Administration. Both already existed by the time this was first written; the level is retired, not missing.

## The Six Levels

The researcher's operational reality spans six levels.

| Level | Operational reality | Example concerns | State |
|-------|---------------------|------------------|-------|
| 1 | **Personal life** | calendar, health, travel, personal tasks | Minimal · by design |
| 2 | **Daily work** | meetings, email, decisions, documents | Thin |
| 3 | **Administration** | institutional procedures, progress reports, training, deadlines | Developed |
| 4 | **Teaching** | courses, lectures, students, exams | Developed |
| 5 | **Research** | projects, publications, grants, reviews | Developed |
| 6 | **Organization** | budget, infrastructure, licenses, compute | Developed |

## One Domain, Many Verticals

No additional root entities are required.

The seven Core Entities already describe every operational level. What changes are the Derived Types.

```
Project
 ├── Doctoral Thesis
 ├── Research Project
 ├── Teaching Course
 ├── Grant
 ├── Administrative Process
 └── Personal Goal

Document
 ├── Paper
 ├── Chapter
 ├── Lecture
 ├── Exam
 ├── Rubric
 ├── Presentation
 ├── Email
 └── Minutes

Activity
 ├── Reading
 ├── Writing
 ├── Teaching
 ├── Experiment
 ├── Meeting
 ├── Coding
 └── Administration

Knowledge
 ├── Concept
 ├── Hypothesis
 ├── Evidence
 ├── Decision
 └── Insight
```

Completing the domain means completing these verticals without changing the seven Core Entities or the frozen Logical Domain Model.

---

# Two Phases

```
Phase A — Complete the Domain
        ↓
Phase B — Build the System
```

---

# Phase A — Complete the Domain

## Objective

Complete the operational coverage of the system.

## Approach

Only Derived Types and Use Cases.

No new root entities.

No changes to the Logical Domain Model.

## Deliverables

| Vertical | Work | Status |
|----------|------|--------|
| Research | Derived Types + 12 Use Cases (Knowledge, Research, Writing) | Done |
| Teaching | Derived Types + 7 Use Cases | Done |
| Administration | Derived Types + 5 Use Cases | Done |
| Organization | Derived Types + 5 Use Cases | Done |
| Daily Work | Derived Types + 7 Use Cases (Planning, Communication) | Thin · sufficient for now |
| Personal | Derived Types + 3 Use Cases | Minimal · by design |

Completion criterion:

Every operational level has its Derived Types defined and representative Use Cases.

This criterion is met. See [DOMAIN_VERTICALS.md](DOMAIN_VERTICALS.md) for the index and [RESEARCH_VERTICAL.md](RESEARCH_VERTICAL.md), [TEACHING_VERTICAL.md](TEACHING_VERTICAL.md), [ADMINISTRATION_VERTICAL.md](ADMINISTRATION_VERTICAL.md) and [ORGANIZATION_VERTICAL.md](ORGANIZATION_VERTICAL.md) for the four verticals substantial enough to carry their own document. Remaining thinness in Daily Work and Personal is a scope decision, not a gap.

---

# Phase B — Build the System

## Objective

Turn the conceptual architecture into executable software.

## Architectural progression

```
Conceptual Models
        ↓
Interaction Model
        ↓
System Architecture
        ↓
Software Architecture
        ↓
Application Services
        ↓
Cognitive Runtime
(Memory · Context · Events · Knowledge)
        ↓
Infrastructure
(Graph · Database · MCP · LLMs · Storage)
        ↓
Interfaces
(Dashboard · Editors · Chat · Automation)
```

The Interaction Model is completed before Software Architecture because the way users interact with the system determines the architecture that follows.

Conversation is one interaction modality among several.

## Deliverables

Design the runtime architecture and implement a first vertical slice.

## Proving milestone

A complete operating cycle running end-to-end:

```
Domain change
        ↓
Domain Event
        ↓
Memory update
        ↓
Context rebuild
        ↓
Agent reasoning
        ↓
Recommendation / Action
        ↓
User accepts or rejects
        ↓
Domain updated
```

The system is considered validated when this cycle executes over real data with traceability and reproducible results.

---

# Document Plan

| # | Document | Phase | Purpose | Status |
|---|----------|-------|---------|--------|
| 1 | ROADMAP.md | — | Build sequence | This document |
| 2 | Domain verticals | A | Complete missing operational domains | Done |
| 3 | INTERACTION_MODEL.md | B | Define every interaction modality | Done |
| 4 | SYSTEM_ARCHITECTURE.md | B | Define authority, planes, trust boundaries and system invariants | Done · v1.0 |
| 5 | SOFTWARE_ARCHITECTURE.md | B | Translate system invariants into modules, contracts and runtimes | Done · v1.0 |
| 6 | DOCUMENTATION_ARCHITECTURE.md | B | Govern document authority, precedence and change impact | Done · v1.0 |
| 7 | IMPLEMENTATION_CONTEXTS.md | B | Define bounded coding-agent context bundles | Done · v1.0 |
| 8 | UML.md / Architecture Views | B | Derive structural, runtime and deployment-neutral implementation views | Next |
| 9 | DECISIONS.md (ADR) | B | Record significant architectural decisions | Pending |
| 10 | TECHNICAL_ARCHITECTURE.md | B | Select technologies and deployment topology | Pending |
| 11 | IMPLEMENTATION_PLAN.md | B | Validate architectural hypotheses through proving experiments | First pass |

DECISIONS.md also owns the promotion (or rejection) of the remaining candidate concepts:

- Decision
- Artifact
- Event
- Curate

---

# Milestones

| Milestone | Phase | Done when | Status |
|-----------|-------|-----------|--------|
| M1 · Domain breadth | A | All six operational levels are represented | Achieved |
| M2 · Interaction model | B | Manual, assisted, conversational and autonomous interactions are defined | Achieved |
| M3 · Reference architecture | B | System and Software Architecture completed | Achieved |
| M3.5 · Documentation operationalization | B | Ownership, precedence and bounded agent contexts defined | Achieved |
| M4 · Implementation views | B | Logical and software architecture projected to structural and runtime views | Next |
| M5 · Build plan | B | Components, epics and MVP defined | Pending |
| M6 · Proving slice | B | End-to-end operating cycle validated | Pending |

---

# Working Principles for This Stage

- **No new root entities.** Breadth comes from Derived Types.
- **Frozen stays frozen.** The conceptual and logical models remain implementation-independent.
- **Interaction precedes implementation.** The system's interaction model defines how software is structured.
- **Mechanisms enter at Software Architecture; concrete products enter only at Technical Architecture.**
- **Conversation is not the system.** It is one interaction modality among several.
- **Document authority is explicit.** Every change starts from the concern owner and follows declared downstream impact.
- **Bounded context for agents.** Coding agents load a task-specific bundle, never the entire repository by default.
- **Architectural decisions are recorded.** Significant technical decisions become ADRs.
- **Thin slice first.** Prove one complete operating cycle before broadening.
- **The objective is execution.** The conceptual architecture is considered complete; value now comes from building.
