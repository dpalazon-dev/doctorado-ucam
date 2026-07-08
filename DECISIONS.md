# Architecture Decision Records

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 4 — Navigation, Delivery and Process |
| **Normative status** | Historical architectural decision register |
| **Authoritative for** | Decision context, status, rationale, consequences, supersession and links to the canonical documents changed by each decision. |
| **Not authoritative for** | Current domain semantics, system behavior, software contracts, capability vocabulary, event semantics, technology selection or roadmap ordering. |
| **Required reading** | `DOCUMENTATION_ARCHITECTURE.md` and the canonical owner documents named by the relevant ADR. |
| **Downstream documents** | Architecture reviews, implementation planning, pull requests and future ADRs. |

> ADRs explain why the canonical specification changed. The current rule always lives in the concern owner identified by `DOCUMENTATION_ARCHITECTURE.md`.

---

## Purpose

This register preserves significant architectural decisions that would otherwise be lost in prose revisions or version history.

Each ADR is immutable after acceptance except for:

- status changes;
- supersession links;
- correction of factual metadata;
- links to later evidence.

A changed decision is superseded by a new ADR rather than silently rewritten.

---

# Decision Index

| ADR | Title | Status | Date | Canonical owners |
|---|---|---|---|---|
| ADR-0001 | Domain State Is Authoritative | Accepted | 2026-07-08 | `SYSTEM_ARCHITECTURE.md`, `LOGICAL_DOMAIN_MODEL.md` |
| ADR-0002 | Modular Monolith as the Initial Deployment | Accepted | 2026-07-08 | `SOFTWARE_ARCHITECTURE.md` |
| ADR-0003 | Commands Express Intent; Events Record Committed Change | Accepted | 2026-07-08 | `EVENT_MODEL.md`, `SOFTWARE_ARCHITECTURE.md` |
| ADR-0004 | Seven Core Entities as a Falsifiable Canonical Baseline | Accepted | 2026-07-08 | `DOMAIN_MODEL.md`, `LOGICAL_DOMAIN_MODEL.md`, `IMPLEMENTATION_PLAN.md` |
| ADR-0005 | Resolve Decision, Artifact, Event and Curate Without New Roots | Accepted | 2026-07-08 | Domain, capability, AI and event owners |
| ADR-0006 | Experimental Technical Baseline Before Stabilized Technical Architecture | Accepted | 2026-07-08 | `ROADMAP.md`, `IMPLEMENTATION_PLAN.md` |

---

# ADR-0001 · Domain State Is Authoritative

**Status:** Accepted  
**Date:** 2026-07-08

## Context

ResearchOS contains several mechanisms that persist, derive or project information: canonical domain state, documents, memory, context packages, event records, search indexes, graph projections, model outputs and external integrations.

Without an explicit authority rule, any mechanism could become a competing source of truth. A vector index might be treated as Knowledge, a model output might mutate state directly, or an external system might silently override domain invariants.

## Decision

The committed Domain Model is the authoritative representation of ResearchOS state.

- Domain aggregates and their validated relationships own business meaning and invariants.
- Content, event logs, memories, contexts, indexes, embeddings and graph/search projections serve the domain.
- Derived stores are rebuildable or reconcilable from canonical state and retained source evidence.
- External systems enter through anti-corruption boundaries and cannot override canonical state directly.
- AI produces interpretations, Artifacts and typed Proposals; it does not obtain mutation authority by generating them.

## Rationale

A single authority surface prevents semantic drift, makes provenance enforceable and allows mechanisms to evolve independently. It also preserves the project's defining domain-first position without tying it to one database or model provider.

## Consequences

### Positive

- One consistency authority.
- Replaceable retrieval, memory and AI mechanisms.
- Explicit mutation and approval paths.
- Rebuildable projections and clearer recovery.

### Negative

- More translation at system boundaries.
- Some operations require proposals and domain validation rather than direct writes.
- Canonical state can become a bottleneck if module boundaries are poorly designed.

## Canonical specification

- `SYSTEM_ARCHITECTURE.md` — state authority and trust boundaries.
- `LOGICAL_DOMAIN_MODEL.md` — canonical logical structure.
- `SOFTWARE_ARCHITECTURE.md` — transactional domain kernel and projection rules.

---

# ADR-0002 · Modular Monolith as the Initial Deployment

**Status:** Accepted  
**Date:** 2026-07-08

## Context

The logical architecture contains distinct domain modules, application services, cognitive processes, event reactions, integrations and projections. Translating each logical boundary directly into a network service would introduce distributed transactions, deployment overhead, operational latency and observability requirements before the project has validated a single end-to-end slice.

ResearchOS is currently a personal project with no production load evidence requiring independent service scaling.

