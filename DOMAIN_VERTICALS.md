# Domain Verticals

> This document is the index of every vertical in the domain.
>
> The Domain Model defines the universal concepts.
> This document defines how those concepts specialize across the operational
> realities of the researcher — inline for the two that stay thin by design,
> by reference for the four substantial enough to carry their own document.
>
> No new Core Entities are introduced.
> Every vertical is expressed exclusively through Derived Types and Use Cases.

---

# Purpose

The Domain Model intentionally contains only seven Core Entities.

Those entities are sufficiently general to describe every operational area of the researcher.

Different domains are not represented by different entity models.

They are represented by different specializations.

This document defines those specializations.

---

# One Domain

```
             Seven Core Entities
```

Project
Document
Knowledge
Task
Activity
Person
Resource

```
remain identical everywhere.

Only the vocabulary changes.
```

Research
Teaching
Administration
Organization
Personal
Daily Work

```
are different views over the same domain.
```

---

# Vertical Overview

| Vertical | Objective | Status | Detail |
|----------|-----------|--------|--------|
| Personal | Personal organization and life management | Minimal · by design | inline below |
| Daily Work | Daily operational execution | Thin | inline below |
| Administration | Institutional obligations of the doctorate | Developed | [ADMINISTRATION_VERTICAL.md](ADMINISTRATION_VERTICAL.md) |
| Research | Scientific work | Developed | [RESEARCH_VERTICAL.md](RESEARCH_VERTICAL.md) |
| Teaching | Academic teaching | Developed | [TEACHING_VERTICAL.md](TEACHING_VERTICAL.md) |
| Organization | Infrastructure and institutional management | Developed | [ORGANIZATION_VERTICAL.md](ORGANIZATION_VERTICAL.md) |

Two further groups of Use Cases in [USE_CASES.md](USE_CASES.md) do not appear above because they are not verticals: **People**, **Context** and **Traceability** operate identically regardless of which vertical produced the entity they touch. They are infrastructure — the behavioural expression of the Context, Search & Retrieval and Memory domains already named in [DOMAIN_MAP.md](DOMAIN_MAP.md) — not specializations of it.

---

# Vertical 1 — Personal

Represents the individual's own life outside institutional work.

Personal life is the lightest vertical by design. It introduces almost nothing new: a personal task is a Task, a reminder is a Task, an appointment is a Task with a time, a personal contact is a Person, a jotting is a Note (Document), and a personal goal is a Project. No personal-specific machinery exists, and none is planned — the same capabilities that serve research serve errands and goals unchanged.

## Typical Projects

- Personal Goal

## Typical Documents

- Note

Personal introduces no specialization of Activity or Knowledge — commitments and reflections reuse Task, Activity and Knowledge unspecialized.

## Use Cases

| Use Case | Intention |
|----------|-----------|
| UC-PL01 · Capture a Personal Note or Reminder | Record a personal item with near-zero friction and route it correctly |
| UC-PL02 · Schedule a Personal Appointment | Register a time-bound commitment and reconcile it against existing obligations |
| UC-PL03 · Track a Personal Goal | Define a personal goal and review its progress over time |

See [USE_CASES.md](USE_CASES.md) → Personal.

---

# Vertical 2 — Daily Work

Represents operational execution: meetings, email, decisions, documents.

## Typical Documents

- Email
- Meeting Minutes
- Decision Log

## Typical Activities

- Meeting
- Phone Call

## Typical Knowledge

- Decision

Daily Work introduces no Project of its own — it attaches to whatever project the work serves. (Administrative Process, sketched here in earlier drafts, is now owned by [ADMINISTRATION_VERTICAL.md](ADMINISTRATION_VERTICAL.md).)

## Use Cases

| Use Case | Intention |
|----------|-----------|
| UC-P01 · Manage Research Tasks | Create, prioritise and review tasks, kept consistent with their projects |
| UC-P02 · Plan Research Activities | Prepare short- and medium-term plans aligned with milestones |
| UC-P03 · Review Research Progress | Summarise completed work, blockers and evolution over a period |
| UC-C01 · Process Incoming Email | Extract operational content from incoming email instead of triaging by hand |
| UC-C02 · Prepare Communication | Generate context-aware drafts for approval |
| UC-C03 · Prepare a Meeting | Assemble every piece of relevant context before an interaction |
| UC-C04 · Record an Interaction | Transform conversations, meetings or calls into structured knowledge |

See [USE_CASES.md](USE_CASES.md) → Planning, Communication.

---

# Vertical 3 — Administration

The institutional obligations that surround the doctorate without directly advancing it: annual progress reports, committee decisions, mandatory training credits, bureaucratic procedures and the deadlines that govern them.

Not part of the original six-vertical sketch — it emerged directly from the Use Case catalogue and is formalized here for the first time. Together with Writing (see [RESEARCH_VERTICAL.md](RESEARCH_VERTICAL.md)), it absorbs what this document used to call **Doctorate**: producing a chapter is Writing; everything institutional around the thesis is this vertical.

Full specification: [ADMINISTRATION_VERTICAL.md](ADMINISTRATION_VERTICAL.md).

---

# Vertical 4 — Research

Scientific production: hypotheses, experiments, evidence, publications.

Full specification: [RESEARCH_VERTICAL.md](RESEARCH_VERTICAL.md).

---

# Vertical 5 — Teaching

Academic teaching: courses, lectures, assessment, supervision.

Full specification: [TEACHING_VERTICAL.md](TEACHING_VERTICAL.md).

---

# Vertical 6 — Organization

Infrastructure and institutional resource management: compute, datasets, licenses, budgets.

Full specification: [ORGANIZATION_VERTICAL.md](ORGANIZATION_VERTICAL.md).

---

# Cross-Vertical Principles

The verticals are not isolated.

Knowledge is shared.

Resources are shared.

Projects interact.

Documents move between verticals.

Example

```

Research
│
▼
Paper
│
▼
Teaching
│
Lecture

```

Knowledge created during research can later support teaching.

Teaching may generate new knowledge.

Administration enables both.

The system therefore models one connected graph rather than independent modules.

---

# Cross-Vertical Workflows

Many workflows naturally span several verticals.

Example

```

Research
│
▼
Publication
├──────────────┐
▼              ▼
Teaching       Administration
│              │
Lecture        Progress Report

```

A single Publication feeds two independent branches: its knowledge is reused to prepare a Lecture in Teaching, while Administration reports on the same work in the annual Progress Report — producing only a Document, changing no upstream state.

The same knowledge evolves through multiple operational contexts.

The graph preserves provenance across every transition.

---

# AI Perspective

The AI does not reason about verticals.

It reasons over the unified domain.

Verticals exist primarily to:

- organize the UI
- specialize use cases
- specialize derived types
- tailor workflows
- adapt recommendations

Internally, every vertical resolves back to the seven Core Entities.

This allows knowledge discovered in one operational context to become immediately reusable everywhere else.

---

# Completion Criteria

Met. Every vertical defines Derived Types and Use Cases without introducing new Core Entities — four with enough substance to carry their own document ([RESEARCH_VERTICAL.md](RESEARCH_VERTICAL.md), [TEACHING_VERTICAL.md](TEACHING_VERTICAL.md), [ADMINISTRATION_VERTICAL.md](ADMINISTRATION_VERTICAL.md), [ORGANIZATION_VERTICAL.md](ORGANIZATION_VERTICAL.md)), two — Personal and Daily Work — thin by design rather than by omission.
