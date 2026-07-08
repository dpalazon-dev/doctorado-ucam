# Software Architecture

> **Status: Canonical · v1.0.** This document translates the technology-independent System Architecture of ResearchOS into implementable software structures.
>
> It defines logical modules, runtime roles, contracts, transaction boundaries, ports, adapters, coordination mechanisms and conformance rules. It does not select programming languages, frameworks, databases, model providers, message brokers, deployment platforms or cloud products.

## Purpose

The System Architecture defines what must remain true across every implementation of ResearchOS.

This document defines how software must be organized so those truths are enforced by code rather than by convention.

It specifies:

- the reference software style;
- logical module boundaries and allowed dependencies;
- application, domain, cognitive, process and integration runtimes;
- command, query, proposal, approval, event and effect contracts;
- transaction and consistency mechanisms;
- persistence ports and reconstructible projections;
- long-running work and failure recovery;
- model and tool isolation;
- security, observability and audit mechanisms;
- test strategies that prove architectural conformance;
- the evolution path from a personal deployment to a physically distributed system.

The objective is not to maximize the number of components.

The objective is to create the smallest software architecture capable of preserving the coherence, continuity, human authority, traceability and evolvability required by ResearchOS.

---

# Position in the Documentation

```text
Conceptual and Logical Models
            ↓
SYSTEM_ARCHITECTURE.md
system-wide authority, planes and invariants
            ↓
SOFTWARE_ARCHITECTURE.md
modules, contracts, runtimes and mechanisms
            ↓
TECHNICAL_ARCHITECTURE.md
concrete technologies and deployment topology
            ↓
Implementation
source code, schemas, configuration and operations
```

This document implements the constraints of `SYSTEM_ARCHITECTURE.md`.

It does not redefine:

- the seven Core Entities;
- aggregate boundaries;
- Knowledge semantics;
- memory categories;
- context semantics;
- event semantics;
- human–AI interaction modes;
- domain verticals;
- system capabilities.

Those definitions remain owned by their canonical documents.

---

# Normative Language

The terms **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT** and **MAY** are normative.

- **MUST / MUST NOT** define software invariants.
- **SHOULD / SHOULD NOT** define the reference implementation unless an ADR records a justified exception.
- **MAY** defines permitted implementation variation.

---

# Architectural Objectives

The software architecture must make the following properties structural.

## Domain authority

Only domain operations may mutate canonical state.

## Human authority

AI-assisted and autonomous behaviours remain bounded by explicit policy, approval and reversibility rules.

## Replaceability

Interfaces, storage mechanisms, model providers, retrieval engines, execution tools and transports remain replaceable behind ports.

## Traceability

Every meaningful operation can be followed from initiating intent through validation, commitment, reaction and external effect.

## Recoverability

A failed worker, model call, projection update or external invocation cannot silently corrupt canonical state.

## Simplicity

Logical boundaries are strong; physical boundaries are introduced only when evidence requires them.

## Testability

Every architectural rule can be tested without requiring the complete deployed system.

---

# Reference Architectural Style

ResearchOS adopts a combination of complementary patterns.

## 1. Domain-centred modular architecture

The Domain Model is implemented as cohesive modules aligned with domain responsibility.

Software is not organized around screens, model providers, agents or database tables.

## 2. Ports and adapters

The application and domain core depend on abstract ports. Infrastructure implements those ports through adapters.

External systems never define the shape of the core.

## 3. Command–query separation

State-changing operations and read operations have different contracts, validation paths and consistency expectations.

This is logical CQRS, not necessarily separate physical stores.

## 4. Transactional domain kernel

Every command executes through an explicit unit of work, enforces aggregate invariants and commits state together with its pending domain events.

## 5. Event-driven reaction

Cross-aggregate coordination, projections and background cognitive work react to committed events.

Events complement commands; they do not replace explicit user intent or process orchestration.

## 6. Durable process execution

Long-running or multi-step operations are represented by explicit process state rather than by an in-memory call chain or an LLM conversation.

## 7. Propositional intelligence

The Cognitive Runtime produces typed outputs, artifacts, plans and proposals. It never receives direct mutation authority.

## 8. Modular monolith as the reference deployment

The initial and default physical architecture is a modular monolith with separately executable runtime roles.

```text
One codebase
One domain model
One architectural contract
        │
        ├── Interactive Runtime
        ├── Background Worker Runtime
        ├── Scheduler / Dispatcher Runtime
        └── Isolated Effect or Tool Runtime when required
```

A modular monolith is not an absence of architecture.

It is the deliberate choice to enforce boundaries in code before paying the coordination cost of distributed systems.

No logical module becomes a microservice by default.

---

# High-Level Software Topology

```text
┌───────────────────────────────────────────────────────────────────────┐
│                         Interface Adapters                            │
│ Workspace · API · CLI · Import · Automation · Notifications          │
└───────────────────────────────┬───────────────────────────────────────┘
                                │ driving ports
                                ▼
┌───────────────────────────────────────────────────────────────────────┐
│                         Application Core                              │
│ Commands · Queries · Proposals · Approvals · Policies · Use Cases    │
└──────────────────────┬──────────────────────────┬─────────────────────┘
                       │                          │
                       ▼                          ▼
┌──────────────────────────────┐    ┌───────────────────────────────────┐
│        Domain Kernel         │    │      Cognitive & Process Runtime │
│ Aggregates · Domain Services │    │ Context · Recall · Reasoning     │
│ Invariants · Domain Events   │    │ Plans · Verification · Jobs      │
└──────────────┬───────────────┘    └──────────────────┬────────────────┘
               │ driven ports                         │ capability ports
               ▼                                      ▼
┌───────────────────────────────────────────────────────────────────────┐
│                    Infrastructure Adapters                            │
│ Persistence · Projections · Event Delivery · Models · Tools          │
│ External Systems · Content · Search · Graph · Vector · Scheduling    │
└───────────────────────────────────────────────────────────────────────┘
```

The diagram shows dependency direction, not process placement.

Infrastructure depends inward on contracts. The Domain Kernel does not depend outward on infrastructure.

---

# Runtime Roles

A runtime role is an executable responsibility. Multiple roles may run in one process or in separate processes without changing the logical architecture.

## 1. Interactive Runtime

Owns low-latency, human-facing operations.

Responsibilities:

- serve Workspace and API interactions;
- authenticate the actor;
- execute synchronous commands and queries;
- create proposals and approval requests;
- return immediate operation status;
- subscribe interfaces to relevant projection updates.

The Interactive Runtime SHOULD NOT execute long model calls, document processing, large indexing operations or unbounded external tool workflows inline.

## 2. Background Worker Runtime

Owns asynchronous work.

Responsibilities:

- consume committed events;
- execute durable jobs;
- process documents;
- build and refresh projections;
- perform context construction and cognitive operations;
- consolidate operating memory;
- reconcile external effects;
- retry recoverable failures.

## 3. Scheduler and Dispatcher Runtime

Owns time- and readiness-based activation.

Responsibilities:

- release scheduled jobs;
- resume waiting processes;
- enforce leases and timeouts;
- apply concurrency and resource budgets;
- dispatch outbox records;
- identify abandoned work.

This is a work scheduler, not an AI authority or general-purpose cognitive kernel.

## 4. Isolated Execution Runtime

Owns operations that require stronger isolation than the main application.

Examples:

- running untrusted code;
- browser automation;
- invoking high-risk external tools;
- processing potentially malicious documents;
- handling credentials scoped to a particular integration.

Isolation MAY be implemented as a process, container, sandbox, remote worker or external service. The software contract remains the same.

## 5. Maintenance Runtime

Owns administrative and reconstructive operations.

Responsibilities:

