# Implementation Contexts

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 4 — Navigation, Delivery and Process |
| **Normative status** | Operational guide for coding-agent context assembly |
| **Authoritative for** | Which documents and sections should be loaded for each class of implementation task. |
| **Not authoritative for** | Domain behavior, architecture, software contracts, technologies or project sequencing. |
| **Required reading** | `DOCUMENTATION_ARCHITECTURE.md`, `CLAUDE.md`, `ROADMAP.md` |
| **Downstream documents** | Coding-agent prompts, implementation plans, pull requests and future development workflows. |

> This document selects context. It never changes the meaning of the documents it selects.

---

## Purpose

ResearchOS contains multiple architectural views because different concerns need different models. A coding agent should not load all of them for every task.

This document defines bounded **implementation context bundles**. Each bundle identifies:

- authoritative documents to load in full;
- supporting documents or sections to load selectively;
- documents that should remain outside context unless a trigger appears;
- invariants the agent must state before implementation;
- expected outputs and checks.

The objective is not to minimize context at any cost. It is to maximize the ratio of relevant authority to distracting material.

---

# Universal Agent Protocol

Before any code or structural repository change:

1. Read `CLAUDE.md`.
2. Check `ROADMAP.md` for the current phase.
3. Select exactly one primary bundle below.
4. Add secondary bundles only when the task genuinely crosses boundaries.
5. Read the relevant accepted or proposed ADRs in `DECISIONS.md`; do not treat them as substitutes for canonical owners.
6. Read authoritative documents in full.
7. Read supporting documents only at the sections named by the task.
8. Write down the invariants and owned concerns before implementation.
9. Identify downstream documentation impact.
10. Implement and test the smallest coherent change.
11. Report conformance, open questions and any required ADR.

Do not use repository-wide context as a substitute for task analysis.

---

# Context Budget Classes

| Class | Intended use | Typical document load |
|---|---|---|
| **S — Local** | Editing prose, isolated mappings, small bug fixes | 1 owner + 1–2 prerequisites |
| **M — Bounded** | One module, use case or runtime concern | 2–4 full documents + selected sections |
| **L — Cross-cutting** | Architecture change, new subsystem, major lifecycle change | Canonical core + all affected owners |
| **XL — Repository-wide** | Rare governance or foundation revision | Explicitly justified; never the default |

Most implementation tasks should be **M**.

---

# Bundle Template

Every implementation prompt should be assembled using this template:

```text
Task:
Primary context bundle:
Concern owner:
Documents loaded in full:
Sections loaded selectively:
Relevant ADRs:
Explicit exclusions:
Invariants to preserve:
Expected state changes:
Expected events/effects:
Tests and conformance checks:
Downstream documents to review:
```

---

# Bundle 1 — Documentation-Only Change

Use for editing, reorganizing or reviewing Markdown without changing system meaning.

## Load in full

- `DOCUMENTATION_ARCHITECTURE.md`
- the document being changed

## Load selectively

- its declared required reading;
- its declared downstream documents only where terminology or mappings are affected.

## Exclude by default

- unrelated models;
- implementation architecture;
- all verticals when changing only navigation.

## Preserve

- one owner per concern;
- document contract accuracy;
- no new normative rule in Level 4 documents;
- working local links.

## Output

- changed files;
- owner and downstream impact statement;
- link and contract validation.

---

# Bundle 2 — Vision or Principle Change

Use only when changing why the system exists or a permanent cross-cutting constraint.

## Load in full

- `VISION.md`
- `SYSTEM_PRINCIPLES.md`
- `SYSTEM_ARCHITECTURE.md`
- `DOCUMENTATION_ARCHITECTURE.md`

## Load selectively

- `SYSTEM_MODEL.md`;
- `ROADMAP.md`;
- affected specialized models.

## Exclude by default

- vertical details;
- individual use cases;
- runtime mechanics unless the principle directly constrains them.

## Preserve

- technology independence;
- internal consistency of the canonical core;
- explicit downstream impact.

## Additional requirement

A change at this level normally requires an ADR and a repository-wide impact review.

---

# Bundle 3 — Domain Concept or Entity Change

Use for changes to Core Entities, derived types, attributes, relationships, cardinalities, lifecycles or invariants.

## Load in full

- `SYSTEM_PRINCIPLES.md`
- `DOMAIN_MODEL.md`
- `LOGICAL_DOMAIN_MODEL.md`
- `DOCUMENTATION_ARCHITECTURE.md`

