# Component Model

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 2 — Specialized Implementation Model |
| **Normative status** | Canonical for implementation component decomposition |
| **Authoritative for** | Logical components, component responsibilities, owned state, public contracts, allowed dependencies, runtime placement, failure boundaries and the minimum component set for the proving slice. |
| **Not authoritative for** | Domain meaning or invariants, system-wide authority, software architectural style, logical data schemas, internal agent execution semantics, concrete technologies or deployment products. |
| **Required reading** | `SYSTEM_PRINCIPLES.md`, `SYSTEM_ARCHITECTURE.md`, `SOFTWARE_ARCHITECTURE.md`, `LOGICAL_DOMAIN_MODEL.md`, `DECISIONS.md` (ADR-0001 through ADR-0006). |
| **Downstream documents** | `DATA_ARCHITECTURE.md`, `AGENT_RUNTIME.md`, `TECHNICAL_ARCHITECTURE.md`, area specifications, source-tree design, architecture tests and deployment manifests. |

> This document derives executable components from the canonical software modules. It does not create a second software architecture.

> **Status: Canonical · v1.0.** This document defines the logical component decomposition that implementation specifications must follow.

---

## Purpose

`SOFTWARE_ARCHITECTURE.md` defines the implementation-neutral software structure of ResearchOS: modules, ports, adapters, runtime roles and execution protocols.

This document makes that structure operational by defining the **logical components** that must exist in an implementation.

It answers:

- Which cohesive components implement the software architecture?
- What responsibility and state does each component own?
- Which contracts does each component expose or consume?
- Which dependencies are allowed?
- In which runtime roles may each component execute?
- Which failures must each component contain or surface?
- Which components are required for the first proving slice?

It deliberately does not answer:

- which programming language is used;
- which database or framework is selected;
- whether a component becomes a package, process, container or service;
- how an agent reasons internally;
- how physical schemas are laid out.

Those questions belong downstream.

---

# Position in the Architecture

```text
System Architecture
        ↓ defines authority and planes
Software Architecture
        ↓ defines modules, ports and runtimes
Component Model
        ↓ defines cohesive executable responsibilities
Data Architecture
        ↓ defines owned data and physical independence rules
Agent Runtime
        ↓ defines the cognitive execution harness
Technical Architecture
        ↓ selects technologies and deployment mechanisms
Area Specifications
        ↓ define implementable contracts and acceptance tests
```

The Component Model is a projection of the Software Architecture.

If this document conflicts with `SOFTWARE_ARCHITECTURE.md`, the Software Architecture owns the rule and this document must be corrected.

---

# Core Definitions

## Module

A **module** is a static ownership and dependency boundary.

Examples:

- Knowledge;
- Documents;
- Application;
- Cognitive;
- Events.

A module organizes code and authority.

## Component

A **component** is a cohesive executable responsibility with:

- one explicit owner;
- a bounded public contract;
- declared dependencies;
- owned state, if any;
- failure and recovery semantics;
- one or more permitted runtime placements.

A component may contain several classes or functions. It is not synonymous with a class.

## Runtime Role

A **runtime role** describes how work executes:

- Interactive Runtime;
- Background Worker Runtime;
- Scheduler and Dispatcher Runtime;
- Isolated Execution Runtime;
- Maintenance Runtime.

A component may execute in more than one runtime role without changing ownership.

## Deployment Unit

A **deployment unit** is a process, container, executable or externally hosted service.

Deployment units are technical decisions. They do not define component boundaries.

Several components may run in one deployment unit. One component may have several runtime instances.

## Adapter

An **adapter** implements a port at a technology or external-system boundary.

Adapters translate. They do not own domain policy.

## Process

A **process** is a durable coordination instance that advances through commands, events, timers and approvals.

A Process Manager component owns process progression. It does not own the aggregates it coordinates.

---

# Component Design Principles

## 1. Components Follow Authority

