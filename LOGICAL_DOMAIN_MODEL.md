# Logical Domain Model

> **Status: Stable · v1.1.** This model changes only when the *domain* changes — never for implementation, AI, storage or tooling. Additions (an attribute, an enum, a Derived Type) are welcome; structural churn is not.
>
> **v1.1** — added Vertical Lifecycle Extensions (Administrative Process, Submission, Final Project) so the Teaching, Administration and Organization verticals have canonical states to reuse instead of inventing their own; fixed Experiment's Block 3 casing to match its own specialized-lifecycle convention. No structural change.

## Purpose

This document is the canonical logical specification of ResearchOS.

The Domain Model introduced the concepts that exist. This document specifies them precisely: their attributes, their types, their relationships, their cardinalities, their lifecycles and their invariants.

It is the bridge between the conceptual domain and its implementation. Where the Domain Model says *a Project exists*, this document says *a Project has these attributes, these states, these relationships and these constraints*.

It is a **logical** model, not a physical one. It defines structure, not storage. It names no database, no language and no framework. Those belong to the Software Architecture.

It owns one thing: the exact structure of the domain.

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

The Domain Model is deliberately minimal and stable — seven Core Entities and a handful of Derived Types. This document does not add concepts. It gives the existing concepts the structural precision an implementation requires, while remaining independent of any particular implementation.

If a future need cannot be expressed by specifying an existing entity, that is a signal to revisit the Domain Model — not to invent structure here.

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

This is the same discipline the whole architecture follows — the domain is the truth, storage is a projection — now applied to the specification itself. It carries two obligations.

- The model must be **precise enough** to derive each of those views: attributes with types, explicit cardinalities, defined state machines, stated invariants.
- The model must be **neutral enough** to derive *all* of them: nothing here may assume a relational store, a graph store or any particular language.

A diagram that drifts from this document is wrong. This document does not drift to match a diagram.

---

# The System at a Glance

This is the single mental model of ResearchOS: the seven-entity domain at its core, and the four models of the Cognitive Architecture operating over it. Everything specified in the rest of this document is a part of this one picture.

```
┌──────────────────────── AI OPERATING LAYER ───────────────────────────┐
│      Capture · Curate · Link · Reason · Plan · Write · Audit · Notify   │
└─────────┬──────────────────────┬───────────────────────┬───────────────┘
     reacts to               uses context            acts on the
      events                      │                     domain
          │                       │                       │
  ┌───────▼────────┐    ┌─────────▼──────────┐            │
  │  EVENT MODEL   │    │   CONTEXT MODEL    │            │
  │  what changes  │    │ what is assembled  │            │
  │                │    │                    │            │
  │ Created Updated│    │ Context Builder    │            │
  │ Derived Linked │    │  (derived,         │            │
  │ + transitions  │    │   per task)        │            │
  └───────┬────────┘    └─────────┬──────────┘            │
          │                       │ assembled from         │
          │             ┌─────────▼──────────┐            │
          │             │   MEMORY MODEL     │            │
          │             │  what persists     │            │
          │             │                    │            │
          │             │ world:    Semantic │            │
          │             │           Episodic │            │
          │             │ operating: Working │            │
          │             │         Procedural │            │
          │             └─────────┬──────────┘            │
    emitted by              recalls over                  │
   domain changes                 │                        │
          │                       │                        │
          ▼                       ▼                        ▼
┌──────────────────────── DOMAIN · source of truth ─────────────────────┐
│                                                                        │
│  Person ─▶ Project ─▶ Task ─▶ Activity ─▶ Knowledge ◀─ Document        │
│                        ▲          │                                    │
│                        └─ informs ┘        Activity ─consumes─▶ Resource│
│                                                                        │
│                        complete graph → Block 2                       │
└───────────────────────────────┬────────────────────────────────────────┘
                                 │  the AI layer writes back;
                                 │  a write is a change → a new Event
                                 └──────────────▶  ⟳  the cycle repeats
```

**Reading the picture.**