## Load selectively

- `DOMAIN_MAP.md` for ownership boundaries;
- `KNOWLEDGE_MODEL.md` when Knowledge is affected;
- relevant use cases from `USE_CASES.md`;
- relevant vertical documents;
- aggregate and persistence sections of `SOFTWARE_ARCHITECTURE.md`.

## Exclude by default

- AI, memory and context documents unless their source objects change;
- unrelated verticals.

## Preserve

- seven Core Entities unless an explicit canonical revision is approved;
- one logical owner per fact and relationship;
- lifecycle and cardinality consistency;
- technology-neutral logical types.

## Tests and outputs

- updated logical invariants;
- migration impact assessment when code exists;
- affected use-case traceability;
- conformance tests for aggregate rules.

---

# Bundle 4 — Implement a Domain Module

Use for implementing one domain module such as Projects, Documents, Knowledge, Tasks, Activities, People or Resources.

## Load in full

- `SYSTEM_PRINCIPLES.md`
- relevant entity section of `LOGICAL_DOMAIN_MODEL.md`
- domain-module and dependency sections of `SOFTWARE_ARCHITECTURE.md`

## Load selectively

- relevant conceptual definitions from `DOMAIN_MODEL.md`;
- relevant domain ownership from `DOMAIN_MAP.md`;
- use cases that mutate or query the module;
- event definitions the module emits.

## Exclude by default

- unrelated entities and verticals;
- AI and interaction models unless the module participates directly in those flows.

## Preserve

- aggregate boundaries;
- command validation and transition invariants;
- no infrastructure dependency inside the domain;
- event-after-commit;
- no projection as write authority.

## Expected output

- domain types and rules;
- application commands/queries;
- ports;
- unit and architecture tests;
- no provider-specific code in the module core.

---

# Bundle 5 — Implement One Use Case

Use for a vertical slice corresponding to one or a small coherent group of use cases.

## Load in full

- the selected use case entries from `USE_CASES.md`;
- relevant sections of `LOGICAL_DOMAIN_MODEL.md`;
- command/query/process sections of `SOFTWARE_ARCHITECTURE.md`.

## Load selectively

- capability definitions from `SYSTEM_CAPABILITIES.md`;
- corresponding vertical file;
- interaction channel and approval sections from `INTERACTION_MODEL.md`;
- relevant event definitions.

## Exclude by default

- the full use-case catalogue;
- unrelated verticals;
- the complete cognitive architecture when no AI operation is involved.

## Preserve

- intention and expected outcome;
- declared state changes;
- human approval boundaries;
- one authoritative mutation path;
- traceable events and effects.

## Expected output

- executable slice;
- acceptance tests derived from the use case;
- documentation impact only when behavior or mappings change.

---

# Bundle 6 — AI Operating Layer or Cognitive Runtime

Use for planning, reasoning, critique, synthesis, agent roles, model invocation or cognitive process coordination.

## Load in full

- `SYSTEM_ARCHITECTURE.md` sections on authority, Cognitive Plane and trust boundaries;
- `AI_ARCHITECTURE.md`;
- cognitive-runtime sections of `SOFTWARE_ARCHITECTURE.md`;
- `AGENT_RUNTIME.md`.

## Load selectively

- `CONTEXT_MODEL.md`;
- `MEMORY_MODEL.md`;
- `EVENT_MODEL.md`;
- capability definitions used by the operation;
- relevant use cases.

## Exclude by default

- unrelated verticals;
- provider-specific research unless the task is Technical Architecture;
- full logical entity catalogue when only a few entity types are used.

## Preserve

- models and agents are proposers, not state authorities;
- all context is explicitly constructed;
- capability access is mediated;
- proposals are validated before commitment or effect execution;
- model-provider state is replaceable.

## Expected output

- typed input/output contracts;
- context manifest;
- proposal and evidence model;
- failure and retry behavior;
- deterministic tests around non-deterministic providers.

---

# Bundle 7 — Memory or Recall

Use for semantic, episodic, working or procedural memory; recall; consolidation; retention or forgetting.

## Load in full

- `MEMORY_MODEL.md`
- relevant authority/state sections of `SYSTEM_ARCHITECTURE.md`
- memory-runtime sections of `SOFTWARE_ARCHITECTURE.md`

## Load selectively

