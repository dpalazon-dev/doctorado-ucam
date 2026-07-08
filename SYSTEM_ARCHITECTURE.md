# System Architecture

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 1 — Canonical Foundation |
| **Normative status** | Canonical |
| **Authoritative for** | System-wide authority, state ownership, logical planes, trust boundaries, dependency rules, operating cycles, governance and runtime invariants. |
| **Not authoritative for** | Entity attributes, specialized model semantics, concrete software components, products, protocols or deployment topology. |
| **Required reading** | `VISION.md`, `SYSTEM_PRINCIPLES.md`, `SYSTEM_MODEL.md`, `LOGICAL_DOMAIN_MODEL.md`, `AI_ARCHITECTURE.md`, `MEMORY_MODEL.md`, `CONTEXT_MODEL.md`, `EVENT_MODEL.md`, `INTERACTION_MODEL.md`. |
| **Downstream documents** | `SOFTWARE_ARCHITECTURE.md`, `IMPLEMENTATION_CONTEXTS.md`, `IMPLEMENTATION_PLAN.md`, `CLAUDE.md` and every implementation artifact. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

> **Status: Canonical · v1.0.** This document defines the technology-independent system architecture of ResearchOS.
>
> It changes only when a system-wide responsibility, authority boundary, dependency rule or runtime invariant changes. It does not change because a framework, database, model provider or deployment topology changes.

## Purpose

This document defines how the conceptual models of ResearchOS cooperate as one coherent system.

The existing architecture already defines:

- why the system exists;
- what exists in its domain;
- what the system can do;
- how knowledge, memory, context and events behave;
- how the AI Operating Layer participates;
- how the researcher and the system collaborate.

This document connects those definitions into a single architectural whole.

It specifies:

- the authoritative state of the system;
- the system's major logical planes and subsystems;
- the boundaries between reasoning, decision, state mutation and execution;
- the direction of dependencies;
- the runtime operating cycles;
- the consistency, governance and recovery rules;
- the quality attributes every implementation must preserve.

It does **not** choose technologies, frameworks, databases, protocols, process boundaries or deployment infrastructure. Software structure is defined in `SOFTWARE_ARCHITECTURE.md`; concrete products, protocols and deployment choices belong to `TECHNICAL_ARCHITECTURE.md` and its accepted ADRs.

---

# Position in the Documentation

ResearchOS is specified through complementary architectural documents. Each owns a distinct question.

```text
Vision                         why the system exists
    ↓
System Principles              permanent design commitments
    ↓
System Model                   what the system is made of
    ↓
Operational Model              the reality in which it operates
    ↓
System Responsibilities        what the system must continuously achieve
    ↓
Domain Map                     areas of responsibility
    ↓
Domain Model                   what concepts exist
    ↓
Knowledge Model                what Knowledge means and how it evolves
    ↓
System Capabilities            the stable vocabulary of behaviour
    ↓
Use Cases                      observable operational behaviour
    ↓
Cognitive Architecture         how intelligent behaviour works
    ├── AI Architecture
    ├── Memory Model
    ├── Context Model
    └── Event Model
    ↓
Logical Domain Model           exact logical structure and invariants
    ↓
Interaction Model              how the researcher and system collaborate
    ↓
System Architecture            how all of the above form one system
    ↓
Software Architecture          modules, ports, adapters and runtime mechanisms
    ↓
Technical Architecture         concrete technologies and deployment
```

This document does not replace any preceding model.

It defines the relationships, authority boundaries and operating rules between them.

---

# Normative Language

The terms **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT** and **MAY** are normative.

- **MUST / MUST NOT** define architectural invariants.
- **SHOULD / SHOULD NOT** define strong defaults that require an explicit architectural decision to override.
- **MAY** defines permitted variation.

---

# Architectural Thesis

ResearchOS is a technology-independent, knowledge-centred operating system for the researcher's complete operational reality.

Its architecture rests on five propositions.

1. **The Domain is the authoritative model of the world.**
2. **The researcher remains the final authority over judgement and irreversible action.**
3. **Intelligence proposes, interprets and prepares; it does not own truth or bypass governance.**
4. **Every persistent change occurs through an explicit, validated domain transition.**
5. **The system coordinates through shared state and committed events, not through hidden agent-to-agent control flow.**

The architecture is therefore not centred on an LLM, an agent, a database, a graph or a user interface.

It is centred on governed evolution of a canonical domain state.

```text
Knowledge and operational state
            ↓
       interpreted by
            ↓
        intelligence
            ↓
    proposed as change
            ↓
 validated by system rules
            ↓
 committed to the Domain
            ↓
   observed through Events
            ↓
       acted upon again
```

The system becomes intelligent not because a model controls it, but because intelligence can operate continuously over durable, coherent and governed state.

---

# Architecture Drivers

The architecture is optimized for the following concerns, in priority order.

## 1. Coherence

Every capability and interaction operates over one shared model of the researcher's world.

No subsystem may create an independent competing truth.

## 2. Continuity

The system preserves knowledge, activity, decisions, relationships and context across sessions, projects, domains and long time horizons.

## 3. Human Authority

The system augments judgement without replacing epistemic, scientific or personal responsibility.

## 4. Traceability

The system must be able to reconstruct what changed, why it changed, what evidence supported it, who or what initiated it, and which effects followed.

## 5. Evolvability

Models, providers, storage mechanisms and interfaces must remain replaceable without changing the conceptual architecture.