- **The Domain** is persistent — the single source of truth. Only the seven entities and their Derived Types are stored (Block 1).
- **Memory, Context and Events** operate *over* the domain and add no truth of their own. Memory recalls over it — and world memory *is* the domain (Semantic = Knowledge, Episodic = Activities). Context is assembled per task by the Context Builder, a derived artifact that is never stored. Events are emitted by the domain's changes.
- **The AI Operating Layer** reacts to events, uses context and acts on the domain — and its actions are themselves changes, which are new events.

**The boundary that matters most** — persistent versus derived — runs horizontally across the picture. The Domain, below the line, is truth. Everything the models produce above it (context, recall results, events-as-records, artifacts) is derived and reconstructible (Block 7).

**The operating cycle.**

1. A change to the Domain is recorded as an **Event**.
2. The Event triggers a responsibility in the **AI Operating Layer**.
3. That responsibility requests **Context**; the **Context Builder** assembles it from **Memory** (recall over the Domain), recent Events and Policies.
4. The responsibility reasons and **acts**, writing the result back to the **Domain** — within the autonomy and human-control limits of the AI Architecture.
5. The write is a new change — a new **Event** — and the cycle continues.

**Policies** are the autonomy and human-control constraints that govern what the AI Operating Layer may do on its own; they are defined in the AI Architecture and consumed here only as an input to the Context Builder.

This one loop composes every use case: acquire or retrieve, understand, reason, produce, integrate. It is what keeps the system alive without continuous instruction from the researcher.

---

# Logical Type Vocabulary

Attributes are specified with logical types. A logical type describes the *nature* of a value, never its storage.

| Type                | Meaning                                                        |
|---------------------|---------------------------------------------------------------|
| `identifier`        | A globally unique identity.                                   |
| `text`              | Free text of any length.                                      |
| `enum(a\|b\|c)`     | Exactly one value from a fixed set.                           |
| `timestamp`         | A single instant in time.                                     |
| `time span`         | A start and an end instant.                                   |
| `date`              | A calendar day.                                               |
| `degree`            | A normalized value in the interval [0, 1].                    |
| `ordinal`           | A ranked value (e.g. priority).                               |
| `quantity`          | A magnitude with a unit.                                      |
| `money`             | An amount with a currency.                                    |
| `locator`           | A logical reference to where content resides, not a path.    |
| `reference → X`     | An identity link to another entity, navigable both ways.     |
| `set<T>` / `list<T>`| An unordered / ordered collection.                           |

How a `degree`, a `timestamp` or a `locator` is physically represented is a decision for the Software Architecture. This document commits only to the logical nature.

---

# Block 0 · Value Objects

A **value object** is a domain concept identified by its values, not by an identity of its own. Value objects are immutable and always live inside an aggregate (Block 5); they are never referenced independently.

Where an attribute holds a single logical value (a `degree`, a `date`, an `ordinal`, an `enum`), it is simply an attribute — not a value object. Value objects are the *composite* concepts below.

| Value Object        | Composition                                                                              | Used by             |
|---------------------|------------------------------------------------------------------------------------------|---------------------|
| `Provenance`        | source, means, moment                                                                     | Document, Knowledge |
| `Version`           | ordinal, timestamp, author (`→ Person`), note                                             | Document            |
| `Metadata`          | a set of descriptive fields                                                               | Document            |
| `KnowledgeRelation` | relation type `enum(supports\|contradicts\|refines\|depends on)`, target (`→ Knowledge`)  | Knowledge           |
| `Objective`         | statement, target measure                                                                 | Project             |
| `Deliverable`       | description, due `date`, status                                                           | Project             |
| `Milestone`         | label, `date`                                                                             | Project             |
| `FundingAllocation` | amount (`money`), source (`→ Resource`), period (`time span`)                             | Project             |
| `Membership`        | member (`→ Person`), role, period (`time span`)                                           | Project             |
| `ContactInfo`       | a set of contact points                                                                   | Person              |
| `ResourceUse`       | resource (`→ Resource`), quantity                                                         | Activity            |

A value object may reference an aggregate root by identity — a `Membership` references a `Person` — but it never has identity itself. Everything else an entity carries is a plain attribute of a type from the vocabulary above.

---

# Block 1 · Core Entities

Each Core Entity is an aggregate root (Block 5). Its attributes are specified below. `reference → X` attributes restate relationships (Block 2) and are shown here only where they are intrinsic to the entity.

