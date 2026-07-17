# Data Architecture

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 2 — Specialized Implementation Model |
| **Normative status** | Canonical for logical data architecture |
| **Authoritative for** | Data authority classes, logical data ownership, record families, transaction boundaries, relationship persistence, event and operational records, projection semantics, lifecycle, retention, migration, backup and recovery requirements. |
| **Not authoritative for** | Domain meaning or invariants, component responsibilities, agent execution semantics, concrete database products, storage engines, cloud services or physical deployment. |
| **Required reading** | `SYSTEM_ARCHITECTURE.md`, `LOGICAL_DOMAIN_MODEL.md`, `SOFTWARE_ARCHITECTURE.md`, `COMPONENT_MODEL.md`, `CANONICAL_DATA_MODEL.md`, `MEMORY_MODEL.md`, `CONTEXT_MODEL.md`, `EVENT_MODEL.md`, `DECISIONS.md`. |
| **Downstream documents** | `AGENT_RUNTIME.md`, `TECHNICAL_ARCHITECTURE.md`, `SPEC_CATALOG.md`, database and vertical Specs, schemas, migrations, repository adapters, projection specifications, retention policies and backup runbooks. |

> Data mechanisms serve the canonical domain. Persistence is not authority by itself.

> **Status: Canonical · v1.0.** This document defines logical data authority, ownership and lifecycle independently from storage products.

---

## Purpose

ResearchOS operates over several kinds of durable and transient information:

- canonical domain state;
- document content;
- commands and process state;
- proposals and approvals;
- events and audit records;
- memories and context manifests;
- lexical, graph and vector projections;
- external observations and effect results.

Treating all of these as one undifferentiated database would create competing truth, accidental coupling and unrecoverable derived state.

This document defines the logical data architecture that preserves:

- one authoritative representation of the researcher's world;
- explicit ownership for every write;
- transactional integrity within declared boundaries;
- traceable provenance and causality;
- replaceable retrieval and AI mechanisms;
- reconstructible projections;
- recoverable long-running work;
- controlled deletion and retention;
- reproducible experiments.

It defines **what data exists, who owns it and how it evolves**. `TECHNICAL_ARCHITECTURE.md` decides where it is physically stored.

---

# Position in the Architecture

```text
Logical Domain Model
        ↓ defines canonical entities and invariants
Canonical Data Model
        ↓ defines shared record form and extension contracts
Software Architecture
        ↓ defines transactions, ports and projections
Component Model
        ↓ assigns component ownership
Data Architecture
        ↓ defines logical record families, ownership and lifecycles
Technical Architecture
        ↓ maps them to embedded local products and process boundaries
Development and Vertical Specs
        ↓ define exact schemas, indexes, migrations and acceptance tests
```

This document may refine persistence semantics but may not redefine a Domain Entity, relationship or invariant.

---

# Data Architecture Thesis

> ResearchOS has one canonical world state and several supporting data classes with different authority, durability and rebuild semantics.

The central distinction is:

```text
Canonical state
    ≠ content bytes
    ≠ operational progress
    ≠ event transport
    ≠ audit history
    ≠ search index
    ≠ context package
    ≠ model output
    ≠ external system state
```

Each class has a purpose. None inherits domain authority from being persistent.

---

# Data Authority Classes

ResearchOS defines eight logical data classes.

