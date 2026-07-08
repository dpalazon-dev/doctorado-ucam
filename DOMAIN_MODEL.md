# Domain Model

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 2 — Specialized Model |
| **Normative status** | Canonical within conceptual-entity scope |
| **Authoritative for** | The meaning, identity and conceptual relationships of the seven Core Entities and their distinction from processes, domains and derived concepts. |
| **Not authoritative for** | Attributes, logical types, cardinalities, lifecycle transitions, persistence or module implementation. |
| **Required reading** | `SYSTEM_PRINCIPLES.md`, `DOMAIN_MAP.md`, `SYSTEM_RESPONSIBILITIES.md`. |
| **Downstream documents** | `KNOWLEDGE_MODEL.md`, `SYSTEM_CAPABILITIES.md`, `USE_CASES.md`, `DOMAIN_VERTICALS.md`, `LOGICAL_DOMAIN_MODEL.md`. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

## Purpose

This document defines the fundamental entities of ResearchOS.

Unlike a database schema or an object model, this document describes the conceptual objects that exist within the Research Operating System.

These entities represent the irreducible concepts that compose the platform.

Every capability, workflow and software component should ultimately operate on one or more of these entities.

The objective of this model is to establish a ubiquitous language for the project.

---

# Design Principles

The Domain Model follows several fundamental principles.

- Every entity represents a unique concept.
- No entity should duplicate the responsibility of another.
- Entities describe the domain, not the implementation.
- Every future feature should manipulate existing entities before introducing new ones.
- Simplicity is preferred over specialization.

Specialized objects should emerge from these entities rather than becoming independent root concepts.

---

# Core Entities

The current version of ResearchOS is built around seven core entities.

```
                   Project
                  /   |   \
                 /    |    \
                /     |     \
         Task ------ Activity ------ Resource
            \          /
             \        /
              \      /
             Knowledge
              /   \
             /     \
      Document    Person
```

Each entity represents a different aspect of the researcher's operational reality.

Together they define the shared operational state of the system.

---

# Document

## Purpose

Represents any persistent digital artifact managed by the platform.

A Document stores information but does not necessarily represent knowledge.

Examples include:

- Scientific papers
- Books
- PDFs
- Word documents
- Excel files
- Images
- Presentations
- Administrative documents
- Regulations
- Emails

A Document may generate Knowledge but is not itself Knowledge.

### Responsibilities

- Preserve content
- Preserve metadata
- Maintain provenance
- Support retrieval
- Support versioning

### Does NOT represent

- Ideas
- Concepts
- Relationships
- Understanding

---

# Knowledge

## Purpose

Represents what the system knows.

Knowledge is independent of the documents from which it originates.

It captures meaning rather than storage.

The full knowledge lifecycle — how knowledge is born, evolves and is represented — is defined in the Knowledge Model.

Knowledge may emerge from:

- Documents
- Activities
- Conversations
- Research
- Experience
- Artificial Intelligence

Future specializations may include:

- Concepts
- Ideas
- Skills
- Observations
- Relationships

### Responsibilities

- Represent meaning
- Connect information
- Evolve over time
- Support reasoning
- Preserve relationships

### Does NOT represent

- Files
- Storage
- Projects
- Tasks

---

# Project

## Purpose

Represents an organized initiative with defined objectives.

Projects coordinate work.

They provide operational structure without owning knowledge itself.

Projects may include:

- Objectives
- Deliverables
- Deadlines
- Funding
- Documentation
- Collaborators
- Tasks
- Research

### Responsibilities

- Organize work
- Track progress
- Coordinate resources
- Maintain objectives

### Does NOT represent

- Knowledge
- Documents
- Activities

Projects reference them.

---

# Person

## Purpose

Represents any human actor interacting with the operational environment.

Examples include:

- Thesis supervisors
- Researchers
- Collaborators
- Students
- Project managers
- Clients
- Reviewers

Future versions may generalize this entity into Actor, allowing organizations and institutions to become first-class entities.

### Responsibilities

- Participate
- Collaborate
- Author
- Review
- Supervise
- Communicate

### Does NOT represent

Organizations or institutions (for now).

---

# Task

## Purpose

Represents an intended piece of work.

A Task exists before execution.

It defines commitment rather than action.

Examples include:

- Read a paper
- Write a chapter
- Meet the supervisor
- Prepare an experiment
- Review literature

A completed Task may generate one or more Activities.

### Responsibilities

- Represent future work
- Express priorities
- Support planning
- Track completion

### Does NOT represent

Work that has already happened.

---

# Activity

## Purpose

Represents work that has actually occurred.

Activities capture the operational history of the researcher.

Examples include:

- Reading
- Writing
- Thinking
- Meeting
- Teaching
- Experimenting
- Reviewing
- Discussing

Activities generate operational state.

