# Canonical Data Model

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 2 — Specialized Implementation Model |
| **Normative status** | Canonical for the shared structural data contract |
| **Authoritative for** | Canonical entity envelopes, root-record shapes, value-object representation, Derived Type registration and extension, canonical relationship structure, provenance anchors, document content/version/segment contracts, projection source contracts, common versioning semantics, and the rules every vertical data specification must follow. |
| **Not authoritative for** | Domain meaning or invariants, vertical-specific semantics, use-case behavior, component ownership, transaction orchestration, retention policy, concrete database products, physical tables, ORM classes, API payloads, indexes, deployment, or user-interface design. |
| **Required reading** | `LOGICAL_DOMAIN_MODEL.md`, `DOMAIN_MODEL.md`, `DOMAIN_VERTICALS.md`, `DATA_ARCHITECTURE.md`, `SOFTWARE_ARCHITECTURE.md`, `EVENT_MODEL.md`, `DECISIONS.md`. |
| **Downstream documents** | Vertical domain specifications, database specifications, schemas, migrations, repository contracts, API schemas, projection specifications, validation suites, and generated language types. |

> Scope and conflict rules are defined in `DOCUMENTATION_ARCHITECTURE.md`.

---

> **Status: Canonical · v1.0.** This document defines the common structural form of ResearchOS data before any vertical or physical implementation specializes it.

---

# Purpose

ResearchOS has one shared domain and several operational verticals.

The seven Core Entities remain identical across Research, Teaching, Administration, Organization, Personal and Daily Work. Verticals may introduce Derived Types, additional fields, relationships, lifecycles, constraints and projections, but they must not invent independent representations of the same world.

The Logical Domain Model defines the meaning and technology-neutral structure of the domain. The Data Architecture defines authority, ownership, durability, movement and lifecycle. A further contract is required before implementation specifications can safely specialize the system:

> **the exact shared data form that every vertical, schema, API and repository must preserve.**

This document provides that contract.

It prevents each vertical from independently deciding:

- how identity is represented;
- how a Core Entity is distinguished from a Derived Type;
- how common lifecycle and audit fields are carried;
- how subtype-specific fields are extended;
- how relationships are typed and promoted;
- how provenance reaches source evidence;
- how document versions, text segments and embeddings connect;
- how canonical data differs from projections;
- how schema evolution and deletion propagate;
- how generated schemas remain projections of one model.

The objective is not to define physical tables. It is to make physical tables, object models, API schemas and vertical specifications derivable without semantic drift.

---

# Position in the Architecture

```text
Domain Model
    ↓ defines what exists
Logical Domain Model
    ↓ defines domain attributes, relationships and invariants
Canonical Data Model
    ↓ defines the shared structural representation
Vertical Data Specifications
    ↓ define Derived Types and vertical extensions
Physical Data Specifications
    ↓ define tables, indexes, schemas and migrations
Implementation
```

The surrounding implementation documents retain their own authority:

```text
LOGICAL_DOMAIN_MODEL.md
    owns meaning, cardinality, lifecycle and invariants

CANONICAL_DATA_MODEL.md
    owns common structural representation and extension contracts

DATA_ARCHITECTURE.md
    owns authority classes, ownership, persistence lifecycle, retention and recovery

TECHNICAL_ARCHITECTURE.md
    owns products, libraries and physical baseline choices

Vertical Specs
    own vertical Derived Types, extension fields and specialized rules
```

When documents appear to conflict:

1. domain meaning and invariants follow `LOGICAL_DOMAIN_MODEL.md`;
2. data authority and lifecycle follow `DATA_ARCHITECTURE.md`;
3. common record form and extension mechanics follow this document;
4. physical implementation follows the relevant technical specification.

---

# Canonical Data Thesis

> ResearchOS stores one canonical world through seven aggregate families, extends them through governed Derived Types, connects them through typed relationships, and derives every retrieval or interface structure as a projection.

The model is:

```text
Canonical Entity
    = Core Entity identity
    + Core payload
    + optional Derived Type
    + optional typed extension
    + provenance
    + version and lifecycle metadata

Canonical World
    = Canonical Entities
    + Canonical Relationships
    + retained Content and Versions
    + promoted provenance
```

Everything else is supporting or derived:

```text
canonical world
    ≠ search index
    ≠ embedding store
    ≠ graph projection
    ≠ context packet
    ≠ model output
    ≠ agent scratchpad
    ≠ event transport
```

---

# Structural Principles

## One Shared Domain

Every vertical uses the same seven Core Entity kinds:

```text
Project
Document
Knowledge
Person
Task
Activity
Resource
```

No vertical may create an alternative root such as `ResearchDocument`, `TeachingPerson` or `AdministrativeTask` as an independent authority.

A specialized object is represented as:

```text
Core Entity
    + Derived Type
    + governed extension data
```

Examples:

```text
Knowledge + research.hypothesis + HypothesisExtension
Project   + teaching.course     + CourseExtension
Document  + research.paper      + PaperExtension
Activity  + daily_work.meeting  + MeetingExtension
```

## Aggregate-Family Storage

The canonical model does not require one universal entity table.

Each Core Entity remains an aggregate family with its own repository and explicit schema. Shared envelopes define consistency across families; they do not erase domain-specific structure.

