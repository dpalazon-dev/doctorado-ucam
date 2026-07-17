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
| ADR-0006 | Experimental Technical Baseline Before Stabilized Technical Architecture | Accepted | 2026-07-08 | `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `TECHNICAL_ARCHITECTURE.md` |
| ADR-0007 | ResearchOS Is a Local-First Single-User Desktop Application | Accepted | 2026-07-08 | `TECHNICAL_ARCHITECTURE.md`, `SYSTEM_ARCHITECTURE.md` |
| ADR-0008 | Tauri and React Form the Cross-Platform Desktop Shell | Accepted | 2026-07-08 | `TECHNICAL_ARCHITECTURE.md` |
| ADR-0009 | Rust Owns Authority; Python Provides Cognitive Capabilities | Accepted | 2026-07-08 | `TECHNICAL_ARCHITECTURE.md`, `AGENT_RUNTIME.md`, `COMPONENT_MODEL.md` |
| ADR-0010 | Embedded Local Persistence Is the Initial Product Baseline | Accepted | 2026-07-08 | `TECHNICAL_ARCHITECTURE.md`, `DATA_ARCHITECTURE.md`, `CANONICAL_DATA_MODEL.md` |
| ADR-0011 | The Canonical Data Model Governs Vertical Extensions | Accepted | 2026-07-08 | `CANONICAL_DATA_MODEL.md`, `LOGICAL_DOMAIN_MODEL.md`, `DOMAIN_VERTICALS.md` |
| ADR-0012 | OKF as a Candidate Knowledge Projection and Interchange Format | Accepted | 2026-07-17 | `DATA_ARCHITECTURE.md`, `CANONICAL_DATA_MODEL.md`, `AI_ARCHITECTURE.md` |

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
- `TECHNICAL_ARCHITECTURE.md` — current concrete selections, process topology and maturity status.

---

# ADR-0007 · ResearchOS Is a Local-First Single-User Desktop Application

**Status:** Accepted  
**Date:** 2026-07-08

## Context

ResearchOS is intended to behave as personal software installed on the user's own computer. The previous provisional baseline still assumed a VPS-oriented web deployment and therefore introduced server administration, network exposure and infrastructure that a single user should not need to operate.

The product must remain capable of using optional remote AI providers, but those providers are capabilities consumed by the application rather than the location or authority of the application itself.

## Decision

ResearchOS is initially a local-first, single-user desktop application distributed through native installers for Windows, macOS and Linux.

- One local application instance owns one primary local workspace.
- Canonical state, source content, operational state and rebuildable projections are stored locally.
- The user is not required to install or administer a database server, container runtime, reverse proxy, message broker or background daemon.
- External AI, search or integration services are optional adapters invoked only by capabilities that require them.
- The initial product does not provide multi-user collaboration, server-hosted workspaces, cross-device synchronization or remote administration.
- Any later synchronization, hosted mode or multi-user authority model requires a new ADR because it changes trust, identity, consistency and conflict-resolution boundaries.

## Rationale

The local-first topology matches the actual product, minimizes operational burden, preserves privacy and gives the project the shortest path to an installable proving slice. It also keeps remote dependencies explicit rather than making the entire system depend on a privately operated server.

## Consequences

### Positive

- Installation and operation resemble ordinary desktop software.
- Canonical data remains under the user's direct control.
- Offline operation is possible for all capabilities that do not require remote providers.
- Deployment, backup and recovery are bounded to one local product.
- Infrastructure decisions are evaluated against a real single-user workload.

### Negative

- Native packaging, signing and update flows must be implemented per platform.
- Cross-device access and collaboration are deferred.
- Local resource limits constrain heavy model execution and document processing.
- Data durability depends on explicit local backup and restoration mechanisms.

## Canonical specification

- `TECHNICAL_ARCHITECTURE.md` — desktop target, process topology, packaging and local storage baseline.
- `SYSTEM_ARCHITECTURE.md` — authority and trust boundaries.
- `ROADMAP.md` — delivery sequence for the desktop product.

---

# ADR-0008 · Tauri and React Form the Cross-Platform Desktop Shell

**Status:** Accepted  
**Date:** 2026-07-08

## Context

ResearchOS requires a rich Workspace with document views, editors, dashboards, timelines, graph exploration, approval surfaces and conversational interaction. It must also expose native desktop functions such as file selection, drag-and-drop, notifications, credential storage, application lifecycle, installers and updates.

A browser-hosted application would retain server assumptions. Electron would bundle an additional Chromium and Node runtime. A Python-native UI would reduce language count but would make the planned Workspace and its visual ecosystem more expensive to develop.

## Decision

The cross-platform desktop shell uses:

```text
Tauri 2
+
React
+
TypeScript
+
Vite
```

- React and TypeScript own Workspace presentation and interaction state.
- Tauri owns native application lifecycle, windows, menus, system integration, installer hooks and supervision of subordinate processes.
- The application is built and packaged separately on Windows, macOS and Linux from one shared source tree.
- Node.js is a development and build-time dependency, not a production server.
- Platform-specific behavior remains behind Tauri commands or adapters rather than leaking into the Workspace.

## Rationale

This combination preserves the mature web UI ecosystem while providing a small native host, cross-platform packaging and a natural Rust boundary for the authoritative runtime.

## Consequences

### Positive

- One Workspace codebase across the three target desktop platforms.
- Access to modern editors, visualization libraries and frontend tooling.
- Native packaging without a permanent local web server.
- Clear separation between presentation and authority.

### Negative

- The product uses both TypeScript and Rust.
- WebView differences require platform testing.
- Code signing, notarization and packaging remain platform-specific.
- IPC contracts between the Workspace and the Rust host must be versioned and tested.

## Alternatives considered

- **Electron:** rejected as the initial baseline because its bundled runtime and process footprint are unnecessary for the current product.
- **PySide6 / Qt:** retained as a credible fallback, but not selected because the Workspace is expected to benefit substantially from the web UI ecosystem.
- **Browser plus local server:** rejected because it weakens the desktop lifecycle and preserves server-shaped operational assumptions.

## Canonical specification

- `TECHNICAL_ARCHITECTURE.md` — desktop stack, repository layout, packaging and interface boundaries.

---

# ADR-0009 · Rust Owns Authority; Python Provides Cognitive Capabilities

**Status:** Accepted  
**Date:** 2026-07-08

## Context

ResearchOS needs both a strict local runtime and access to the strongest ecosystem for models, embeddings, document intelligence, evaluation and scientific processing. Implementing everything in Python would place canonical persistence, authorization, durable execution and desktop lifecycle in a highly dynamic runtime. Implementing everything in Rust would make cognitive experimentation and integration unnecessarily slow.

A language boundary is acceptable only if authority and data ownership remain unambiguous.

## Decision

ResearchOS uses a controlled two-runtime architecture:

```text
React + TypeScript Workspace
        ↓