## 6. Recoverability

Failures must not silently corrupt canonical state. Derived state must be rebuildable and incomplete effects must be detectable and reconcilable.

## 7. Low Cognitive Load

The architecture must make complexity an internal concern. The researcher interacts with one Workspace, not with the system's internal machinery.

## 8. Privacy and Control

The system handles a long-lived model of a person's work and life. Access, disclosure and external execution must therefore be explicit, constrained and auditable.

---

# The Fundamental Authority Model

## The Canonical State

The authoritative state of ResearchOS is the **Domain State** defined by the Domain Model and specified by the Logical Domain Model.

It consists of the seven Core Entities, their Derived Types, their Value Objects, their relationships, their lifecycle states and their provenance.

```text
Project
Document
Knowledge
Task
Activity
Person
Resource
```

The Domain State is the system's single source of truth.

ResearchOS does **not** introduce a separate canonical cognitive state alongside the domain.

A second authoritative state for what the system “believes”, “intends” or “remembers” would duplicate the Domain, create reconciliation problems and violate the single-source-of-truth principle.

Instead:

- semantic memory is recall over Knowledge;
- episodic memory is recall over Activities, decisions and outcomes;
- active goals are represented through Projects, Tasks and their states;
- context is derived for an operation;
- working and procedural memory support cognition but are not authoritative;
- AI outputs remain proposals or Artifacts until accepted through governed transitions.

The world is held by the Domain.

The intelligence operates over it.

## State Classes

ResearchOS distinguishes five classes of state.

| State class | Examples | Authority | Lifetime | Reconstructible |
|---|---|---:|---|---:|
| **Canonical domain state** | Entities, relationships, lifecycles, provenance | Authoritative | Durable | No — this is the source |
| **Historical state** | Activities, versions, transition causes, accepted decisions | Authoritative record | Durable | Partially, from canonical history |
| **Operating state** | Working memory, execution progress, leases, retries | Non-authoritative | Temporary or bounded | Yes |
| **Derived state** | Context, recall results, search indexes, graph projections, read models | Non-authoritative | Disposable | Yes |
| **External state** | Email systems, calendars, repositories, external tools | Authoritative only in its own system | External | Re-observed or synchronized |

No derived or operating state may silently become authoritative.

External state becomes part of ResearchOS only after it is captured, normalized and committed through the Domain boundary.

---

# The ResearchOS Kernel

ResearchOS defines a **Domain Kernel**.

The Domain Kernel is an architectural boundary, not necessarily a process, service, library or deployment unit.

It is the minimal part of the system that owns authority over persistent state evolution.

The Domain Kernel is responsible for:

- the canonical Domain State;
- aggregate boundaries;
- entity and value-object invariants;
- lifecycle transition rules;
- validation of domain changes;
- causal provenance of committed transitions;
- versioning and conflict detection;
- production of domain events after successful commitment.

The Domain Kernel is not responsible for:

- user-interface rendering;
- context construction;
- model inference;
- retrieval implementation;
- external tool execution;
- storage technology;
- notification transport;
- agent orchestration.

The kernel's defining rule is:

> No interface, agent, integration, automation, projection or storage adapter may mutate canonical state directly.

Every persistent mutation MUST pass through the Domain Kernel.

---

# High-Level Architecture

ResearchOS is organized into six logical planes and two cross-cutting concerns.

```text
┌──────────────────────────────────────────────────────────────────────┐
│                         RESEARCHER / ACTORS                          │
└───────────────────────────────┬──────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────────┐
│  EXPERIENCE PLANE                                                   │
│  Workspace · Navigation · Forms · Search · Conversation · Alerts    │
└───────────────────────────────┬──────────────────────────────────────┘
                                │ intents · queries · approvals
                                ▼
┌──────────────────────────────────────────────────────────────────────┐
│  CONTROL PLANE                                                      │
│  Application Coordination · Policy · Approval · Operation Lifecycle │
└───────────────────────────────┬──────────────────────────────────────┘
                                │ governed commands / queries
                                ▼
┌──────────────────────────────────────────────────────────────────────┐
│  DOMAIN PLANE — THE DOMAIN KERNEL                                   │
│  Aggregates · Invariants · State Transitions · Provenance            │
└───────────────┬───────────────────────────────────────▲──────────────┘
                │ committed facts                       │ validated proposals
                ▼                                       │
┌──────────────────────────────────────────────────────────────────────┐
│  COGNITIVE PLANE                                                    │
│  Events · Memory · Context · AI Operating Layer                      │
│  Recall · Interpret · Reason · Plan · Curate · Produce               │
└───────────────────────────────┬──────────────────────────────────────┘
                                │ authorized action requests
                                ▼
┌──────────────────────────────────────────────────────────────────────┐
│  EXECUTION & INTEGRATION PLANE                                      │
│  Capability Execution · Tools · Automations · External Systems      │
└───────────────────────────────┬──────────────────────────────────────┘
                                │ observations / outcomes
                                └───────────────────────► Control Plane

┌──────────────────────────────────────────────────────────────────────┐
│  PERSISTENCE & PROJECTION PLANE                                     │
│  Durable storage · indexes · read models · caches · derived views    │
└──────────────────────────────────────────────────────────────────────┘

Cross-cutting: GOVERNANCE & SECURITY · OBSERVABILITY & AUDIT
```

These are logical planes.

They MUST NOT be interpreted as a requirement for microservices, distributed deployment or separate processes.