A component may only own state and decisions permitted by its architectural plane.

No component gains authority merely because it executes asynchronously or invokes an AI model.

## 2. Components Are Not Microservices by Default

The reference implementation is a modular monolith.

Component boundaries MUST be enforceable in code before any network boundary is introduced.

## 3. One Cohesive Responsibility

A component should have one primary reason to change.

A component that owns unrelated policies, storage, orchestration and external effects is incorrectly scoped.

## 4. Contracts Before Mechanisms

Components depend on typed ports and records, not on another component's private classes, tables or provider SDKs.

## 5. State Has One Owner

Every durable state family has one logical write owner.

Other components may observe it through contracts or projections but do not mutate it directly.

## 6. Failures Are Contained

A component declares:

- what can fail;
- whether failure is retryable;
- what state survives;
- how the caller observes incomplete work;
- when human escalation is required.

## 7. Components Remain Replaceable

Provider-specific behavior remains behind adapters.

The component contract must survive a change of database, model provider, search engine or deployment topology.

## 8. Thin Slice Before Completeness

The proving slice implements the smallest component set that preserves the complete authority path.

Deferred components remain architectural placeholders, not empty code packages.

---

# High-Level Component Topology

```text
┌───────────────────────────────────────────────────────────────────────┐
│                         Experience Components                         │
│ Workspace · HTTP API · CLI · Import · Notification Delivery          │
└───────────────────────────────┬───────────────────────────────────────┘
                                │ driving contracts
                                ▼
┌───────────────────────────────────────────────────────────────────────┐
│                         Application Components                        │
│ Command · Query · Proposal/Approval · Policy · Use-Case Coordination  │
└──────────────────────┬──────────────────────────┬─────────────────────┘
                       │                          │
                       ▼                          ▼
┌──────────────────────────────┐    ┌───────────────────────────────────┐
│       Domain Components      │    │ Cognitive & Process Components   │
│ Seven modules + Shared Kernel│    │ Context · Agent Runtime · Jobs   │
└──────────────┬───────────────┘    └──────────────────┬────────────────┘
               │ driven contracts                     │ capability contracts
               ▼                                      ▼
┌───────────────────────────────────────────────────────────────────────┐
│                    Platform Adapter Components                        │
│ Data · Content · Events · Search · Models · Tools · Integrations      │
└───────────────────────────────┬───────────────────────────────────────┘
                                │
                                ▼
┌───────────────────────────────────────────────────────────────────────┐
│                       Operational Components                          │
│ Identity · Policy · Secrets · Telemetry · Audit · Health · Maintenance│
└───────────────────────────────────────────────────────────────────────┘
```

This diagram shows responsibility flow. It does not imply that every box is a process.

---

# Component Groups

ResearchOS defines seven component groups.

| Group | Architectural plane | Primary responsibility |
|---|---|---|
| Experience | Experience Plane | Human and machine interaction adapters |
| Application | Control Plane | Intent, orchestration, policy and approval |
| Domain | Domain Plane | Canonical state, identity and invariants |
| Cognitive | Cognitive Plane | Contextual reasoning and proposal production |
| Process & Event | Control / Execution | Durable progression and asynchronous reaction |
| Platform Adapters | Persistence / Integration | Mechanism implementation behind ports |
| Operations | Cross-cutting | Security, observability, configuration and recovery |

---

# Experience Components

## CMP-EXP-01 · Workspace Gateway

**Responsibility:** expose the coherent human workspace without leaking internal runtimes, repositories or model providers.

**Owns:**

- interface-local state;
- user navigation state;
- presentation composition;
- optimistic interaction state that is explicitly non-canonical.

**Consumes:**

- Command Port;
- Query Port;
- Proposal Port;
- approval and notification streams.

**Must not:**

- mutate repositories;
- embed domain invariants;
- invoke model providers directly;
- treat cached UI state as canonical.

**Runtime:** Interactive.

