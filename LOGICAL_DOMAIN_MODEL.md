# Logical Domain Model

## Purpose

This document is the canonical logical specification of Doctorado_UCAM.

The Domain Model introduced the concepts that exist. This document specifies them precisely: their attributes, their states, their relationships, their cardinalities and their invariants.

It is the bridge between the conceptual domain and its implementation. Where the Domain Model says *a Project exists*, this document says *a Project has these attributes, these states, these relationships and these constraints*.

It is a **logical** model, not a physical one. It defines structure, not storage. It names no database, no language and no framework. Those belong to the Software Architecture.

It owns one thing: the exact structure of the domain.

This first version establishes the overview — the master view, the boundaries and the specification format. The detailed specification of each entity is developed from that overview, block by block.

---

# From Conceptual to Logical

The architecture moves through three levels of structural precision.

```
Domain Model          what exists                    (conceptual)
      ↓
Logical Domain Model  exactly how it is structured   (logical)   ← this document
      ↓
Software Architecture how it is built and stored     (physical)
```

Each level refines the one above without contradicting it.

The Domain Model is deliberately minimal and stable — seven Core Entities and a handful of Derived Types. This document does not add concepts. It gives the existing concepts the structural precision an implementation will require, while remaining independent of any particular implementation.

If a future need cannot be expressed by specifying an existing entity, it is a signal to revisit the Domain Model — not to invent structure here.

---

# Canonical Specification, Derived Views

This document is the single authoritative specification of the domain's structure.

Every other structural representation of the system is a **projection** derived from it.

```
                       Logical Domain Model
                      (the canonical spec)
                              │
        ┌──────────┬──────────┼──────────┬─────────────┐
        ▼          ▼          ▼          ▼             ▼
   UML class   relational   knowledge   JSON        language
   diagram      schema      graph model  schema      types
                                                   (TS · Python)
```

A UML diagram is a view of this model. A relational schema is a view. A knowledge-graph model is a view. A JSON schema for an API is a view. Types in TypeScript or Python are views.

None of them is the source. This document is the source; they are derived from it.

This is the same discipline the whole architecture follows — the domain is the truth, and storage is a projection — now applied to the specification itself. It carries two obligations:

- The model must be **precise enough** to derive each of those views: attributes with types, explicit cardinalities, defined state machines, stated invariants.
- The model must be **neutral enough** to derive *all* of them: nothing here may assume a relational store, a graph store or any particular language.

A diagram that drifts from this document is wrong. This document does not drift to match a diagram.

---

# The Master View

The overview has two halves: the graph of the domain itself, and the way the four models of the cognitive architecture operate over it.

## The Domain Graph

The seven Core Entities and their primary relationships.

```
   Person ──▶ Project ──▶ Task ──▶ Activity ──▶ Knowledge
                             ▲                      │
                             └──────── informs ─────┘

   Document ──supports──▶ Knowledge      Activity ──consumes──▶ Resource
```

The diagram shows the primary operational flow. The complete and authoritative set of relationships is the list below — the logical graph, independent of any drawing.

- Person — *participates in* → Project
- Person — *performs* → Activity
- Project — *organizes* → Task
- Project — *organizes* → Document
- Project — *references* → Knowledge
- Task — *becomes* → Activity
- Task — *produces* → Knowledge
- Activity — *produces* → Knowledge
- Activity — *consumes* → Resource
- Document — *supports* → Knowledge
- Knowledge — *informs* → Task

These relationships restate, at logical precision, the Relationship Philosophy of the Domain Model. The Research Spine (Hypothesis → Experiment → Evidence) specializes this graph through Derived Types and is specified with them.

## The System of Models

The domain does not stand alone. The four models of the Cognitive Architecture operate over it — and only over it.

```
                          AI OPERATING LAYER
           Capture · Curate · Link · Reason · Plan · Write · Audit · Notify
                 │             │             │              │
             operates on    recalls      assembles      reacts to
                 │             │             │              │
                 ▼             ▼             ▼              ▼
   ┌───────────────┐   ┌─────────────┐ ┌────────────┐ ┌────────────┐
   │    DOMAIN     │   │   MEMORY    │ │  CONTEXT   │ │   EVENTS    │
   │  persistent   │◀──│ recall over │ │ assembled  │ │ emitted by  │
   │  the entity   │   │ the domain  │ │ from the   │ │ the domain's│
   │    graph      │   │             │ │   domain   │ │  changes    │
   └───────────────┘   └─────────────┘ └────────────┘ └────────────┘
```