A future implementation may realize several planes in one deployable unit while preserving their boundaries.

---

# Architectural Planes

## 1. Experience Plane

The Experience Plane is the visible surface of ResearchOS.

Its canonical abstraction is the **Workspace**, as defined by the Interaction Model.

### Responsibilities

- present coherent views of domain state;
- receive user actions, requests, approvals and corrections;
- expose the appropriate interaction channel for each task;
- preserve continuity while the researcher moves across domains;
- surface recommendations, pending approvals, failures and relevant change;
- hide internal mechanisms such as agents, indexes, event transport and model providers.

### Authority

The Experience Plane may collect intent and display results.

It has no authority to mutate persistence directly, execute external effects directly or redefine domain rules.

### Constraints

- Direct software operations SHOULD remain direct and deterministic.
- Reasoning MUST NOT be invoked when ordinary software behaviour is sufficient.
- Conversation MUST remain one interaction channel, never the system's privileged control surface.
- Every visible view MUST be a projection of canonical or explicitly labelled derived state.

---

## 2. Control Plane

The Control Plane governs the lifecycle of operations entering and leaving the system.

It translates interaction intent, external observations and cognitive proposals into governed application operations.

### Responsibilities

- identify the requested use case or system responsibility;
- distinguish commands, queries, proposals, approvals and observations;
- authenticate the actor and authorize the operation;
- apply policy and autonomy rules;
- coordinate application-level workflows;
- establish operation identity, causality and correlation;
- route valid commands to the Domain Kernel;
- request external effects from the Execution Plane;
- track long-running operation state;
- expose completion, failure or required human intervention.

### Authority

The Control Plane owns orchestration, not truth.

It may coordinate multiple domain transitions, but it MUST NOT duplicate domain invariants or maintain a shadow representation of aggregates.

### The Proposal–Commit Boundary

The Control Plane is the mandatory boundary between a proposal and a committed change.

Inputs from an LLM, an agent, an external system or the user are not state changes merely because they were produced.

They become state changes only after:

1. normalization into a typed operation;
2. authorization;
3. policy evaluation;
4. domain validation;
5. successful commitment.

---

## 3. Domain Plane

The Domain Plane contains the Domain Kernel and is the authoritative centre of ResearchOS.

### Responsibilities

- represent the researcher's operational world;
- enforce entity and aggregate invariants;
- govern lifecycle transitions;
- preserve provenance and historical continuity;
- reject invalid, stale or conflicting changes;
- commit valid state transitions atomically within aggregate boundaries;
- emit factual events describing committed change.

### Authority

Only the Domain Plane may determine whether a proposed persistent state transition is valid.

A model may infer.

A policy may constrain.

An application operation may coordinate.

Only the Domain Kernel commits.

### Independence

The Domain Plane MUST remain independent of:

- model providers;
- prompt formats;
- agents;
- user-interface frameworks;
- database products;
- event brokers;
- external APIs;
- deployment topology.

This is the most important dependency rule in the architecture.

---

## 4. Cognitive Plane

The Cognitive Plane contains the system's intelligent operating behaviour.

It is composed conceptually from:

- the Event Model;
- the Memory Model;
- the Context Model;
- the AI Operating Layer.

### Responsibilities

- detect or receive committed domain change;
- recall relevant semantic, episodic and procedural memory;
- construct a bounded context for each intelligent operation;
- interpret information and intent;
- produce comparisons, critiques, syntheses and recommendations;
- maintain and curate knowledge continuously;
- prepare plans and proposed actions;
- produce Artifacts with provenance;
- express all intended changes as proposals.

### Authority

The Cognitive Plane has no direct mutation authority.

Its outputs are:

- interpretations;
- recall results;
- constructed contexts;
- recommendations;
- Artifacts;
- typed proposals for domain change;
- typed requests for execution.

These outputs remain non-authoritative until accepted by the appropriate boundary.

### Model Independence

Models are replaceable reasoning resources.

No model, prompt, agent or provider may own durable domain truth.

The Cognitive Plane MUST express needs in domain and capability language, not in storage or provider language.

---

## 5. Execution & Integration Plane

The Execution & Integration Plane performs authorized effects and communicates with systems outside ResearchOS.

### Responsibilities

- execute deterministic capabilities;
- invoke tools and automations;
- communicate with external systems;
- translate between external schemas and ResearchOS concepts;
- enforce capability-scoped permissions;
- return structured observations and outcomes;
- detect ambiguous or partial external effects;
- support retries, compensation or reconciliation where appropriate.

### Authority

The Execution Plane may perform only actions that have been authorized by the Control Plane.

It MUST NOT decide what should be done.

It MUST NOT treat a successful external response as an implicit domain mutation.

External outcomes return as observations and are incorporated through a new governed transition.

### Integration Boundary

Every external integration is an anti-corruption boundary.

External concepts MUST be translated into ResearchOS domain concepts rather than allowed to reshape the Domain Model around a vendor's API.

---

## 6. Persistence & Projection Plane

The Persistence & Projection Plane realizes storage and access without becoming the model of the system.

### Responsibilities

- durably persist canonical aggregates and provenance;
- provide repositories for aggregate access;
- maintain projections for search, graph traversal, timelines and dashboards;
- maintain indexes and caches;
- support version recovery and reconstruction;
- rebuild derived views from canonical state and committed history.

### Authority

Persistence mechanisms store the Domain State; they do not define it.

Projections are disposable.