## Explicit Schema Before Flexibility

Core state must be represented by explicit typed fields.

Extension payloads may use a schema-governed flexible representation when justified, but unrestricted property bags are forbidden as the sole representation of canonical state.

## References Instead of Embedded Aggregates

An aggregate may contain its own immutable value objects. It references other aggregate roots by identity.

Cross-aggregate links are represented through canonical relationships or owned association records, not by copying foreign aggregate state into the owner.

## Canonical Before Projection

Lexical, vector, graph, timeline and dashboard structures are projections. They are rebuildable from canonical records and may never become the only surviving representation of a fact.

## Evidence Before Authority

An inferred type, field value or relationship remains a proposal until a policy promotes it. Similarity, repetition or model confidence alone does not grant canonical authority.

---

# Canonical Type Vocabulary

This document reuses the logical types defined by `LOGICAL_DOMAIN_MODEL.md` and adds structural types required for data contracts.

| Type | Meaning |
|---|---|
| `identifier` | Stable identity of a domain or supporting record. |
| `type_id` | Namespaced identifier of a registered Derived Type or relationship type. |
| `schema_version` | Version of the contract used to interpret a record. |
| `record_version` | Monotonic version of one mutable record for optimistic concurrency. |
| `generation` | Rebuild generation of a derived projection. |
| `digest` | Content-derived integrity value. |
| `field_path` | Stable logical path to a field within a governed schema. |
| `classification` | Access and provider-handling category. |
| `authority_state` | Whether a value is proposed, canonical, rejected or superseded. |
| `reference<T>` | Identity reference to another record of type `T`. |
| `optional<T>` | A value that may be absent by contract. |

Physical encodings are not defined here.

---

# Canonical Entity Envelope

Every persisted Core Entity satisfies the following logical envelope.

```text
CanonicalEntityEnvelope
├── id
├── entity_kind
├── derived_type_id?
├── lifecycle_state?
├── authority_state
├── schema_version
├── record_version
├── classification
├── created_at
├── created_by?
├── updated_at
├── updated_by?
├── archived_at?
├── deleted_at?
└── primary_provenance_ref?
```

## Envelope Fields

| Field | Type | Rule |
|---|---|---|
| `id` | `identifier` | Stable for the lifetime of the entity. Never reused. |
| `entity_kind` | `enum(Project\|Document\|Knowledge\|Person\|Task\|Activity\|Resource)` | Exactly one Core Entity family. |
| `derived_type_id` | `optional<type_id>` | Registered specialization. Never invents a new root. |
| `lifecycle_state` | `optional<enum>` | State defined by the base entity or registered Derived Type lifecycle. |
| `authority_state` | `enum(canonical\|superseded)` | Canonical entity records are never merely inferred. Candidate entities live as proposals until promoted. |
| `schema_version` | `schema_version` | Contract version required to interpret the record. |
| `record_version` | `record_version` | Increments on every committed mutation. |
| `classification` | `classification` | Governs access, external model eligibility, retention and redaction. |
| `created_at` | `timestamp` | Recorded time of creation. |
| `created_by` | `optional<reference<Person or System Actor>>` | Human or governed system actor responsible for creation. |
| `updated_at` | `timestamp` | Recorded time of the latest committed mutation. |
| `updated_by` | `optional<reference<Person or System Actor>>` | Human or governed system actor responsible for the mutation. |
| `archived_at` | `optional<timestamp>` | Set when operationally archived without deletion. |
| `deleted_at` | `optional<timestamp>` | Set only when deletion policy marks the canonical record as deleted. |
| `primary_provenance_ref` | `optional<reference<ProvenanceRecord>>` | Primary origin when one exists; additional provenance remains append-oriented. |

`created_by` and `updated_by` are actor references, not free text. A future Actor generalization may replace the current Person-or-system representation without changing this contract.

## Envelope and Domain Payload

The envelope wraps an explicit root-specific payload:

```text
CanonicalEntityRecord<TPayload>
├── envelope
└── payload: TPayload
```

The envelope does not replace root fields such as `title`, `confidence`, `priority`, `period` or `capacity`.

---

# Core Aggregate Record Shapes

The following structures materialize the seven Core Entity families without redefining their semantics. Field meaning, optionality constraints, cardinality and lifecycle remain owned by `LOGICAL_DOMAIN_MODEL.md`.

Cross-aggregate references shown in the Logical Domain Model are represented through relationships or owned associations defined later in this document.

## Document Record

```text
DocumentRecord
├── envelope(entity_kind = Document)
└── payload
    ├── title
    ├── medium
    ├── current_content_version_ref?
    ├── metadata
    ├── registered_at
    └── domain timestamps required by the logical model
```

Rules:

- `derived_type_id` replaces an ungoverned free-form `kind` discriminator.
- authors are represented through `authored_by` relationships;
- project attachment is represented through relationships;
- content bytes and extracted representations are stored through the content model;
- version history is represented through `DocumentVersionRecord`;
- provenance remains immutable and append-oriented.

## Knowledge Record

```text
KnowledgeRecord
├── envelope(entity_kind = Knowledge)
└── payload
    ├── title
    ├── summary
    ├── body
    ├── confidence
    └── domain timestamps required by the logical model
```

Rules:

- Knowledge-to-Knowledge semantics are canonical relationship records;
- sources are represented through provenance and `derived_from` or equivalent relationships;
- validated Knowledge must resolve to retained provenance;
- `Decision`, `Hypothesis` and `Evidence` are Derived Types, not new roots.

## Project Record

```text
ProjectRecord
├── envelope(entity_kind = Project)
└── payload
    ├── title
    ├── description
    ├── objectives
    ├── deliverables
    ├── milestones
    └── period
```

Rules:

- objectives, deliverables and milestones are owned value objects;
- members are represented through `MembershipRecord`;
- funding is represented through `FundingAllocationRecord`;
- linked tasks, documents and knowledge are relationships, not embedded aggregates.

## Person Record

```text
PersonRecord
├── envelope(entity_kind = Person)
└── payload
    ├── name
    ├── affiliation
    ├── contact
    └── global_roles?
```

Rules:

- contextual roles belong to memberships or relationships;
- contact data is classified according to privacy policy;
- a Person is never duplicated merely because the same person appears in different verticals;
- organizations remain affiliation data until the Domain Model explicitly promotes Actor or Organization.

## Task Record

```text
TaskRecord
├── envelope(entity_kind = Task)
└── payload
    ├── title
    ├── description
    ├── status
    ├── priority
    ├── due_date?
    └── completed_at?
```

Rules:

- Project membership is a `belongs_to` relationship with cardinality `0..1`;
- informing Knowledge is represented through relationships;
- realized Activities are represented through relationships;
- completion time is required by the Done-state invariant.

## Activity Record

```text
ActivityRecord
├── envelope(entity_kind = Activity)
└── payload
    ├── description
    ├── occurred
    └── domain timestamps required by the logical model
```

Rules:

- the Derived Type identifies Reading, Writing, Meeting, Experiment or another registered specialization;
- performers are canonical relationships;
- consumed resources use `ResourceUseRecord`;
- produced Documents and Knowledge are relationships;
- realized Task is a relationship;
- completed historical Activities are append-oriented.

## Resource Record

```text
ResourceRecord
├── envelope(entity_kind = Resource)
└── payload
    ├── name
    ├── capacity
    ├── cost
    ├── provider
    └── availability
```

Rules:

- the Derived Type identifies API, Software, Equipment, Compute, Dataset, License, Funding or another registered specialization;
- consumption is represented through `ResourceUseRecord`;
- credentials are never stored inside the Resource record; only secret references are allowed.

---

# Value Object Representation

A value object has no domain identity and exists only inside its aggregate.

The data layer may assign an internal row or document key for persistence, ordering or migration. That key is not domain identity and must never be referenced externally as though the value object were an Entity.

## Representation Classes

### Inline Value Object

Used when the object:

- is small;
- changes with its owner;
- has no cross-aggregate reference;
- is not independently queried at scale.

Examples:

- `Objective`;
- `Deliverable`;
- `Milestone`;
- `ContactInfo`;
- simple `Metadata` fields.

### Owned Child Record

Used when the object:

- forms a collection;
- requires ordering or independent mutation tracking;
- needs queryable fields;
- remains semantically owned by one aggregate.

Conceptual envelope:

```text
OwnedValueRecord
├── owner_entity_id
├── owner_entity_kind
├── value_type
├── ordinal_or_key?
├── schema_version
├── record_version
└── typed_payload
```

### Owned Association Record

Used when a value object carries both attributes and a reference to another aggregate.

```text
OwnedAssociationRecord
├── owner_entity_id
├── association_type
├── referenced_entity_id
├── schema_version
├── record_version
└── typed_attributes
```

Canonical examples follow.

## Membership Record

```text
MembershipRecord
├── project_id
├── person_id
├── role
├── period
├── schema_version
└── record_version
```

The Project owns the Membership value. The Person remains an independent aggregate.

## Funding Allocation Record

```text
FundingAllocationRecord
├── project_id
├── resource_id
├── amount
├── period
├── schema_version
└── record_version
```

## Resource Use Record

```text
ResourceUseRecord
├── activity_id
├── resource_id
├── quantity
├── recorded_at
├── schema_version
└── record_version
```

## Document Version Record

`Version` is a value object in the domain and a durable owned record in the data model.

```text
DocumentVersionRecord
├── document_id
├── version_ordinal
├── content_version_ref
├── authored_by?
├── created_at
├── change_note?
├── content_digest
├── schema_version
└── record_version
```

Its internal persistence key is not a new domain entity.

---

# Derived Type Model

Derived Types specialize Core Entities without creating new roots.

## Derived Type Identifier

Every Derived Type uses a stable namespaced identifier:

```text
<owning_vertical>.<type_name>
```

Examples:

```text
research.hypothesis
research.evidence
research.paper
teaching.course
teaching.assessment
administration.submission
organization.server
daily_work.meeting
personal.goal
```

Identifiers are stable machine contracts. Display names may change without changing identity.

## Derived Type Definition

```text
DerivedTypeDefinition
├── type_id
├── base_entity_kind
├── owning_vertical
├── definition_version
├── status
├── extension_schema_ref?
├── lifecycle_definition_ref?
├── allowed_relationship_types
├── required_capabilities?
├── projection_hints?
├── introduced_at
└── deprecated_at?
```