## Document

A persistent digital artifact. Stores information; is not itself Knowledge (Domain Model → Document).

| Attribute      | Type                                                             | Notes                                          |
|----------------|------------------------------------------------------------------|------------------------------------------------|
| `id`           | `identifier`                                                     |                                                |
| `title`        | `text`                                                           |                                                |
| `kind`         | `enum(Paper\|Book\|Article\|Report\|Regulation\|Email\|Presentation\|Spreadsheet\|Image\|Note\|…)` | Derived-Type discriminator      |
| `medium`       | `enum(text\|document\|tabular\|image\|slide\|message)`           | Logical medium, not file format                |
| `provenance`   | `Provenance`                                                     | Origin: source, means, moment (value object)   |
| `content`      | `locator`                                                        | Logical pointer to preserved content           |
| `authors`      | `set<reference → Person>`                                        |                                                |
| `versions`     | `list<Version>`                                                  | Version history (value objects)                |
| `metadata`     | `Metadata`                                                       | Descriptive fields (value object)              |
| `state`        | `enum` (Block 3)                                                 |                                                |
| `registered_at`| `timestamp`                                                     |                                                |
| `updated_at`   | `timestamp`                                                      |                                                |

## Knowledge

What the system knows. Meaning, independent of any Document (Knowledge Model).

| Attribute     | Type                                                              | Notes                                             |
|---------------|------------------------------------------------------------------|---------------------------------------------------|
| `id`          | `identifier`                                                     |                                                   |
| `title`       | `text`                                                           |                                                   |
| `summary`     | `text`                                                           |                                                   |
| `body`        | `text`                                                           | The meaning itself                                |
| `kind`        | `enum(Concept\|Insight\|Note\|Methodology\|Skill\|Observation\|Hypothesis\|Evidence\|Decision\|…)` | Derived-Type discriminator |
| `confidence`  | `degree`                                                         | How strongly the knowledge is held                |
| `provenance`  | `set<Provenance>`                                                | Accumulates as knowledge is reused                |
| `relations`   | `set<KnowledgeRelation>`                                         | Typed edges to other Knowledge (value objects)    |
| `sources`     | `set<reference → Document \| Activity>`                          | What it emerged from                              |
| `state`       | `enum` (Block 3)                                                 |                                                   |
| `created_at`  | `timestamp`                                                     |                                                   |
| `updated_at`  | `timestamp`                                                     |                                                   |

## Project

An organized initiative with defined objectives. Coordinates work; does not own Knowledge (Domain Model → Project).

| Attribute      | Type                                          | Notes                              |
|----------------|-----------------------------------------------|------------------------------------|
| `id`           | `identifier`                                  |                                    |
| `title`        | `text`                                        |                                    |
| `description`  | `text`                                        |                                    |
| `objectives`   | `set<Objective>`                              | At least one required (Block 6)    |
| `deliverables` | `set<Deliverable>`                            |                                    |
| `milestones`   | `list<Milestone>`                             |                                    |
| `period`       | `time span`                                   | Start → deadline                   |
| `funding`      | `set<FundingAllocation>`                      | References Resource                |
| `members`      | `set<Membership>`                             | Person + Role (value object)       |
| `state`        | `enum` (Block 3)                              |                                    |
| `created_at`   | `timestamp`                                   |                                    |
| `updated_at`   | `timestamp`                                   |                                    |

## Person

A human actor in the operational environment (Domain Model → Person).

| Attribute     | Type                                                          | Notes                                  |
|---------------|--------------------------------------------------------------|----------------------------------------|
| `id`          | `identifier`                                                |                                        |
| `name`        | `text`                                                      |                                        |
| `roles`       | `set<enum(Supervisor\|Collaborator\|Student\|Reviewer\|Coordinator\|Author\|Researcher\|Client)>` | Contextual, may vary per Project; `Coordinator` covers the project-manager role |
| `affiliation` | `text`                                                     | Organization is not yet an entity      |
| `contact`     | `ContactInfo`                                              | Value object                           |
| `created_at`  | `timestamp`                                               |                                        |
| `updated_at`  | `timestamp`                                                |                                        |

