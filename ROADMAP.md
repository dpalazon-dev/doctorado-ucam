# Roadmap

> **Status: Living.** Updated as phases progress. This document owns *what gets built, in what order, and why*. It is the plan of record.

## Purpose

The conceptual work is mature. This document turns it into a build plan.

It does not add models. It sequences the work that carries a frozen conceptual foundation into a real system — first by completing the *breadth* of the domain, then by building the *system* that runs it.

---

# Where We Stand

| Layer                     | Documents                                                        | State        |
|---------------------------|------------------------------------------------------------------|--------------|
| Conceptual                | Vision, Principles, System Model, Operational Model, Responsibilities, Domain Map, Domain Model, Knowledge Model, System Capabilities, Use Cases | Mature |
| Cognitive Architecture    | AI Architecture, Memory Model, Context Model, Event Model         | Complete     |
| Logical                   | Logical Domain Model                                             | Frozen · v1.0|
| **Domain breadth**        | Derived Types + Use Cases across all operational domains          | **Incomplete** |
| **System design**         | Software Architecture and below                                  | **Not started** |

Two gaps remain, and they define the next two phases: the domain has **collapsed to research**, and the **system has not been designed**.

---

# The Gap: Breadth

A researcher is not only a researcher.

The Vision and the Operational Model always said so — the Operational Model names Research, Project Management, Knowledge Management, Communication, Teaching and Institutional Responsibilities as parallel domains. But the Derived Types and the Use Cases drifted, over weeks of modelling the cognitive core, into a research-only view.

The Domain Model is not wrong. Its breadth is simply unfinished.

## The Six Levels

The researcher's operational reality spans six levels. Only the third and fifth are currently developed.

| Level | Operational reality      | Example concerns                                            | State       |
|-------|--------------------------|------------------------------------------------------------|-------------|
| 1     | **Personal life**        | calendar, personal tasks, health, travel, notes            | Absent      |
| 2     | **Daily work**           | email, meetings, documents, calls, decisions               | Thin        |
| 3     | **Doctorate**            | hypotheses, papers, experiments, bibliography, chapters     | Developed   |
| 4     | **Teaching**             | courses, lectures, practicals, tutoring, students, exams, rubrics | Absent |
| 5     | **Research**             | competitive projects, publications, reviews, congresses, grants | Developed |
| 6     | **Organization**         | resources, budget, licenses, GPUs, datasets, infrastructure | Thin       |

## One Domain, Many Verticals

The critical insight: **no new root entities are needed.** The seven Core Entities already support all six levels. What differs per level is the *Derived Types* — the specializations of those seven roots.

```
Project ── Doctoral Thesis · Research Project · Teaching Course · Grant
           Administrative Process · Personal Goal

Document ─ Paper · Thesis Chapter · Lecture · Exam · Rubric · Presentation
           Email · Meeting Minutes · Publication · Review

Activity ─ Reading · Writing · Experiment · Teaching · Meeting · Coding
           Review · Thinking · Administration

Person ─── Supervisor · Collaborator · Student · Reviewer · Coordinator

Resource ─ GPU · Dataset · License · Budget · Compute · Software

Knowledge ─ Concept · Hypothesis · Evidence · Decision · Insight · Methodology
```

Completing the domain means developing these verticals — Derived Types and their Use Cases — **without touching the seven roots or the frozen Logical Domain Model.** The model was built for exactly this.

---

# Two Phases

```
Phase A — Complete the Domain      breadth · the verticals
      ↓
Phase B — Build the System         depth · the implementation
```

Phase A finishes *what the system is and does* across the researcher's whole life. Phase B builds *how it runs*.

## Phase A — Complete the Domain

**Objective.** Develop the missing operational verticals so the domain covers the full six levels, not only research.

**Approach.** Derived Types and Use Cases only. No new root entities. No changes to the frozen Logical Domain Model.

**Deliverables.**