| Field | Rule |
|---|---|
| `type_id` | Globally unique and namespaced. |
| `base_entity_kind` | Exactly one of the seven Core Entity kinds. |
| `owning_vertical` | One canonical owner of the type definition. |
| `definition_version` | Changes when interpretation or extension schema changes. |
| `status` | `draft`, `active`, `deprecated` or `retired`. |
| `extension_schema_ref` | Resolves to a governed extension contract. |
| `lifecycle_definition_ref` | Present only when the Derived Type specializes the base lifecycle. |
| `allowed_relationship_types` | Relationship vocabulary valid for the type. |

## Derived Type Assignment

Each entity has at most one primary Derived Type.

```text
Canonical Entity
    1 ── 0..1 Derived Type
```

Multiple classification labels must not be simulated through multiple inheritance. Cross-cutting roles, themes and uses are represented through relationships, tags or projections.

Changing an entity's Derived Type is a governed semantic migration. It is not a routine field edit.

## Derived Type Extension Record

Subtype-specific fields are held through a typed extension:

```text
DerivedTypeExtensionRecord
├── entity_id
├── type_id
├── extension_schema_version
├── record_version
├── typed_payload
├── created_at
├── updated_at
└── provenance_refs
```

Rules:

- `entity_id` must reference a Core Entity whose `derived_type_id` equals `type_id`;
- the extension schema may only add subtype-specific fields;
- it may not redefine common envelope or base payload fields;
- required fields are validated before the type becomes canonical;
- query-critical extension fields must be exposed through typed storage or governed indexes;
- free JSON without a registered schema and version is forbidden;
- deleting an extension does not delete the Core Entity unless an explicit command performs that transition.

## Extension Strategy

A physical specification may choose:

- one typed extension table per Derived Type;
- one typed extension table per vertical;
- a validated document column with registered JSON Schema;
- a hybrid approach.

The choice is valid only if all canonical rules above remain enforceable.

---

# Relationship Type Registry

Relationships are semantic contracts, not arbitrary graph labels.

## Relationship Type Definition

```text
RelationshipTypeDefinition
├── relationship_type_id
├── owning_area
├── definition_version
├── source_entity_kinds
├── source_derived_types?
├── target_entity_kinds
├── target_derived_types?
├── inverse_name
├── source_cardinality
├── target_cardinality
├── temporal_semantics
├── confidence_allowed
├── promotion_policy_ref
├── status
└── deprecated_at?
```

Examples:

```text
core.belongs_to
core.produced_by
core.informed_by
knowledge.supports
knowledge.contradicts
research.tests
research.resolves
teaching.evaluates
organization.allocated_to
```

The registry prevents different verticals from independently creating synonymous or incompatible edges.

---

# Canonical Relationship Model

A canonical relationship is a first-class authoritative record.

```text
CanonicalRelationshipRecord
├── id
├── relationship_type_id
├── source_entity_id
├── source_entity_kind
├── target_entity_id
├── target_entity_kind
├── lifecycle_state
├── schema_version
├── record_version
├── valid_from?
├── valid_to?
├── confidence?
├── created_at
├── created_by?
├── updated_at
├── updated_by?
└── provenance_refs
```

## Relationship Rules

- source and target must resolve to canonical entities;
- source and target kinds must match the registered relationship definition;
- cardinality must satisfy the Logical Domain Model;
- inverse navigation is derived from the same record, not separately written;
- one logical owner controls mutation of the relationship;
- relationship history is retained when provenance or traceability requires it;
- temporal validity is distinct from record creation time;
- confidence is permitted only for relationship types whose semantics support epistemic uncertainty;
- a graph projection may mirror the relationship but never replace it.

## Relationship Identity

A relationship may use:

- an explicit stable identifier; or
- a stable composite identity when the type contract guarantees uniqueness.

A physical specification must select one strategy per relationship type and preserve idempotency.

## Relationship Lifecycle

Canonical relationships use a bounded lifecycle such as:

```text
Active
    ↓
Deprecated / Expired / Removed
```

`Removed` does not imply silent erasure. Historical retention follows Data Architecture policy.

---

# Candidate Relationships and Promotion

An inferred relationship is not canonical data.

It is represented as an operational proposal:

```text
RelationshipCandidate
├── candidate_id
├── proposed_relationship_type_id
├── source_entity_id
├── target_entity_id
├── inference_method
├── confidence
├── evidence_refs
├── model_or_rule_version
├── created_at
├── expires_at?
└── proposal_state
```

Candidate states:

```text
Proposed
    ├── Accepted  → creates CanonicalRelationshipRecord
    ├── Rejected  → retained according to evaluation policy
    └── Expired   → never becomes canonical
```

Promotion requires the policy registered for the relationship type. Policies may allow:

- explicit human approval;
- deterministic rule validation;
- trusted-source import;
- automatic promotion above a governed evidence threshold;
- no automatic promotion.

Vector similarity, co-occurrence and model repetition are evidence signals only.

---

# Provenance Model

Provenance records why a canonical value or relationship may be trusted and where its evidence can be inspected.

## Provenance Record