Future generalization to **Actor** (Domain Model) would let organizations become first-class.

## Task

Intended work. Exists before execution; expresses commitment, not action (Domain Model → Task).

| Attribute      | Type                          | Notes                                    |
|----------------|-------------------------------|------------------------------------------|
| `id`           | `identifier`                  |                                          |
| `title`        | `text`                        |                                          |
| `description`  | `text`                        |                                          |
| `status`       | `enum` (Block 3)              |                                          |
| `priority`     | `ordinal`                     |                                          |
| `due_date`     | `date`                        |                                          |
| `project`      | `reference → Project` (0..1)  | Not every Task belongs to a Project      |
| `informed_by`  | `set<reference → Knowledge>`  |                                          |
| `created_at`   | `timestamp`                   |                                          |
| `completed_at` | `timestamp`                   | Set on transition to Done                |

## Activity

Work that has occurred or is occurring. The operational history (Domain Model → Activity).

| Attribute     | Type                                        | Notes                                         |
|---------------|---------------------------------------------|-----------------------------------------------|
| `id`          | `identifier`                                |                                               |
| `kind`        | `enum(Reading\|Writing\|Thinking\|Meeting\|Teaching\|Experimenting\|Reviewing\|Discussing)` | Derived-Type discriminator |
| `description` | `text`                                      |                                               |
| `occurred`    | `time span`                                 | When it happened or began                     |
| `state`       | `enum` (Block 3)                            |                                               |
| `performers`  | `set<reference → Person>`                   | At least one (Block 6)                         |
| `consumed`    | `set<ResourceUse>`                          | Resource + quantity (value object)            |
| `realizes`    | `reference → Task` (0..1)                   | The Task it came from, if any                 |
| `produced`    | `set<reference → Knowledge \| Document>`    | Its outputs                                   |
| `created_at`  | `timestamp`                                 |                                               |

## Resource

Anything consumed or required to perform work (Domain Model → Resource).

| Attribute      | Type                                                        | Notes                    |
|----------------|------------------------------------------------------------|--------------------------|
| `id`           | `identifier`                                              |                          |
| `name`         | `text`                                                    |                          |
| `kind`         | `enum(API\|Software\|Equipment\|Compute\|GPU\|Dataset\|License\|Funding)` | Derived-Type discriminator |
| `capacity`     | `quantity`                                                | Available amount         |
| `cost`         | `money`                                                   |                          |
| `provider`     | `text`                                                    |                          |
| `availability` | `enum` (Block 3)                                          |                          |
| `created_at`   | `timestamp`                                               |                          |
| `updated_at`   | `timestamp`                                               |                          |

## The Research Spine

Three Derived Types — **Hypothesis**, **Experiment** and **Evidence** — specialize the Core Entities into the operational spine of scientific work. They are specializations, not roots, and are specified separately in *The Research Extension*.

## Other Derived Types in use

The Use Cases already exercise further specializations through the `Entity (Specialization)` notation: **Draft** and **Chapter** (Document), **Bibliography** and **Research Journal** (Document), **Meeting** (Activity), and **Decision** (Knowledge). These inherit their base entity's structure. Those recurring enough to warrant formal promotion are tracked under *Scope and Boundaries*; **Decision** is currently modelled as `Knowledge (Decision)`.

---

# Block 2 · Relationships

The complete logical graph. Every edge is directed, named, and navigable in both directions (Block 6). "Inverse" names the reverse traversal.

| From      | Relationship        | To        | Inverse            | Cardinality |
|-----------|---------------------|-----------|--------------------|-------------|
| Person    | participates in     | Project   | has member         | N : N       |
| Person    | performs            | Activity  | performed by       | N : N       |
| Person    | authors             | Document  | authored by        | N : N       |
| Project   | organizes           | Task      | belongs to         | 1 : N       |
| Project   | organizes           | Document  | supports project   | N : N       |
| Project   | references          | Knowledge | referenced by      | N : N       |
| Task      | becomes             | Activity  | realizes           | 1 : N       |
| Task      | produces            | Knowledge | produced by        | N : N       |
| Task      | informed by         | Knowledge | informs            | N : N       |
| Activity  | produces            | Knowledge | produced by        | N : N       |
| Activity  | produces            | Document  | produced by        | N : N       |
| Activity  | consumes            | Resource  | consumed by        | N : N       |
| Document  | supports            | Knowledge | supported by       | N : N       |
| Knowledge | relates to          | Knowledge | relates to         | N : N       |