| Class | Authority | Durability | Rebuildable | Examples |
|---|---|---:|---:|---|
| **Canonical Domain Data** | authoritative for world state | durable | no, except from controlled backup/history | Projects, Documents, Knowledge, Tasks, Activities, People, Resources, canonical relationships |
| **Content Data** | authoritative for retained bytes/content versions | durable | only from source or backup | PDFs, text extractions, generated documents, attachments |
| **Operational Data** | authoritative for execution progress | durable when recovery/audit requires | partially | Commands, Proposals, approvals, Jobs, Process Instances, cognitive operations, effects |
| **Event Data** | authoritative as a record that a committed fact was emitted or delivered | durable according to event policy | outbox from transaction; projections from source | Domain Event envelopes, Integration Events, Inbox receipts |
| **Projection Data** | non-authoritative query acceleration | durable but disposable | yes | read models, lexical search, vectors, graph, timelines, dashboards |
| **Audit Data** | authoritative for security/authority trace | durable and append-oriented | no, though exports may be regenerated | access decisions, approval actions, privileged maintenance, secret access metadata |
| **Configuration Data** | authoritative for runtime configuration and policy version | durable/versioned | from repository or secret manager | feature flags, capability policy, prompt versions, model routing config |
| **Transient Execution Data** | temporary working material | bounded and expiring | no requirement | Context body, model prompt assembly, parser buffers, unpersisted Artifact bodies |

## External State

External systems own their own state.

ResearchOS stores:

- external references;
- observations;
- synchronization cursors;
- Effect Intents;
- Effect Results;
- reconciliation status.

An external observation becomes canonical only through an explicit Command and domain validation.

---

# Logical Data Topology

```text
                            ┌─────────────────────┐
                            │ External Systems    │
                            └──────────┬──────────┘
                                       │ observations/effects
                                       ▼
┌───────────────────────────────────────────────────────────────────────┐
│ Operational Data                                                     │
│ Commands · Jobs · Processes · Proposals · Approvals · Effects        │
└───────────────────────────────┬───────────────────────────────────────┘
                                │ validated intent
                                ▼
┌───────────────────────────────────────────────────────────────────────┐
│ Canonical Domain Data                                                 │
│ Seven aggregate families · canonical relationships · provenance       │
└───────────────┬───────────────────────────────┬───────────────────────┘
                │ content references            │ committed events
                ▼                               ▼
┌──────────────────────────┐       ┌────────────────────────────────────┐
│ Content Data             │       │ Event Data                         │
│ immutable/versioned body │       │ Outbox · envelopes · Inbox         │
└──────────────────────────┘       └─────────────────┬──────────────────┘
                                                    │ derives
                                                    ▼
                                      ┌────────────────────────────────┐
                                      │ Projection Data                │
                                      │ views · lexical · vector · graph│
                                      └────────────────────────────────┘

Audit data observes authority-relevant operations across every boundary.
Transient execution data exists only within a bounded operation.
```

---

# Data Ownership

## Ownership Rule

Every durable record family has exactly one logical write owner.

The owner is responsible for:

- schema meaning;
- creation and mutation rules;
- versioning;
- lifecycle and retention;
- migration;
- access policy;
- integrity checks;
- emitted facts.

Physical co-location does not change logical ownership.

## Ownership Matrix

| Data family | Logical write owner |
|---|---|
| Knowledge aggregate records | Knowledge Domain component |
| Document aggregate records | Documents Domain component |
| Project aggregate records | Projects Domain component |
| Person aggregate records | People Domain component |
| Task aggregate records | Tasks Domain component |
| Activity aggregate records | Activities Domain component |
| Resource aggregate records | Resources Domain component |
| Canonical relationship | declared source/relationship owner in Logical Domain Model |
| Content objects and versions | Documents contract through Content Repository Adapter |
| Command receipt/idempotency | Command Gateway |
| Proposal and approval | Proposal and Approval Service |
| Job | Job Coordinator |
| Process Instance | Process Manager Host |
| Cognitive Operation | Cognitive Operation Manager |
| Context Manifest | Context Builder |
| Effect Intent/Result | External Effect Service |
| Outbox entry | Unit of Work / Canonical Persistence Adapter |
| Inbox receipt | Event Reaction Host |
| Projection generation | owning Projection Builder |
| Audit entry | Audit Service |
| Secret value | Secret and Credential Service or external secret mechanism |

No component may update another owner's record by direct storage access.

---

# Canonical Domain Data

## Persistence Model

Canonical data is organized by aggregate family, not by one universal entity table.

The physical implementation MAY share a database and common infrastructure, but it MUST preserve:

- one repository boundary per aggregate root;
- domain-specific lifecycle fields;
- explicit logical types;
- aggregate version;
- provenance and causal metadata;
- domain constraints;
- no generic property bag as the sole representation of core state.

JSON-like extension fields MAY support derived-type attributes when:

- the base schema remains explicit;
- required subtype fields are validated;
- query-critical fields are promoted to typed columns or equivalent indexes;
- the extension does not become an ungoverned schema bypass.

## Canonical Aggregate Record

Every persisted aggregate has, conceptually:

```text
Aggregate Record
├── identifier
├── aggregate_type
├── derived_type? 
├── version
├── lifecycle_state
├── created_at
├── updated_at
├── created_by / updated_by
├── provenance
├── domain attributes
└── pending events committed through the Unit of Work
```

These fields are logical. Area specifications define exact names and optionality.

## Identity and Version

- Identifier is stable for the life of the aggregate.
- Version increases monotonically on committed mutation.
- Version is used for optimistic concurrency.
- Content version and aggregate version are distinct.
- A new representation of the same entity does not create a new identity.
- A genuinely different entity does not reuse an identifier merely because content is similar.

## Temporal Fields

ResearchOS distinguishes:

- **recorded time** — when the system recorded the fact;
- **effective time** — when the fact applies in the modeled world;
- **observed time** — when an external fact was observed;
- **event time** — when a committed transition occurred;
- **processing time** — when a worker handled it.

Specifications MUST not collapse these when semantic differences matter.

---

# Canonical Relationships

Relationships are first-class logical records even when a physical database implements them as foreign keys or join tables.

## Relationship Record

A canonical relationship contains:

```text
Relationship
├── identifier or stable composite identity
├── relationship_type
├── source_identifier
├── target_identifier
├── source_version_at_creation?
├── lifecycle_state
├── valid_from / valid_to?
├── provenance
├── confidence?              # only when the domain permits epistemic uncertainty
├── created_at
├── created_by
└── owning_component
```

## Relationship Rules

- Every relationship type has one authoritative write owner.
- Reverse navigation is normally a projection.
- Bidirectional queryability does not imply dual write ownership.
- Inferred relationships remain candidate/projection data until explicitly promoted.
- Deleting an aggregate does not silently erase historical provenance.
- Cardinality and lifecycle rules come from `LOGICAL_DOMAIN_MODEL.md`.
- Cross-aggregate referential integrity may be enforced synchronously or through validation/reconciliation according to the canonical consistency model.

## Semantic Relationship Vocabulary

Knowledge relationships such as `supports`, `contradicts`, `extends`, `derived_from` and `relates_to` MUST retain:

- semantic type;
- provenance;
- source evidence;
- validation state;
- confidence where applicable.

A graph edge without these semantics is only a traversal aid, not canonical knowledge.

---

# Provenance Architecture

Provenance is required for any state that claims origin, evidence or derivation.

## Provenance Record

A provenance reference may include:

- source entity identifier and version;
- source content digest and segment reference;
- external locator and external version;
- actor or cognitive operation;
- method/capability used;
- model/tool identity and configuration version;
- observed/effective time;
- transformation or extraction policy version;
- parent provenance references;
- confidence and validation status.

## Provenance Rules

- Provenance is append-oriented.
- A newer interpretation does not erase the earlier source path.
- Model-generated content retains model and prompt/instruction version.
- A human validation decision is part of provenance.
- A citation must resolve to retained or recoverable source evidence.
- Derived projections preserve source identifiers and versions sufficient for staleness detection.

---

# Content Data

Content Data stores bytes or normalized bodies associated with Documents and Artifacts.

## Content Object

```text
Content Object
├── content_id
├── cryptographic_digest
├── media_type
├── byte_length
├── storage_locator
├── created_at
├── encryption/classification metadata
└── integrity status
```

## Content Version

```text
Content Version
├── document_id
├── document_version
├── content_id
├── source_locator?
├── extraction status
├── supersedes?
└── provenance
```

