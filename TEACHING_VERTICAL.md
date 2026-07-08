# Teaching Vertical

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 3 — Functional and Vertical Specification |
| **Normative status** | Canonical within Teaching vertical scope |
| **Authoritative for** | Teaching-specific scope, derived types, authorship boundaries and mapping to canonical use cases. |
| **Not authoritative for** | Copied use-case definitions, Core Entity changes, grading implementation or UI workflows. |
| **Required reading** | `DOMAIN_VERTICALS.md`, `DOMAIN_MODEL.md`, `USE_CASES.md`. |
| **Downstream documents** | Teaching implementation slices, `LOGICAL_DOMAIN_MODEL.md` impact reviews and `ROADMAP.md` status. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

> Materializes the Teaching vertical named in [DOMAIN_VERTICALS.md](DOMAIN_VERTICALS.md).
>
> Prepare, deliver and evaluate university teaching.

---

# Purpose

Teaching reuses the same knowledge, documents and people as Research rather than forming an isolated workflow. Courses draw on the researcher's own knowledge; student work feeds back as evaluated understanding and, over editions, as accumulated teaching experience.

---

# Scope

Teaching covers course preparation, delivery, assessment and reflection. It does not cover the institutional side of the doctorate — that is [ADMINISTRATION_VERTICAL.md](ADMINISTRATION_VERTICAL.md) — nor the scientific content Teaching draws on, which is [RESEARCH_VERTICAL.md](RESEARCH_VERTICAL.md).

Two use cases in this vertical carry a deliberate authorship boundary, recorded directly in the catalogue: grading (UC-TE04) is prepared by the system and always approved by the teacher, never assigned autonomously; supervising a final degree project (UC-TE05) applies only when acting as co-director alongside the thesis supervisor, not as sole supervisor.

---

# Derived Types

## Projects
- Course

## Documents
- Teaching Guide
- Lecture
- Slides
- Exam / Assignment
- Rubric
- Submission
- Course Review

## Activities
- Teaching Session

## Knowledge
- Teaching Insight
- Evaluation *(not yet formalized in the Domain Model — plays the same structural role for assessment that Evidence plays for the Research Spine; under observation, see USE_CASES.md → Evolution)*

## People
- Student

---

# Use Cases

Full specifications live in [USE_CASES.md](USE_CASES.md) → Teaching.

| Use Case | Intention |
|----------|-----------|
| UC-TE01 · Prepare a Course | Set up a course as an organized initiative, reusing existing knowledge |
| UC-TE02 · Prepare a Lecture | Turn a session slot into ready-to-deliver teaching material |
| UC-TE03 · Design an Assessment | Create an exam or assignment with a rubric aligned to learning outcomes |
| UC-TE04 · Grade Student Work | Evaluate submissions against a rubric with consistent, justified feedback |
| UC-TE05 · Supervise a Final Project | Direct a student's TFG/TFM from proposal to defence |
| UC-TE06 · Track a Course | Monitor delivery, assessment and workload throughout the term |
| UC-TE07 · Review a Course | Consolidate a completed course's experience into reusable teaching knowledge |

---

# Relationships

Teaching draws its material from Research's Knowledge and Documents — UC-TE02 retrieves "related knowledge, documents and previous lectures." Course Reviews (UC-TE07) feed Teaching Insight back into the graph, available to future editions the same way any other Knowledge is. Course records and training obligations surface in Administration's annual progress report (UC-AD01).