The **Research Spine** adds specialized edges over this graph:

| From       | Relationship  | To         | Cardinality |
|------------|---------------|------------|-------------|
| Experiment | tests         | Hypothesis | N : 1       |
| Experiment | produces      | Evidence   | 1 : N       |
| Evidence   | resolves      | Hypothesis | N : 1       |

`Knowledge relates to Knowledge` is the edge set that forms the knowledge graph. Each such edge is a `KnowledgeRelation` value object carrying its own type (e.g. *supports*, *contradicts*, *refines*, *depends on*).

---

# Block 3 · Entity Lifecycles

An entity with a lifecycle carries a `state`. Transitions are constrained: only the moves shown are legal. Every transition records its cause (Block 6).

## Knowledge

```
Draft ──▶ Candidate ──▶ Validated ──▶ Deprecated ──▶ Archived
              │              ▲
          (rejected)   (revalidated on new evidence)
              ▼
          Discarded
```

Validation is the researcher's act (AI Architecture → The Improvement Loop). New evidence may move Validated back to Deprecated; nothing is deleted.

## Hypothesis · specialized Knowledge lifecycle

```
captured ──▶ developing ──▶ experimenting ──▶ evidenced
                                           ──▶ falsified
                                           ──▶ unresolved
```

Canonical, as exercised by the Use Cases (UC-R01 … UC-R04). It replaces the generic Knowledge lifecycle for Hypotheses.

## Task

```
Proposed ──▶ Todo ──▶ In Progress ──▶ Done
                          ▲   │
                          └ Blocked
   (any active state) ──▶ Cancelled
```

## Project

```
Proposed ──▶ Active ──▶ Completed ──▶ Archived
               ▲  │
               └ On Hold
   (Active | On Hold) ──▶ Cancelled
```

## Document

```
Registered ──▶ Processed ──▶ Available ──▶ Superseded ──▶ Archived
```

A new version moves the prior one to Superseded; superseded versions are retained (Block 6).

## Activity

```
Active ⇄ Suspended ──▶ Completed
```

An Activity may be suspended and resumed (UC-C05). Many Activities are recorded directly as Completed. A Completed Activity is immutable (Block 6).

## Experiment · specialized Activity lifecycle

```
planned ──▶ running ──▶ analysed
                    ──▶ aborted
```

Lowercase, matching the casing convention the Research Extension already established for specialized lifecycles (Hypothesis uses `captured`/`developing`/…), and the casing already used where this lifecycle is exercised (Use Cases → UC-R01, UC-R02).

## Resource

```
Available ⇄ In Use ──▶ Depleted
                   ──▶ Expired
```

---

# Block 4 · Cardinalities

The cardinality of each relationship, read as *"one From relates to how many To, and one To to how many From."*

| Relationship                         | From → To | To → From |
|--------------------------------------|-----------|-----------|
| Person — participates in — Project   | 0..N      | 0..N      |
| Person — performs — Activity         | 0..N      | 1..N      |
| Person — authors — Document          | 0..N      | 0..N      |
| Project — organizes — Task           | 0..N      | 0..1      |
| Project — organizes — Document       | 0..N      | 0..N      |
| Project — references — Knowledge     | 0..N      | 0..N      |
| Task — becomes — Activity            | 0..N      | 0..1      |
| Task — produces — Knowledge          | 0..N      | 0..N      |
| Task — informed by — Knowledge       | 0..N      | 0..N      |
| Activity — produces — Knowledge      | 0..N      | 0..N      |
| Activity — produces — Document       | 0..N      | 0..N      |
| Activity — consumes — Resource       | 0..N      | 0..N      |
| Document — supports — Knowledge      | 0..N      | 0..N      |
| Knowledge — relates to — Knowledge   | 0..N      | 0..N      |
| Experiment — tests — Hypothesis      | 1..1      | 0..N      |
| Evidence — resolves — Hypothesis     | 1..1      | 0..N      |
| Experiment — produces — Evidence     | 0..N      | 1..1      |