Search indexes, vector representations, graph projections, caches, generated summaries and read models MUST always remain reconstructible from authoritative state or explicitly preserved source material.

A projection may be useful, optimized and long-lived.

It may never become the source of truth.

---

# Cross-Cutting Concerns

## Governance and Security

Governance applies across every plane.

It includes:

- identity and actor attribution;
- access control;
- data sensitivity;
- autonomy policy;
- approval requirements;
- capability permissions;
- external disclosure rules;
- retention and deletion rules;
- secret isolation;
- model and tool trust boundaries.

Policy is evaluated before context exposure, state commitment and external execution.

A component MUST receive only the information and capabilities required for its responsibility.

## Observability and Audit

Every meaningful operation MUST be reconstructible without depending on hidden model reasoning.

The system SHOULD record, as appropriate:

- operation identity;
- initiating actor or event;
- use case or responsibility;
- prior relevant state version;
- command or proposal type;
- policy and approval decisions;
- evidence and provenance references;
- committed transition;
- emitted events;
- requested external effects;
- observed outcomes;
- failure and recovery actions;
- model, tool and procedure metadata when intelligence participated.

ResearchOS MUST NOT depend on private chain-of-thought traces for explainability.

Explainability is grounded in explicit inputs, evidence, domain state, policies, proposals, approvals and committed outcomes.

---

# Subsystem Contracts

A subsystem is a logical owner of a responsibility. It is not automatically a service.

| Subsystem | Owns | Consumes | Produces | MUST NOT own |
|---|---|---|---|---|
| **Workspace** | interaction state and presentation | queries, projections, notifications | intents, approvals, corrections | domain truth |
| **Application Coordination** | use-case and operation lifecycle | intents, proposals, observations | governed commands, queries, effect requests | domain invariants |
| **Domain Kernel** | canonical state and transition authority | validated commands | committed state, domain events | UI, model or storage concerns |
| **Event Subsystem** | event semantics, subscriptions and bounded reactions | committed domain events | reaction activations, event history views | commands or domain truth |
| **Memory Subsystem** | recall and consolidation faculties | domain state, Activities, procedures | recall results, consolidation proposals | a parallel world model |
| **Context Subsystem** | task-specific context construction | intent, anchors, recall, events, policy | bounded transient context | durable truth |
| **AI Operating Layer** | intelligent responsibilities and proposals | constructed context, capabilities | interpretations, Artifacts, proposals | mutation or execution authority |
| **Capability Execution** | authorized capability performance | effect requests, permissions | outcomes and observations | decisions about goals or truth |
| **Integration Subsystem** | translation across external boundaries | external events and authorized requests | normalized observations and effects | vendor-shaped domain concepts |
| **Persistence** | durable realization of aggregates and history | canonical state transitions | durable records | domain semantics |
| **Projection Subsystem** | optimized derived views | state and events | search, graph, timeline and dashboard views | authoritative state |
| **Policy & Approval** | governance decisions | actor, action, data, risk, autonomy | permit, deny, require approval | domain facts |
| **Audit & Observability** | causal and operational evidence | operations, transitions, effects | traces, metrics, audit records | business authority |

---

# The State Transition Protocol

Every persistent domain change follows one conceptual protocol.

```text
Intent / Observation / Proposal
            ↓
Normalize into a typed operation
            ↓
Authenticate actor and establish causality
            ↓
Authorize and apply policy
            ↓
Load expected canonical state
            ↓
Validate domain rules and preconditions
            ↓
Commit state transition
            ↓
Record provenance and transition cause
            ↓
Publish committed domain event
            ↓
Update projections and trigger bounded reactions
```

## Required Properties

### Explicitness

No persistent state mutation may be implicit.

A change MUST have an explicit operation, target, cause and outcome.

### Validation Before Commitment

Validation is a prerequisite, not a later correction.

Invalid or unauthorized proposals MUST be rejected without contaminating canonical state.

### Deterministic Application

Given the same prior aggregate state and the same accepted typed operation, application of the domain transition SHOULD produce the same resulting aggregate state.

Generative proposal production may remain probabilistic.

Commit semantics must not.

### Atomicity Within an Aggregate

A transition within one aggregate MUST either commit completely or not commit.

### Version Awareness

A state-changing operation SHOULD declare the state version or preconditions on which it was based.

Stale proposals MUST be rejected, re-evaluated or explicitly reconciled.

### Causality

Every committed transition MUST retain its initiating cause.

### Event-After-Commit

A domain event describes a fact that happened.

It MUST NOT be visible as committed before the corresponding state transition succeeds.

The commitment of state and the durable intent to publish its event MUST be treated as one logical unit, regardless of the future implementation mechanism.

---

# Typed Proposals

Intelligent components do not return unstructured authority.

When an AI operation intends to change the system or cause an effect, it SHOULD produce a typed proposal containing, where relevant:

- proposal type;
- target entity or capability;
- intended change or action;
- expected prior state or preconditions;
- evidence and provenance references;
- uncertainty or confidence;
- reversibility classification;
- requested autonomy level;
- predicted external effects;
- expiration or freshness boundary.

A proposal is a derived object.

It is never itself proof that the proposed change is correct.

Free-form model output may inform a proposal, but only the typed proposal crosses the governance boundary.

---

# Validation Gates

A proposal may pass through several independent gates.

## Structural Validation

Is the proposal well-formed and expressed in the system's vocabulary?

## Authorization Validation

Is the initiating actor or behaviour permitted to request this operation?