```text
ProvenanceRecord
├── id
├── subject_type
├── subject_id
├── subject_field_path?
├── origin_type
├── source_entity_id?
├── source_entity_version?
├── source_content_version_ref?
├── source_segment_ref?
├── external_locator?
├── external_version?
├── method
├── capability_id?
├── cognitive_operation_id?
├── actor_id?
├── model_ref?
├── tool_ref?
├── instruction_version?
├── transformation_version?
├── observed_at?
├── effective_from?
├── effective_to?
├── confidence?
├── validation_state
├── parent_provenance_refs
└── recorded_at
```

## Provenance Subject

A provenance record may attach to:

- an entity;
- one entity field;
- a canonical relationship;
- one relationship field;
- a document version;
- a document segment;
- a promoted AI result.

Field-level provenance uses a stable logical `subject_field_path`, not a database column name.

## Provenance Origin Types

```text
UserAction
ImportedSource
DocumentContent
DocumentSegment
Activity
ExternalObservation
DeterministicRule
AIExecution
Migration
Reconciliation
```

## Provenance Rules

- provenance is append-oriented;
- source content and source version must remain resolvable or explicitly marked unavailable;
- a model-generated value records provider/model compatibility identity, instruction version and operation identifier;
- human acceptance is a provenance step, not an overwrite of machine origin;
- confidence never substitutes for source evidence;
- provenance chains may reference earlier provenance but must remain acyclic;
- redaction may hide sensitive values while retaining the existence and authority trail required by policy.

---

# Temporal Model

ResearchOS distinguishes several clocks.

| Time | Meaning |
|---|---|
| `recorded_at` | When ResearchOS persisted the record. |
| `effective_from / effective_to` | When a fact or relationship applies in the modeled world. |
| `observed_at` | When an external fact was observed. |
| `event_time` | When a committed domain transition occurred. |
| `processing_time` | When a worker or runtime processed data. |
| `content_created_at` | When source content was originally created, when known. |

Specifications must not collapse these times when doing so would destroy meaning.

Bitemporal representation is required only for records whose world validity and system history both matter. It is not mandatory for every field.

---

# Document Content Model

The Document aggregate describes the retained artifact. Content records preserve its bytes and machine-operable representations without converting those representations into Knowledge.

## Content Topology

```text
Document
    1 ── N DocumentVersionRecord

DocumentVersionRecord
    1 ── 1 ContentObjectVersion
    1 ── N ContentRepresentation
    1 ── N DocumentSegment

DocumentSegment
    1 ── N EmbeddingProjection
    0 ── N ProvenanceRecord
```

## Content Object

```text
ContentObject
├── content_object_id
├── logical_locator
├── classification
├── created_at
└── retention_policy_ref
```

The locator is logical. It does not expose a filesystem path or vendor-specific object key as domain meaning.

## Content Object Version

```text
ContentObjectVersion
├── content_version_id
├── content_object_id
├── content_digest
├── media_type
├── byte_size
├── encryption_metadata_ref?
├── storage_generation
├── created_at
└── source_provenance_ref
```

Content bytes are immutable within one version.

## Content Representation

A source may have multiple derived representations:

```text
ContentRepresentation
├── representation_id
├── content_version_id
├── representation_kind
├── language?
├── representation_locator
├── representation_digest
├── extraction_method
├── extraction_version
├── quality_metrics?
├── created_at
└── provenance_ref
```

Allowed representation kinds may include:

```text
raw
extracted_text
ocr_text
normalized_text
structured_sections
thumbnail
transcript
```

A derived representation remains non-authoritative relative to retained source content.

## Document Segment

```text
DocumentSegment
├── segment_id
├── document_id
├── document_version_ordinal
├── content_version_id
├── source_representation_id
├── ordinal
├── parent_segment_id?
├── section_path?
├── page_or_region?
├── start_offset?
├── end_offset?
├── normalized_text
├── text_digest
├── segmentation_policy_id
├── segmentation_policy_version
├── language?
├── classification
└── created_at
```

Rules:

- a segment always belongs to one immutable content version;
- offsets refer to the declared source representation;
- segment text must be verifiable through `text_digest`;
- segmentation policy and version are mandatory;
- re-segmentation creates a new segment generation rather than silently changing existing segment identity;
- segments are evidence anchors, not Core Entities;
- deleting or restricting source content invalidates dependent retrieval projections.

---

# Knowledge Evidence Anchoring

A Knowledge record may cite precise source evidence without embedding document text into the Knowledge aggregate.

The evidence path is:

```text
Knowledge
    ── canonical relationship / provenance ──▶ Document
                                                │
                                                ▼
                                        DocumentVersion
                                                │
                                                ▼
                                         DocumentSegment
```

A citation or evidence anchor must resolve at least:

```text
EvidenceAnchor
├── document_id
├── document_version_ordinal
├── segment_id?
├── source_span?
├── content_digest
└── provenance_ref
```

A document-level source is valid when finer granularity is unavailable. Claims presented as precise evidence should use segment- or span-level anchors.

---

# Projection Source Contract

Every projection must identify the exact canonical source from which it can be rebuilt.

## Projection Envelope

```text
ProjectionRecordEnvelope
├── projection_name
├── projection_schema_version
├── generation
├── source_record_id
├── source_record_version
├── transformation_version
├── built_at
├── checkpoint?
└── classification
```

Projection records do not use `record_version` as canonical optimistic concurrency. Their replaceability is expressed through generation and source version.