- rebuild projections;
- verify integrity;
- migrate canonical data;
- reconcile external systems;
- replay safe reactions;
- rotate secrets and credentials;
- produce audit exports.

Maintenance operations MUST pass through explicit privileged application ports. They MUST NOT mutate storage through ad hoc scripts that bypass domain invariants, except for controlled disaster recovery governed by an ADR and an auditable runbook.

---

# Logical Module Map

ResearchOS is divided into nine top-level logical areas.

| Area | Owns | May depend on | Must not depend on |
|---|---|---|---|
| **Interfaces** | Workspace/API/CLI adapters | Application contracts | Domain implementation, persistence implementation |
| **Application** | use cases, commands, queries, policies, approvals | Domain contracts, runtime ports | concrete infrastructure adapters |
| **Domain** | aggregates, value objects, invariants, domain services | minimal shared kernel | application, cognitive, infrastructure |
| **Cognitive** | context, recall, reasoning, verification, proposal production | application/cognitive ports, domain read contracts | domain mutation, concrete providers |
| **Processes** | durable workflows, jobs, timers, compensation | application contracts, event contracts | direct aggregate persistence |
| **Events** | envelopes, outbox/inbox, dispatch contracts | domain event contracts | domain mutation |
| **Integrations** | external-system and tool adapters | capability/effect ports | domain internals |
| **Persistence & Projections** | repositories, read models, indexes | domain/application ports | interface concerns |
| **Operations** | telemetry, configuration, security, health, audit | stable contracts from all layers | business decisions |

The dependency graph MUST remain acyclic at the module level.

---

# Domain Modules

The Domain Kernel is composed of modules aligned with the Domain Map.

```text
Domain
├── Knowledge
├── Documents
├── Projects
├── People
├── Planning
│   ├── Tasks
│   └── Activities
├── Resources
└── Shared Kernel
```

Research is not a separate root module because it is an emergent process expressed through specializations and relationships across the existing modules.

Verticals such as Teaching, Administration and Organization extend the vocabulary of the same modules through Derived Types and policies; they do not introduce parallel domain stacks.

## Aggregate implementation

Each Core Entity remains an aggregate root as defined by the Logical Domain Model.

An aggregate implementation MUST:

- encapsulate its state;
- expose intention-revealing operations rather than generic setters;
- enforce its own invariants before producing a new state;
- record pending domain events caused by successful operations;
- reject invalid lifecycle transitions;
- retain causal and provenance information;
- reference other aggregates by identifier rather than by mutable object graph.

## Domain services

A Domain Service is permitted only when domain logic:

- is genuinely domain-specific;
- cannot naturally belong to one aggregate;
- remains independent of application orchestration and infrastructure.

Domain Services MUST NOT become generic service classes containing arbitrary use-case logic.

## Repository contracts

There is one repository abstraction per aggregate root.

A repository port exposes domain-oriented operations such as:

```text
get(identifier, expected_version?) → Aggregate | NotFound
save(aggregate, expected_version) → NewVersion
exists(identifier) → boolean
```

Repositories MUST NOT expose persistence-specific query languages to the Domain.

Complex retrieval belongs to query and projection ports, not aggregate repositories.

## Shared Kernel

The Shared Kernel MUST remain intentionally small.

It MAY contain:

- identifiers;
- timestamps and time ranges;
- provenance primitives;
- version primitives;
- domain-event base contracts;
- invariant and domain-error abstractions;
- actor and causation references.

It MUST NOT become a dumping ground for unrelated utilities, base entities, framework abstractions or infrastructure concerns.

---

# Application Core

The Application Core implements the system's use cases.

It owns orchestration but not domain rules.

Its responsibilities are:

- receive commands and queries through driving ports;
- establish actor, policy and operation context;
- validate request shape and authorization;
- load aggregates through repository ports;
- invoke domain behaviour;
- coordinate the unit of work;
- create proposals and approval requests;
- initiate long-running processes;
- return stable application results;
- expose no infrastructure-specific details to callers.

## Use-case handlers

Every externally meaningful operation maps to an explicit handler.

Handlers SHOULD be narrow and named after intent.

Examples:

```text
RegisterDocument
ClassifyDocument
CreateKnowledgeCandidate
ValidateKnowledge
RelateKnowledge
ScheduleTask
RecordActivity
RequestReasonedAnswer
ApproveProposal
ExecuteApprovedEffect
```

A handler MUST NOT be a generic CRUD endpoint when the domain expresses a more meaningful operation.

## Application services

An Application Service may coordinate:

- multiple reads;
- one aggregate mutation by default;
- policy checks;
- proposal creation;
- process initiation;
- calls to side-effect-free domain services.

A single command SHOULD commit at most one aggregate root.

A command that requires durable change across multiple aggregates SHOULD coordinate those changes through committed events or a Process Manager.

A multi-aggregate transaction requires an explicit ADR demonstrating why eventual consistency is insufficient and why the involved aggregates do not form a missing consistency boundary.

---

# Core Software Contracts

ResearchOS distinguishes contracts by semantic role.

## Command

A Command expresses an authorized intent to change canonical domain state.

A Command is imperative, typed and directed to one application handler.

Minimum envelope:

```text
Command
- command_id
- command_type
- actor
- issued_at
- payload
- target_identifier, when known
- expected_version, when applicable
- correlation_id
- causation_id
- idempotency_key
- policy_context
```

Commands are not facts. They may be rejected.

## Query

A Query requests information without mutating canonical state.

Queries MAY read projections, indexes and aggregate snapshots according to their consistency requirement.

Minimum envelope:

```text
Query
- query_id
- query_type
- actor
- issued_at
- parameters
- consistency_requirement
- correlation_id
- policy_context
```

Query handlers MUST NOT produce hidden domain mutation.

Incidental operational telemetry is permitted; business state change is not.

## Proposal

A Proposal is a typed suggestion produced by a human, rule, integration or cognitive operation before mutation authority is granted.

Minimum contents:

```text
Proposal
- proposal_id
- proposal_type
- proposer
- proposed_commands or proposed_effects
- rationale
- evidence references
- confidence or uncertainty, when meaningful
- provenance
- created_at
- expiry, when applicable
- required_approval_class
- correlation_id
```

A Proposal MAY be persisted as operating or historical state.

It is not canonical domain truth merely because it has been generated or stored.

## Approval Decision

An Approval Decision records a human or policy decision over a Proposal or Effect Intent.

```text
ApprovalDecision
- approval_id
- subject_reference
- decision: approved | rejected | revised | expired
- actor
- decided_at
- constraints
- rationale, when required
- subject_version or digest
```

Approval applies only to the exact version or digest reviewed. Material revision invalidates prior approval.

## Domain Event

A Domain Event records a fact caused by a successfully committed domain transition.

```text
DomainEvent
- event_id
- event_type
- aggregate_type
- aggregate_id
- aggregate_version
- occurred_at
- actor
- correlation_id
- causation_id
- payload
- schema_version
```

An event MUST NOT be published as committed before the state transition commits.

## Integration Event

An Integration Event is an outward-facing, versioned representation of a committed fact.

It protects external consumers from internal domain-event evolution.

Domain Events and Integration Events MAY initially share transport, but they remain different contracts.

## Effect Intent

An Effect Intent represents a requested change to an external system.

```text
EffectIntent
- effect_id
- capability
- target_system
- actor
- parameters
- risk_class
- reversibility
- approval_requirement
- idempotency_key
- preconditions
- correlation_id
- causation_id
```

The intent is not evidence that the external effect occurred.

## Effect Result

An Effect Result records the observed outcome of execution.

```text
EffectResult
- effect_id
- status: succeeded | failed | ambiguous | cancelled
- executed_at
- external_reference
- observation
- retryability
- error classification
- evidence
```