Tauri + authoritative Rust runtime
        ↓ controlled, versioned IPC
subordinate Python cognitive sidecar
```

Rust owns:

- canonical state and persistence;
- commands, queries and application use cases;
- domain invariant enforcement;
- authorization and approval gates;
- durable jobs, events, retries, cancellation and checkpoints;
- filesystem authority, secrets and process supervision;
- validation and commitment of cognitive Proposals.

Python owns bounded cognitive capabilities such as:

- model-provider adapters;
- document processing and OCR coordination;
- embeddings and reranking;
- extraction, classification, synthesis and critique;
- retrieval experiments and evaluation;
- scientific and machine-learning libraries.

The following constraints are mandatory:

- Python does not write directly to canonical persistence.
- The sidecar receives typed operations with explicit authority, context and budgets.
- The sidecar returns typed results, Artifacts, observations and Proposals.
- Rust validates every result before any canonical mutation.
- The LLM never controls scheduling, permissions, budget enforcement or commit authority.
- Cognitive frameworks may be used behind adapters but cannot become a hidden state owner or second orchestration authority.
- Additional languages enter only through a capability adapter justified by a concrete dependency or measured requirement.

## Rationale

Rust provides a strict native core for authority, security and durable execution. Python preserves access to the rapidly evolving AI and scientific ecosystem. The sidecar boundary contains failure and allows the cognitive runtime to evolve without transferring ownership of the system.

## Consequences

### Positive

- Canonical authority is independent of model and Python framework behavior.
- Cognitive capabilities remain fast to prototype and replace.
- Python failures can be isolated, cancelled and restarted.
- The IPC boundary makes budgets, provenance and capability contracts explicit.
- Native or JVM tools can later enter as specialized adapters without changing the core.

### Negative

- Two runtime toolchains must be built, packaged and tested per platform.
- IPC schemas, compatibility and process supervision become first-class engineering work.
- Large payloads require reference-based exchange rather than naive serialization.
- End-to-end debugging spans process and language boundaries.

## Canonical specification

- `TECHNICAL_ARCHITECTURE.md` — language ownership, process topology and IPC baseline.
- `AGENT_RUNTIME.md` — bounded cognitive operation lifecycle and authority limits.
- `COMPONENT_MODEL.md` — component responsibility and runtime placement.

---

# ADR-0010 · Embedded Local Persistence Is the Initial Product Baseline

**Status:** Accepted  
**Date:** 2026-07-08

## Context

A single-user desktop application does not justify a separately installed database server, broker, graph server or vector service. The system still requires transactions, migrations, full-text retrieval, durable jobs, event delivery, relationship traversal, backup and rebuildable projections.

The storage baseline must therefore be embedded, locally operable and replaceable where specialized workload evidence later requires extraction.

## Decision

The initial physical data baseline is:

```text
SQLite
+
content-addressed local filesystem
+
embedded, rebuildable projections
```

- SQLite stores canonical records, relationships, provenance, operational state, durable jobs, events, audit data, configuration metadata and relational projections.
- SQLite FTS5 provides the initial lexical-search projection.
- Canonical graph semantics are represented by first-class relationship records; bounded traversal initially uses relational queries and recursive CTEs.
- Original and derived content objects are stored in a content-addressed local filesystem under application-managed paths.
- The initial vector projection may use vectors stored locally with Python/NumPy retrieval for proving work. A dedicated embedded vector adapter may be introduced only after measured volume or latency justifies it.
- One authoritative write coordinator, short transactions, WAL mode and optimistic record versions govern local concurrency.
- Backups use application-controlled consistent database copies plus content and manifest capture.
- Search, vector, graph and UI projections remain rebuildable and never become canonical authority.

The following are not initial product prerequisites:

- PostgreSQL or `pgvector`;
- Redis;
- Neo4j or another graph server;
- an external vector database;
- Kafka, RabbitMQ or another broker;
- Docker or a separately managed storage service.

Replacement or extraction requires measured evidence such as:

- concurrency that the single-writer model cannot satisfy;
- vector volume or latency outside accepted targets;
- relationship traversal that cannot be maintained reasonably in SQLite;
- cross-device synchronization;
- multi-user authority;
- independently deployed or scaled execution.

## Rationale

Embedded persistence satisfies the actual product topology while preserving the logical separation among canonical, operational, event, audit and projection data. It removes installation burden without erasing future extraction boundaries.

## Consequences

### Positive

- No database or broker administration for the user.
- Atomic local transactions and simple backup ownership.
- Small deployment surface and fewer failure modes.
- The complete proving slice can run inside the installed product.

### Negative

- Write concurrency must remain deliberately bounded.
- Specialized graph and vector performance may eventually require adapters.
- Large local collections require disk-space and rebuild management.
- Backup, migration and corruption recovery become product responsibilities.

## Canonical specification

- `TECHNICAL_ARCHITECTURE.md` — physical storage, local paths, concurrency and packaging.
- `DATA_ARCHITECTURE.md` — logical data classes, ownership, lifecycle and projection rules.
- `CANONICAL_DATA_MODEL.md` — shared structural contracts independent of physical storage.

---

# ADR-0011 · The Canonical Data Model Governs Vertical Extensions

**Status:** Accepted  
**Date:** 2026-07-08

## Context

ResearchOS spans Personal, Daily Work, Administration, Teaching, Research and Organization. Each vertical needs specialized types, fields, relationships, invariants, lifecycles and projections. Allowing every vertical to invent its own root records, identity, provenance, relationship representation or persistence authority would fragment the system into disconnected applications.

The Logical Domain Model defines meaning, but implementation specifications also require one shared structural contract before physical schemas and vertical extensions are designed.

## Decision

`CANONICAL_DATA_MODEL.md` governs the structural form that every vertical and persistence specification must extend.

- The seven Core Entities remain the only routine root entity kinds.
- A vertical introduces Derived Types, typed extension profiles, vertical relationships, invariants, state machines and projections.
- A vertical does not redefine root identity, entity envelopes, record versioning, provenance anchors, content-version contracts or canonical relationship structure.
- Derived Types are registered with a namespaced owner, base entity kind, schema version and extension contract.
- Vertical extension data must be schema-governed and versioned; unowned free-form JSON is not a valid substitute for a contract.
- All verticals share the same canonical state, relationship graph, event mechanisms, retrieval infrastructure and cognitive runtime.
- A vertical may own a projection or workflow but not a parallel source of truth.
- A new Core Entity still requires the evidence and ADR process defined by ADR-0004.

## Rationale

One canonical structural model prevents vertical drift while allowing domain-specific richness. It also gives database, API, UI, agent and migration specifications a stable contract to target.

## Consequences

### Positive

- Cross-vertical relationships retain one identity and provenance model.
- Shared infrastructure can validate and operate on all vertical data.
- Vertical Specs become smaller and focus on genuine specialization.
- Graph, search and cognitive contexts remain connected across the user's work.

### Negative

- Vertical designers must work through the extension mechanism rather than choosing arbitrary schemas.
- Changes to the canonical contract have broad downstream impact.
- Some specialized fields require explicit extension tables or versioned schemas.

## Canonical specification

- `CANONICAL_DATA_MODEL.md` — entity envelopes, relationships, provenance, content segmentation and extension mechanism.
- `LOGICAL_DOMAIN_MODEL.md` — semantic meaning, aggregates and invariants.
- `DOMAIN_VERTICALS.md` — vertical ownership and specialization boundaries.
- Future vertical Specs — concrete Derived Types, profiles, relationships, lifecycles and projections.

---

# ADR-0012 · OKF as a Candidate Knowledge Projection and Interchange Format

**Status:** Accepted  
**Date:** 2026-07-17

## Context

The Knowledge graph is an internal mechanism: `INTERACTION_MODEL.md` states the researcher never interacts with it directly. It is stored as canonical records plus typed relationships, with the graph, lexical and vector views held as rebuildable projections (`DATA_ARCHITECTURE.md`, `CANONICAL_DATA_MODEL.md`). There is today no human-readable, navigable, portable rendering of what the system knows.

Two external developments describe exactly such a rendering. The "LLM-wiki" pattern compiles sources into an interlinked Markdown knowledge base that is maintained over time rather than re-derived on every query. The Open Knowledge Format (OKF) is an external open specification that packages such a base as a directory of Markdown files with YAML frontmatter — portable, model-agnostic and readable by ordinary tools.

`AI_ARCHITECTURE.md` already anticipates this shape and pre-classifies it: no document folder, index, vector store or generated wiki is ever the source of truth; any such structure is a projection of the domain, reconstructible from it, never authoritative over it. `SPEC_CATALOG.md` and `TECHNICAL_ARCHITECTURE.md` already establish how to evaluate an external format as a deferred, measured candidate without a mandatory dependency: DocLang, for Document representations. OKF is the analogous candidate one layer up — for the Knowledge projection rather than the Document representation.

A naive adoption of the pattern would let a model edit the Markdown wiki as the source of truth. That directly violates ADR-0001 and ADR-0009. This decision adopts the useful half and forecloses the harmful half.

## Decision

OKF is adopted as a **deferred candidate** for a human-readable Knowledge projection and interchange format, evaluated within `SPEC-011` (Retrieval and Projection System) on the same terms the DocLang candidate is evaluated within SPEC-010.

- The OKF bundle is a **rebuildable, non-authoritative projection**. Canonical Knowledge (ADR-0001, ADR-0010) remains the single source of truth; the bundle is derived from committed Knowledge and reconstructible from it.
- It is **generated from** committed Knowledge, never **edited into** it. Consistent with ADR-0009 and the human-gated Improvement Loop, any agent-originated change to Knowledge flows through a typed Proposal that Rust validates and the researcher accepts. The cognitive sidecar never writes the projection as authority.
- OKF introduces **no technology-specific canonical `representation_kind`**. A retained projection is described semantically and carries explicit `format`, `format_version`, `generator` and `generator_version` metadata, mirroring the DocLang rule in `TECHNICAL_ARCHITECTURE.md`.
- **Bidirectional editing** — reconciling human edits of OKF files back into canonical Knowledge — is **out of scope** and requires its own future ADR, because it changes the mutation and authority boundary owned by ADR-0001 and ADR-0009.
- Adoption is **evidence-gated**. It requires measured benefit — human legibility, cheaper agent context assembly, or portability — over the baseline of the existing graph and retrieval projections, sufficient to justify the runtime, packaging and maintenance cost of a second Knowledge rendering. Absent that evidence it remains deferred.

This decision changes no upstream invariant. It reinforces ADR-0001 and ADR-0009 and gives the generated-wiki-as-projection principle of `AI_ARCHITECTURE.md` a concrete, named, deferred realization.

## Rationale

The Knowledge graph is authoritative and machine-oriented but not legible; a wiki projection is legible and portable but must not be authoritative. Separating the two lets the system gain a human- and agent-facing surface without a second source of truth. The pattern's documented failure modes — drift, stale contradictions, lossy compression — are precisely what the human gate, supersession-not-overwrite and retained source evidence already prevent, and the local-first single-user topology (ADR-0007) is the context in which the pattern is strongest. Treating OKF as a measured candidate rather than a mandatory dependency matches the discipline already applied to DocLang.

## Consequences

### Positive

- A legible, navigable, exportable rendering of canonical Knowledge becomes possible without introducing new authority.
- The projection is a natural, low-cost context body for the cognitive runtime — starting from synthesis rather than raw sources.
- Portability and interchange through git and ordinary Markdown tools fit the local-first product.
- The authority boundary is stated explicitly, so the useful pattern cannot silently become a second source of truth.

### Negative

- A second Knowledge rendering adds generation and maintenance cost that the evidence gate must justify.
- A read-only projection forgoes the edit-the-wiki-directly affordance until a separate ADR addresses reconciliation.
- Projection consistency depends on disciplined rebuilds tied to Knowledge change.

## Canonical specification

- `DATA_ARCHITECTURE.md` — projection families, ownership and the rebuild contract that would carry an OKF Knowledge-projection family.
- `CANONICAL_DATA_MODEL.md` — projection source contract and the rule that projections never become authoritative.
- `AI_ARCHITECTURE.md` — the generated-wiki-as-projection principle and the human-gated Improvement Loop.
- `SPEC_CATALOG.md` — `SPEC-011` Retrieval and Projection System, the evaluation home for the candidate.
- `KNOWLEDGE_MODEL.md` — Knowledge forms and evolution that the projection renders.