Two cardinalities carry design intent worth stating explicitly.

- An Activity has **at least one** performer (`1..N`): work is always done by someone.
- A Task belongs to **at most one** Project (`0..1`): tasks may be personal, administrative or cross-cutting, not only research (System Model → Tasks).

---

# Block 5 · Aggregates

This model uses the Domain-Driven Design building blocks, kept deliberately distinct.

- **Entity** — a concept with its own `identifier` and a lifecycle (the seven of Block 1, and the Research Extension).
- **Value Object** — a concept identified by its values, immutable, with no identity of its own (Block 0).
- **Aggregate** — an Entity together with the Value Objects it owns, treated as a single unit of consistency.
- **Aggregate Root** — the Entity that guards an aggregate's invariants and is its only entry point.

*Repository* and *Domain Service* — the two remaining DDD building blocks — describe how aggregates are stored and how cross-aggregate operations are coordinated. Those are implementation concerns and belong to the Software Architecture, not here.

Each Core Entity is an aggregate root. Aggregates are kept small: a root **contains** its value objects and **references** other roots by identity rather than containing them.

| Aggregate root | Contains (value objects)                                        | References (by identity)          |
|----------------|-----------------------------------------------------------------|-----------------------------------|
| **Project**    | Objective, Deliverable, Milestone, FundingAllocation, Membership | Task, Document, Knowledge, Person |
| **Knowledge**  | Provenance, KnowledgeRelation                                   | Document, Activity, Project        |
| **Document**   | Provenance, Version, Metadata                                  | Person, Project                    |
| **Person**     | ContactInfo                                                     | Project, Activity, Document        |
| **Task**       | —                                                               | Project, Knowledge, Activity       |
| **Activity**   | ResourceUse                                                     | Person, Resource, Task, Knowledge, Document |
| **Resource**   | —                                                               | Activity                           |

Simple attributes (a Task's `priority` and `status`, a Resource's `capacity` and `cost`) are not value objects; they are plain typed attributes of the root.

This yields the model's most important structural consequence.

Because aggregates are small and reference one another by identity, a change confined to one aggregate is **immediately consistent**, while consistency *across* aggregates is **eventual** — achieved through domain Events (Event Model), not through large transactions.

```
within an aggregate   →  immediate consistency
across aggregates      →  eventual consistency, via Events
```

The logical model and the Event Model meet here: aggregate boundaries are exactly the boundaries across which the system coordinates by reacting to change rather than by locking state.

---

# Block 6 · Domain Constraints

Invariants that must always hold. They are the rules a projection — schema, type or API — must enforce.

## Global invariants

- **Bidirectional navigability.** Every relationship is navigable in both directions. Provenance traces backward and impact traces forward over the same edges (Use Cases → Traceability, UC-T04).
- **No silent overwrite.** No entity is destructively overwritten. Superseded state is retained with its provenance, so prior reasoning stays recoverable (Knowledge Model; UC-R04).
- **Caused transitions.** Every lifecycle transition records its cause — the Activity or Evidence that drove it — making evolution reconstructible (UC-T05).
- **Knowledge is shared, never owned.** Knowledge is reusable across Projects; no Project owns Knowledge exclusively (System Principle 6).
- **Derived objects are not truth.** Every derived object is reconstructible from persistent entities and is never a source of truth (Block 7).

## Per-entity invariants

- **Knowledge** — `confidence` ∈ [0, 1]; a Validated Knowledge must retain provenance; Knowledge is never deleted, only Deprecated or Archived.
- **Document** — provenance is immutable and preserved across every version; a superseded version is retained, never discarded.
- **Task** — has exactly one `status`; a Done Task has a `completed_at`; references at most one Project.
- **Activity** — has at least one performer; `occurred` never lies in the future; once Completed it is append-only (immutable history).
- **Project** — has at least one objective; `period.end` ≥ `period.start`.
- **Resource** — total quantity consumed across Activities never exceeds `capacity`.
- **Hypothesis** — an evidenced or falsified Hypothesis references at least one Evidence; transitions follow the specialized order.
- **Experiment** — tests exactly one Hypothesis; only a Running or Analysed Experiment may yield Evidence.
- **Evidence** — references exactly one Hypothesis and exactly one Experiment; `polarity` is always set.