An ambiguous result MUST trigger reconciliation before blind retry when duplicate external action is possible.

## Job

A Job is a durable unit of background work.

```text
Job
- job_id
- job_type
- input_reference
- state
- priority
- attempts
- next_attempt_at
- lease_owner
- lease_expires_at
- timeout
- correlation_id
- causation_id
```

## Process Instance

A Process Instance is a durable state machine coordinating work across time, aggregates or external systems.

```text
ProcessInstance
- process_id
- process_type
- state
- current_step
- data references
- waiting_condition
- deadlines
- compensation state
- version
- correlation_id
```

A Process Instance is application operating state, not a new Core Entity.

## Trace Record

A Trace Record captures operational evidence required for audit and diagnosis.

It MAY reference:

- commands and queries;
- proposals and approvals;
- domain events;
- jobs and process steps;
- model invocations;
- tool invocations;
- context manifests;
- external effects;
- failures and retries.

Trace Records do not replace domain provenance.

---

# Driving Ports

Driving ports expose application behaviour to actors and runtimes.

The reference set is:

## Command Port

```text
execute(command) → CommandResult
```

## Query Port

```text
query(request) → QueryResult
```

## Proposal Port

```text
submit(proposal) → ProposalReceipt
review(proposal_id, approval_decision) → ReviewResult
```

## Process Port

```text
start(process_request) → ProcessReference
signal(process_id, signal) → ProcessState
cancel(process_id, reason) → ProcessState
```

## Event Reaction Port

```text
react(committed_event) → ReactionResult
```

## Maintenance Port

```text
rebuild_projection(name, range?)
verify_integrity(scope)
reconcile_external_state(scope)
```

Interfaces MUST invoke these ports rather than application internals.

---

# Driven Ports

Driven ports describe capabilities the core requires from infrastructure.

## Domain Repository Ports

One per aggregate root.

## Unit of Work Port

```text
begin()
commit()
rollback()
collect_domain_events()
```

The actual interface MAY be language-idiomatic, but the semantic transaction boundary is mandatory.

## Outbox Port

Stores events and dispatchable messages atomically with canonical state.

## Inbox Port

Records consumed message identifiers and reaction outcomes for idempotent processing.

## Projection Ports

```text
project(event)
rebuild(source_range)
checkpoint()
```

## Content Port

Stores and retrieves immutable or versioned binary/textual content addressed by locator or content digest.

## Search Ports

Separate ports MAY exist for:

- lexical search;
- semantic/vector retrieval;
- graph traversal;
- metadata filtering;
- temporal retrieval.

The application requests retrieval semantics, not a particular index technology.

## Model Port

```text
invoke(reasoning_request) → ModelResult
```

The Model Port hides provider protocols, but it does not erase relevant properties such as modality, context limit, structured-output support, cost class, latency class or data-governance constraints.

## Embedding Port

```text
embed(content, embedding_policy) → EmbeddingResult
```

Embeddings are derived state and MUST be versioned by model and policy.

## Capability Port

```text
invoke(capability_request) → CapabilityResult
```

The Capability Port is the only route by which cognitive execution may access tools or integrations.

## Clock, Identifier and Entropy Ports

Time, identity and controlled randomness are injected where deterministic tests or replay require them.

## Notification Port

Delivers user-facing notifications after policy and preference evaluation.

## Secret and Credential Port

Provides scoped credentials without exposing secret material to domain or reasoning layers.

## Scheduler Port

Schedules jobs and process wake-ups without embedding business semantics in the scheduler.

---

# Command Execution Protocol

Every domain-changing command follows this protocol.

```text
1. Receive Command
2. Establish actor and policy context
3. Validate schema and authorization
4. Begin Unit of Work
5. Load target aggregate at expected version
6. Invoke domain operation
7. Validate aggregate invariants
8. Persist aggregate
9. Persist pending Domain Events to Outbox
10. Persist required audit metadata
11. Commit atomically
12. Return committed result
13. Dispatch Outbox asynchronously
```

## Required properties

### Atomic domain commitment

Aggregate state and its pending Domain Events MUST commit atomically.

### Optimistic concurrency

Aggregate writes SHOULD use version-aware optimistic concurrency.

A stale command is rejected or explicitly reconciled; it is never silently applied over a newer state.

### Idempotency

Commands entering through retryable boundaries MUST carry an idempotency key.

Repeated delivery of the same command MUST return the prior result or a semantically equivalent result without applying the transition twice.

### No external effects inside the domain transaction

The transaction MUST NOT wait on model providers, email systems, calendars, web browsers or other external effects.

External work is represented by a committed intent and executed afterward.

### Stable error taxonomy

Command results distinguish at least:

- invalid request;
- unauthorized operation;
- invariant violation;
- version conflict;
- missing dependency;
- policy rejection;
- transient infrastructure failure;
- permanent infrastructure failure.

---

# Query Architecture

The write model exists to preserve domain integrity. It is not required to satisfy every retrieval shape efficiently.

ResearchOS therefore uses reconstructible read models.

## Query sources

A query may read from:

- an aggregate repository for strongly consistent detail;
- a relational or document projection;
- a graph projection;
- a lexical index;
- a vector index;
- a timeline or activity projection;
- a cached composite view.

## Consistency classes

Every query contract declares one of the following expectations.

| Class | Meaning |
|---|---|
| **Committed** | must reflect the latest committed version of one aggregate |
| **Session-consistent** | must include changes made earlier in the current interaction |
| **Eventually consistent** | may lag while projections converge |
| **Historical** | reads an explicit prior version or time range |

The interface SHOULD communicate material staleness when eventual projections are still catching up.

## Projection ownership

Each projection has one owner responsible for:

- schema and version;
- subscribed event types;
- rebuild procedure;
- checkpoint;
- lag and failure metrics;
- compatibility with query contracts.

No projection is authoritative.

---

# Event Runtime

The Event Runtime carries committed facts from the Domain Kernel to interested reactions.

## Outbox pattern

Pending Domain Events are stored in an Outbox within the same transaction as aggregate state.

A dispatcher later publishes them.

This prevents:

- committed state without its event;
- an event representing state that later rolled back.

## At-least-once delivery

The reference delivery guarantee is **at least once**.

Exactly-once processing is achieved semantically through idempotent consumers, inbox records and domain checks rather than assumed from transport.

## Inbox pattern

Every durable consumer records:

- event identifier;
- consumer identity;
- processing state;
- outcome;
- retry metadata.

A duplicate event does not repeat a completed reaction.

## Ordering

Ordering is guaranteed only where the architecture requires it:

- Domain Events for one aggregate are processed in aggregate-version order.
- No global total order is assumed.
- Cross-aggregate concurrency is resolved through process logic, idempotency and version checks.

## Bounded reactions

Every event reaction must declare:

- subscribed event types;
- produced commands, jobs or projections;
- idempotency strategy;
- retry policy;
- termination condition;
- maximum cascade depth or equivalent loop protection where applicable.

A reaction MUST NOT call another reaction directly.

It may issue a Command, enqueue a Job, update a Projection or signal a Process Instance.

## Event sourcing

ResearchOS does not use event sourcing as the canonical persistence model by default.

Domain Events record committed change and coordinate reactions. Aggregate state remains canonical.

Adopting event sourcing for any module requires a separate ADR and evidence that its additional complexity is justified.

---

# Durable Process Runtime

Not every use case completes in one command.

Examples include:

- document ingestion and enrichment;
- multi-stage cognitive analysis;
- waiting for human approval;
- synchronizing an external calendar;
- monitoring a deadline;
- reconciling an ambiguous external effect;
- producing a complex artifact from several sources.