## CMP-EXP-02 · HTTP API Adapter

**Responsibility:** translate authenticated HTTP requests into application commands, queries, proposal decisions and process operations.

**Owns:**

- transport schemas;
- request correlation;
- protocol-level validation;
- response serialization.

**Must not:**

- expose aggregate persistence models;
- execute domain mutations outside handlers;
- return provider-specific errors as application contracts.

**Runtime:** Interactive.

## CMP-EXP-03 · CLI Adapter

**Responsibility:** provide expert and maintenance interaction through the same application ports used by other interfaces.

**Runtime:** Interactive or Maintenance according to the invoked port.

## CMP-EXP-04 · Ingestion Adapter

**Responsibility:** receive files, URLs or external payloads and normalize them into explicit ingestion commands or proposals.

**Owns:** transient upload/session state only.

**Must not:** write Documents or Knowledge directly.

**Runtime:** Interactive for admission; Background or Isolated for processing.

## CMP-EXP-05 · Notification Delivery Adapter

**Responsibility:** render and deliver notifications prepared by the Application or Process layer.

**Owns:** delivery attempts and transport references, not notification meaning.

**Runtime:** Background.

---

# Application Components

## CMP-APP-01 · Command Gateway

**Responsibility:** execute explicit mutation intent through the canonical command protocol.

**Owns:**

- command admission;
- idempotency checks;
- actor and operation context;
- handler routing;
- unit-of-work coordination;
- stable application results.

**Consumes:** Domain repositories, policy decisions, Unit of Work and Outbox ports.

**Produces:** committed state transitions and pending Domain Events.

**Runtime:** Interactive, Background, Scheduler or Maintenance depending on actor and command source.

## CMP-APP-02 · Query Gateway

**Responsibility:** execute read requests against authoritative or declared projection sources.

**Owns:**

- query routing;
- consistency-class enforcement;
- pagination and filtering contracts;
- projection-lag disclosure where relevant.

**Must not:** mutate canonical state.

**Runtime:** Interactive and Background.

## CMP-APP-03 · Proposal and Approval Service

**Responsibility:** persist, present and resolve typed Proposals and approval decisions.

**Owns:**

- Proposal lifecycle;
- proposal digest and source evidence references;
- approval/rejection history;
- approval expiry and constraints;
- conversion of an accepted proposal into an explicit Command or Effect Intent.

**Must not:** allow an approval to authorize materially changed content.

**Runtime:** Interactive and Background.

## CMP-APP-04 · Policy and Authorization Service

**Responsibility:** evaluate whether an actor or autonomous behaviour may read, propose, mutate or execute an effect.

**Owns:**

- policy evaluation contracts;
- autonomy grants;
- risk and reversibility classification;
- authorization decisions and reasons.

**Must not:** allow a cognitive component to widen its own authority.

**Runtime:** all roles through a stable port.

## CMP-APP-05 · Use-Case Coordinator

**Responsibility:** implement bounded application orchestration that does not require durable process state.

**May coordinate:**

- multiple reads;
- policy checks;
- one aggregate mutation by default;
- proposal creation;
- process initiation;
- side-effect-free domain services.

**Must not:** become a generic service layer or coordinate long-running work in memory.

**Runtime:** Interactive or Background.

## CMP-APP-06 · External Effect Service

**Responsibility:** implement Prepare–Authorize–Execute–Observe–Reconcile for external effects.

**Owns:**

- Effect Intent lifecycle;
- approval binding;
- idempotency key;
- effect observation and ambiguity state;
- reconciliation commands.

**Runtime:** Background; execution may delegate to Isolated.

---

# Domain Components

Each Domain component implements one Core Entity aggregate family and its domain-specific contracts.

