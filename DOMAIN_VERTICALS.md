# Domain Verticals

> This document completes the breadth of the domain.
>
> The Domain Model defines the universal concepts.
> This document defines how those concepts specialize across the operational
> realities of the researcher.
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
Doctorate
Projects
Administration
Personal

```

are different views over the same domain.

---

# Vertical Overview

| Vertical | Objective |
|----------|-----------|
| Personal | Personal organization and life management |
| Daily Work | Daily operational execution |
| Doctorate | Thesis management |
| Research | Scientific work |
| Teaching | Academic teaching |
| Organization | Infrastructure and institutional management |

---

# Vertical 1 — Personal

Represents the individual's own life outside institutional work.

## Typical Projects

- Personal Goal
- Trip
- Health Plan

## Typical Documents

- Personal Notes
- Receipts
- Travel Documents

## Typical Activities

- Exercise
- Personal Review
- Planning

## Typical Knowledge

- Reflection
- Habit
- Personal Insight

## Representative Use Cases

- Personal planning
- Habit tracking
- Weekly review

---

# Vertical 2 — Daily Work

Represents operational execution.

## Projects

- Administrative Process

## Documents

- Email
- Meeting Minutes
- Decision Log

## Activities

- Meeting
- Phone Call
- Coding
- Writing

## Knowledge

- Decision
- Operational Note

Representative Use Cases

- Process email
- Record meeting
- Track decisions

---

# Vertical 3 — Doctorate

Represents management of the doctoral journey itself.

This is different from research.

Research produces science.

Doctorate produces a thesis.

## Projects

- Doctoral Thesis

## Documents

- Thesis Chapter
- Draft
- Bibliography
- Supervisor Feedback

## Activities

- Thesis Writing
- Literature Review
- Supervisor Meeting

## Knowledge

- Chapter Insight
- Thesis Decision

Representative Use Cases

- Produce chapter
- Prepare defense
- Track thesis milestones

---

# Vertical 4 — Research

Represents scientific production.

## Projects

- Research Project
- Grant

## Documents

- Paper
- Review
- Dataset
- Protocol

## Activities

- Experiment
- Reading
- Analysis

## Knowledge

- Hypothesis
- Evidence
- Methodology
- Insight

Representative Use Cases

- Validate hypothesis
- Produce publication
- Literature synthesis

---

# Vertical 5 — Teaching

Represents academic teaching.

## Projects

- Course
- Subject

## Documents

- Lecture
- Slides
- Exam
- Rubric

## Activities

- Lecture
- Tutoring
- Exam Correction

## Knowledge

- Teaching Material
- Student Feedback

Representative Use Cases

- Prepare lecture
- Evaluate students
- Publish grades

---

# Vertical 6 — Organization

Represents institutional and infrastructure management.

## Projects

- Infrastructure Upgrade
- Budget Allocation

## Documents

- License
- Contract
- Budget

## Activities

- Procurement
- Maintenance

## Resources

- GPU
- Server
- API
- License
- Dataset

Representative Use Cases

- Allocate resources
- Renew licenses
- Monitor costs

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
│
▼
Doctorate
│
Thesis Chapter
│
▼
Teaching
│
Lecture

```

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

This document is complete when every vertical defines:

- representative Projects
- representative Documents
- representative Activities
- representative Knowledge
- representative Use Cases

without introducing new Core Entities.

El siguiente documento ya sería **`SOFTWARE_ARCHITECTURE.md`**, que es donde empieza realmente el diseño de cómo construir ResearcherOS. Ahí es donde empezaremos a hablar de UI, Backend, Knowledge Graph, agentes, MCP, memoria, eventos, motores, etc., y cómo todas las piezas encajan en una arquitectura coherente.