- `KNOWLEDGE_MODEL.md` for semantic memory;
- `EVENT_MODEL.md` for episodic encoding;
- `CONTEXT_MODEL.md` for recall consumption;
- relevant logical entity sections.

## Exclude by default

- UI and vertical files;
- unrelated AI roles.

## Preserve

- world memory derives authority from canonical domain or historical state;
- operating memory never becomes shadow truth;
- recalled material is evidence, not an implicit mutation;
- retention and deletion remain policy-controlled and auditable.

---

# Bundle 8 — Context Construction

Use for context builders, relevance budgets, source selection, context manifests and task-specific views.

## Load in full

- `CONTEXT_MODEL.md`
- context and projection sections of `SYSTEM_ARCHITECTURE.md`
- context-construction sections of `SOFTWARE_ARCHITECTURE.md`

## Load selectively

- `MEMORY_MODEL.md` recall interfaces;
- `EVENT_MODEL.md` recent-change inputs;
- `INTERACTION_MODEL.md` user intent and workspace anchors;
- relevant AI operation contract.

## Exclude by default

- full domain and use-case catalogues;
- unrelated persistence details.

## Preserve

- context is bounded and operation-specific;
- context is non-authoritative;
- construction is reproducible from a manifest where required;
- permissions and policy filter every source;
- stale or missing sources are explicit.

---

# Bundle 9 — Event Runtime or Reaction

Use for domain events, outbox/inbox, subscriptions, process managers, retries or event-driven reactions.

## Load in full

- `EVENT_MODEL.md`
- event, consistency and operating-cycle sections of `SYSTEM_ARCHITECTURE.md`
- event-runtime, outbox, inbox and process-manager sections of `SOFTWARE_ARCHITECTURE.md`

## Load selectively

- aggregate boundaries in `LOGICAL_DOMAIN_MODEL.md`;
- use cases that trigger the reaction;
- interaction approval rules when effects are possible.

## Exclude by default

- cognitive documents unless a reaction invokes cognition;
- unrelated verticals.

## Preserve

- facts are emitted only after successful commitment;
- at-least-once delivery is handled idempotently;
- ordering assumptions are explicit;
- cascades terminate;
- events do not become commands by implication;
- external effects pass through authorization and reconciliation.

---

# Bundle 10 — Interaction, Workspace or UI

Use for dashboards, forms, search, graph views, calendars, kanban, chat, notifications or approval surfaces.

## Load in full

- `INTERACTION_MODEL.md`
- Experience and Control Plane sections of `SYSTEM_ARCHITECTURE.md`
- inbound-adapter, query and command sections of `SOFTWARE_ARCHITECTURE.md`

## Load selectively

- selected use cases;
- relevant projections and query contracts;
- context rules when AI interaction is involved;
- approval policy for consequential actions.

## Exclude by default

- storage implementation;
- unrelated cognitive internals;
- all use cases.

## Preserve

- chat is one channel, not the system;
- UI never mutates persistence directly;
- views are projections;
- intent is converted into typed commands or queries;
- approval and uncertainty are visible.

---

# Bundle 11 — External Integration or Effect

Use for email, calendar, repositories, external search, publication, communication, APIs or tool execution.

## Load in full

- execution and trust-boundary sections of `SYSTEM_ARCHITECTURE.md`;
- effect protocol, capability registry and adapter sections of `SOFTWARE_ARCHITECTURE.md`.

## Load selectively

- affected use cases;
- interaction approval rules;
- relevant event definitions;
- Resource or Document logical definitions when synchronized.

## Exclude by default

- internal models unrelated to the effect;
- all provider documentation until the abstract contract is fixed.

## Preserve

- prepare → authorize → execute → observe → reconcile;
- idempotency or explicit outcome-unknown handling;
- no direct provider call from domain or cognitive code;
- credentials and permissions are least-privilege;
- externally authoritative facts are re-observed rather than guessed.

---

# Bundle 12 — Persistence, Projection, Search or Graph

Use for repositories, data mapping, indexes, vector retrieval, graph projections, read models, caches or migrations.

## Load in full

- relevant entity and relationship sections of `LOGICAL_DOMAIN_MODEL.md`;
- persistence/projection sections of `SYSTEM_ARCHITECTURE.md`;
- port, repository, transaction and projection sections of `SOFTWARE_ARCHITECTURE.md`.

## Load selectively