| Component | Aggregate root | Primary ownership |
|---|---|---|
| CMP-DOM-01 · Knowledge Domain | Knowledge | epistemic state, provenance, validation, semantic relations |
| CMP-DOM-02 · Documents Domain | Document | identity, content references, metadata, versions and document lifecycle |
| CMP-DOM-03 · Projects Domain | Project | objectives, status, memberships and project relationships |
| CMP-DOM-04 · People Domain | Person | identity, roles, affiliations and person relationships |
| CMP-DOM-05 · Tasks Domain | Task | actionable commitments, assignment, priority and lifecycle |
| CMP-DOM-06 · Activities Domain | Activity | observed or recorded work episodes and outcomes |
| CMP-DOM-07 · Resources Domain | Resource | capacity, allocation, availability and resource lifecycle |
| CMP-DOM-08 · Shared Kernel | — | identifiers, versions, provenance primitives, actor/causation references and domain errors |

## Domain Component Contract

Every Domain component MUST:

- expose intention-revealing aggregate operations;
- enforce local invariants before state changes;
- record pending Domain Events;
- reference other aggregates by identifier;
- remain independent of application, cognitive and infrastructure code;
- expose repository ports without persistence-specific query languages;
- preserve identity across state and version changes.

## Cross-Domain Coordination

No Domain component directly mutates another Domain component.

Cross-aggregate progression uses:

- an Application command sequence;
- committed Domain Events;
- or a durable Process Manager.

A relationship has one canonical write owner even when it is navigable from both ends.

---

# Cognitive Components

## CMP-COG-01 · Cognitive Operation Manager

**Responsibility:** admit, persist and coordinate bounded cognitive operations.

**Owns:**

- cognitive-operation lifecycle;
- budgets and deadlines;
- checkpoint references;
- correlation and causation;
- final Artifact, Proposal or failure result.

**Delegates:** execution semantics to the Agent Runtime.

**Must not:** commit domain state.

**Runtime:** Background.

## CMP-COG-02 · Context Builder

**Responsibility:** construct task-specific Context from authorized sources.

**Owns:**

- context-construction execution;
- source selection and ranking;
- Context Manifest;
- transient Context Package.

**Consumes:** Query, recall, policy and projection ports.

**Must not:** treat assembled context as canonical state.

**Runtime:** Background; small deterministic builds may execute interactively.

## CMP-COG-03 · Recall Coordinator

**Responsibility:** perform governed recall across World Memory and Operating Memory mechanisms.

**Owns:**

- recall request normalization;
- source selection;
- result scoring and deduplication;
- traceable Recall Results.

**Must not:** create Knowledge merely because content was retrieved.

**Runtime:** Background and Interactive query support.

## CMP-COG-04 · Agent Runtime Host

**Responsibility:** execute typed cognitive tasks through planning, context use, capability calls, verification and bounded recovery.

**Owns:** ephemeral execution state and durable checkpoints delegated through operational ports.

**Internal semantics:** owned by `AGENT_RUNTIME.md`.

**Must not:** own persistent world state, provider conversations or permissions.

**Runtime:** Background; risky steps may delegate to Isolated.

## CMP-COG-05 · Verification Service

**Responsibility:** verify structure, evidence, policy compliance and declared quality properties of cognitive output.

**Owns:** verification records and reasons.

**May use:** deterministic validators, retrieval, tools or bounded model review.

**Must not:** convert a verified output directly into canonical state.

**Runtime:** Background.

## CMP-COG-06 · Artifact and Proposal Builder

**Responsibility:** transform cognitive results into typed Artifacts or Proposals with provenance, evidence, confidence and intended next action.

**Must distinguish:**

- transient Artifact;
- persisted Document candidate;
- Knowledge candidate;
- proposed Command;
- proposed Effect Intent.

**Runtime:** Background.

---

# Process and Event Components

## CMP-PRO-01 · Process Manager Host

**Responsibility:** own durable multi-step workflows with explicit state, deadlines, compensation and completion conditions.

**Owns:** Process Instances and process-specific state machines.