## Decision

The initial deployment is a modular monolith with explicit internal module boundaries, ports and adapters.

Separate runtime processes may be used where execution characteristics justify them — for example an interactive process, background worker or isolated tool runner — but they share the same architectural contracts and do not imply microservice ownership.

Physical service extraction requires a later ADR justified by at least one of:

- independent scaling;
- security or isolation;
- failure containment;
- materially different runtime requirements;
- independent release cadence;
- demonstrated organizational ownership.

## Rationale

A modular monolith preserves conceptual boundaries while minimizing distributed-systems cost. It gives the proving slice the shortest path to real evidence and leaves extraction possible after module contracts are exercised.

## Consequences

### Positive

- Faster first implementation.
- Atomic local transactions around aggregates and outbox state.
- Simpler debugging, testing and deployment.
- Architecture boundaries can be enforced through imports and tests.

### Negative

- Discipline is required to prevent a layered monolith from becoming entangled.
- Some background or isolated workloads still need process boundaries.
- Later extraction may require data ownership migrations.

## Canonical specification

- `SOFTWARE_ARCHITECTURE.md` — reference architectural style, runtime roles and package rules.

---

# ADR-0003 · Commands Express Intent; Events Record Committed Change

**Status:** Accepted  
**Date:** 2026-07-08

## Context

Earlier event prose implied that ResearchOS was built on events instead of commands and that workers coordinated exclusively through event cascades. The Software Architecture already distinguished explicit commands, atomic domain transitions, committed Domain Events and durable Process Managers.

Treating every workflow as choreography would obscure ownership, deadlines, compensation and completion conditions. Treating events as commands would weaken factual semantics and make replay or audit ambiguous.

## Decision

ResearchOS uses the following canonical cycle:

```text
Command
    ↓
validated domain transition
    ↓
atomic commit
    ↓
Domain Event
    ↓
reaction · projection · durable process
```

- Commands express human, policy or process intent.
- Domain handlers validate authority and invariants before mutation.
- Aggregate state and pending Domain Events commit atomically.
- Events report facts that already happened; they are never disguised instructions.
- Independent reactions use event choreography.
- Workflows requiring ownership, deadlines, compensation or explicit completion use Application Services or durable Process Managers that issue commands and observe resulting events.
- Workers do not call another worker's private implementation.

## Rationale

The distinction preserves explicit intent, factual event semantics and controlled orchestration while retaining decoupled reactions.

## Consequences

### Positive

- Clear ownership of mutations.
- Reliable audit and event semantics.
- Bounded use of choreography.
- Process managers can handle long-running workflows without inventing fake events.

### Negative

- The runtime must support both command handling and event delivery.
- Designers must decide deliberately between choreography and orchestration.
- Duplicate delivery and idempotency remain implementation obligations.

## Canonical specification

- `EVENT_MODEL.md` — event meaning and reaction discipline.
- `SOFTWARE_ARCHITECTURE.md` — command protocol, outbox/inbox and Process Managers.

---

# ADR-0004 · Seven Core Entities as a Falsifiable Canonical Baseline

**Status:** Accepted  
**Date:** 2026-07-08

## Context

The current domain is organized around seven Core Entities: Project, Document, Knowledge, Task, Activity, Person and Resource. This compact ontology prevents uncontrolled entity growth and supports all six documented verticals through Derived Types.

However, declaring the seven roots permanently unchangeable would bias Experiment 0: an implementation agent could force real concepts into artificial representations merely to preserve the rule.

## Decision

The seven Core Entities are the canonical baseline for routine design and implementation, but their sufficiency is falsifiable.

- New ordinary concepts must first be represented as Derived Types, Value Objects, relationships, events, process state or projections.
- A new root may be proposed only when real operational evidence shows that every existing representation would break invariants, erase identity or create artificial ownership/relationships.
- Root changes require Experiment 0 evidence, an ADR and updates to `DOMAIN_MODEL.md` and `LOGICAL_DOMAIN_MODEL.md` before code adopts them.
- Convenience, naming preference or storage shape are not sufficient evidence.

## Rationale

This decision protects the domain from speculative growth while keeping the architecture scientifically honest and able to learn from real use.

## Consequences

### Positive

- Stable implementation target.
- Explicit burden of proof for ontology changes.
- Experiments can genuinely challenge the design.
- Derived Types remain the normal extension mechanism.

### Negative

- Some concepts may remain provisional until enough evidence exists.
- A falsified root model causes significant downstream revision.
- Teams must maintain a friction log rather than normalizing model strain.

## Canonical specification