## Policy Validation

May the relevant information be accessed, disclosed or acted upon in this context?

## Domain Validation

Would the transition preserve aggregate invariants, lifecycle rules and cardinalities?

## Concurrency Validation

Is the proposal still valid against the current state version?

## Epistemic Validation

Does a proposed Knowledge change have sufficient provenance and the required human acceptance?

## Effect Validation

Is the requested action reversible, external, destructive or otherwise approval-sensitive?

Passing one gate does not imply passing the others.

---

# Runtime Operating Cycles

## 1. Direct Human Operation

Used when no reasoning is required.

```text
Researcher action
      ↓
Workspace
      ↓
Application operation
      ↓
Domain validation and commit
      ↓
Domain event
      ↓
Updated Workspace projection
```

The AI Operating Layer does not participate.

This is the preferred path for deterministic operations.

---

## 2. Assisted Human Operation

Used when intelligence adds value but the researcher remains in direct control.

```text
Researcher request
      ↓
Application Coordination
      ↓
Context construction
      ↓
AI reasoning
      ↓
Recommendation / Artifact / Proposal
      ↓
Researcher accepts, modifies or rejects
      ↓
Governed domain transition
```

Rejection produces no persistent domain change unless the rejection itself is intentionally recorded as an Activity or preference.

---

## 3. Event-Driven Cognitive Operation

Used for continuous maintenance and proactive assistance.

```text
Committed domain change
      ↓
Domain event
      ↓
Matching bounded reaction
      ↓
Context construction
      ↓
AI responsibility executes
      ↓
No-op, Artifact, notification or typed proposal
      ↓
Policy / approval / domain validation
      ↓
Optional new domain transition
      ↓
New event
```

The cascade terminates when no meaningful new change is produced or when a configured bound is reached.

Workers coordinate through state and events, not through direct worker-to-worker calls.

---

## 4. External Observation

Used when the external world changes.

```text
External signal
      ↓
Integration boundary
      ↓
Normalize and verify
      ↓
Observation
      ↓
Application operation
      ↓
Governed domain transition
      ↓
Domain event
```

An incoming email, calendar change, repository update or external publication is not automatically domain truth.

It becomes part of ResearchOS through capture.

---

## 5. External Effect

Used when ResearchOS acts outside itself.

```text
Validated intent
      ↓
Policy and approval gate
      ↓
Authorized effect request
      ↓
Execution / integration adapter
      ↓
External effect
      ↓
Structured outcome or ambiguity
      ↓
Observation
      ↓
Governed domain transition
```

The system MUST distinguish between:

- requested;
- dispatched;
- confirmed;
- failed;
- outcome unknown;
- reconciled.

A timeout is not proof that an effect did not occur.

---

## 6. Long-Running Operation

Long-running work is represented as explicit process state, not hidden inside an agent loop or chat session.

A long-running operation MUST support:

- identity;
- objective;
- current state;
- initiating cause;
- dependencies;
- progress evidence;
- pause and resume;
- cancellation or abandonment;
- approval checkpoints;
- failure and recovery state;
- final outcome.

The Software Architecture will determine whether this is realized through Tasks, Activities, a process manager or a dedicated application construct. The state may not exist only inside a model context window.

---

# Events and Coordination

## Events Are Facts

Events are past-tense records of committed domain change.

They are not commands, requests, plans or agent messages.

## Events Do Not Address Consumers

An event MUST NOT know which subsystem will react to it.

Consumers subscribe independently.

## Reactions Are Idempotent

A reaction SHOULD be safe to evaluate more than once.

Duplicate delivery must not create duplicate canonical effects.

## Reactions Are Bounded

Every cascade MUST have a termination condition.

A component may not generate change solely to keep a cascade alive.

## Event Ordering Is Scoped

The system MUST preserve causal ordering where correctness requires it, particularly for transitions of the same aggregate.

It need not impose a meaningless global order over unrelated changes.

## Events Are Not the Source of Truth by Default

The canonical current state remains the Domain State.

Committed events provide causality, history and coordination. `SOFTWARE_ARCHITECTURE.md` may permit event-sourced persistence as a realization, but this document does not require it.

---

# Memory and Context in the System Architecture

## World Memory

World memory is authoritative because it is the Domain recalled over time.

- Semantic memory is Knowledge.
- Episodic memory is Activities, decisions and outcomes.

The Memory Subsystem does not copy these into a second truth.

## Operating Memory

Working and procedural memory support the Cognitive Plane.

They are replaceable and non-authoritative.

Loss of operating memory may reduce convenience or performance, but MUST NOT destroy the researcher's world state.

## Context

Context is constructed per operation.

It is:

- intent-driven;
- anchored in domain entities;
- expanded through relationships;
- enriched through recall and recent events;
- constrained by policy;
- prioritized and bounded;
- transient and non-authoritative.

A context cache, if introduced, remains a projection.

## Context Isolation

Different operations MAY receive different context slices from the same canonical state because their intent, permissions and relevance differ.

Different context does not imply different truth.

---

# Reasoning and Execution

ResearchOS enforces a strict separation between reasoning and execution.

## Reasoning

Reasoning may:

- interpret;
- compare;
- critique;
- synthesize;
- recommend;
- plan;
- produce Artifacts;
- propose changes or actions.

Reasoning MUST NOT directly:

- commit domain state;
- invoke unrestricted tools;
- send external communication;
- delete durable information;
- escalate its own autonomy;
- convert generated output into validated Knowledge.