**Consumes:** Domain Events, timers, approval decisions and Effect Results.

**Produces:** Commands, jobs, proposals and effect intents.

**Must not:** mutate aggregate storage directly.

**Runtime:** Background and Scheduler.

## CMP-PRO-02 · Job Coordinator

**Responsibility:** persist and lease executable background jobs.

**Owns:**

- job state;
- readiness;
- attempts;
- lease owner and expiry;
- retry classification;
- terminal failure or escalation.

**Runtime:** Background and Scheduler.

## CMP-EVT-01 · Outbox Dispatcher

**Responsibility:** publish committed Outbox entries without changing event meaning.

**Owns:** dispatch attempts and checkpoints.

**Guarantee:** at-least-once delivery.

**Runtime:** Scheduler and Dispatcher.

## CMP-EVT-02 · Event Reaction Host

**Responsibility:** route event envelopes to bounded, idempotent consumers.

**Owns:** Inbox/deduplication records and consumer checkpoints.

**Must not:** reinterpret an event as an instruction.

**Runtime:** Background.

## CMP-PRO-03 · Scheduler

**Responsibility:** activate time-based or readiness-based work.

**Owns:** schedules, wake-up times and scheduler leases.

**Must not:** decide domain or cognitive priorities outside declared policies.

**Runtime:** Scheduler and Dispatcher.

## CMP-PRJ-01 · Projection Coordinator

**Responsibility:** build, checkpoint, invalidate and rebuild query projections.

**Owns:** projection definitions, generation versions and rebuild status.

**Runtime:** Background and Maintenance.

---

# Platform Adapter Components

## CMP-DAT-01 · Canonical Persistence Adapter

Implements aggregate repositories, Unit of Work and atomic Outbox commitment.

It does not own domain meaning.

## CMP-DAT-02 · Operational Persistence Adapter

Persists jobs, processes, proposals, approvals, cognitive operations, model/tool invocation metadata, Inbox state and effect records.

## CMP-DAT-03 · Content Repository Adapter

Stores and retrieves immutable or versioned content through locator and digest contracts.

## CMP-DAT-04 · Projection Store Adapters

Implement read models, lexical indexes, vector indexes, graph projections, timelines and caches.

Every adapter declares rebuild semantics.

## CMP-INT-01 · Capability Registry

Owns the executable catalogue that maps stable system capabilities to policy-scoped implementations.

The registry exposes typed capability metadata, not arbitrary callable code.

## CMP-INT-02 · Model Gateway

Normalizes model invocation, structured output, token/cost usage, provider errors and model identity.

Provider-managed conversation state is not accepted as system state.

## CMP-INT-03 · Tool Gateway

Validates and executes typed tool calls through the Capability Registry.

It enforces parameter schemas, actor scope, budgets, isolation requirement and result normalization.

## CMP-INT-04 · External Integration Gateway

Coordinates adapters for external information systems and communication platforms.

External payloads are normalized before they enter application commands or proposals.

## CMP-INT-05 · Isolated Execution Gateway

Delegates untrusted code, browser automation, risky document processing or privileged tools to a stronger isolation boundary.

It returns a normalized result and captured evidence.

---

# Operational Components

## CMP-OPS-01 · Identity and Session Service

Authenticates actors and supplies stable actor identity to application operations.

## CMP-OPS-02 · Secret and Credential Service

Provides scoped credentials to authorized adapters without exposing them to domain objects, prompts or ordinary logs.

## CMP-OPS-03 · Telemetry and Trace Service

Collects correlated logs, traces and metrics across command, event, process, cognitive and effect lifecycles.

## CMP-OPS-04 · Audit Service

Persists append-oriented security and authority-relevant records.

Audit state does not replace domain history or provenance.

## CMP-OPS-05 · Configuration Service

Provides validated configuration and feature-policy values with explicit environment and version identity.

## CMP-OPS-06 · Health and Readiness Service