## Content Rules

- Stored content is immutable by digest.
- Editing creates a new content version.
- Canonical Document state references content; it does not embed provider-specific paths.
- Duplicate bytes may share one content object while retaining distinct Document identities.
- Original source content is preserved when legally and operationally permitted.
- Normalized text and OCR output are separate derived content versions with provenance.
- Generated Artifact content becomes durable only when persisted as a Document.
- Content integrity is checked on write and during maintenance verification.

## Content Deletion

Deletion distinguishes:

- removing a Document from active use;
- revoking access;
- deleting a content version;
- cryptographic or physical erasure;
- retaining minimal audit/provenance tombstones.

The exact policy depends on legal basis, source rights and user request.

---

# Operational Data

Operational Data makes intent, progress, authority and recovery explicit.

## Command Receipt

A Command Receipt records:

- command identifier;
- command type and schema version;
- actor;
- target identifier and expected version;
- idempotency key;
- correlation and causation;
- received time;
- terminal application result reference.

The full command payload may be retained, redacted or hashed according to sensitivity.

## Proposal

A Proposal records:

- proposal identifier and type;
- producing cognitive/process operation;
- intended command or effect type;
- subject digest;
- evidence and provenance;
- confidence and limitations;
- risk/reversibility classification;
- status;
- expiry;
- review requirements.

## Approval Decision

An Approval Decision records:

- proposal/effect reference;
- subject digest;
- actor;
- decision;
- constraints or edits;
- timestamp;
- reason;
- resulting command/effect reference.

Approval is invalid when the subject digest materially changes.

## Job

A Job records:

- job type and payload reference;
- status;
- priority;
- ready time;
- attempt count;
- lease owner and expiry;
- timeout and retry policy;
- result or failure classification;
- correlation and causation.

## Process Instance

A Process Instance records:

- process type and version;
- current state;
- business subject references;
- expected events/results;
- pending commands/jobs/effects;
- deadlines and timers;
- compensation state;
- failure/escalation state;
- history or transition log sufficient for recovery.

## Cognitive Operation

A Cognitive Operation records:

- operation type and version;
- objective;
- actor/initiator;
- policy and budget snapshot;
- state;
- Context Manifest reference;
- plan/checkpoint references;
- capability/model/tool invocation references;
- output Artifact/Proposal references;
- verification result;
- final status and failure reason.

The runtime MUST NOT persist hidden chain-of-thought. It persists plans, actions, observations, evidence, decisions and concise rationale needed for audit and recovery.

## Model Invocation

A Model Invocation record contains:

- provider and model identifier;
- request/instruction version;
- input/output digest or redacted body reference;
- structured-output schema version;
- token/usage/cost metrics when available;
- latency;
- finish status;
- safety/filter result;
- correlation and step identity.

## Capability/Tool Invocation

A capability invocation records:

- capability and implementation version;
- normalized parameters or digest;
- authorization scope;
- isolation mode;
- idempotency key where supported;
- result reference;
- duration and failure classification.

## External Effect Intent and Result

The intent and observed result are separate records.

An Effect Result may be:

- confirmed success;
- confirmed failure;
- partial;
- ambiguous;
- compensated;
- manually reconciled.

A transport timeout is not stored as confirmed failure without observation.

---

# Event Data

Events are committed facts, not commands and not Core Entities.

## Event Envelope

Every Domain or Integration Event envelope contains:

```text
Event Envelope
├── event_id
├── event_type
├── event_schema_version
├── aggregate_type?
├── aggregate_id?
├── aggregate_version?
├── occurred_at
├── recorded_at
├── actor_id?
├── correlation_id
├── causation_id
├── payload
├── metadata/classification
└── trace reference
```

## Outbox Record

An Outbox record adds delivery state:

- committed transaction reference;
- availability time;
- dispatch attempts;
- lease/checkpoint;
- dispatched time;
- terminal dispatch error if any.

Canonical mutation and Outbox insertion are atomic.

## Inbox Record

An Inbox record contains:

- consumer identity;
- event identifier;
- first/last seen time;
- processing status;
- result/checkpoint;
- attempts;
- terminal failure or dead-letter reference.

Inbox uniqueness enforces duplicate-safe consumption.

## Event Retention

The system retains enough event and transition history to support:

- causal trace;
- operational diagnosis;
- projection repair within declared windows;
- experiment reproducibility;
- audit obligations.

Because event sourcing is not the default, indefinite retention of every event is not automatically required. Retention is declared per event class.

---

# Projection Data

Projection Data is derived and rebuildable.

## Projection Families

### Relational Read Models

Optimized records for workspace lists, dashboards, filters and cross-aggregate summaries.

### Lexical Search

Tokenized/normalized text and field indexes for exact and full-text retrieval.

### Vector Projection

Embeddings associated with source segments and source versions.

### Graph Projection

Canonical and candidate relationships represented for traversal and analysis.

### Timeline Projection

Chronological view derived from Activities, Events, Documents, Tasks and process state.

### Cache

Short-lived computed values with explicit invalidation or expiry.

### Human-Readable Knowledge Projection

A legible, navigable, portable rendering of canonical Knowledge as interlinked documents; the Open Knowledge Format (OKF) is the candidate encoding. A **candidate** family — deferred and evidence-gated, evaluated in `SPEC-011` — generated from committed Knowledge and never edited back into it. Like every projection here it is non-authoritative and rebuildable; a retained bundle records its concrete `format`, `format_version`, `generator` and `generator_version`. See ADR-0012.

## Projection Record Requirements

Every projection record includes or can resolve:

- projection name;
- projection schema version;
- generation/build identifier;
- source entity/event identifier;
- source version;
- transformation version;
- created/updated time;
- staleness or checkpoint metadata.

## Rebuild Contract

Every projection specification defines:

- authoritative source;
- bootstrap strategy;
- incremental update events;
- idempotency key;
- checkpoint semantics;
- rebuild procedure;
- validation method;
- cutover strategy;
- acceptable lag.

A projection that cannot be rebuilt or reconciled has accidentally become authoritative and violates this architecture.

## Vector Records

A vector record contains:

- source document/entity identifier and version;
- segment identifier and offsets;
- normalized text digest;
- embedding provider/model identifier;
- embedding dimension/compatibility class;
- preprocessing policy version;
- vector generation;
- creation time;
- access classification.

Changing model or preprocessing policy creates a new generation.

## Graph Records

A graph projection distinguishes:

- canonical node/edge;
- candidate/inferred node/edge;
- projection-only traversal structure;
- community/cluster/summary output.

No inferred edge becomes canonical merely by appearing in the graph.

---

# Memory Mapping

The Memory Model is conceptual; this section maps it to data classes.

| Memory concept | Data representation |
|---|---|
| Semantic Memory | validated canonical Knowledge and related domain state |
| Episodic Memory | Activities, selected Events, process/effect outcomes and traceable episodes |
| Working Memory | transient execution state plus bounded Cognitive Operation checkpoints |
| Procedural Memory | versioned instructions, capability policies, templates, code and configuration |
| Preferences | canonical Person/configuration data according to ownership, not a separate memory store |

A vector database is not “memory.” It is one possible projection mechanism used during recall.

---

# Context Data

Context is transient selection, not canonical state.

## Context Manifest

The durable Context Manifest records:

- context identifier;
- task/intent reference;
- builder version and policy;
- source identifiers and versions;
- retrieval queries and filters;
- ranking/selection reasons;
- exclusions and truncation;
- token/size budget;
- creation and expiry time;
- sensitivity classification;
- digest of the assembled context body.

## Context Body

The body may contain:

- selected canonical facts;
- document excerpts;
- recalled episodes;
- process state;
- user instructions;
- capability descriptions.

The body is transient by default and must not be promoted to Knowledge without a separate validated transition.

Sensitive context should expire promptly and avoid ordinary logs.

---

# Audit Data