## Execution

Execution may:

- perform an authorized capability;
- invoke a bounded tool;
- call an external system;
- return an observation;
- report success, failure or ambiguity.

Execution MUST NOT decide:

- whether a scientific interpretation is valid;
- whether a proposal should become Knowledge;
- whether an irreversible action should be approved;
- whether policy may be bypassed.

Reasoning determines what might be done.

Governance determines what may be done.

Execution determines what actually happened.

---

# Human Authority and Autonomy

The researcher's authority is structural, not ceremonial.

## Permanent Human Decisions

The following remain human decisions:

- scientific judgement;
- acceptance of generated Knowledge;
- approval of irreversible actions;
- approval of outward-facing communication;
- delegation of autonomy to behaviours;
- correction of the system's interpretation of personal intent.

## Autonomy Is Granted Per Behaviour

Autonomy MUST be scoped to a specific behaviour or capability.

It MUST NOT be granted globally to “the AI”.

## Autonomy Cannot Self-Escalate

No intelligent component may increase its own permissions, access scope or autonomy level.

## Reversibility Governs Automation

Reversible, low-risk actions may be automated within policy.

Irreversible, destructive, externally visible or high-consequence actions require explicit approval.

## Generated Output Is Not Knowledge

An Artifact may become a Document with provenance.

It becomes validated Knowledge only through the human-gated Improvement Loop defined by the AI Architecture.

---

# Consistency Model

ResearchOS follows the aggregate boundaries of the Logical Domain Model.

## Within an Aggregate

Consistency is immediate.

All invariants of the affected aggregate MUST hold at commitment.

## Across Aggregates

Consistency is eventual unless a future explicit decision establishes a stronger boundary.

Cross-aggregate coordination occurs through committed events and application processes.

## No Silent Overwrite

State evolution MUST preserve superseded versions and provenance where required by the Logical Domain Model.

## Conflict Handling

Concurrent or stale changes MUST result in one of:

- rejection;
- re-evaluation against current state;
- explicit merge;
- human resolution.

Silent last-write-wins behaviour is not acceptable for semantically meaningful state.

## Derived Consistency

Projections may lag behind canonical state.

The system MUST make that lag detectable where it affects user trust or decision quality.

---

# Failure and Recovery Model

Failure is expected and must remain contained.

| Failure | Required architectural response |
|---|---|
| **Reasoning failure** | Produce no state mutation; retry, use another procedure or request human input |
| **Invalid proposal** | Reject before commitment and record the reason when useful |
| **Authorization failure** | Deny without exposing restricted context or capability |
| **Domain validation failure** | Preserve prior canonical state unchanged |
| **Commit failure** | Publish no committed event and perform no dependent effect |
| **Event-delivery failure** | Retry or recover without duplicating canonical effects |
| **Projection failure** | Continue protecting canonical state; rebuild the projection |
| **External-effect failure** | Record failure and decide retry, compensation or escalation |
| **Ambiguous external outcome** | Mark outcome unknown and reconcile before repeating unsafe effects |
| **Provider outage** | Defer, degrade or route to an alternative without corrupting state |
| **Context-construction failure** | Abort the intelligent operation rather than reason on knowingly incomplete context |
| **Operating-memory loss** | Reconstruct from canonical state where possible; world state remains intact |
| **Canonical corruption** | Stop unsafe mutation, restore from durable versions and verify provenance |

## No Silent Failure

A failure that affects canonical state, an external effect, an approval or user trust MUST become visible to the relevant operator or recovery process.

## Recovery Before Progress

A failed or ambiguous irreversible effect MUST be reconciled before the system issues an equivalent effect again.

## Derived State Is Rebuildable

The architecture assumes indexes, caches, read models and context representations can be deleted and rebuilt without losing truth.

---

# Trust Boundaries

ResearchOS contains explicit trust boundaries.

## Researcher Boundary

The system receives intent and approval from a human authority, but still validates structural and domain correctness.

## Cognitive Boundary

Models and agents are untrusted proposers with bounded capabilities.

Their output is treated as data, not authority.

## External-System Boundary

External content may be incomplete, malicious, stale or semantically incompatible.

It must be normalized, constrained and validated before affecting the domain.

## Tool Boundary

Tools execute with least privilege and capability-scoped access.

A reasoning component does not inherit unrestricted environment access merely because it can request a tool.

## Projection Boundary

Derived stores may be stale or inconsistent.

Canonical decisions must not rely on a projection when the current authoritative state is required.

## Provider Boundary

No provider is trusted with more information or authority than its operation requires.

Provider-specific state is never the only copy of system-critical state.

---

# Dependency Rules

The following dependency rules are mandatory.

1. **The Domain Kernel depends on no outer plane.**
2. **The Experience Plane depends on application contracts, not persistence or AI internals.**
3. **The Control Plane coordinates use cases but does not redefine domain invariants.**
4. **The Cognitive Plane accesses the world through domain-oriented contracts, never physical storage.**
5. **The AI Operating Layer may propose changes but may not commit them.**
6. **The Execution Plane accepts only authorized effect requests.**
7. **External integrations translate into domain language at the boundary.**
8. **Projections depend on canonical state and events; canonical state never depends on projections.**
9. **Agents or workers do not coordinate by direct private calls. They coordinate through shared state, events and governed application operations.**
10. **No subsystem may maintain an authoritative shadow copy of another subsystem's state.**
11. **Cross-cutting policy and observability may inspect or constrain operations but may not bypass domain rules.**
12. **Technology-specific types must not leak into the Domain Model or its public contracts.**