- `KNOWLEDGE_MODEL.md` for semantic projections;
- `MEMORY_MODEL.md` for recall indexes;
- query requirements from selected use cases.

## Exclude by default

- provider products and benchmarks unless performing Technical Architecture;
- UI and unrelated verticals.

## Preserve

- persistence realizes the model; it does not define it;
- projections are rebuildable;
- one authoritative write owner per fact;
- migration and replay behavior is explicit;
- canonical decisions do not depend on stale projections.

---

# Bundle 13 — Security, Policy, Approval or Audit

Use for permissions, autonomy levels, approval gates, policy evaluation, audit records, provenance or sensitive-data boundaries.

## Load in full

- security, governance and trust-boundary sections of `SYSTEM_ARCHITECTURE.md`;
- policy, authorization, audit and effect sections of `SOFTWARE_ARCHITECTURE.md`;
- approval and initiative sections of `INTERACTION_MODEL.md`.

## Load selectively

- `SYSTEM_PRINCIPLES.md`;
- relevant use cases;
- event definitions;
- logical provenance fields.

## Exclude by default

- unrelated domain breadth;
- provider-specific security features before requirements are fixed.

## Preserve

- least privilege;
- explicit policy decisions;
- human authority for high-consequence behavior;
- audit distinct from domain-event semantics;
- no secrets in domain state, prompts or logs.

---

# Bundle 14 — Vertical Extension

Use when adding or expanding Personal, Daily Work, Research, Teaching, Administration or Organization behavior.

## Load in full

- `DOMAIN_VERTICALS.md`
- the relevant vertical document;
- relevant use cases;
- relevant parts of `DOMAIN_MODEL.md` and `LOGICAL_DOMAIN_MODEL.md`;
- owning component sections from `COMPONENT_MODEL.md`. 

## Load selectively

- capability definitions;
- neighboring verticals only to test overlap;
- operational model for real-world justification.

## Exclude by default

- software runtime documents when the task is still domain discovery;
- unrelated verticals.

## Preserve

- no new Core Entity for vocabulary that can be a Derived Type;
- use cases remain canonical in `USE_CASES.md` and are referenced, not copied;
- verticals are views over one domain, not separate models.

---

# Bundle 15 — System or Software Architecture Change

Use for new subsystems, authority changes, dependency changes, consistency semantics or runtime topology changes.

## Load in full

- `VISION.md`
- `SYSTEM_PRINCIPLES.md`
- `SYSTEM_ARCHITECTURE.md`
- `SOFTWARE_ARCHITECTURE.md`
- `DOCUMENTATION_ARCHITECTURE.md`

## Load selectively

- every specialized owner affected by the change;
- `LOGICAL_DOMAIN_MODEL.md` when state or aggregates change;
- `ROADMAP.md` and `IMPLEMENTATION_PLAN.md` for sequencing and evidence.

## Exclude by default

- nothing relevant may be excluded merely to save context; instead narrow the architectural concern first.

## Preserve

- explicit ownership and trust boundaries;
- inward dependencies;
- replaceable mechanisms;
- failure containment and recovery;
- testable conformance rules.

## Additional requirement

Produce or update an ADR before implementation.

---

# Bundle 16 — Technical Architecture or Deployment

Use only after the roadmap authorizes concrete technology selection.

## Load in full

- `SYSTEM_ARCHITECTURE.md`
- `SOFTWARE_ARCHITECTURE.md`
- `ROADMAP.md`
- relevant ADRs when present.

## Load selectively

- workload and quality requirements from use cases;
- `COMPONENT_MODEL.md` for the components being deployed;
- `DATA_ARCHITECTURE.md` for stores, schemas, retention and recovery;
- `AGENT_RUNTIME.md` when cognitive execution is involved;
- persistence, model, integration and deployment contracts being realized;
- results from `IMPLEMENTATION_PLAN.md` experiments.

## Exclude by default

- vendor comparisons unrelated to a required contract;
- speculative scale requirements.

## Preserve

- technology choices remain adapters to architectural ports;
- the proving slice informs choices;
- operational simplicity is preferred until evidence demands distribution;
- every selected product has an exit boundary.

---

# Bundle 17 — Local Bug Fix or Refactor

Use after code exists for changes that do not intentionally alter behavior or architecture.

## Load in full

- the affected source module and tests;
- the authoritative document for the local contract.

## Load selectively

- nearest inbound/outbound interface;
- relevant use case or invariant;
- architecture test rules.