Audit Data records authority-relevant actions independently from ordinary telemetry.

Audit events include:

- authentication and session changes;
- authorization decisions;
- approval and rejection;
- privileged maintenance;
- secret/credential access metadata;
- external effect authorization and reconciliation;
- data export or deletion;
- policy changes;
- model/tool actions that cross a configured risk threshold.

Audit records are:

- append-oriented;
- actor and time attributed;
- tamper-evident according to deployment risk;
- access controlled;
- retained according to policy;
- exportable for inspection.

Audit does not duplicate full sensitive payloads unless necessary.

---

# Configuration and Secret Data

## Configuration

Configuration is versioned and environment-specific.

It includes:

- feature flags;
- runtime limits;
- retry policies;
- capability registry entries;
- model routing policy;
- prompt/instruction identifiers;
- projection versions;
- retention policy;
- autonomy grants.

A running operation records the relevant configuration version or snapshot.

## Secrets

Secrets are never stored in:

- domain aggregates;
- event payloads;
- model prompts;
- ordinary logs;
- source-controlled configuration;
- projection records.

Records store only secret references, scope and non-sensitive access metadata.

---

# Transaction Architecture

## Aggregate Transaction

One command commits, by default:

- one aggregate root;
- owned Value Objects/relationship changes within that consistency boundary;
- aggregate version update;
- pending Domain Events into the Outbox;
- command idempotency result when implemented in the same unit.

## Unit of Work

The Unit of Work guarantees:

- atomicity;
- optimistic version check;
- rollback on invariant or persistence failure;
- event-after-commit;
- no external effect inside the transaction.

## Cross-Aggregate Change

Cross-aggregate workflows use:

- subsequent commands triggered by events;
- or a Process Manager that records progress.

A physical database transaction across several aggregate roots requires an ADR proving the need.

## Operational Transactions

Operational components may commit their own state independently:

- lease acquisition;
- job attempt;
- process transition;
- proposal decision;
- model/tool invocation record;
- Inbox receipt;
- projection checkpoint.

They must not combine these updates with hidden canonical mutations.

---

# Concurrency and Locking

## Optimistic Concurrency

Aggregate version comparison is the default.

A stale command returns a first-class conflict result.

## Leasing

Jobs, outbox records, process activations and effect execution use bounded leases with:

- owner;
- acquired time;
- expiry;
- heartbeat when required;
- safe reclaim semantics.

A lease is not a domain lock.

## Resource Reservation

Capacity-sensitive Resource operations use domain reservation semantics and version checks.

Eventually consistent projections are insufficient when over-allocation would violate an invariant.

## Ordering

Ordering guarantees are scoped:

- aggregate version order within one aggregate;
- process transition order within one Process Instance;
- no assumption of total global event order.

---

# Idempotency and Deduplication

Idempotency is required at:

- command ingress;
- document import;
- event consumption;
- job execution;
- projection update;
- model/tool step where repeat cost or effect matters;
- external effect dispatch;
- synchronization import.

## Idempotency Record

A logical idempotency record contains:

- scope/consumer;
- idempotency key;
- request digest;
- first received time;
- status;
- result reference;
- expiry policy.

The same key with a different digest is a conflict, not a replay.

---

# Data Lifecycle

Every data family declares:

1. creation authority;
2. active lifecycle;
3. archival criteria;
4. retention period or retention basis;
5. deletion/erasure behavior;
6. backup inclusion;
7. projection invalidation;
8. audit/tombstone requirement.

## Lifecycle Classes

| Class | Default approach |
|---|---|
| Canonical domain | retain while semantically active; archive/deprecate before destructive deletion where possible |
| Content | retain by Document/version policy and rights; erase when requested/required if no valid basis remains |
| Operational | retain through recovery/audit window; summarize or purge after completion according to policy |
| Events | retain by event class and rebuild/audit need |
| Projections | disposable; rebuild or delete freely after source and policy checks |
| Audit | retain under explicit security/legal policy |
| Context body | short-lived and minimized |
| Manifests and experiment traces | retain sufficiently for reproducibility, with sensitive content redacted or referenced |