`SOFTWARE_ARCHITECTURE.md` translates these rules into component contracts and conformance tests that make violations difficult by construction.

---

# Quality Attributes

## Correctness and Coherence

**Goal:** one reliable model of the researcher's world.

**Architectural response:** canonical domain state, aggregate invariants, validation before commitment, no shadow truth.

## Auditability and Provenance

**Goal:** reconstruct meaningful decisions and state evolution.

**Architectural response:** explicit operations, caused transitions, provenance, committed events and effect outcomes.

## Recoverability

**Goal:** recover from partial failure without silent corruption.

**Architectural response:** atomic aggregate transitions, version history, idempotent reactions, rebuildable projections and explicit ambiguous states.

## Evolvability

**Goal:** replace implementation mechanisms without redesigning the system.

**Architectural response:** logical subsystem boundaries, technology-neutral contracts, domain independence and projection discipline.

## Provider Independence

**Goal:** models and external services remain replaceable.

**Architectural response:** provider adapters, typed proposals, domain-language contracts and no provider-owned truth.

## Security and Privacy

**Goal:** protect long-lived personal and research state.

**Architectural response:** least privilege, policy-constrained context, explicit approval, trust boundaries and auditable disclosure.

## Explainability

**Goal:** explain recommendations and actions in terms the researcher can inspect.

**Architectural response:** evidence references, explicit proposals, policy decisions, provenance and state transitions rather than hidden reasoning traces.

## Usability

**Goal:** reduce cognitive load and preserve focus.

**Architectural response:** one Workspace, progressive disclosure, context-adaptive views and software-before-AI interactions.

## Efficiency

**Goal:** use intelligence and infrastructure only when they add value.

**Architectural response:** direct deterministic paths, bounded context, bounded reactions, replaceable projections and capability-specific model selection.

## Availability and Graceful Degradation

**Goal:** preserve useful operation when optional intelligent services fail.

**Architectural response:** canonical state independent of models, direct operations that do not require AI, deferred cognitive work and explicit degraded modes.

## Testability

**Goal:** verify architecture and behaviour reproducibly.

**Architectural response:** deterministic domain transitions, typed operations, isolated subsystem contracts, replayable events and explicit acceptance criteria.

---

# Architectural Invariants

The following invariants summarize the non-negotiable architecture.

1. ResearchOS exposes one coherent operational environment.
2. The Domain State is the single authoritative representation of the researcher's world.
3. No separate cognitive, agent, index or provider state may compete with the Domain as truth.
4. Every persistent mutation passes through the Domain Kernel.
5. Every mutation is explicit, validated and causally attributable.
6. Domain invariants are enforced before commitment.
7. A domain event describes a committed fact, never an intention.
8. Events are published only after their corresponding state transition succeeds.
9. Context is constructed per operation, bounded, policy-constrained and transient.
10. World memory is the Domain recalled; operating memory is disposable.
11. AI models and agents are proposers, never state authorities.
12. Reasoning and execution are separated by governance.
13. External effects require authorization and return as observations.
14. Irreversible and outward-facing actions require human approval.
15. Generated Artifacts do not become validated Knowledge without human acceptance.
16. Autonomy is granted per behaviour and cannot self-escalate.
17. Workers coordinate through shared state and events, not direct hidden control flow.
18. Reactive cascades are bounded and safe under duplicate delivery.
19. Projections, indexes, caches and graphs are derived and rebuildable.
20. The Domain remains independent of UI, AI, storage, providers and deployment.
21. Failures never silently corrupt canonical state.
22. Explainability is grounded in evidence, provenance and explicit decisions, not private chain-of-thought.
23. Logical boundaries do not imply distributed deployment.
24. Architectural complexity must be justified by a system property, not by analogy or fashion.

---

# Architectural Decisions Established Here

## SA-001 · Domain State Is Canonical

ResearchOS uses its Domain State as the sole authoritative model of the world.

**Consequence:** no independent canonical cognitive state is introduced.

## SA-002 · The Domain Kernel Owns Mutation Authority

Every persistent state transition is validated and committed through one governed boundary.

**Consequence:** UI, AI, integrations and persistence adapters cannot mutate state directly.

## SA-003 · Intelligence Is Propositional

Models and agents produce interpretations, Artifacts and typed proposals.

**Consequence:** intelligence remains replaceable and cannot contaminate truth merely by generating output.

## SA-004 · Reasoning and Execution Are Separate

A proposed action is validated and authorized before execution.

**Consequence:** tool access, external effects and destructive operations remain controllable and auditable.

## SA-005 · Events Follow Commitment

Events describe committed facts and drive decoupled reactions.

**Consequence:** event-driven coordination does not replace domain authority.

## SA-006 · Projections Are Disposable

Every optimized view is subordinate to canonical state.

**Consequence:** the architecture may use relational, graph, vector, document or other projections without making any of them the conceptual source.

## SA-007 · Logical Boundaries Precede Physical Distribution

Subsystems are defined by responsibility and authority, not deployment.

**Consequence:** the future implementation may begin as a modular unit and distribute only where an operational need justifies it.

## SA-008 · Human Authority Is Structural

Scientific validation, irreversible effects and outward communication remain human-gated.

**Consequence:** autonomy can grow without transferring final epistemic responsibility.

---

# Relationship to Existing Documents