These operations are implemented as durable processes.

## Process Manager responsibilities

A Process Manager:

- owns explicit process state;
- reacts to events and signals;
- decides the next application command or job;
- manages timers and deadlines;
- persists progress after each step;
- resumes after process failure;
- supports cancellation;
- invokes compensation when defined;
- terminates explicitly.

## Process state machine

Every process type defines:

- valid states;
- valid transitions;
- required inputs;
- waiting conditions;
- timeout behaviour;
- retryable and terminal failures;
- compensation actions;
- completion criteria.

Natural-language plans may inform a process, but they do not replace its executable state machine when durable control is required.

## Orchestration versus choreography

Use choreography when a reaction is local, independent and naturally triggered by one committed fact.

Use a Process Manager when:

- several steps form one business outcome;
- ordering matters;
- progress must be visible;
- failures require coordinated recovery;
- human approval interrupts the flow;
- external effects must be reconciled;
- the chain would otherwise become difficult to understand from events alone.

ResearchOS deliberately uses both patterns.

---

# Cognitive Runtime

The Cognitive Runtime operationalizes `AI_ARCHITECTURE.md`, `MEMORY_MODEL.md` and `CONTEXT_MODEL.md`.

It is a subordinate runtime, not the owner of the system.

## Responsibilities

The Cognitive Runtime may:

- interpret intent;
- request and assemble context;
- retrieve world and operating memory;
- decompose work;
- choose permitted capabilities;
- invoke models;
- inspect tool observations;
- compare, critique, synthesize and explain;
- generate Artifacts;
- create typed Proposals;
- request approval;
- report uncertainty and failure.

It may not:

- mutate aggregates directly;
- write to canonical repositories;
- publish committed Domain Events;
- self-grant permissions;
- bypass policy or approval gates;
- treat model output as validated Knowledge;
- infer that an external action succeeded without observation.

## Cognitive operation

A cognitive operation is a bounded execution scope.

```text
CognitiveOperation
- operation_id
- purpose
- actor
- requested capability
- input references
- policy context
- budget
- deadline
- state
- context_manifest
- model/tool invocation references
- output references
- correlation_id
```

A cognitive operation may contain one model call or a bounded multi-step loop.

## Agent semantics

An Agent is an ephemeral role within a cognitive operation.

It is defined by:

- responsibility;
- allowed capabilities;
- context view;
- output schema;
- budgets;
- stopping conditions;
- escalation policy.

An Agent is not:

- a source of truth;
- a persistent identity with independent authority;
- a repository owner;
- a direct event consumer with arbitrary side effects;
- a justification for creating a microservice.

Multi-agent execution MAY be used when role separation measurably improves quality, independence or verification. It is not the default.

## Reference cognitive pipeline

```text
Request
  ↓
Intent Interpretation
  ↓
Context Construction
  ↓
Execution Strategy
  ↓
Model and Capability Invocation
  ↓
Verification
  ↓
Artifact and/or Typed Proposal
  ↓
Application Policy and Approval
  ↓
Command or Effect Intent
```

The pipeline MAY omit stages when the operation does not need them.

## Execution strategy

The runtime chooses among:

- direct deterministic software;
- one model invocation;
- retrieve–reason–produce;
- fixed pipeline;
- bounded plan execution;
- parallel independent analyses;
- critic or verifier pass;
- human escalation.

The simplest strategy capable of meeting the operation's quality and risk requirements SHOULD be selected.

---

# Context Construction Runtime

Context is transient, bounded and operation-specific.

The Context Builder implements the conceptual pipeline defined in `CONTEXT_MODEL.md`.

## Context Builder stages

The reference stages are:

1. interpret purpose and intent;
2. establish domain anchors;
3. retrieve directly relevant entities and documents;
4. expand relationships where justified;
5. recall relevant episodic or procedural memory;
6. include recent material changes;
7. apply authorization and disclosure policy;
8. rank by relevance, provenance, freshness and risk;
9. bound to the operation's budget;
10. assemble a typed context package.

## Context package

A Context Package may include:

- task and purpose;
- selected entity snapshots;
- document excerpts;
- Knowledge and provenance;
- relationship paths;
- relevant prior Activities and outcomes;
- current policies and constraints;
- unresolved conflicts;
- uncertainty indicators;
- output requirements.

## Context Manifest

The full assembled context is operating state and may be discarded.

For traceability, the runtime persists a **Context Manifest** containing:

- source identifiers and versions;
- excerpt or segment references;
- retrieval methods and scores;
- relationship paths;
- policy filters applied;
- ranking and budget policy versions;
- construction time;
- content digests where required.

The manifest enables audit and approximate reconstruction without turning transient context into a second source of truth.

## Isolation

Context for one operation MUST NOT leak into another unless the relevant information is explicitly recalled from governed memory or canonical state.

Provider-managed conversation state MUST NOT be treated as system memory.

---

# Memory Mechanisms

The software architecture implements the distinction between World Memory and Operating Memory.

## World Memory

World Memory is retrieved from canonical and historical domain state through query and projection ports.

- Semantic memory is grounded in Knowledge and its relationships.
- Episodic memory is grounded in Activities, transitions, decisions, outcomes and provenance.

World Memory is not implemented as a standalone competing truth store.

## Operating Memory

Operating Memory supports execution and may include:

- current cognitive-operation state;
- intermediate observations;
- process checkpoints;
- recent failures;
- execution heuristics;
- reusable procedures;
- temporary summaries;
- model or tool performance observations.

Operating Memory MUST be scoped, versioned and disposable according to policy.

Its loss may reduce efficiency, but MUST NOT destroy canonical world state.

## Consolidation

Consolidation is an explicit application process.

It may propose:

- new Knowledge candidates;
- updates to procedural guidance;
- links between episodes and outcomes;
- deprecation of stale operating hints.

Consolidation does not directly promote generated content to validated Knowledge.

---

# Model Runtime and Provider Abstraction

Models are external compute dependencies with probabilistic behaviour.

They are accessed through the Model Port.

## Reasoning Request

A Reasoning Request declares:

```text
- operation purpose
- context package
- required input/output modalities
- output schema
- capability or role
- privacy classification
- latency and cost budget
- determinism preference
- allowed provider classes
- tool-use permission
- maximum iterations
- stop conditions
```

## Model Result

A Model Result contains:

```text
- structured output or failure
- provider/model identity
- model configuration
- usage and latency
- finish reason
- safety or policy signals
- raw response reference when retention is permitted
- output digest
```

## Provider selection

Provider selection is a policy decision based on declared requirements.

No domain or application handler names a concrete provider.

## Structured outputs

Cognitive operations that feed software behaviour MUST request typed outputs and validate them before use.

Free-form text may be produced for human consumption, but it does not become an executable command without parsing, schema validation and policy evaluation.

## Prompt and instruction versioning

System instructions, templates and tool descriptions that materially influence output MUST be versioned and traceable.

They are implementation artifacts, not domain truth.

## Determinism

Model generation is not assumed deterministic.

Determinism is required for:

- applying validated domain transitions;
- enforcing policies;
- validating schemas;
- calculating idempotency;
- ordering aggregate versions;
- reconstructing committed history.

---

# Verification Architecture

Verification is a chain, not one model prompt.

## Deterministic verification

Where applicable, the runtime performs:

- schema validation;
- type and range checks;
- identifier and version checks;
- permission checks;
- citation existence checks;
- provenance completeness checks;
- duplicate detection;
- constraint and budget checks;
- domain-invariant validation through Commands.

## Evidence verification

Claims that depend on sources are checked for:

- source availability;
- claim-to-source support;
- citation correctness;
- contradiction with selected evidence;
- freshness requirements;
- distinction between evidence and interpretation.

