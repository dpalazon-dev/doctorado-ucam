# Domain Model

## Purpose

This document defines the fundamental entities of Doctorado_UCAM.

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

# Fundamental Entities

The current version of Doctorado_UCAM is built around seven fundamental entities.

```
                   Project
                  /   |   \
                 /    |    \
                /     |     \
         Task ------ Activity
            \          /
             \        /
              \      /
             Knowledge
              /   \
             /     \
      Document    Person
             \
              \
            Resource
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
- Datasets

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
- Insights
- Methodologies
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
- Context
- New Tasks

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
- Funding allocations

### Responsibilities

- Enable work
- Provide capabilities
- Support execution

### Does NOT represent

Knowledge or documents.

Resources are consumed rather than interpreted.

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

Several important concepts are intentionally **not** represented as fundamental entities.

They emerge from combinations of the entities defined above.

## Context

Context is the current operational interpretation of the complete system state.

It emerges from the relationships between entities.

---

## Research

Research is not an entity.

Research is an operational process emerging from Projects, Knowledge, Activities, Documents, Tasks and People.

---

## Intelligence

Intelligence is not stored.

It is a capability operating over the complete domain.

---

## Memory

Memory is the long-term persistence of relationships and historical state.

It is a system capability rather than a domain entity.

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