| Document | Owns | This document uses it for |
|---|---|---|
| **VISION.md** | purpose and long-term outcome | architecture drivers |
| **SYSTEM_PRINCIPLES.md** | permanent principles | architectural invariants |
| **SYSTEM_MODEL.md** | Assets, Capabilities, Interfaces, Unified State | system composition |
| **RESEARCHER_OPERATIONAL_MODEL.md** | operational reality and friction | context and interaction requirements |
| **SYSTEM_RESPONSIBILITIES.md** | permanent system obligations | subsystem responsibilities |
| **DOMAIN_MAP.md** | domain areas and relationships | responsibility boundaries |
| **DOMAIN_MODEL.md** | entities and conceptual meanings | canonical world model |
| **KNOWLEDGE_MODEL.md** | meaning and evolution of Knowledge | epistemic authority and validation |
| **SYSTEM_CAPABILITIES.md** | stable behaviour vocabulary | execution and proposal language |
| **USE_CASES.md** | concrete behaviours and state changes | application operations and events |
| **AI_ARCHITECTURE.md** | AI Operating Layer, autonomy and human control | Cognitive Plane |
| **MEMORY_MODEL.md** | retention and recall | memory contract |
| **CONTEXT_MODEL.md** | per-task context construction | context contract |
| **EVENT_MODEL.md** | facts and reactions | event coordination contract |
| **LOGICAL_DOMAIN_MODEL.md** | attributes, aggregates, lifecycles and invariants | Domain Kernel specification |
| **INTERACTION_MODEL.md** | Workspace and collaboration modes | Experience Plane |
| **DOMAIN_VERTICALS.md** | specialization across operational domains | proof that the architecture is domain-wide |
| **ROADMAP.md** | build sequence | downstream architectural work |
| **IMPLEMENTATION_PLAN.md** | experiments and proving slices | validation of architectural claims |

This document owns only the system-wide arrangement of those responsibilities.

---

# Relationship to Software Architecture

`SOFTWARE_ARCHITECTURE.md` translates this specification into implementable, technology-neutral structures.

It must define, at minimum:

- module boundaries;
- application services and use-case handlers;
- domain repositories and domain services;
- ports and adapters;
- command, query, proposal and event contracts;
- transaction and event-publication mechanisms;
- context construction runtime;
- memory recall and consolidation mechanisms;
- AI runtime and model-provider abstraction;
- capability and tool execution;
- policy and approval enforcement;
- integration adapters;
- projection and indexing mechanisms;
- long-running process management;
- observability, audit and recovery mechanisms;
- conformance tests for the architectural invariants.

The Software Architecture MUST NOT assume that each subsystem becomes a microservice.

Physical separation is a later optimization and must be justified by isolation, scaling, reliability, security or independent evolution needs.

---

# Deferred Decisions

This document intentionally leaves the following decisions to their owning documents or future ADRs.

- Whether **Event** becomes a persisted domain concept.
- Whether **Decision** receives a promoted domain representation.
- Whether **Artifact** remains a Document specialization or is promoted.
- Whether **Curate** becomes a formal System Capability.
- Whether Policy requires a dedicated conceptual model.
- The exact representation of long-running application processes.
- The event-delivery and state-commit mechanism.
- The physical persistence strategy.
- The read-model, graph, vector and search projection strategy.
- The model-provider and agent-runtime strategy.
- The deployment and infrastructure topology.

Deferral does not weaken the architecture.

It protects the distinction between permanent system commitments and replaceable implementation choices.

---

# Conformance Criteria

A future implementation conforms to this architecture only if it can demonstrate that:

1. canonical state has one authoritative owner;
2. no AI or interface path bypasses domain validation;
3. proposals and committed facts are distinguishable;
4. event publication cannot represent an uncommitted transition;
5. derived projections can be rebuilt;
6. irreversible and outward actions are human-gated;
7. context is constructed and policy-constrained per operation;
8. operating-memory loss does not destroy world state;
9. external effects and ambiguous outcomes are traceable;
10. cross-aggregate processes tolerate eventual consistency;
11. direct software operation remains possible without an AI dependency;
12. model and storage providers can be replaced without changing the Domain Model;
13. failures preserve or recover canonical state;
14. every major state transition can be explained through explicit evidence and provenance;
15. architectural boundaries are enforced by code structure and tests, not only by documentation.

---

# Evolution Strategy

This architecture is expected to evolve by refinement, not by accumulation.

A new subsystem is justified only when an existing owner cannot carry a responsibility without violating cohesion, authority or dependency rules.

A new source of truth is never introduced for convenience.

A new distributed boundary is never introduced merely because the system has logical layers.

A new intelligent behaviour is expressed through existing domain concepts and system capabilities before new primitives are invented.

Every significant change should answer four questions.

1. Which architectural responsibility changes?
2. Which authority boundary changes?
3. Which invariant is added, removed or weakened?
4. Why can the change not be expressed within the existing architecture?

If none of those questions has an answer, the change belongs to Software Architecture, Technical Architecture or implementation — not here.

---

# Final Architectural Statement

ResearchOS is a governed system for preserving and operating on the researcher's world over time.

Its Domain is durable.

Its intelligence is replaceable.

Its context is transient.

Its events are factual.

Its execution is authorized.

Its projections are reconstructible.

Its researcher remains the final authority.

The architecture succeeds when models, tools, interfaces and infrastructure can evolve continuously while the coherence, provenance and ownership of the researcher's world remain intact.