Reports component readiness, dependency degradation, backlog and projection lag.

## CMP-OPS-07 · Maintenance Coordinator

Executes privileged rebuild, migration, integrity and reconciliation operations through explicit Maintenance ports.

---

# Component Contract Categories

Components communicate using the contracts already defined by the Software Architecture.

| Category | Examples | Semantic rule |
|---|---|---|
| Intent | Command, Query, Proposal decision | asks the system to do or return something |
| Fact | Domain Event, Integration Event, Effect Result | records something that happened |
| Durable work | Job, Process Instance | survives restart and has explicit lifecycle |
| Cognitive | Cognitive Operation, Context Manifest, Artifact | bounded reasoning state and output |
| Capability | Capability Request, Tool Result, Model Result | typed access to replaceable mechanisms |
| Data | Repository, Content, Projection, Recall ports | reads or persists declared state classes |
| Operations | Trace, Audit, Health, Configuration | cross-cutting support without domain authority |

Private data structures MUST NOT cross a component boundary merely because the components share a process.

---

# Allowed Dependency Matrix

`✓` means a direct code dependency on the target component's public contract is permitted.

| From \ To | Experience | Application | Domain | Cognitive | Process/Event | Platform Adapters | Operations |
|---|---:|---:|---:|---:|---:|---:|---:|
| Experience | — | ✓ | — | — | — | — | ✓ |
| Application | — | — | ✓ | via ports | via contracts | via ports | ✓ |
| Domain | — | — | Shared Kernel only | — | event base only | — | — |
| Cognitive | — | application/cognitive ports | read contracts only | — | job/process ports | capability/data ports | ✓ |
| Process/Event | — | command/process ports | event contracts only | cognitive-operation port | — | dispatch/persistence ports | ✓ |
| Platform Adapters | — | implements ports | implements ports | implements ports | implements ports | — | ✓ |
| Operations | — | stable metadata only | — | stable metadata only | stable metadata only | stable metadata only | — |

## Prohibited Dependencies

The following are architecture violations:

- Domain → Application;
- Domain → Cognitive;
- Domain → persistence or provider SDK;
- Experience → repository implementation;
- Cognitive → canonical write repository;
- Process Manager → aggregate table;
- adapter → another adapter's private implementation;
- projection builder → canonical mutation path;
- model provider → capability implementation without the Tool Gateway;
- any component → secrets through ordinary configuration objects or prompts.

---

# Runtime Placement

| Component group | Interactive | Background | Scheduler/Dispatcher | Isolated | Maintenance |
|---|---:|---:|---:|---:|---:|
| Experience | Primary | notification only | — | — | CLI only |
| Application | Primary | commands from processes | scheduled commands | effect admission only | privileged ports |
| Domain | In-process | In-process | In-process through commands | never exposed directly | In-process through commands |
| Cognitive | small admission only | Primary | activation only | delegated steps | evaluation/rebuild support |
| Process & Event | status/query only | Primary | Primary | — | replay/rebuild |
| Platform Adapters | request-scoped | Primary | Primary | Primary for risky tools | migration/rebuild |
| Operations | Primary | Primary | Primary | Primary | Primary |

This table permits placement. `TECHNICAL_ARCHITECTURE.md` decides the actual deployment topology.

---

# State Ownership Matrix

| State family | Logical write owner | Readers |
|---|---|---|
| Aggregate state | corresponding Domain component through Command Gateway | Query Gateway, Context Builder, projections |
| Domain relationships | declared owning Domain component | queries and graph projections |
| Document content | Content Repository Adapter under Documents contract | processors, context and interfaces |
| Proposals and approvals | Proposal and Approval Service | Workspace, processes, audit |
| Jobs | Job Coordinator | workers, scheduler, operations |
| Process Instances | Process Manager Host | Workspace, scheduler, operations |
| Cognitive operations | Cognitive Operation Manager | Agent Runtime, Workspace, audit |
| Context Manifest | Context Builder | Agent Runtime, audit, evaluation |
| Context body | transient execution owner | Agent Runtime only within scope |
| Outbox | Unit of Work / Canonical Persistence Adapter | Outbox Dispatcher |
| Inbox | Event Reaction Host | operations and maintenance |
| Projection state | Projection Coordinator / specific builder | Query Gateway and Workspace |
| Effect intents/results | External Effect Service | Workspace, processes, audit |
| Audit records | Audit Service | authorized audit and maintenance |
| Secrets | Secret and Credential Service | scoped adapters only |