## Model-based verification

A separate model or critic MAY review:

- completeness;
- reasoning gaps;
- alternative interpretations;
- contradiction;
- clarity;
- uncertainty.

Model-based verification MUST NOT be the sole enforcement mechanism for security, authorization, domain invariants or irreversible effects.

## Human verification

Human review is required according to risk and authority policy.

The review surface SHOULD present:

- the proposed change or effect;
- supporting evidence;
- material uncertainty;
- alternatives;
- affected entities or external systems;
- reversibility;
- provenance and trace links.

---

# Capability and Tool Architecture

The System Capabilities provide the stable behavioural vocabulary. Software realizes them through a Capability Registry.

## Capability definition

Every registered capability declares:

```text
CapabilityDefinition
- capability_id
- semantic name
- input schema
- output schema
- effect class
- required permissions
- reversibility
- idempotency characteristics
- timeout
- retry policy
- data classifications accepted
- cost/resource class
- implementation adapter
```

## Effect classes

Capabilities are classified at minimum as:

1. **Pure read** — observes without external or domain mutation.
2. **Derived computation** — creates disposable or proposed output.
3. **Internal reversible effect** — changes application operating state and can be undone.
4. **Canonical mutation request** — creates a Command; never writes directly.
5. **External reversible effect** — changes an external system with compensation available.
6. **External irreversible or sensitive effect** — requires explicit human approval.

## Tool invocation

The Cognitive Runtime may invoke only capabilities present in the registry and allowed by the operation's policy.

Tool adapters MUST treat inputs from documents, websites and models as untrusted.

Tool results are observations, not automatically trusted facts.

## No invented tools

A model may select from declared capabilities. It may not invent a tool name or executable operation and expect the system to comply.

---

# External Integration Architecture

Every external system is behind an adapter and anti-corruption boundary.

Examples include:

- calendar;
- email;
- file repositories;
- bibliographic services;
- academic search systems;
- source-control systems;
- communication platforms;
- institutional systems;
- model providers;
- browser or code-execution environments.

## Adapter responsibilities

An adapter:

- translates external schemas into internal contracts;
- normalizes identifiers and timestamps;
- applies authentication and rate limits;
- classifies failures;
- preserves external references;
- implements idempotency where possible;
- reports observed results;
- does not embed domain policy.

## Inbound synchronization

External observations enter through an ingestion application service.

```text
Observe external change
        ↓
Normalize external payload
        ↓
Deduplicate and authorize
        ↓
Create Command or Proposal
        ↓
Domain validation and commitment
```

External payloads MUST NOT be written directly into canonical tables.

## Outbound effects

```text
Approved Effect Intent
        ↓
Capability Executor
        ↓
External Adapter
        ↓
Effect Result
        ↓
Observation / Reconciliation
        ↓
Command recording outcome when relevant
```

## Anti-corruption boundary

External terminology MUST NOT leak into the core unless it represents a genuine domain concept.

Adapter-specific fields remain adapter metadata or are translated into established Value Objects and Derived Types.

---

# External Effect Protocol

External effects require stronger semantics than ordinary background jobs.

## Phase 1 — Prepare

- validate target and parameters;
- classify risk and reversibility;
- determine approval requirement;
- create an Effect Intent with idempotency key;
- capture relevant preconditions.

## Phase 2 — Authorize

- apply autonomy policy;
- obtain explicit human approval where required;
- bind approval to the effect digest;
- reject expired or materially changed intents.

## Phase 3 — Execute

- acquire a lease;
- invoke the adapter once under the idempotency key;
- capture response and external reference;
- avoid holding a domain transaction open.

## Phase 4 — Observe

- verify the external state when possible;
- distinguish success, failure and ambiguity;
- persist an Effect Result.

## Phase 5 — Reconcile

- record relevant outcome through an application Command;
- retry only when safe;
- compensate when defined;
- escalate ambiguous outcomes to the researcher.

A timeout is not proof of failure. The system MUST check whether the external effect occurred before retrying a potentially non-idempotent action.

---

# Approval and Autonomy Architecture

Approval is enforced in the Application Core, not merely displayed in the interface.

## Autonomy policy

Every behaviour is assigned one of four authority modes.

| Mode | Meaning |
|---|---|
| **Direct human** | operation occurs only from explicit human command |
| **Pre-authorized** | system may perform the bounded behaviour under recorded policy |
| **Approval required** | system may prepare but must wait for review |
| **Prohibited** | operation cannot be performed in the current context |

## Policy inputs

Policy evaluation may consider:

- actor identity and role;
- domain and vertical;
- data sensitivity;
- capability effect class;
- target system;
- reversibility;
- confidence and evidence;
- cost and resource budget;
- current autonomy grant;
- time or location constraints;
- prior approval scope.

## Non-escalation

A cognitive operation, process or adapter MUST NOT widen its own permission scope.

Only an authorized human or administrative policy operation may change autonomy grants.

## Approval persistence

Approvals and rejections are durable historical records with actor, subject digest, constraints and timestamp.

They are not hidden inside chat history.

---

# Persistence Architecture

Persistence is split by semantic responsibility, not by product.

## Canonical write store

Stores:

- aggregates and versions;
- value objects;
- canonical relationships;
- lifecycle state;
- provenance;
- pending Outbox entries;
- required domain history.

It is authoritative.

## Content store

Stores immutable or versioned document content and large artifacts.

Canonical Document state references content through locator and digest rather than embedding implementation-specific storage paths in the domain.

## Operational store

Stores:

- jobs;
- process instances;
- proposals;
- approvals;
- cognitive-operation state;
- model/tool invocation metadata;
- effect intents and results;
- inbox and dispatch state;
- operating memory.

Operational records are durable where recovery or audit requires them, but they are not canonical representations of the researcher's world.

## Projection stores

May include:

- query-optimized records;
- graph projections;
- vector indexes;
- lexical indexes;
- timelines;
- dashboards;
- cached summaries.

They are reconstructible.

## Audit store

Stores append-oriented security and operational audit records with tamper-evident controls appropriate to the deployment.

Audit does not substitute for domain history or provenance.

## Storage independence

One physical database MAY initially implement several logical stores.

Logical separation MUST be preserved through schemas, modules, ownership and access rules even when physical infrastructure is shared.

---

# Projection and Indexing Architecture

Derived stores exist to answer questions efficiently, not to own truth.

## Projection pipeline

```text
Committed Domain Event
        ↓
Outbox Dispatcher
        ↓
Projection Consumer
        ↓
Idempotent transformation
        ↓
Projection checkpoint
```

## Versioning

A projection definition has:

- projection name;
- schema version;
- transformation version;
- source event versions;
- checkpoint;
- rebuild status.

## Rebuildability

The system MUST provide a supported rebuild path for every projection.

A projection that cannot be rebuilt has accidentally become a source of truth and violates the System Architecture.

## Embeddings

Embedding records retain:

- source identifier and version;
- segment reference;
- embedding model identifier;
- preprocessing policy version;
- vector dimension or compatibility class;
- creation timestamp.

Changing the embedding model creates a new projection generation; it does not mutate Knowledge.

## Graph projection

The graph is a navigable representation of canonical relationships and derived semantic links.

A graph adapter MUST distinguish:

- canonical relationships committed by the Domain;
- candidate or inferred relationships;
- projection-only traversal aids.

Inferred edges do not silently become validated canonical relationships.

---

# Consistency and Concurrency

## Aggregate consistency

Within one aggregate transaction:

- invariants are immediate;
- version is monotonic;
- pending events correspond to the committed version;
- partial state is not visible.

## Cross-aggregate consistency

Across aggregates:

- coordination is eventual by default;
- process state makes incomplete work visible;
- reactions are idempotent;
- compensating or corrective operations are explicit;
- no consumer assumes global locking.

## Optimistic concurrency

The reference concurrency mechanism is aggregate version comparison.

A conflict returns a first-class application result.

Resolution may be:

- reload and reapply when semantically safe;
- ask the user to reconcile;
- merge through a domain-specific operation;
- abandon the stale proposal;
- restart the affected process step.

## Relationship consistency

Because relationships are navigable in both directions, the canonical representation MUST define one authoritative write owner for each relationship type.

Reverse navigation may be implemented through projections.

The software MUST NOT attempt to maintain duplicated relationship truth in two aggregates through a distributed dual write.

## Resource capacity

Operations affecting constrained Resources require version-aware reservation or equivalent domain semantics. They MUST NOT rely solely on an eventually consistent projection when over-allocation would violate a Resource invariant.

---

# Idempotency, Retry and Deduplication

Retries are expected in every asynchronous and external boundary.

## Idempotency scopes

ResearchOS uses idempotency at:

- command ingress;
- event consumption;
- job execution;
- model/tool steps where supported;
- external effect dispatch;
- projection updates;
- synchronization imports.

## Retry classification

Failures are classified as:

- transient and retryable;
- rate-limited and delayable;
- conflict requiring fresh state;
- invalid and permanent;
- unauthorized and permanent until policy changes;
- ambiguous and requiring reconciliation;
- non-deterministic quality failure requiring revision or escalation.

Blind retries are prohibited for ambiguous non-idempotent external effects.

## Backoff and limits

Every retrying component declares:

- maximum attempts;
- delay/backoff policy;
- timeout;
- dead-letter or escalation destination;
- idempotency strategy;
- operator-visible failure state.

No component retries indefinitely.

---

# Failure and Recovery Architecture

Failure is modeled explicitly.

## Failure domains

The software distinguishes:

- request validation failure;
- domain rejection;
- concurrency conflict;
- projection lag or failure;
- event-delivery failure;
- process-step failure;
- model-provider failure;
- cognitive-quality failure;
- tool failure;
- external-effect ambiguity;
- storage failure;
- security or policy violation.

## Recovery principles

1. Canonical state is protected before progress is optimized.
2. A failed asynchronous step remains visible and resumable.
3. Retry never bypasses validation.
4. Derived state may be discarded and rebuilt.
5. External ambiguity is reconciled before repeating effects.
6. Human escalation is a valid terminal strategy.
7. Recovery action is auditable and correlated with the original operation.

## Dead-letter handling

A dead-letter record is not a graveyard.

It includes:

- original message or job reference;
- failure classification;
- attempts;
- last error;
- affected process and correlation;
- recommended recovery action;
- operator disposition.

## Reconciliation

Reconciliation processes compare:

- expected canonical state;
- projection checkpoints;
- outbox and inbox status;
- external-system observations;
- process and effect records.

They produce commands or operator tasks; they do not patch data silently.

---

# Security Architecture

Security is enforced across all layers.

## Identity and actor context

Every command, proposal, approval, effect and privileged query carries an actor identity.

Actors may represent:

- the researcher;
- another authorized human;
- a scheduled policy;
- an integration;
- a cognitive operation;
- a maintenance operator.

Non-human actors act only within explicitly delegated scopes.

## Authorization

Authorization is evaluated at the Application boundary and rechecked at sensitive adapters.

Interface visibility alone is not authorization.

## Data classification

Data SHOULD be classified at least by:

- personal sensitivity;
- institutional confidentiality;
- research confidentiality;
- third-party restrictions;
- provider disclosure eligibility;
- retention requirements.

Model and tool requests are filtered by classification policy before leaving the trust boundary.

## Secret isolation

Secrets are held by credential adapters.

Models, prompts, domain objects, logs and trace payloads MUST NOT contain reusable secret material.

## Untrusted content

Documents, web pages, emails, tool outputs and model outputs are untrusted inputs.

Instructions embedded in retrieved content have no authority over the runtime.

The system MUST separate:

- user/system instructions;
- retrieved evidence;
- executable capability requests.

## Least privilege

Each adapter and runtime role receives only the credentials and capabilities required for its current operation.

## Audit

Security-significant events include:

- authentication and authorization decisions;
- policy changes;
- autonomy grants;
- approval decisions;
- sensitive data disclosure;
- secret access;
- external effects;
- privileged maintenance operations;
- repeated or anomalous failures.

---

# Observability and Audit Architecture

ResearchOS requires causal observability, not only logs.

## Correlation model

Every operation carries:

- `correlation_id` — groups the complete user or system objective;
- `causation_id` — identifies the immediate cause;
- `operation_id` — identifies the current execution;
- actor and policy context;
- target entity or process references.

These identifiers propagate through commands, events, jobs, model calls, tools and effects.

## Telemetry classes

### Logs

Structured operational records for diagnosis.

### Metrics

At minimum:

- command success and rejection rates;
- concurrency conflicts;
- outbox and projection lag;
- job queue depth and age;
- process duration and stuck states;
- model/tool latency, failure and cost;
- context size and source count;
- approval wait time;
- external-effect ambiguity;
- recovery and reconciliation outcomes.

### Traces

End-to-end causal paths across runtime boundaries.

### Audit records

Durable records of authority, policy and sensitive effects.

## Redaction

Observability data MUST obey privacy policy.

Raw prompts, document content, model responses and personal data are not logged by default. Traceability uses references, digests and permitted excerpts unless full retention is explicitly justified.

---

# Interface Adapter Architecture

The Workspace is the primary interaction environment, but software behaviour is interface-independent.

## Interface adapters

Potential adapters include:

- graphical Workspace;
- command line;
- HTTP or local API;
- conversational interface;
- file-system watcher;
- email/calendar synchronization;
- notification channel;
- automation trigger.

All adapters call the same Application ports.

## View models

Interfaces consume query-specific view models rather than exposing aggregates directly.

A view model may compose several projections and include:

- data;
- freshness or consistency indicator;
- available commands;
- required approvals;
- relevant trace references;
- pending background work.

## Optimistic interaction

An interface MAY show an operation as pending before all projections converge, but it MUST distinguish:

- accepted command;
- committed state;
- background processing;
- completed effect;
- failure or required attention.

## Conversation

Conversation is an adapter to Queries, Commands and Cognitive Operations.

The conversation transcript is not the application state and does not own workflow progress.

---

# Package and Source Organization

The following structure is normative in responsibility, not in exact directory names.

```text
src/
├── domain/
│   ├── shared/
│   ├── knowledge/
│   ├── documents/
│   ├── projects/
│   ├── people/
│   ├── planning/
│   └── resources/
├── application/
│   ├── commands/
│   ├── queries/
│   ├── proposals/
│   ├── approvals/
│   ├── policies/
│   └── services/
├── cognitive/
│   ├── context/
│   ├── recall/
│   ├── reasoning/
│   ├── verification/
│   └── capabilities/
├── processes/
│   ├── definitions/
│   ├── jobs/
│   └── scheduling/
├── events/
│   ├── contracts/
│   ├── reactions/
│   └── dispatch/
├── integrations/
│   ├── inbound/
│   ├── outbound/
│   └── tools/
├── infrastructure/
│   ├── persistence/
│   ├── projections/
│   ├── models/
│   ├── messaging/
│   ├── content/
│   └── security/
├── interfaces/
│   ├── workspace/
│   ├── api/
│   ├── cli/
│   └── automation/
└── bootstrap/
    ├── interactive_runtime/
    ├── worker_runtime/
    ├── scheduler_runtime/
    └── maintenance_runtime/
```