Every cognitive model resolves back to the domain. Memory recalls over it. Context is assembled from it. Events are emitted by its changes. The AI Operating Layer operates on it. The domain is the one surface they all share — the logical expression of the single-source-of-truth principle.

---

# Persistent and Derived

A single line runs through the whole logical model, and drawing it correctly is the point of this overview.

```
   PERSISTENT (the source of truth)        DERIVED (produced, never stored as truth)
   ─────────────────────────────────       ────────────────────────────────────────
   The Core Entities:                       Context           (Context Model)
     Project · Knowledge · Document          Recall results    (Memory Model)
     Person · Task · Activity · Resource     Generated Artifacts (AI Architecture)
```

Only the Core Entities are persistent. They are what the system stores and what it holds true.

Everything the Cognitive Architecture produces on top of them — a constructed context, the result of a recall, a generated report — is **derived**. It is computed from the domain when needed and is never a source of truth. If it is ever cached, the cache is a projection, reconstructible from the domain.

This distinction is why the logical model can specify the domain exhaustively and leave the cognitive products unspecified as stored structures: they have no persistent structure to specify. They are functions of the domain, not additions to it.

*(Events sit at this boundary: a domain change, recorded. Whether an Event becomes a persisted, first-class record is one of the open promotions below.)*

---

# What This Document Will Specify

From this overview, the model is developed in eight blocks. Each block sharpens one aspect of the entities above.

1. **Core Entities** — each entity's attributes, with logical types.
2. **Relationships** — the logical graph above, each edge given direction and meaning.
3. **Entity Lifecycle** — the state machine of each entity that has one.
4. **Cardinalities** — how many of each entity may relate to another.
5. **Aggregates** — the aggregate roots (in the DDD sense) and what belongs to each.
6. **Domain Constraints** — the invariants that must always hold.
7. **Derived Objects** — what is computed rather than stored (Context, recall, artifacts), specified as derivations, not records.
8. **Logical Views** — focused diagrams over the model (a Knowledge view, a Project view, an AI view), each a projection of the one specification.

To calibrate the intended precision, here is the target format applied to one entity — illustrative, to be finalized in the Core Entities and Lifecycle blocks.

```
Knowledge
  attributes
    id           identifier
    title        text
    summary      text
    confidence   degree
    state        enum  (see lifecycle)
    provenance   origin reference
    created_at   timestamp
    updated_at   timestamp
  lifecycle
    Draft ──▶ Candidate ──▶ Validated ──▶ Deprecated ──▶ Archived
  invariants
    a Validated Knowledge must retain its provenance
    Knowledge is never deleted; it is Deprecated or Archived
```

The types shown (`text`, `degree`, `enum`, `timestamp`) are logical, not physical. How a `degree` or a `timestamp` is stored is a decision for the Software Architecture.

---

# Scope and Boundaries

**In scope.** The seven Core Entities and the formalized Derived Types (the Research Spine: Hypothesis, Experiment, Evidence), specified to logical precision.

**Pending.** Four concepts surfaced by the cognitive architecture are candidates for the Domain Model but are not yet promoted: **Curate** (a capability), **Artifact**, **Decision** and **Event**. They are specified here only once promotion is decided — deliberately, through a recorded decision. Until then they are named as pending, never half-specified.

**Out of scope.** Everything physical: storage, indexing, database choice, language, service boundaries. Those belong to the Software Architecture, which will treat this model as its specification.

---

# Relationship to Other Documents

To avoid duplication, this document does not redefine shared concepts.

- What the entities *are* — Domain Model
- What Knowledge *means* and how it evolves — Knowledge Model
- The behaviour that operates on the entities — System Capabilities; AI Architecture
- The cognitive products that derive from the entities — Memory Model; Context Model; Event Model
- The state changes that behaviour produces — Use Cases → State Changes

The Logical Domain Model owns one thing: the exact structure of the domain, from which every schema, diagram and type is derived.

---

# How This Document Grows

This document is built from its overview outward.

The master view fixed here is the single mental model of the system's structure. Each subsequent block develops one part of it — attributes, cardinalities, states, invariants — without ever contradicting the whole.

This order is deliberate. A consistent overview, developed inward, keeps one representation of the system. It is the structural answer to the same requirement that shaped every document before it: coherence over convenience.