- `DOMAIN_MODEL.md` and `LOGICAL_DOMAIN_MODEL.md` — current roots and extension rules.
- `IMPLEMENTATION_PLAN.md` — Experiment 0 and falsification criteria.

---

# ADR-0005 · Resolve Decision, Artifact, Event and Curate Without New Roots

**Status:** Accepted  
**Date:** 2026-07-08

## Context

Four concepts recurred across the architecture but remained ambiguously labelled as candidates: Decision, Artifact, Event and Curate. Leaving them unresolved would make the Component Model and Agent Runtime depend on unstable semantics.

## Decision

### Decision

`Decision` is a formal Derived Type of `Knowledge`.

It represents a traceable conclusion or commitment and carries decision-specific semantics such as rationale, conditions, provenance and consequences. The episode in which a decision was made may also appear in episodic memory or as an Activity, but those are related records rather than the decision's meaning.

### Artifact

`Artifact` is a typed output contract of the Cognitive Runtime, not a Core Entity.

It is transient by default. When retained, it persists as a `Document` with explicit provenance. Its meaning may enter Knowledge only through the human-gated Knowledge lifecycle.

### Event

`Event` is a committed runtime and audit record, not a Core Entity.

Domain Events, Integration Events and audit records may be persisted and projected for queryability. Their persistence does not add Event to the domain ontology.

### Curate

`Curate` is a composite AI Operating Layer responsibility, not a ninth capability.

It composes `Understand + Organize + Reason` to improve existing knowledge through consolidation, duplicate detection, relationship repair and inconsistency analysis.

## Rationale

Each concept receives the smallest representation that preserves its semantics and lifecycle. The resolution removes ambiguity without expanding the root ontology or capability vocabulary prematurely.

## Consequences

### Positive

- Component and runtime contracts can be designed against stable concepts.
- No parallel persistence category is created for generated output.
- Event history remains queryable without contaminating the domain ontology.
- Capability vocabulary stays atomic.

### Negative

- Decision-specific logical attributes still require refinement.
- Artifact implementations must handle transient and persisted forms explicitly.
- Curate spans multiple capability handlers and therefore needs orchestration.

## Canonical specification

- `DOMAIN_MODEL.md` and `LOGICAL_DOMAIN_MODEL.md` — Decision and Artifact persistence boundary.
- `AI_ARCHITECTURE.md` and `SYSTEM_CAPABILITIES.md` — Artifact and Curate semantics.
- `EVENT_MODEL.md` and `SOFTWARE_ARCHITECTURE.md` — Event semantics and persistence.

---

# ADR-0006 · Experimental Technical Baseline Before Stabilized Technical Architecture

**Status:** Accepted  
**Date:** 2026-07-08

## Context

The proving experiments require executable tools: a language, storage, document parser, model adapter and worker mechanism. Waiting for a complete final Technical Architecture would block evidence. Conversely, treating the first convenient tools as permanent architecture would turn prototype accidents into commitments.

The roadmap also needs an ordered path through Component Model, Data Architecture, Agent Runtime and Technical Architecture.

## Decision

ResearchOS distinguishes two levels of technical choice:

### Experimental Technical Baseline

A minimal, reversible set of tools selected only to execute experiments and collect evidence. It must:

- satisfy the current software contracts;
- be replaceable through ports/adapters;
- avoid infrastructure that the experiment does not need;
- record versions and configuration for reproducibility;
- create no presumption of production adoption.

### Stabilized Technical Architecture

The evidence-backed technology, deployment and operational decisions documented in `TECHNICAL_ARCHITECTURE.md`. Significant commitments receive their own ADRs.

The document sequence is:

```text
COMPONENT_MODEL.md
        ↓
DATA_ARCHITECTURE.md
        ↓
AGENT_RUNTIME.md
        ↓
TECHNICAL_ARCHITECTURE.md
        ↓
Proving Experiments
        ↓
Architecture stabilization from evidence
```

`TECHNICAL_ARCHITECTURE.md` may begin with a clearly labelled provisional baseline and is updated or superseded after experiment results.

## Rationale

This distinction permits immediate empirical work without confusing a prototype with the production design.

## Consequences

### Positive

- Experiments are not blocked by complete product selection.
- Tooling remains deliberately reversible.
- Final choices can cite measured evidence.
- The requested document sequence remains coherent.

### Negative

- Some technical choices may be made twice.
- The Technical Architecture must label provisional and stabilized decisions precisely.
- Reproducibility metadata becomes mandatory even for prototypes.

## Canonical specification

- `ROADMAP.md` — sequence and status.
- `IMPLEMENTATION_PLAN.md` — experiment boundary and baseline rules.
- Future `TECHNICAL_ARCHITECTURE.md` — concrete selections and maturity status.