They may produce:

- Knowledge
- Documents
- Decisions
- New Tasks

Context is not produced here; it emerges from the relationships between these outputs (see Context below).

### Responsibilities

- Record execution
- Capture operational history
- Produce knowledge
- Preserve traceability

### Does NOT represent

Future intentions.

Those belong to Tasks.

---

# Resource

## Purpose

Represents anything consumed or required to perform work.

Resources enable activities but are not themselves operational work.

Examples include:

- APIs
- Software
- Laboratory equipment
- Compute resources
- GPUs
- External datasets
- Licenses
- Budgets and grants

### Responsibilities

- Enable work
- Provide capabilities
- Support execution

### Does NOT represent

Knowledge or documents.

Resources are consumed rather than interpreted.

---

# Derived Types

Core Entities are intentionally minimal.

Most real-world concepts are not new entities. They are specializations of an existing Core Entity.

These specializations are called Derived Types.

| Derived Type     | Base Entity | Notes                                                                        |
|------------------|-------------|------------------------------------------------------------------------------|
| Paper            | Document    | A scientific article                                                         |
| Book             | Document    | A monograph or textbook                                                      |
| Bibliography     | Document    | A curated collection of references                                           |
| Research Journal | Document    | A chronological research log                                                 |
| Meeting          | Activity    | Work that occurred with other People                                         |
| Dataset          | Resource    | Data consumed to perform work                                                |
| Hypothesis       | Knowledge   | A proposed explanation with a lifecycle (captured → experimenting → evidenced / falsified) |
| Experiment       | Activity    | Structured work that tests a Hypothesis, consumes Resources and produces Evidence |
| Evidence         | Knowledge   | The interpreted result of an Experiment that supports or falsifies a Hypothesis |
| Decision         | Knowledge   | A traceable conclusion or commitment with rationale, conditions, provenance and consequences |

Deriving from a Base Entity means a Derived Type inherits its responsibilities and relationships.

Because of this:

- Research Journal does not compete with Document. It is a Document.
- Bibliography does not compete with Document. It is a Document.

New Derived Types can be added freely, as long as they specialize an existing Core Entity rather than introducing a new root concept.

## The Research Spine

Three Derived Types form the operational spine of scientific work.

A **Hypothesis** (Knowledge) is proposed, tested by an **Experiment** (Activity), which produces **Evidence** (Knowledge) that in turn resolves the Hypothesis.

```
Hypothesis ──tested by──▶ Experiment ──produces──▶ Evidence
     ▲                                                 │
     └──────────────────── resolves ───────────────────┘
```

The Hypothesis carries an explicit operational lifecycle:

```
captured ──▶ … ──▶ experimenting ──▶ evidenced | falsified
```

Intermediate states are defined by the researcher.

This spine is why the three concepts were promoted from recurring use-case concepts into formal Derived Types: they satisfy the Domain Consistency principle by appearing across many use cases rather than in isolation.

The Teaching vertical exhibits a structural echo of this spine: a Rubric, Exam or Assignment is set, a Student produces a Submission, and grading produces an **Evaluation** (Knowledge) that resolves the mark — the same shape as Hypothesis → Experiment → Evidence, with Evaluation playing Evidence's role. This parallel is noted, not promoted; see Use Cases → Evolution.

---

# Relationship Philosophy

The platform is intentionally highly connected.

Entities should reference one another whenever meaningful relationships exist.

Examples include:

- Documents contribute to Knowledge.
- Activities generate Knowledge.
- Tasks become Activities.
- Projects organize Tasks.
- Projects organize Documents.
- People participate in Projects.
- People perform Activities.
- Activities consume Resources.
- Knowledge informs future Tasks.
- Knowledge guides Research.
- Documents support Projects.

The objective is not relational complexity.

The objective is contextual continuity.

---

# Emergent Concepts

Several important concepts are intentionally **not** represented as core entities.

They emerge from combinations of the entities defined above.

## Context

Context is not an entity. It emerges from the relationships between entities.

See System Model → Context.

---

## Research

Research is not an entity.

Research is an operational process emerging from Projects, Knowledge, Activities, Documents, Tasks and People.

---

## Intelligence

Intelligence is not an entity. It is a capability operating over the complete domain.

See Domain Map → Intelligence and System Capabilities → Reason.

---

## Memory

Memory is not an entity. It is a system capability that persists relationships and historical state.

See Domain Map → Memory.

---

## Time

Time is a transversal dimension shared by every entity.

It is never modeled independently.

---

# Evolution Strategy

This model is intentionally minimal.

New entities should only be introduced when they represent genuinely new concepts that cannot be expressed by existing ones.

Complexity should emerge through relationships rather than through an ever-growing number of entity types.

The long-term objective is to preserve a stable conceptual model while allowing the implementation to evolve indefinitely.