## Right to Delete and Forget

Deletion must propagate through:

- canonical records;
- content objects when no other owner/reference exists;
- projections and caches;
- vector and graph generations;
- pending jobs and contexts;
- external systems where an effect is authorized;
- backups according to documented expiry and restore handling.

Audit may retain a minimal non-sensitive deletion record where required.

---

# Privacy and Data Classification

Every record may carry a classification such as:

- public;
- internal;
- confidential;
- restricted/personal;
- secret credential (reference only).

Classification influences:

- authorized readers;
- model/tool eligibility;
- external provider use;
- encryption requirements;
- logging/redaction;
- retention;
- export and deletion.

A Context Builder must never select data solely by relevance; it must apply access and provider policy first.

---

# Schema and Contract Versioning

ResearchOS versions independently:

- domain schemas;
- command/query schemas;
- event schemas;
- process definitions;
- projection schemas;
- context manifests;
- capability/tool schemas;
- model structured-output schemas;
- configuration and instruction sets.

## Compatibility Rules

- persisted records identify their schema/definition version where interpretation may change;
- event consumers support declared compatibility windows;
- breaking event changes create a new version/type;
- migrations are explicit, reviewed and reversible where practical;
- provider payload versions are normalized at the adapter boundary;
- old projections may be rebuilt into a new generation before cutover.

## Migration Classes

- **expand** — add compatible structures;
- **backfill** — derive/populate new fields;
- **switch** — move readers/writers to new representation;
- **contract** — remove obsolete structures after verification;
- **semantic migration** — requires domain review and often an ADR because meaning changes.

Schema autogeneration may assist but never replaces migration review.

---

# Backup and Recovery

## Recovery Objectives

`TECHNICAL_ARCHITECTURE.md` assigns concrete RPO/RTO values. This architecture requires the categories below.

## Backup Sets

### Set A — Canonical and Operational State

Includes:

- aggregate data;
- canonical relationships;
- proposals/approvals;
- active jobs/processes;
- Outbox/Inbox;
- effect state;
- configuration versions required for recovery.

### Set B — Content

Includes retained content objects and version metadata.

### Set C — Audit and Critical Trace

Includes security and authority-relevant records.

### Set D — Projections

Normally excluded from primary backup when fully rebuildable, except when retaining them materially reduces recovery time and does not confuse authority.

## Restore Requirements

A restore procedure must:

- restore canonical state and content consistently;
- identify incomplete jobs/processes;
- reconcile Outbox dispatch and Inbox receipts;
- invalidate or rebuild projections;
- preserve identifiers and versions;
- verify content digests;
- prevent duplicate unsafe external effects;
- record the recovery operation in audit.

A backup is not considered valid until restore is tested.

---

# Integrity and Reconciliation

Maintenance checks include:

- aggregate version monotonicity;
- relationship target existence or documented tombstone;
- content digest verification;
- canonical Document ↔ content reference consistency;
- Outbox entries without dispatch progress;
- Inbox duplication/conflict;
- expired leases;
- orphaned jobs/processes;
- projection checkpoint/source mismatch;
- vector source version mismatch;
- graph canonical/candidate classification;
- approval digest mismatch;
- ambiguous external effects;
- missing provenance for validated Knowledge.

Integrity repair uses explicit maintenance/application operations. It does not silently patch canonical tables.

---

# Experimental Data Baseline

The proving experiments may begin with a physically simple layout if logical separation remains explicit.

The minimum baseline MAY use:

- one relational database for canonical, operational, event and initial projection schemas;
- one managed filesystem or object namespace for content;
- relational full-text retrieval before a separate search engine;
- one vector extension/store only when Experiment 2 requires semantic retrieval;
- relational relationship records and recursive queries before a dedicated graph database;
- one application-owned job/outbox mechanism before an external broker.

This is not permission to merge authority classes.