`DATA_ARCHITECTURE.md` defines the logical records and lifecycle of these state families.

---

# Primary Interaction Protocols

## 1. Direct Command

```text
Interface
    → Command Gateway
    → Policy
    → Domain Component
    → Unit of Work + Outbox
    → Application Result
```

## 2. Query

```text
Interface / Context Builder
    → Query Gateway
    → authoritative read or declared projection
    → Query Result + consistency metadata
```

## 3. Event Reaction

```text
Committed Outbox entry
    → Outbox Dispatcher
    → Event Reaction Host
    → idempotent reaction
    → Command / Job / Projection update
```

## 4. Durable Process

```text
Trigger
    → Process Manager Host
    → persist process state
    → issue Command or Job
    → observe Event / Result / Timer
    → advance, compensate or escalate
```

## 5. Cognitive Operation

```text
Application or Process request
    → Cognitive Operation Manager
    → Context Builder
    → Agent Runtime Host
    → Verification Service
    → Artifact / Proposal Builder
    → Proposal and Approval Service or caller
```

## 6. External Effect

```text
Effect request
    → External Effect Service
    → Policy / Approval
    → Capability or Integration Gateway
    → observed Effect Result
    → reconciliation Command
```

## 7. Projection Update

```text
Domain Event
    → Event Reaction Host
    → Projection Coordinator
    → idempotent projection builder
    → checkpoint
```

---

# Failure Boundaries

## Synchronous Boundary

A synchronous request returns one of:

- success;
- validation rejection;
- authorization rejection;
- domain rejection;
- concurrency conflict;
- accepted asynchronous operation reference;
- infrastructure unavailable.

It does not wait indefinitely for cognitive or external work.

## Background Boundary

A background component persists progress before acknowledging work.

Unexpected termination must leave the job or process recoverable after lease expiry.

## Cognitive Boundary

Model or tool failure cannot corrupt canonical state.

Partial cognitive output remains operational state until verified and promoted through an explicit proposal or command.

## External Effect Boundary

Timeout is ambiguous, not automatically failed.

The External Effect Service observes and reconciles before repeating non-idempotent work.

## Projection Boundary

Projection failure degrades queries but does not invalidate canonical state.

The failed projection remains visible, checkpointed and rebuildable.

## Security Boundary

A denied capability or secret request is terminal for that attempt and is auditable.

The component does not retry until policy or authorization changes.

---

# Minimum Component Set for the Proving Slice

The first implementation MUST NOT scaffold every component in this document.

The required set is:

| Required component | Proving responsibility |
|---|---|
| HTTP API or CLI Adapter | admit a real paper and user decisions |
| Command Gateway | register Document, validate Knowledge and record decisions |
| Query Gateway | retrieve Document, candidate Knowledge and answer evidence |
| Policy and Authorization Service | enforce human validation and bounded AI access |
| Documents Domain | own document identity, metadata and content reference |
| Knowledge Domain | own candidate and validated Knowledge |
| Activities Domain | record ingestion/validation activity when required by the slice |
| Proposal and Approval Service | present and resolve Knowledge candidates |
| Cognitive Operation Manager | persist extraction and answer operations |
| Context Builder | construct source-bounded context |
| Agent Runtime Host | execute extraction and answer tasks |
| Verification Service | validate structured output and evidence references |
| Process Manager Host | coordinate document processing lifecycle |
| Job Coordinator | make processing durable |
| Outbox Dispatcher + Event Reaction Host | react after committed change |
| Canonical and Operational Persistence Adapters | persist authoritative and recoverable state |
| Content Repository Adapter | retain source document content |
| one Projection Adapter | lexical retrieval; vector projection only when Experiment 2 requires it |
| Model Gateway | invoke one configured model provider |
| Capability Registry / Tool Gateway | expose parsing and retrieval capabilities |
| Telemetry and Audit | preserve causality and experiment evidence |