## Exclude by default

- repository-wide documentation;
- unrelated models and verticals.

## Preserve

- existing public behavior;
- module dependency rules;
- state ownership;
- test coverage.

Escalate to another bundle if the fix reveals that an architectural contract must change.

---


# Bundle 18 — Development Specification

Use when producing an implementable specification for one bounded component, data family, runtime operation or proving-slice area.

## Load in full

- `COMPONENT_MODEL.md` section for the owning component;
- the relevant owner among `DATA_ARCHITECTURE.md`, `AGENT_RUNTIME.md` or `TECHNICAL_ARCHITECTURE.md`;
- relevant command/query/process sections of `SOFTWARE_ARCHITECTURE.md`;
- the selected use case or experiment acceptance criteria.

## Load selectively

- affected entity/invariant sections from `LOGICAL_DOMAIN_MODEL.md`;
- related event definitions;
- relevant capability definitions;
- interaction/approval semantics;
- relevant ADRs and Technical Architecture sections.

## Exclude by default

- unrelated components and verticals;
- implementation details belonging to another future specification;
- speculative scale or provider features outside the proving slice.

## Preserve

- one owning component and one write owner per state family;
- exact traceability from use case to command/query/event/data/test;
- no code task without acceptance criteria;
- no technology that contradicts `TECHNICAL_ARCHITECTURE.md`;
- non-deterministic behavior surrounded by deterministic contracts and tests.

## Required specification sections

1. Scope and non-goals.
2. Concern owner and implementing component.
3. Relevant use cases and experiment hypothesis.
4. Domain entities, invariants and state transitions.
5. Commands, queries, events, jobs, processes, proposals and effects.
6. Public and driven ports.
7. Exact logical/physical data schema and migration.
8. API/UI or worker contracts.
9. Authorization, approval and data classification.
10. Failure, idempotency, retry and recovery.
11. Observability and audit.
12. Unit, property, architecture, integration and acceptance tests.
13. Implementation tasks and dependency order.
14. Completion criteria and known deferrals.

## Expected output

A standalone specification that a coding agent can implement without loading the entire repository or inventing architectural decisions.

# Combining Bundles

Combine bundles only when one task genuinely crosses ownership boundaries.

Examples:

- AI-assisted literature-question flow: Bundle 5 + Bundle 6 + Bundle 8.
- Calendar synchronization with approval: Bundle 5 + Bundle 10 + Bundle 11 + Bundle 13.
- Knowledge graph projection: Bundle 3 + Bundle 12.

When combining bundles:

1. Name one primary bundle.
2. Load duplicate prerequisites once.
3. Preserve the union of invariants.
4. Identify each concern owner explicitly.
5. Split the task if more than four bundles are required.

A task requiring five or more bundles is probably too broad for one coding-agent iteration.

---

# Context Assembly Example

```text
Task:
Implement UC-K05 Support Research Question.

Primary context bundle:
Bundle 5 — Implement One Use Case.

Secondary bundles:
Bundle 6 — AI Operating Layer.
Bundle 8 — Context Construction.

Concern owners:
USE_CASES.md, AI_ARCHITECTURE.md, CONTEXT_MODEL.md.

Load in full:
- UC-K05 from USE_CASES.md
- AI_ARCHITECTURE.md
- CONTEXT_MODEL.md

Load selectively:
- SYSTEM_ARCHITECTURE.md: authority and Cognitive Plane
- SOFTWARE_ARCHITECTURE.md: cognitive runtime, proposals, commands and artifacts
- LOGICAL_DOMAIN_MODEL.md: Knowledge, Document and Activity
- MEMORY_MODEL.md: Recall interfaces

Explicit exclusions:
- Teaching, Administration and Organization verticals
- deployment and provider selection

Invariants:
- context is constructed and transient
- generated answers are artifacts until accepted
- claims remain attributable to evidence
- AI cannot commit domain state directly
```

---

# Agent Completion Report

Every coding-agent task should end with:

```text
Context bundle used:
Concern owners consulted:
Invariants preserved:
Tests executed:
Architecture checks executed:
Documentation reviewed:
Downstream files changed:
Downstream files reviewed but unchanged:
ADR required: yes/no
Known limitations:
```

---

# Final Rule

> Load the smallest context that contains every authority required by the task, and no context merely because it exists.

The goal is bounded correctness, not maximal prompt size.