---

# Lexical Projection

Lexical search projections may contain:

- normalized text;
- language-specific tokens;
- weighted fields;
- exact identifiers and metadata;
- full-text ranking data.

Every lexical record must resolve to the source entity, content version or segment and its version.

Lexical indexes may be rebuilt or replaced without canonical migration.

---

# Vector Projection

Embeddings are retrieval projections, not Knowledge and not canonical memory.

## Embedding Projection Record

```text
EmbeddingProjection
├── embedding_id
├── source_kind
├── source_entity_id
├── source_record_version
├── source_content_version_id?
├── source_segment_id?
├── source_text_digest
├── embedding_model_id
├── embedding_compatibility_class
├── embedding_dimension
├── preprocessing_policy_id
├── preprocessing_policy_version
├── vector_generation
├── vector
├── classification
└── created_at
```

## Embedding Rules

- the vector always resolves to the exact text digest it represents;
- source text is retained outside the vector record;
- one document may have document-, section- and segment-level embeddings when use cases justify them;
- segment-level embeddings are the default evidence retrieval unit;
- changing model, dimensions or preprocessing creates a new generation;
- incompatible generations are never compared without an explicit compatibility adapter;
- deleting, redacting or reclassifying the source invalidates or removes dependent embeddings;
- embeddings may suggest candidate relationships but never create canonical semantics directly;
- secrets and ineligible restricted data are never embedded.

---

# Graph Projection

The graph projection is derived from canonical entities and canonical relationships.

```text
Graph Projection
├── canonical nodes      ← Core Entity records
├── canonical edges      ← CanonicalRelationshipRecord
├── candidate edges      ← RelationshipCandidate
└── analytical outputs   ← communities, clusters, summaries
```

## Graph Node Contract

```text
GraphNodeProjection
├── entity_id
├── entity_kind
├── derived_type_id?
├── source_record_version
├── selected display properties
├── generation
└── classification
```

## Graph Edge Contract

```text
GraphEdgeProjection
├── relationship_id_or_candidate_id
├── relationship_type_id
├── source_entity_id
├── target_entity_id
├── edge_authority
├── source_record_version
├── confidence?
├── generation
└── provenance_summary_ref?
```

`edge_authority` distinguishes:

```text
canonical
candidate
analytical
```

## Graph Rules

- the graph does not own node or edge truth;
- canonical and inferred edges must remain visibly distinct;
- analytical edges such as similarity, community membership or centrality are projection-only;
- a graph rebuild must reproduce all canonical nodes and edges from authoritative records;
- graph-specific identifiers never escape as canonical entity identifiers;
- graph traversal results carry source versions sufficient for staleness detection.

---

# Candidate Entity Model

An AI extraction may propose a new entity before that entity is canonical.

```text
EntityCandidate
├── candidate_id
├── proposed_entity_kind
├── proposed_derived_type_id?
├── proposed_payload
├── extension_payload?
├── evidence_refs
├── extraction_method
├── model_or_rule_version
├── confidence
├── created_at
├── expires_at?
└── proposal_state
```

Promotion performs a normal command and domain validation:

```text
EntityCandidate
    ↓ accepted by policy
Create Canonical Entity Command
    ↓ validates domain and extension invariants
Canonical Entity Record
    ↓ committed
Domain Event
```

No candidate identifier becomes the canonical entity identifier by assumption. The creation command decides identity according to idempotency and deduplication rules.

---

# Schema Registry

ResearchOS maintains a governed registry for every interpreted data contract.

```text
SchemaDefinition
├── schema_id
├── schema_kind
├── schema_version
├── owner
├── status
├── compatibility_mode
├── definition_ref
├── introduced_at
├── deprecated_at?
└── migration_ref?
```

Schema kinds include:

```text
core_entity_payload
derived_type_extension
value_object
relationship_type
command
query
event
projection
context_manifest
model_structured_output
```

## Compatibility Modes

```text
backward_compatible
forward_compatible
fully_compatible
breaking
```

Compatibility declarations must be tested, not assumed.

---

# Version Semantics

ResearchOS uses several independent version dimensions.

| Version | Meaning |
|---|---|
| `record_version` | Mutation version of one canonical or operational record. |
| `schema_version` | Contract used to interpret a record. |
| `definition_version` | Version of a Derived Type or relationship definition. |
| `content_version` | Immutable version of retained document content. |
| `projection_generation` | Rebuild generation of a derived projection. |
| `model_version` | Model identity or compatibility class used by an AI operation. |
| `instruction_version` | Version of prompts, policies or structured-output instructions. |
| `transformation_version` | Version of parsing, normalization, segmentation or enrichment logic. |

These versions must never be collapsed into one generic `version` field.

## Identity Rule

A record may change many times while retaining one identity.

```text
same modeled thing + changed state
    → same id, higher record_version

different modeled thing
    → new id

same Document + new content revision
    → same Document id, new DocumentVersionRecord
```

---

# Concurrency Contract

Every mutable canonical record supports optimistic concurrency.

A mutation command declares the expected `record_version`. The commit succeeds only if the stored version still matches.

```text
read version 7
    ↓
command expects version 7
    ↓
commit writes version 8
```

Conflicts do not produce silent last-write-wins behavior.

Cross-aggregate operations follow the Software Architecture:

- one aggregate changes atomically;
- committed events communicate facts;
- process managers or reactions coordinate additional aggregates;
- no distributed transaction is implied by the shared data model.

---

# Data Classification Contract

Every canonical, content, operational and projection record carries or inherits a classification.

Baseline classifications are:

```text
public
internal
confidential
restricted_personal
secret_reference
```

Rules:

- `secret_reference` stores only a reference to a secret mechanism, never the secret value;
- a derived record inherits the most restrictive applicable source classification unless a policy explicitly transforms it;
- a Context Builder filters by access and provider eligibility before relevance;
- embeddings and graph projections preserve classification;
- exports and logs may redact values but must not silently lower classification;
- vertical specifications may refine handling policy but may not weaken the baseline.

---

# Archive, Delete, Redact and Expire

These operations are distinct.

| Operation | Meaning |
|---|---|
| `Archive` | Retain canonical data but remove it from active operational views. |
| `Soft Delete` | Mark canonical data deleted while retaining it according to policy. |
| `Hard Delete` | Irreversibly remove eligible data after policy and dependency checks. |
| `Redact` | Remove or mask sensitive values while preserving permitted structural/audit information. |
| `Expire` | End temporal validity or retention eligibility. |
| `Invalidate Projection` | Mark derived records stale or remove them without changing canonical truth. |

## Deletion Propagation

Deleting or redacting a source triggers governed propagation:

```text
Source canonical/content record
    ↓
validate legal and domain retention obligations
    ↓
remove or redact eligible source values
    ↓
invalidate dependent segments
    ↓
remove dependent embeddings and search records
    ↓
rebuild graph/read projections
    ↓
review derived Knowledge and relationships
    ↓
retain only permitted audit/provenance traces
```

Derived Knowledge is not automatically deleted merely because one source disappears. It must be re-evaluated according to its remaining provenance and domain policy.

---

# Vertical Extension Contract

Every vertical specification extends this model rather than replacing it.

## Required Declaration

A vertical data specification must declare:

```text
Vertical
├── Core Entity kinds used
├── Derived Types introduced
├── extension schemas
├── value objects introduced
├── relationship types introduced
├── specialized lifecycles
├── vertical invariants
├── commands and events affected
├── projections required
├── migration strategy
└── conformance tests
```

## Vertical Spec Template

```markdown
# <Vertical> Data Extension

## Extends
- `CANONICAL_DATA_MODEL.md`
- `LOGICAL_DOMAIN_MODEL.md`
- `<VERTICAL>.md`

## Core Entity Kinds Used
- ...

## Derived Types
| Type ID | Base Entity | Extension Schema | Lifecycle |
|---|---|---|---|

## Extension Schemas
...

## Relationship Types
...

## Invariants
...

## Commands and Events
...

## Projections
...

## Migrations
...

## Conformance Tests
...
```

## Vertical Restrictions

A vertical specification must not:

- introduce an eighth Core Entity without a governed Domain Model revision;
- redefine a common envelope field;
- redefine a base entity field with a different meaning;
- create a private Person, Document, Project, Task, Activity, Knowledge or Resource authority;
- create an independent graph or memory model;
- write directly into another vertical's owned extension;
- store unversioned free-form subtype data;
- promote inferred data without the registered policy;
- choose a database product or deployment topology unless the specification is explicitly technical.

---

# Cross-Vertical Data Rules

## Shared Identity

The same real entity retains one canonical identity across all verticals.

```text
one Person
    ├── supervises Doctoral Thesis
    ├── teaches Course
    ├── collaborates_on Research Project
    └── participates_in Meeting
```

Vertical membership is expressed through Derived Types and relationships, not duplicate entities.

## Shared Knowledge

Knowledge may be created in one vertical and reused in another without copying it.

```text
Research Evidence
    ── informs ──▶ Teaching Material
```

Reuse adds relationships and provenance. It does not clone Knowledge into a vertical-specific store.

## Shared Documents

A Document keeps one identity even when used in several contexts.

A Paper referenced by a thesis chapter and a lecture remains one Paper Document connected to several Projects, Activities and Knowledge records.

## Shared Resources

A Resource may be allocated across multiple projects and verticals. Capacity and usage are reconciled globally.

---

# API and Language Projection Rules

API schemas, Python models and TypeScript types are generated or manually derived views of this document.

They must preserve:

- canonical identifiers;
- entity and Derived Type discrimination;
- schema and record versions;
- classification;
- relationship semantics;
- provenance references;
- optionality and cardinality;
- extension schema boundaries.

They may add transport-only fields such as links, pagination cursors or presentation labels, but those fields do not become canonical data.

Transport payloads must not expose:

- storage-specific keys as domain identity;
- secret values;
- internal projection identifiers;
- unfiltered restricted provenance;
- arbitrary extension fields not allowed by the registered schema.

---

# Physical Mapping Rules

A physical data specification may map this model to relational, document, graph or mixed storage.

Whatever mapping is selected, it must demonstrate:

- one authoritative representation for each canonical field;
- explicit aggregate repository boundaries;
- enforceable schema versions;
- enforceable Derived Type compatibility;
- relationship cardinality and ownership;
- optimistic concurrency;
- provenance resolvability;
- content and segment integrity;
- projection rebuildability;
- deletion propagation;
- migration safety.