## Import rules

- `domain` imports only its Shared Kernel and standard language facilities.
- `application` may import domain modules and port contracts.
- `cognitive` may import read contracts, cognitive contracts and application proposal/process ports; it does not import repository adapters.
- `processes` invokes Application ports rather than domain internals.
- `infrastructure` implements inward-facing ports.
- `interfaces` invoke Application ports and consume view models.
- `bootstrap` is the composition root and may know concrete implementations.

Architecture tests MUST enforce these rules.

---

# Composition Root

Concrete adapters are assembled only at the Composition Root.

The Composition Root owns:

- configuration loading;
- dependency construction;
- port-to-adapter binding;
- runtime role selection;
- lifecycle management;
- health checks;
- startup validation.

Domain and application modules MUST NOT use global service locators to retrieve infrastructure dependencies.

---

# Configuration Architecture

Configuration is divided into:

## Static application configuration

Examples:

- enabled modules;
- adapter bindings;
- projection definitions;
- queue classes;
- provider capability metadata.

## Runtime policy

Examples:

- autonomy grants;
- model disclosure policy;
- retry limits;
- cost budgets;
- data retention;
- approval thresholds.

Runtime policy changes are authorized, versioned and auditable.

## Secrets

Secrets are stored and accessed separately from both static configuration and domain state.

Configuration MUST be validated on startup. An invalid configuration fails closed rather than silently disabling governance.

---

# Testing Strategy

Testing proves both behaviour and architecture.

## 1. Domain tests

For every aggregate:

- valid operations;
- invariant rejection;
- lifecycle transitions;
- emitted events;
- version behaviour;
- provenance and caused transitions.

These tests use no infrastructure.

## 2. Application tests

For every handler:

- authorization and policy;
- repository interactions;
- unit-of-work behaviour;
- command idempotency;
- conflict handling;
- proposal and approval flow;
- process initiation.

Use in-memory or contract-test ports.

## 3. Repository contract tests

Every repository adapter passes the same suite for:

- round-trip fidelity;
- optimistic concurrency;
- atomicity;
- version retention;
- failure behaviour.

## 4. Event contract tests

Prove:

- event-after-commit;
- outbox atomicity;
- duplicate delivery safety;
- ordering per aggregate;
- bounded reactions;
- schema compatibility.

## 5. Projection tests

Prove:

- deterministic transformation;
- idempotency;
- rebuild equivalence;
- checkpoint recovery;
- stale-version handling.

## 6. Process tests

Use deterministic clocks and simulated signals to prove:

- state transitions;
- timers;
- retries;
- cancellation;
- compensation;
- recovery after interruption;
- explicit termination.

## 7. Cognitive tests

Separate evaluation dimensions:

- context retrieval quality;
- source coverage and provenance;
- structured-output validity;
- groundedness;
- claim support;
- uncertainty calibration;
- tool selection;
- stopping behaviour;
- model/provider portability;
- resistance to untrusted-content instructions.

Probabilistic evaluation does not replace deterministic contract tests.

## 8. Effect tests

Use fake or sandboxed adapters to prove:

- approval enforcement;
- idempotency;
- ambiguous-result reconciliation;
- compensation;
- secret isolation;
- no domain transaction spans the external call.

## 9. Architecture tests

Automated tests inspect package dependencies and fail when:

- Domain imports infrastructure;
- Interfaces access repositories directly;
- Cognitive code writes canonical stores;
- Process code bypasses Application ports;
- concrete providers leak into core contracts;
- modules form cycles.

## 10. Conformance scenarios

The fifteen conformance criteria from `SYSTEM_ARCHITECTURE.md` are implemented as executable integration scenarios.

---

# Reference End-to-End Flows

## Flow A — Direct human mutation

```text
Workspace
  → Command Port
  → Authorization
  → Handler
  → Aggregate
  → Unit of Work
  → Canonical Store + Outbox
  → Committed Result
  → Projection updates
  → Workspace refresh
```

No AI dependency exists.

## Flow B — Assisted Knowledge proposal

```text
Researcher request
  → Cognitive Operation
  → Context Builder
  → Model / Retrieval / Verification
  → Knowledge Proposal
  → Human review
  → ValidateKnowledge or CreateKnowledgeCandidate Command
  → Domain commitment
  → Domain Event
```

Generated output becomes canonical only through the command path.

## Flow C — Document ingestion

```text
Import adapter
  → RegisterDocument Command
  → Document committed
  → DocumentRegistered Event
  → Ingestion Process starts
  → content extraction and normalization jobs
  → MarkDocumentProcessed Command
  → DocumentProcessed Event
  → cognitive extraction
  → candidate Knowledge Proposals
```

Each stage is independently recoverable.

## Flow D — Event-driven recommendation

```text
Task state changes
  → TaskChanged Event
  → bounded reaction starts cognitive operation
  → Context constructed
  → recommendation Artifact produced
  → notification projection updated
  → researcher accepts, rejects or ignores
```

The reaction does not mutate the Task merely because it generated a recommendation.

## Flow E — External calendar action

```text
Proposed calendar event
  → Effect Intent
  → approval policy
  → human approval when required
  → calendar adapter
  → observed external result
  → RecordExternalActivity Command
  → Domain Event
```

The calendar response does not directly update canonical tables.

---

# Reference Proving Slice

The first implementation slice SHOULD prove the architecture with one complete flow rather than implementing every module superficially.

The reference slice is:

> Import one real document, process it asynchronously, generate traceable Knowledge candidates, allow the researcher to validate one candidate, and answer one grounded question using a constructed Context.

## Required modules

- Document aggregate and repository;
- Knowledge aggregate and repository;
- Register/Process Document handlers;
- Create/Validate Knowledge handlers;
- Unit of Work and Outbox;
- Worker and durable jobs;
- Document content adapter;
- one projection for document/knowledge browsing;
- Context Builder;
- one Model adapter;
- Proposal and approval path;
- causal trace and basic observability.

## Explicitly not required

- microservices;
- multiple model providers;
- multi-agent debate;
- a general-purpose scheduler kernel;
- every vertical;
- every projection type;
- graph and vector stores simultaneously unless Experiment 2 justifies both;
- autonomous external effects.

The slice succeeds only when the complete authority path is preserved. A prototype that bypasses the Domain Kernel does not validate the architecture.

---

# Deployment Evolution

Physical architecture evolves in stages.

## Stage 0 — In-process validation

- one executable may host interactive and background roles;
- adapters may be local;
- events may be dispatched in-process after transactional persistence;
- architecture boundaries remain enforced in modules and tests.

Purpose: falsify domain and workflow assumptions with minimal operational complexity.

## Stage 1 — Separated worker roles

- interactive and worker runtimes execute separately;
- durable job and event delivery are required;
- shared canonical state remains authoritative;
- isolated execution is introduced for risky tools.

Purpose: support long-running work and independent recovery.

## Stage 2 — Selective physical isolation

A logical module may become separately deployed only when justified by one or more of:

- strong security isolation;
- independent scaling profile;
- failure containment;
- specialized runtime requirements;
- independent release cadence;
- regulatory or data-residency boundary.

## Stage 3 — Distributed platform, if ever required

Distribution introduces:

- explicit network contracts;
- schema compatibility policy;
- distributed tracing;
- stronger reconciliation;
- service ownership;
- operational automation.

Distribution is not an architectural maturity badge. It is a cost accepted for a demonstrated need.

---

# Architectural Decisions Established Here

## SWA-001 · Modular Monolith First

ResearchOS begins as a modular monolith with executable runtime roles.

**Rationale:** preserve strong boundaries without premature distributed-systems cost.

## SWA-002 · Ports and Adapters