Separate schemas, repositories, access paths and naming must preserve ownership even when one product stores them.

---

# Proving Slice Data Flow

```text
1. User imports a paper
        ↓
2. Command Receipt + Document aggregate + content object commit
        ↓
3. DocumentRegistered event enters Outbox
        ↓
4. Event consumer starts durable processing Job/Process
        ↓
5. Parser creates normalized derived content with provenance
        ↓
6. Cognitive Operation + Context Manifest + model/tool traces
        ↓
7. Knowledge candidate Proposals persist as operational data
        ↓
8. User approves/rejects each candidate
        ↓
9. Accepted command commits Knowledge + canonical relationships + events
        ↓
10. Lexical/vector/graph projections update idempotently
        ↓
11. Question request builds a new Context Manifest from source versions
        ↓
12. Answer Artifact cites retained evidence; it does not become Knowledge automatically
```

## Evidence Collected

The experiment must retain enough data to measure:

- parse success and content integrity;
- metadata extraction accuracy;
- candidate precision/recall under a reviewed sample;
- provenance completeness;
- projection lag and rebuildability;
- context source accuracy;
- answer citation validity;
- processing latency and cost;
- retries, failures and human corrections.

---

# Data Specifications Required Before Development

After this architecture, implementation specifications should be produced per area.

Minimum specifications include:

- aggregate persistence specification for each Domain component;
- canonical relationship catalogue;
- Document content/version specification;
- Knowledge provenance and evidence specification;
- Command and idempotency records;
- Event envelope and event catalogue;
- Job and Process Instance schemas;
- Proposal/Approval schemas;
- Cognitive Operation and Agent Trace schemas;
- Context Manifest schema;
- projection definitions and rebuild procedures;
- retention/classification matrix;
- migration and backup runbooks.

These specifications define exact fields and indexes without changing the authority model in this document.

---

# Conformance Criteria

An implementation conforms to this Data Architecture only if:

1. canonical domain data is distinguishable from every supporting class;
2. every durable record family has one logical write owner;
3. one command commits one aggregate by default with pending events atomically;
4. no external effect occurs inside the canonical transaction;
5. canonical relationships have one write owner;
6. content is immutable/versioned and integrity-addressable;
7. validated Knowledge retains provenance to evidence;
8. Proposal and Approval state is durable and digest-bound;
9. Jobs and Process Instances survive restart;
10. event envelopes propagate actor, correlation and causation;
11. Inbox/Outbox records support duplicate-safe at-least-once delivery;
12. Context bodies remain transient while manifests remain traceable;
13. model/tool outputs remain operational until explicitly promoted;
14. projections declare source, version, generation and rebuild procedure;
15. vector and graph records distinguish canonical from inferred data;
16. deletion propagates to derived stores and contexts;
17. secrets never enter ordinary data paths;
18. migrations preserve semantic and version integrity;
19. backups are restore-tested;
20. the proving slice can reconstruct every accepted Knowledge item and answer citation from retained source evidence.

---

# Evolution Rules

A new data store or record family requires:

- declared authority class;
- logical write owner;
- access pattern that existing mechanisms cannot responsibly support;
- lifecycle and retention policy;
- consistency and transaction semantics;
- migration and backup impact;
- rebuild or reconciliation strategy;
- privacy classification;
- conformance tests.

A new database product is not a new data architecture.

A new projection does not create a new source of truth.

A model output does not create Knowledge without domain validation.

A persistent event does not become a Core Entity.

---

# Final Data Statement

ResearchOS stores one canonical representation of the researcher's world and several explicitly subordinate forms of content, progress, evidence, history and retrieval state.

Canonical data owns truth.

Content data owns retained bytes and versions.

Operational data owns recoverable work and authority decisions.

Event data records committed facts and delivery state.

Projection data accelerates questions and remains rebuildable.

Audit data preserves accountability.

Context and model output remain bounded execution material until the system deliberately promotes them.

This separation allows ResearchOS to change databases, indexes, model providers and deployment topology without changing what the system knows.