---

# Block 7 · Derived Objects

Not everything the system produces is persistent. Much of what the Cognitive Architecture generates is **derived**: computed from the domain when needed, never stored as truth.

| Derived object        | Derived from                                   | Nature                | Defined in        |
|-----------------------|------------------------------------------------|-----------------------|-------------------|
| **Context**           | task + intent, over Memory · Domain · Events · Policies | Transient       | Context Model     |
| **Recall result**     | a query, over Memory                           | Transient             | Memory Model      |
| **Traceability view** | the knowledge graph (provenance, impact, decision history, evolution) | Transient synthesis, read-only | Use Cases → Traceability |
| **Artifact**          | reasoning, over the domain                     | Produced output       | AI Architecture   |

A derived object has **no persistent structure to specify** in this model, because it is a function of the persistent entities, not an addition to them. This is why Block 1 specifies the seven entities exhaustively and stops there.

The **persistent / derived** line, stated precisely:

```
   PERSISTENT (source of truth)          DERIVED (function of the domain)
   ─────────────────────────────         ───────────────────────────────
   Project · Knowledge · Document        Context · Recall result
   Person · Task · Activity · Resource   Traceability view · Artifact
```

An **Artifact** is the boundary case. When a generated artifact must persist — a saved report, an accepted draft — it is stored as a `Document` with explicit provenance, and only enters Knowledge through the human-gated Improvement Loop (AI Architecture). It becomes persistent by *becoming a Document*, not by being a fourth kind of stored thing.

---

# Block 8 · Logical Views

A logical view is a projection of this specification for a particular concern — the knowledge graph, a project's structure, the research spine, the intelligence loop.

These views are drawn where they are used, not duplicated here. The whole-system view is *The System at a Glance* above; the domain graph is Block 2; the intelligence loop belongs to the AI Architecture; the memory, context and event perspectives belong to their respective models.

> See the corresponding architectural documents.

This keeps the document to a single responsibility — the structure of the domain — and lets each other document own the view it is best placed to draw.

---

# The Research Extension

The Research Spine is not part of the Core. It is a **specialization layer** over three Core Entities, formalized because it recurs across the scientific use cases (Domain Model → The Research Spine). Each type inherits every attribute and relationship of its base entity and adds its own.

## Hypothesis · specializes Knowledge

| Added attribute | Type                          | Notes                    |
|-----------------|-------------------------------|--------------------------|
| `statement`     | `text`                        | The proposed explanation |
| `assumptions`   | `set<text>`                   |                          |
| `tested_by`     | `set<reference → Experiment>` |                          |
| `resolved_by`   | `set<reference → Evidence>`   |                          |

Lifecycle: `captured → developing → experimenting → evidenced | falsified | unresolved` — replacing the generic Knowledge lifecycle (Block 3).

## Experiment · specializes Activity

| Added attribute | Type                         | Notes                                       |
|-----------------|------------------------------|---------------------------------------------|
| `tests`         | `reference → Hypothesis` (1) | Exactly one (Block 6)                       |
| `design`        | `text`                       | Objectives, protocol, expected observations |
| `produced`      | `set<reference → Evidence>`  | Narrows Activity's `produced`               |

Lifecycle: `planned → running → analysed | aborted` — replacing the generic Activity lifecycle (Block 3).

## Evidence · specializes Knowledge

| Added attribute | Type                                      | Notes |
|-----------------|-------------------------------------------|-------|
| `polarity`      | `enum(supports\|falsifies\|inconclusive)` |       |
| `resolves`      | `reference → Hypothesis` (1)              | Mirrors `Hypothesis.resolved_by`; the `resolves` edge (Blocks 2, 4) |
| `produced_by`   | `reference → Experiment` (1)              |       |
| `strength`      | `degree`                                  |       |

Evidence keeps the generic Knowledge lifecycle.

The Spine's relationships (`tests`, `resolves`, `produces`), cardinalities and invariants extend the same graph the Core Entities form, and are specified inline in Blocks 2, 4 and 6.

