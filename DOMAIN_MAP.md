# Domain Map

## Purpose

This document defines the conceptual domains of Doctorado_UCAM.

A domain represents a coherent area of responsibility within the Research Operating System.

The objective of the Domain Map is not to describe software modules or microservices.

Instead, it identifies the major conceptual boundaries that compose the system and explains how they interact.

Every future capability, entity and implementation should belong to one of these domains.

---

# Domain Philosophy

Doctorado_UCAM is not organized around software features.

It is organized around the operational reality of a researcher.

Each domain represents a different aspect of that reality.

Together they compose a single unified operational state.

No domain should evolve in isolation.

---

# Domain Classification

The platform is organized into three categories:

• Core Domains
• Supporting Domains
• Cross-Cutting Domains

---

# Domains and Entities

A domain is an area of responsibility.

An entity is a concrete concept that lives inside a domain.

The two should not be confused. A domain typically owns one or more entities, which are defined in the Domain Model.

| Domain    | Primary Entities |
|-----------|------------------|
| Knowledge | Knowledge        |
| Documents | Document         |
| Projects  | Project          |
| People    | Person           |
| Planning  | Task, Activity   |
| Resources | Resource         |

The Research domain owns no entity of its own. It is an emergent process (see Domain Model → Research).

Cross-cutting domains (Intelligence, Context, Search & Retrieval, Memory) own no entities. They operate over the entities owned by other domains.

---

# Core Domains

Core Domains represent the intellectual heart of the Research Operating System.

They define the unique value of the platform.

Without them, Doctorado_UCAM would simply become another productivity application.

---

## Knowledge Domain

Purpose

Represent, organize and evolve the researcher's knowledge.

Responsibilities

- Concepts
- Notes
- Ideas
- Skills
- Methodologies
- Relationships
- Insights
- Personal knowledge
- Semantic connections

The Knowledge Domain answers:

> What does the researcher know?

---

## Research Domain

Purpose

Represent the scientific process itself.

Responsibilities

- Hypotheses
- Experiments
- Evidence
- Publications
- Analysis
- Datasets
- Scientific writing
- Literature review
- Research methodology

The Research Domain answers:

> What is the researcher investigating?

---

## Project Domain

Purpose

Coordinate operational execution.

Responsibilities

- Projects
- Objectives
- Deliverables
- Milestones
- Funding
- Deadlines
- Responsibilities
- Project documentation

The Project Domain answers:

> Why is this work being done?

---

# Supporting Domains

Supporting Domains enable the Core Domains to function effectively.

Although essential, they are not the primary source of differentiation.

---

## Document Domain

Purpose

Manage every digital artifact handled by the platform.

Examples

- Papers
- PDFs
- Word documents
- Excel files
- Images
- Presentations
- Administrative documents
- Regulations
- Emails
- Meeting minutes

Documents represent information containers.

They should not be confused with Knowledge.

Knowledge may originate from documents but is conceptually independent.

---

## People Domain

Purpose

Represent the network of individuals involved in the researcher's work.

Examples

- Supervisors
- Collaborators
- Project managers
- Students
- Colleagues
- Clients
- Reviewers

The People Domain captures relationships rather than communication itself.

---

## Planning Domain

Purpose

Coordinate operational commitments.

Responsibilities

- Tasks
- Calendar
- Events
- Deadlines
- Meetings
- Reminders
- Priorities

Planning organizes future work.

It does not generate knowledge.

---

## Resource Domain

Purpose

Represent resources required to perform research.

Examples

- Datasets
- APIs
- Compute resources
- Funding allocations
- Software licenses
- Laboratory resources
- External services

Resources support research activities.

---

# Cross-Cutting Domains

Cross-Cutting Domains operate across every other domain.

They do not own business entities.

Instead, they provide system-wide capabilities.

---

## Intelligence

Purpose

Operate intelligently over the complete system state.

Responsibilities

- Reasoning
- Recommendations
- Planning
- Context reconstruction
- Decision support
- Knowledge synthesis
- Agent orchestration

Intelligence consumes information from every domain.

It owns none.

---

## Context

Purpose

Provide the relevant operational state required for reasoning.

Context is not stored independently.

It is continuously derived from:

- Knowledge
- Research
- Projects
- Documents
- Planning
- People
- Historical activity

Every intelligent operation depends on Context.

---

## Search & Retrieval

Purpose

Locate relevant information regardless of its originating domain.

The researcher should search concepts rather than storage locations.

---

## Memory

Purpose

Preserve long-term continuity.

Memory records:

- historical decisions
- conversations
- project evolution
- research evolution
- behavioural patterns

Memory transforms isolated events into long-term understanding.

---

# Domain Relationships

Knowledge is informed by Research.

Research consumes Knowledge.

Projects organize Research.

Documents support every domain.

People participate in Projects and Research.

Planning coordinates operational execution.

Resources enable research activities.

Context emerges from every domain.

Intelligence operates over the complete system state.

---

# Core Architectural Principle

The Research Operating System does not consist of independent applications.

It consists of multiple domains operating over one shared operational state.

Every domain contributes information.

Every capability consumes information.

Every interface exposes different perspectives of the same underlying reality.

Maintaining this coherence is the defining architectural characteristic of Doctorado_UCAM.