The following are implementation choices, not canonical requirements:

- table names;
- primary-key encoding;
- UUID version;
- JSON versus extension tables;
- foreign-key implementation;
- index types;
- graph database products;
- vector extensions;
- ORM inheritance strategy.

---

# Minimal Canonical Schema Set

Before any vertical feature is implemented, the physical baseline must be able to represent at least:

```text
Seven Core Entity record families
DerivedTypeDefinition
DerivedTypeExtensionRecord
RelationshipTypeDefinition
CanonicalRelationshipRecord
RelationshipCandidate
EntityCandidate
ProvenanceRecord
DocumentVersionRecord
ContentObject
ContentObjectVersion
ContentRepresentation
DocumentSegment
SchemaDefinition
ProjectionRecordEnvelope
EmbeddingProjection
Graph node/edge projection contracts
```

Operational records such as Commands, Proposals, Approvals, Jobs, Cognitive Operations, Events, Outbox and Inbox remain defined by Software and Data Architecture and are not canonical domain records.

---

# Conformance Rules

An implementation or specification conforms to this model only if all applicable statements are true.

## Entity Conformance

- every canonical entity belongs to exactly one Core Entity family;
- every Derived Type is registered and compatible with its base family;
- every record carries schema and record versions;
- base fields are explicit and typed;
- extension data is validated against a registered versioned schema;
- identity is stable and never reused;
- cross-aggregate state is referenced, not copied as authority.

## Relationship Conformance

- every relationship uses a registered semantic type;
- cardinality and endpoint kinds are validated;
- inverse navigation derives from one authoritative record;
- candidate and canonical relationships remain distinct;
- provenance resolves to evidence where semantics require it.

## Content Conformance

- document content versions are immutable;
- every segment resolves to one content version and segmentation policy;
- every embedding resolves to exact source text and generation metadata;
- deleting or reclassifying content invalidates dependent projections.

## Projection Conformance

- every projection identifies canonical source and source version;
- every projection has a rebuild procedure;
- inferred graph edges remain visibly non-canonical;
- projection identifiers never replace canonical identifiers.

## Vertical Conformance

- the vertical introduces only Derived Types, extensions, relationships, rules and projections;
- no duplicate Core Entity authority exists;
- no vertical bypasses common provenance, versioning, classification or deletion rules;
- vertical migrations declare compatibility and rollback or recovery strategy.

---

# Validation Strategy

Conformance is validated through:

- schema tests;
- generated type compatibility tests;
- aggregate invariant tests;
- relationship cardinality tests;
- subtype registration tests;
- migration tests;
- provenance resolution tests;
- content digest tests;
- projection rebuild tests;
- deletion and redaction propagation tests;
- cross-vertical identity tests;
- optimistic concurrency tests.

A vertical is not implementation-ready until its extension passes the shared conformance suite.

---

# Evolution Rules

## Compatible Evolution

Normally compatible:

- adding an optional base or extension field;
- adding a Derived Type;
- adding a relationship type;
- adding a projection;
- expanding an enum when all consumers declare support;
- adding a new provenance method;
- introducing a new projection generation.

## Governed Breaking Evolution

Requires migration and impact review:

- changing field meaning;
- changing requiredness;
- changing the base Core Entity of a Derived Type;
- changing relationship cardinality;
- changing canonical identity rules;
- splitting or merging Derived Types;
- changing provenance requirements;
- lowering data classification;
- changing deletion semantics.

## Domain Revision

Introducing a new Core Entity, changing the meaning of an existing root or moving a concept across aggregate boundaries requires:

1. evidence that the current seven-root baseline cannot represent the domain cleanly;
2. an ADR;
3. revision of `DOMAIN_MODEL.md` and `LOGICAL_DOMAIN_MODEL.md`;
4. revision of this document;
5. explicit migration and compatibility plans.

No vertical specification may perform such a change locally.

---

# What This Model Does Not Define

This document intentionally does not define:

- SQL DDL;
- table or collection names;
- ORM classes;
- indexes and query plans;
- API routes;
- event transport;
- message broker topics;
- file-storage products;
- embedding providers;
- graph database products;
- prompt formats;
- UI forms;
- complete vertical schemas;
- implementation milestones.

Those belong to downstream specifications.

---

# Completion Criteria

This document is complete when:

- every Core Entity can be represented through one common envelope and explicit root payload;
- Derived Types can extend roots without redefining them;
- vertical specifications have one enforceable extension mechanism;
- canonical and candidate entities and relationships are structurally distinct;
- provenance can reach exact source evidence;
- document content, versions, representations and segments are connected;
- embeddings and graph structures are unambiguously projections;
- version, concurrency, classification and deletion semantics are shared;
- physical schemas and language types can be derived without inventing domain meaning.

---

# Final Canonical Statement

```text
One shared domain
    ↓
Seven canonical aggregate families
    ↓
Governed Derived Types and typed extensions
    ↓
Canonical semantic relationships
    ↓
Traceable provenance and retained content
    ↓
Rebuildable lexical, vector and graph projections
    ↓
Vertical specifications that specialize without fragmenting truth
```

> **ResearchOS has one canonical data model. Verticals extend its vocabulary and behavior; they do not create separate worlds.**