---

# Vertical Lifecycle Extensions

The Teaching, Administration and Organization verticals (Use Cases; `TEACHING_VERTICAL.md`, `ADMINISTRATION_VERTICAL.md`, `ORGANIZATION_VERTICAL.md`) introduce Derived Types with their own operational shape. Most reuse a base entity's generic Block 3 lifecycle verbatim. A few genuinely do not fit it and need their own specialized lifecycle — the same treatment the Research Extension already gives Hypothesis and Experiment.

This section is deliberately narrow: it specifies lifecycles only, not full attribute tables, because that is the only gap the verticals actually exposed.

## Reuse the generic lifecycle, exactly

These Derived Types map cleanly onto their base entity's existing Block 3 states and need nothing new — only the canonical state names, not invented ones.

| Derived Type | Base Entity | Reuses |
|---|---|---|
| Course | Project | `Proposed → Active → Completed → Archived` |
| Procedure | Task | `Proposed → Todo → In Progress → Done` |
| Progress Report | Document | `Registered → Processed → Available → Superseded → Archived` |

## Specialized lifecycles

Three Derived Types have a genuinely different shape from their base entity's generic lifecycle and are specialized here, exactly as Hypothesis and Experiment were.

### Administrative Process · specializes Project

```
Open ──▶ In Progress ──▶ Under Review ──▶ Resolved
```

An institutional procedure is opened, worked on, submitted for external evaluation and then resolved — a distinct "awaiting external verdict" phase the generic Project lifecycle has no state for.

### Submission · specializes Document

```
Submitted ──▶ Graded
```

A student submission is not registered, processed and made available in the sense the generic Document lifecycle describes — it is handed in, then evaluated.

### Final Project · specializes Project

```
Proposed ──▶ In Progress ──▶ Submitted ──▶ Defended
```

A TFG/TFM has a genuine "awaiting defence" phase and a true terminal state (Defended) that is not Completed/Archived in the generic sense.

These three additions follow this document's own evolution rule: new Derived Types and their lifecycles are welcome; the seven Core Entities and their relationships are not touched.

---

# Scope and Boundaries

**In scope.** The seven Core Entities, the formalized Research Spine (Hypothesis, Experiment, Evidence), and the Vertical Lifecycle Extensions (Administrative Process, Submission, Final Project), specified to logical precision.

**Pending promotion.** Four concepts surfaced by the architecture are candidates for the Domain Model but are not yet promoted, and so are not fully specified here:

- **Decision** — currently modelled as `Knowledge (Decision)`; produced in UC-R03 and UC-C04, consumed across Traceability (Use Cases).
- **Artifact** — the generated product (AI Architecture); until promoted, a `Document` with provenance (Block 7).
- **Event** — a recorded domain change (Event Model); its promotion to a persisted record would make the system's reactive history queryable.
- **Curate** — a candidate *capability*, not an entity (System Capabilities pending).

Each awaits a deliberate, recorded decision before being specified. Until then they are named, never half-specified.

**Out of scope.** Everything physical: storage, indexing, database choice, language, service boundaries. Those belong to the Software Architecture, which treats this model as its specification.

---

# Relationship to Other Documents

To avoid duplication, this document does not redefine shared concepts.

- What the entities *are* — Domain Model
- What Knowledge *means* and how it evolves — Knowledge Model
- The behaviour that operates on the entities — System Capabilities; AI Architecture
- The cognitive products derived from the entities — Memory Model; Context Model; Event Model
- The state changes behaviour produces, and the Traceability the graph must support — Use Cases
- How the researcher and the system collaborate over the entities — Interaction Model

The Logical Domain Model owns one thing: the exact structure of the domain, from which every schema, diagram and type is derived.

---

# Evolution Strategy

This specification refines as the entities are exercised, but along stable lines.

- Attributes and states may be added; the seven roots and their meanings should not change.
- Candidate concepts enter only through promotion, recorded as a decision.
- Every physical realization — relational, graph, document, type — is derived from this document and re-derived when it changes.

The model is expected to grow in precision while remaining, in its shape, as stable as the Domain Model it specifies.
