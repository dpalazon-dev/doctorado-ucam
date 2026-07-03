# Roadmap

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
| Logical | Logical Domain Model | Frozen · v1.0 |
| **Domain breadth** | Derived Types + Use Cases across all operational domains | **Incomplete** |
| **Interaction model** | System interaction modalities | **Not started** |
| **System design** | Software Architecture and below | **Not started** |

Two gaps remain.

The first is **breadth**: the domain currently over-represents research.

The second is **execution**: although the cognitive architecture is complete, the runtime architecture of the system has not yet been designed.

---

# The Gap: Breadth

A researcher is not only a researcher.

The Vision and the Operational Model always said so — the Operational Model names Research, Project Management, Knowledge Management, Communication, Teaching and Institutional Responsibilities as parallel domains. But the Derived Types and the Use Cases drifted, over weeks of modelling the cognitive core, into a research-only view.

The Domain Model is not wrong. Its breadth is simply unfinished.

## The Six Levels

The researcher's operational reality spans six levels.

| Level | Operational reality | Example concerns | State |
|-------|---------------------|------------------|-------|
| 1 | **Personal life** | calendar, health, travel, personal tasks | Absent |
| 2 | **Daily work** | meetings, email, decisions, documents | Thin |
| 3 | **Doctorate** | thesis, bibliography, hypotheses, chapters | Developed |
| 4 | **Teaching** | courses, lectures, students, exams | Absent |
| 5 | **Research** | projects, publications, grants, reviews | Developed |
| 6 | **Organization** | budget, infrastructure, licenses, compute | Thin |

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

| Vertical | Work |
|----------|------|
| Daily Work | Meetings, communication, decision flow |
| Teaching | Course, lecture, student, assessment |
| Doctorate | Thesis management, milestones |
| Personal | Personal goals and routines |
| Organization | Resources, budgets, infrastructure |

Completion criterion:

Every operational level has its Derived Types defined and representative Use Cases.

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
| 2 | Domain verticals | A | Complete missing operational domains | Next |
| 3 | INTERACTION_MODEL.md | B | Define every interaction modality | Pending |
| 4 | SOFTWARE_ARCHITECTURE.md | B | Translate conceptual models into runtime architecture | Pending |
| 5 | UML.md / Views | B | Derived implementation views | Pending |
| 6 | IMPLEMENTATION_PLAN.md | B | Components, milestones, MVP | Pending |
| 7 | DECISIONS.md (ADR) | B | Record architectural decisions | Pending |

DECISIONS.md also owns the promotion (or rejection) of the remaining candidate concepts:

- Decision
- Artifact
- Event
- Curate

---

# Milestones

| Milestone | Phase | Done when |
|-----------|-------|-----------|
| M1 · Domain breadth | A | All six operational levels are represented |
| M2 · Interaction model | B | Manual, assisted, conversational and autonomous interactions are defined |
| M3 · Reference architecture | B | Software Architecture completed |
| M4 · Implementation views | B | Logical model projected to implementation views |
| M5 · Build plan | B | Components, epics and MVP defined |
| M6 · Proving slice | B | End-to-end operating cycle validated |

---

# Working Principles for This Stage

- **No new root entities.** Breadth comes from Derived Types.
- **Frozen stays frozen.** The conceptual and logical models remain implementation-independent.
- **Interaction precedes implementation.** The system's interaction model defines how software is structured.
- **Technology enters only at Software Architecture.**
- **Conversation is not the system.** It is one interaction modality among several.
- **Architectural decisions are recorded.** Significant technical decisions become ADRs.
- **Thin slice first.** Prove one complete operating cycle before broadening.
- **The objective is execution.** The conceptual architecture is considered complete; value now comes from building.