| Vertical                | Work                                                                 |
|-------------------------|----------------------------------------------------------------------|
| Daily Work & Communication | Use Cases for email, meetings, decisions, document flow            |
| Teaching                | Derived Types (Course, Lecture, Exam, Rubric, Student) + Use Cases   |
| Doctorate management    | Derived Types for thesis/administrative process + milestone tracking |
| Personal & Organization | Derived Types (Personal Goal, Budget, License) + light Use Cases     |

These extend `DOMAIN_MODEL.md` (Derived Types) and `USE_CASES.md` (new catalogues), and may be consolidated in a `DOMAIN_VERTICALS.md` if the volume warrants it.

**Completion criterion.** Every one of the six levels has at least its Derived Types named and one representative use case, so the system's scope visibly matches the Vision.

## Phase B — Build the System

**Objective.** Materialize the architecture — the point at which technology finally enters.

**The stack.**

```
AI Architecture            (done)
      ↓
Software Architecture      services · persistence · boundaries
      ↓
Memory Engine · Context Engine · Knowledge Graph · Event Bus
      ↓
MCP Layer                  tool/interface surface for the intelligence
      ↓
Interfaces                 dashboard · conversation · editors
```

**Deliverables.** The design and planning documents below, then a first working slice.

**The proving milestone.** A thin **vertical slice, end to end**: ingest documents → extract and link knowledge → answer a research question with provenance. Validated as the deep-research report proposed — retrieval relevance ≥ 85%, measurable reduction in manual effort, reproducible answers across document order.

**MVP.** The smallest slice that runs the operating cycle (change → event → context → act → change) over real data in one vertical.

---

# Document Plan

The documents that carry the project from here, in priority order.

| # | Document                  | Phase | Purpose                                                           | Status   |
|---|---------------------------|-------|------------------------------------------------------------------|----------|
| 1 | `ROADMAP.md`              | —     | What gets built, in what order, why                              | This doc |
| 2 | Domain verticals          | A     | Derived Types + Use Cases for the missing levels                 | Next     |
| 3 | `SOFTWARE_ARCHITECTURE.md`| B     | How the conceptual and logical models are materialized           | Pending  |
| 4 | `UML.md` / views          | B     | Implementation views derived from the Logical Domain Model       | Pending  |
| 5 | `IMPLEMENTATION_PLAN.md`  | B     | Epics, components, milestones, MVP                               | Pending  |
| 6 | `DECISIONS.md` (ADR)      | B     | Architectural decisions, recorded as they are taken              | Pending  |

`DECISIONS.md` also resolves the four concepts left pending by the Logical Domain Model — **Curate** (capability), **Artifact**, **Decision** and **Event** — each promoted, or not, by a recorded decision.

---

# Milestones

| Milestone | Phase | Done when                                                                     |
|-----------|-------|-------------------------------------------------------------------------------|
| M1 · Domain breadth | A | All six levels have Derived Types and at least one use case each             |
| M2 · Reference architecture | B | `SOFTWARE_ARCHITECTURE.md` defines services, persistence and boundaries |
| M3 · Implementation views | B | Logical model projected to schema, graph and types                        |
| M4 · Build plan | B | Epics, components and MVP scoped in `IMPLEMENTATION_PLAN.md`                   |
| M5 · Proving slice | B | The end-to-end vertical slice runs and meets its validation criteria         |

---

# Working Principles for This Stage

- **No new root entities.** Breadth comes from Derived Types, never from new roots.
- **Frozen stays frozen.** The Logical Domain Model and the conceptual layer do not change for implementation, AI or storage reasons.
- **Technology enters at Software Architecture.** Not before. Concrete products are chosen in ADRs.
- **Decisions are recorded.** Every significant architectural choice becomes an ADR in `DECISIONS.md`.
- **Thin slice first.** Prove the whole cycle on one vertical before broadening — depth over surface area.
- **The plan lands the models; it does not add more.** From here, value comes from building, not from modelling.
