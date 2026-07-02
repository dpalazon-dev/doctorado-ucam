# Knowledge Model

## Purpose

This document defines how Knowledge exists and behaves inside Doctorado_UCAM.

Knowledge is the primary intellectual asset of the platform.

Every other document treats Knowledge as a single concept.

This document explains what that concept actually means: how knowledge is born, how it evolves, how it relates to other entities and how it is represented.

For the definition of Knowledge as a domain entity, see Domain Model → Knowledge.

---

# What is Knowledge

Knowledge represents meaning.

A Document stores information. Knowledge captures what that information *means* to the researcher.

The same paper produces different knowledge for different researchers, because knowledge is interpretation, not storage.

Knowledge is:

- semantic rather than physical
- connected rather than isolated
- reusable across projects
- continuously evolving

Knowledge is never a file. A file is a Document.

---

# How Knowledge is Born

Knowledge is not created directly.

It emerges from a pipeline that transforms raw artifacts into connected meaning.

```
Paper
        │
        ▼
Document
        │
Extract
        ▼
Knowledge
        │
Relate
        ▼
Knowledge Graph
        │
Reason
        ▼
Insights
```

Each step has a distinct responsibility:

- A **Paper** enters the system as a **Document** (Acquire).
- Meaning is **extracted** from the Document, producing **Knowledge** (Understand).
- Knowledge is **related** to existing knowledge, forming a **Knowledge Graph** (Organize).
- The system **reasons** over the graph, producing **Insights** (Reason).

The corresponding capabilities are defined in System Capabilities.

---

# Can Knowledge Exist Without a Document?

Yes.

Documents are the most common origin of knowledge, but not the only one.

Knowledge may also be born from:

- Activities (a meeting, an experiment, a conversation)
- direct capture (an idea written by the researcher)
- reasoning (an insight synthesized from existing knowledge)
- Artificial Intelligence operating over the system state

A Document is one possible *source* of Knowledge, never a requirement for it.

This is why Knowledge and Document are separate entities.

---

# How Knowledge Evolves

Knowledge is not static.

It changes as the researcher's understanding changes.

Knowledge evolves by:

- gaining new relationships to other knowledge
- being refined, corrected or contradicted by new evidence
- being promoted from a note into a concept, hypothesis or methodology
- accumulating provenance as it is reused

Older knowledge is never silently overwritten.

Its evolution is preserved through Memory (see Domain Map → Memory), so past reasoning remains recoverable.

---

# Knowledge and Activities

Knowledge and Activities are deeply connected.

Activities *produce* Knowledge. Reading, thinking, experimenting and discussing all generate new meaning.

Knowledge *informs* Activities. Existing knowledge shapes what the researcher does next.

This creates a continuous cycle:

```
Activity → Knowledge → Activity → Knowledge → …
```

An Activity is the operational moment in which Knowledge is created or applied.

---

# Knowledge and Projects

Projects organize work, but they do not own Knowledge.

Knowledge is intentionally reusable across Projects.

A concept learned in one Project remains available to every other Project.

Projects therefore *reference* Knowledge rather than containing it.

This preserves a single source of truth and prevents the same understanding from being duplicated per project.

---

# How Knowledge is Represented

Knowledge is represented as a connected graph rather than as isolated records.

Each unit of knowledge is a node.

Each meaningful relationship is an edge.

The value of the system grows with the density of these relationships, not merely with the amount of stored information.

This graph is not an independent store.

It is one perspective over the Unified System State (see System Model → Unified System State).

---

# Relationship to Other Documents

To avoid duplication, this document does not redefine shared concepts.

- Context — see System Model → Context
- Unified System State — see System Model → Unified System State
- Assets — see System Model → Core Assets
- Capabilities used by the knowledge pipeline — see System Capabilities

The Knowledge Model owns only one concept: Knowledge itself.