Core modules depend on contracts; external mechanisms implement those contracts.

**Rationale:** protect the Domain from infrastructure and provider churn.

## SWA-003 · Explicit Commands and Queries

State mutation and retrieval have different contracts.

**Rationale:** make intent, consistency and side effects visible.

## SWA-004 · Single-Aggregate Transaction by Default

One command commits at most one aggregate unless an ADR proves otherwise.

**Rationale:** preserve small consistency boundaries and avoid hidden distributed transactions.

## SWA-005 · Transactional Outbox

Canonical state and pending events commit atomically.

**Rationale:** enforce event-after-commit without dual-write failure.

## SWA-006 · At-Least-Once Delivery with Idempotent Consumers

The system does not depend on magical exactly-once transport.

**Rationale:** make duplicate handling explicit and testable.

## SWA-007 · Durable Process Managers for Long-Running Work

Long operations persist their state outside model conversations and process memory.

**Rationale:** enable interruption, recovery, approval and observability.

## SWA-008 · Cognitive Runtime Is Propositional

The Cognitive Runtime returns Artifacts, Proposals and observations through Application ports.

**Rationale:** preserve domain and human authority.

## SWA-009 · Capability Registry Mediates Tools

Models invoke only typed, policy-scoped capabilities.

**Rationale:** prevent arbitrary tool authority and provider-specific coupling.

## SWA-010 · External Effects Use Prepare–Authorize–Execute–Observe–Reconcile

External action is not conflated with intention or transport response.

**Rationale:** handle approval, idempotency and ambiguity safely.

## SWA-011 · Projections Are Rebuildable

Search, graph, vector and view stores remain derived.

**Rationale:** prevent competing truth and enable technology replacement.

## SWA-012 · Event Sourcing Is Not the Default

Events coordinate and record change; aggregate state remains canonical.

**Rationale:** avoid complexity not required by the current domain.

## SWA-013 · Context Body Is Transient; Context Manifest Is Traceable

The system retains source/version evidence without promoting assembled context to truth.

**Rationale:** reconcile Context Model semantics with auditability.

## SWA-014 · Agents Are Ephemeral Runtime Roles

Agents do not own persistent state, permissions or services.

**Rationale:** avoid agent-centric authority and uncontrolled architecture growth.

## SWA-015 · Architecture Is Enforced by Automated Tests

Dependency and authority rules are executable constraints.

**Rationale:** documentation alone cannot prevent architectural erosion.

---

# Deferred Technical Decisions

The following remain for `TECHNICAL_ARCHITECTURE.md` or dedicated ADRs.

- programming language and runtime;
- application framework;
- canonical database product;
- binary/content storage product;
- event and job transport;
- workflow engine or custom process runtime;
- graph representation and product;
- lexical and vector search products;
- model and embedding providers;
- local versus remote model execution;
- user-interface framework;
- authentication mechanism;
- secret-management mechanism;
- sandbox technology;
- packaging and deployment platform;
- backup and disaster-recovery technology;
- telemetry stack.

A technical choice must implement the contracts in this document. It does not redefine them.

---

# Relationship to Existing Documents

| Document | Software Architecture responsibility derived from it |
|---|---|
| `VISION.md` | one coherent operational environment and reduced cognitive load |
| `SYSTEM_PRINCIPLES.md` | provider independence, shared state, human control and traceability |
| `SYSTEM_MODEL.md` | Assets, Capabilities and Interfaces become domain, application and adapter structures |
| `SYSTEM_RESPONSIBILITIES.md` | allocation of responsibilities to modules and runtimes |
| `DOMAIN_MAP.md` | domain-module boundaries |
| `DOMAIN_MODEL.md` | aggregate concepts implemented by the Domain Kernel |
| `LOGICAL_DOMAIN_MODEL.md` | attributes, invariants, relationships, lifecycles and aggregate roots |
| `KNOWLEDGE_MODEL.md` | Knowledge proposal, validation, provenance and lifecycle enforcement |
| `SYSTEM_CAPABILITIES.md` | Capability Registry vocabulary |
| `USE_CASES.md` | command, query, process and cognitive-operation handlers |
| `AI_ARCHITECTURE.md` | Cognitive Runtime, agents, autonomy and Artifact production |
| `MEMORY_MODEL.md` | World/Operating Memory mechanisms |
| `CONTEXT_MODEL.md` | Context Builder and Context Manifest |
| `EVENT_MODEL.md` | Outbox, Inbox, reactions and bounded cascades |
| `INTERACTION_MODEL.md` | Interface adapters, Workspace and approval surfaces |
| `DOMAIN_VERTICALS.md` | extensions of the same modules rather than separate stacks |
| `SYSTEM_ARCHITECTURE.md` | authority, planes, trust boundaries and conformance criteria |
| `IMPLEMENTATION_PLAN.md` | experiments implemented through the proving slice |

---

# Conformance Criteria

A software implementation conforms to this architecture only if it can demonstrate that:

1. Domain modules have no dependency on infrastructure or model providers.
2. Every canonical mutation enters through an explicit Command handler.
3. Aggregate state and pending Domain Events commit atomically.
4. Event consumers are idempotent and duplicate-safe.
5. Projections can be rebuilt from authoritative sources.
6. Cognitive code cannot access canonical repository write adapters.
7. Model outputs are validated and converted into Proposals or Artifacts before mutation.
8. Approval rules are enforced in the Application Core.
9. External effects are represented separately from their observed outcomes.
10. Ambiguous effects are reconciled before unsafe retry.
11. Long-running operations survive process restart.
12. Provider-managed chat state is not treated as system state.
13. Context construction is policy-scoped and traceable through a manifest.
14. Interface adapters do not expose or mutate aggregates directly.
15. Cross-aggregate workflows use events or durable Process Managers by default.
16. Every retrying boundary defines idempotency and termination.
17. Search, graph, vector and cache data are marked and treated as derived.
18. actor, correlation and causation identity propagate end to end.
19. secrets do not enter domain objects, prompts or ordinary logs.
20. automated architecture tests enforce dependency rules.
21. the system can execute core direct operations without an AI provider.
22. the proving slice preserves the complete proposal–validation–commit authority path.

---

# Evolution Rules

The software architecture evolves by evidence and explicit decisions.

A new module is created only when it owns a cohesive responsibility that cannot remain in an existing module without reducing clarity or violating dependency direction.

A new runtime role is created only when it requires distinct lifecycle, isolation, scaling or recovery semantics.

A new physical service is created only after the corresponding logical boundary is stable and distribution solves a measured problem.

A new store is introduced only when its access pattern cannot be served responsibly by existing mechanisms. Its authority class and rebuild strategy must be declared.

A new agent role is introduced only when it improves a measured cognitive outcome and has explicit capabilities, limits and stopping conditions.

A new capability is introduced through the stable Capability vocabulary before an adapter-specific tool is exposed.

Every significant change records:

1. the responsibility being introduced or moved;
2. the owner before and after the change;
3. the dependency impact;
4. the authority and trust-boundary impact;
5. the failure and recovery impact;
6. the evidence justifying the change;
7. the conformance tests added or changed.

---

# Final Software Statement

ResearchOS is implemented as a domain-centred modular system in which explicit application contracts mediate every interaction with canonical state.

The Domain Kernel owns truth and invariants.

The Application Core owns intent, orchestration, policy and approval.

The Cognitive Runtime owns bounded interpretation and proposal generation.

The Process and Event Runtimes own durable progression across time.

Adapters own contact with technologies and external systems.

Projections own efficient views but never truth.

No model, agent, interface, worker, tool or integration bypasses these boundaries.

This organization is the software mechanism by which ResearchOS remains coherent while becoming executable.