## Explicitly Deferred from the First Slice

- independent microservices;
- external message broker;
- general multi-agent coordination;
- graph database;
- dedicated vector database;
- multiple model providers;
- browser automation;
- arbitrary code execution;
- generalized workflow engine;
- multi-user organization administration;
- automatic external effects beyond reversible notifications.

Deferred does not mean forbidden. It means evidence is required before implementation.

---

# Component-to-Specification Traceability

Every future area specification MUST identify:

1. the owning component;
2. the public contracts it implements or consumes;
3. owned and observed state;
4. commands, queries, events, jobs and proposals involved;
5. allowed runtime placement;
6. failure and retry semantics;
7. authorization and approval rules;
8. telemetry and audit requirements;
9. architecture tests required;
10. downstream data and deployment impact.

A specification that cannot name its owning component is not ready for implementation.

---

# Architecture Tests

The implementation MUST automate at least these component rules:

- Domain packages import only the Shared Kernel and their own domain code.
- Interface packages depend only on application-facing contracts.
- Cognitive packages cannot import canonical write repository adapters.
- Process packages issue commands through ports rather than repositories.
- Infrastructure packages implement inward-facing ports.
- No adapter imports another adapter's private package.
- Component public contracts contain no provider SDK types.
- Runtime bootstrap code is the only place allowed to compose concrete adapters.
- Projection code cannot be referenced by Domain mutation code.
- secrets are obtained only through the Secret and Credential port.

`TECHNICAL_ARCHITECTURE.md` selects the concrete tooling used to enforce these rules.

---

# Evolution Rules

A new component requires evidence that:

- it owns a cohesive responsibility not already owned;
- placing it inside an existing component would violate authority, lifecycle, isolation or failure boundaries;
- its public contract is stable enough to name;
- its state ownership is explicit;
- its runtime and recovery semantics are understood.

A component MUST NOT be created merely because:

- a class has grown large;
- a framework encourages one service per feature;
- an external API exists;
- an AI prompt has a different persona;
- a separate database is being considered;
- deployment could theoretically scale independently.

Component extraction into a separate deployment unit requires a later technical decision and, when significant, an ADR.

---

# Conformance Criteria

An implementation conforms to this Component Model only if:

1. every implemented responsibility maps to one declared component;
2. every durable state family has one logical write owner;
3. component dependencies obey the allowed dependency matrix;
4. runtime processes do not redefine component ownership;
5. Domain components remain independent of Application, Cognitive and infrastructure code;
6. Cognitive components produce Artifacts, Proposals or observations rather than canonical mutations;
7. Process Managers coordinate through Commands, Events, timers and results;
8. adapters translate mechanisms without owning domain policy;
9. synchronous interfaces do not execute unbounded background work inline;
10. failure, retry and recovery state remain observable;
11. the proving slice can be implemented without introducing an undeclared component;
12. architecture tests enforce the most important boundaries.

---

# Final Component Statement

ResearchOS is implemented through cohesive components that preserve the authority and dependency rules of the canonical architecture.

Domain components own truth.

Application components own intent, orchestration, policy and approval.

Cognitive components own bounded interpretation and proposal production.

Process and Event components own durable progression through time.

Platform adapters own contact with technologies and external systems.

Operational components make every path secure, observable and recoverable.

These components may share a deployment, but they never share authority by accident.
