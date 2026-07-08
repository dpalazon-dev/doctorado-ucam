# Technical Architecture

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 2 — Specialized Implementation Model |
| **Normative status** | Provisional canonical technical baseline · v0.3 |
| **Authoritative for** | Current implementation languages, desktop runtime, process topology, physical storage baseline, inter-process protocol, repository layout, packaging, installation, update, security mechanisms, observability, backup, development toolchain and technical acceptance constraints. |
| **Not authoritative for** | Product vision, domain meaning, system authority, component ownership, logical data meaning, canonical record structure, agent-runtime semantics, vertical rules or experiment conclusions. |
| **Required reading** | `SYSTEM_ARCHITECTURE.md`, `SOFTWARE_ARCHITECTURE.md`, `COMPONENT_MODEL.md`, `CANONICAL_DATA_MODEL.md`, `DATA_ARCHITECTURE.md`, `AGENT_RUNTIME.md`, `IMPLEMENTATION_PLAN.md`, `DECISIONS.md`. |
| **Downstream documents** | Repository bootstrap, desktop-shell specification, Rust-core specifications, cognitive-runtime specifications, IPC contracts, database specifications, migrations, packaging workflows, release runbooks and technology ADRs. |

> This document selects a reversible local-first technical baseline. A technology is retained only while it remains the simplest mechanism that satisfies upstream contracts and measured requirements.

---

## Status

**Baseline date:** 2026-07-08  
**Target:** single-user, local-first ResearchOS desktop application distributed through native installers for Windows, macOS and Linux  
**Architecture stage:** modular desktop monolith with one authoritative Rust runtime and one subordinated Python cognitive sidecar  
**Stability:** provisional until the proving experiments and desktop packaging tests produce evidence

This baseline supersedes the previous VPS-oriented Python/PostgreSQL deployment model.

ResearchOS is not initially deployed as a web service.

The user installs and runs one local application. External services are optional capability providers, not runtime prerequisites except when a selected AI capability explicitly requires them.

---

## Purpose

The technical platform must support a product that behaves like ordinary desktop software:

```text
Download installer
        ↓
Install ResearchOS
        ↓
Open the application
        ↓
Work locally
```

The user must not be required to install or administer:

- Python;
- Node.js;
- PostgreSQL;
- Docker;
- a message broker;
- a reverse proxy;
- a graph database;
- a vector database server;
- a separate background service.

The baseline must nevertheless support:

- a rich cross-platform Workspace;
- canonical local persistence;
- durable long-running operations;
- document ingestion and processing;
- lexical, vector and graph retrieval;
- bounded cognitive execution;
- optional remote and local model providers;
- auditability and provenance;
- backup and restoration;
- safe upgrades and schema migrations;
- replacement of specialized engines when evidence justifies them.

---

# Position in the Architecture

```text
Canonical models and architecture
        ↓ constrain
Component Model
        ↓ defines executable responsibilities
Canonical Data Model
        ↓ defines shared structural form
Data Architecture
        ↓ defines authority and lifecycle
Agent Runtime
        ↓ defines cognitive execution semantics
Technical Architecture
        ↓ selects languages, products and packaging
Development Specifications
        ↓ define exact contracts and code changes
Implementation
```

A technical convenience never overrides a domain invariant, authority boundary or human-control policy.

---

# Architectural Thesis

> ResearchOS is a local desktop application whose authoritative state, execution control and security live in a native Rust runtime. Python is a specialized cognitive runtime that performs bounded AI, document and scientific operations under Rust supervision.

The baseline is deliberately polyglot but contains only three first-class implementation languages:

```text
TypeScript
    owns presentation

Rust
    owns authority, state, execution and operating-system integration

Python
    owns bounded cognitive and scientific computation
```

Other languages may enter only behind explicit adapters when a concrete capability cannot be implemented reasonably with the baseline stack.

---

# Baseline Selection Principles

## 1. Local-First Product

The installed application remains useful without a server deployment.

Canonical state and retained content reside on the user's machine unless the user explicitly configures synchronization or an external provider.

## 2. One Authoritative Runtime

Rust owns:

- canonical writes;
- transactions;
- authorization;
- durable jobs;
- process supervision;
- event dispatch;
- filesystem authority;
- secret access;
- approval and commit paths.

No sidecar, model or frontend may write canonical state directly.

## 3. Specialized Cognitive Runtime

Python executes bounded capabilities such as:

- parsing;
- OCR coordination;
- extraction;
- embeddings;
- reranking;
- classification;
- synthesis;
- evaluation;
- model invocation.

Python returns Artifacts, evidence and Proposals. Rust validates and mediates every canonical effect.

## 4. Desktop Shell, Not Browser Deployment

React renders inside a Tauri WebView.

The production package does not contain a Node.js server and does not expose the Workspace through a public web endpoint.

## 5. Embedded Storage Before Servers

SQLite and the local filesystem form the initial persistence baseline.

Specialized database servers are introduced only after measured insufficiency.

## 6. Two Long-Lived Processes Before Distribution

The initial application contains:

```text
Process 1: ResearchOS Desktop
           Tauri + Rust + React WebView

Process 2: Cognitive Runtime
           supervised Python sidecar
```

Additional processes are ephemeral adapters for isolated or native work, not permanent services.

## 7. Progressive Retrieval

The baseline begins with:

- structured SQLite queries;
- SQLite FTS5;
- canonical relationships and recursive queries;
- small-scale vector projection stored locally.

Dedicated vector or graph engines require explicit extraction triggers.

## 8. Progressive Cognitive Complexity

The runtime prefers, in order:

1. deterministic logic;
2. one structured model call;
3. a static pipeline;
4. plan–act–verify;
5. maker–checker or bounded parallelism;
6. experimental multi-agent coordination.

Infrastructure must not force a more complex execution pattern than the operation requires.

## 9. Native Packaging Without External Prerequisites

Every production dependency required to launch ResearchOS is included in the platform-specific application package or is supplied by the operating system.

## 10. Exact Versions Live in Lockfiles

This document selects technology families and compatibility lines.

Exact versions, checksums and build inputs live in repository manifests and release provenance.

---

# Technology Baseline

| Concern | Selected baseline | Role |
|---|---|---|
| Desktop framework | Tauri 2 | native lifecycle, windows, menus, tray, notifications, installer and sidecar supervision |
| Workspace | React + TypeScript + Vite | rich desktop UI rendered inside the system WebView |
| Authoritative runtime | Rust stable toolchain | Domain/Application execution, persistence, jobs, events, security and OS integration |
| Rust async runtime | Tokio | bounded asynchronous I/O and supervised runtime tasks |
| Rust serialization | Serde | canonical technical and IPC serialization |
| Rust validation/schema | typed domain structures plus JSON Schema generation where needed | compile-time and boundary validation |
| Canonical database | SQLite with WAL | canonical, operational, event, audit and initial projection records |
| Rust database access | SQLx SQLite adapter | explicit SQL, transactions, migrations and typed repository mapping |
| Lexical retrieval | SQLite FTS5 | initial full-text projection |
| Graph baseline | canonical relationship tables + recursive SQL | typed relationships and bounded traversal |
| Vector baseline | SQLite metadata/BLOB projection + NumPy similarity in the cognitive runtime | small-corpus semantic retrieval proving baseline |
| Future embedded vector option | replaceable embedded adapter, initially evaluated against LanceDB or equivalent | introduced only by measured retrieval/scale need |
| Content storage | content-addressed local filesystem | retained source bytes, versions, normalized text and generated Documents |
| Cognitive runtime | Python supported release packaged as a sidecar | model, document, retrieval and evaluation capabilities |
| Python environment | `uv`, `pyproject.toml` and lockfile | deterministic development and sidecar build |
| Python schemas | Pydantic v2 | IPC payloads and cognitive capability contracts; not canonical Domain authority |
| Scientific stack | NumPy plus capability-specific libraries | vector operations and scientific processing |
| PDF parsing | PyMuPDF adapter | initial text and metadata extraction |
| OCR | optional Tesseract or native OCR adapter | invoked only when extraction quality requires it |
| Model integration | provider-neutral Python adapters | remote or local reasoning and embedding providers |
| Inter-process protocol | framed, versioned JSON-RPC-like messages over stdin/stdout | Rust–Python commands, progress, results and cancellation |
| Secrets | OS credential-store abstraction | API keys and sensitive local configuration |
| Logging | Rust `tracing` + structured Python logging | local correlated diagnostics |
| Rust tests | `cargo test`, property and architecture tests | authoritative runtime verification |
| Python tests | pytest and property/evaluation suites | cognitive capability verification |
| Frontend tests | Vitest + Playwright where applicable | UI and interaction verification |
| Packaging Python | PyInstaller baseline; Nuitka evaluated only by evidence | self-contained sidecar binaries |
| Packaging desktop | Tauri bundler | Windows, macOS and Linux application packages |
| CI | native GitHub Actions runners per target OS | test, build, sign, package and verify installers |

---

# Language Responsibility Model

## TypeScript

TypeScript owns presentation and interaction concerns:

- Workspace composition;
- navigation;
- forms;
- tables;
- editors;
- dashboards;
- graph visualization;
- process-status presentation;
- conversation surfaces;
- optimistic local UI state;
- accessibility behavior.

TypeScript does not own:

- domain invariants;
- authorization;
- canonical transactions;
- durable job state;
- secrets;
- model-provider credentials;
- direct SQLite access.

## Rust

Rust owns the authoritative application runtime.

### Domain and Application

- Core Entity and Value Object implementations;
- domain invariants;
- Commands and Queries;
- Application Services;
- proposal and approval paths;
- Unit of Work;
- optimistic concurrency;
- canonical event creation.

### Data and Execution

- SQLite access;
- schema migration;
- content-addressed storage;
- durable Jobs and Process Instances;
- Outbox/Inbox;
- event dispatch and reactions;
- checkpoints and recovery;
- cancellation;
- process supervision;
- backup and restore coordination.

### Security and Native Integration

- capability authorization;
- secret retrieval;
- filesystem permissions;
- file selection and drag-and-drop boundary;
- notification and tray integration;
- updater coordination;
- application-instance locking;
- sidecar lifecycle.

## Python

Python owns bounded cognitive and scientific execution.

### Appropriate Responsibilities

- model-provider invocation;
- prompt rendering;
- PDF and scientific document processing;
- OCR orchestration;
- embeddings;
- reranking;
- semantic extraction;
- classification;
- summarization;
- synthesis;
- evaluation;
- experimental retrieval algorithms;
- optional local-model adapters.

### Forbidden Responsibilities

Python must not:

- open the canonical database for direct writes;
- bypass Rust authorization;
- commit Domain Entities;
- own the canonical event stream;
- store hidden provider memory as ResearchOS memory;
- modify prompts, policies or routing autonomously;
- retain unrestricted filesystem access;
- become a second Application Layer.

## Other Languages

C, C++, Java, Swift, Kotlin or other languages may be introduced only when:

1. a required capability has no acceptable baseline implementation;
2. the adapter boundary is explicit and versioned;
3. the dependency is independently testable and replaceable;
4. the language does not acquire canonical authority;
5. packaging and security impact are documented through an ADR.

Typical acceptable uses include:

- native inference engines;
- OCR or media libraries;
- hardware-specific acceleration;
- institutional JVM-only integrations;
- platform-specific system adapters.

---

# Runtime Topology

## Installed Application

```text
┌──────────────────────────────────────────────────────────────┐
│ ResearchOS Desktop                                           │
│ Tauri process                                                │
│                                                              │
│  React Workspace                                             │
│          │ Tauri commands/events/channels                    │
│          ▼                                                   │
│  Rust Authoritative Runtime                                  │
│  ├── Domain and Application                                  │
│  ├── SQLite repositories                                     │
│  ├── Jobs and Process Managers                               │
│  ├── Event dispatcher                                        │
│  ├── Content store                                           │
│  ├── Security and credentials                                │
│  └── Cognitive Runtime Supervisor                            │
│                    │ framed IPC                              │
└────────────────────┼─────────────────────────────────────────┘
                     ▼
┌──────────────────────────────────────────────────────────────┐
│ Python Cognitive Sidecar                                     │
│ ├── capability registry                                      │
│ ├── document processing                                      │
│ ├── model adapters                                           │
│ ├── embeddings and retrieval                                 │
│ ├── verification helpers                                     │
│ └── evaluation                                               │
└──────────────────────────────────────────────────────────────┘
```

## Optional Ephemeral Processes

The Rust runtime may start short-lived processes for:

- OCR;
- media conversion;
- local inference;
- isolated code execution;
- format-specific parsing;
- migration or repair utilities.

Such processes receive the minimum required authority and terminate after the operation.

## No Public Server Baseline

The production application does not expose:

- a public HTTP API;
- a fixed localhost port;
- a database port;
- a message broker;
- an externally reachable administration interface.

A local integration API may be introduced later behind an explicit user-controlled feature and security specification.

---

# Desktop Architecture

## Tauri Host

Tauri owns:

- application startup and shutdown;
- single-instance enforcement;
- window lifecycle;
- menus and shortcuts;
- tray behavior;
- native notifications;
- file and directory dialogs;
- drag-and-drop admission;
- deep links and file associations when introduced;
- updater integration;
- platform package metadata;
- cognitive-sidecar startup and supervision.

## React Workspace

The React application is compiled into static assets included in the desktop bundle.

Node.js is a build-time dependency only.

The Workspace communicates with Rust through typed Tauri commands and event channels.

It never communicates directly with Python.

## Tauri Boundary

Every command exposed to the Workspace must define:

- request schema;
- response schema;
- required authority;
- synchronous or durable execution behavior;
- cancellation semantics;
- error codes;
- redaction behavior.

Large payloads are referenced by identifiers or authorized local handles rather than copied through the WebView boundary.

---

# Authoritative Rust Runtime

## Crate Boundaries

The initial repository should contain cohesive Rust crates rather than one unrestricted application crate.

```text
crates/
├── domain
├── application
├── canonical-data
├── persistence-sqlite
├── content-store
├── events
├── jobs
├── process-runtime
├── agent-control
├── security
├── ipc-contracts
├── cognitive-client
└── diagnostics
```

The exact decomposition may evolve, but dependency direction remains inward toward Domain and Application contracts.

## Async Model

Tokio supports:

- sidecar I/O;
- filesystem operations;
- provider-independent progress streams;
- timers;
- durable job dispatch;
- cancellation propagation.

Domain operations remain synchronous and deterministic inside explicit transaction boundaries.

Async execution must not leak into Domain semantics.

## Error Model

Rust errors are typed by boundary:

- Domain Error;
- Validation Error;
- Authorization Error;
- Conflict Error;
- Persistence Error;
- Content Error;
- Cognitive Runtime Error;
- Provider Error;
- Process Error;
- Technical Failure.

User-facing messages are derived from typed errors and do not expose secrets, raw prompts or stack traces.

---

# Cognitive Runtime Sidecar

## Packaging

The cognitive runtime is built into a platform-specific executable containing:

- the Python interpreter;
- locked Python dependencies;
- ResearchOS cognitive packages;
- provider adapters;
- required native libraries.

The user does not install Python separately.

PyInstaller is the initial packaging mechanism because it minimizes bootstrap work.

Nuitka may replace it if measurements demonstrate meaningful advantages in:

- startup time;
- package size;
- native-library reliability;
- antivirus compatibility;
- runtime performance.

## Process Contract

The Rust supervisor:

- starts the sidecar;
- performs protocol negotiation;
- sends only admitted tasks;
- monitors heartbeat and progress;
- enforces time and cancellation budgets;
- terminates and restarts unhealthy processes;
- reconciles interrupted operations;
- records process metadata.

The Python sidecar:

- advertises supported capabilities and versions;
- accepts bounded tasks;
- requests only authorized resources;
- streams progress;
- returns typed results;
- emits no canonical event directly;
- exits cleanly when instructed.

## Capability Isolation

A capability receives:

```text
operation identifier
capability identifier and version
objective
context manifest reference
authorized content references
tool policy
model policy
budget
output schema
stopping conditions
```

It returns:

```text
status
Artifacts
Proposals
Evidence references
metrics
provider invocation records
warnings
failure details
```

## Provider Independence

Provider SDKs remain inside Python adapters.

The Rust runtime sees provider-neutral operation contracts and usage records.

One provider may be implemented first. Multi-provider routing is introduced only after a concrete resilience, quality or cost requirement is measured.

---

# Rust–Python IPC

## Baseline Transport

The initial protocol uses framed messages over the sidecar's standard input and output.

```text
Rust supervisor
      ⇅
length-prefixed JSON frames
      ⇅
Python cognitive sidecar
```

Standard error is reserved for structured sidecar diagnostics and must not carry protocol messages.

## Protocol Requirements

Every message includes:

- protocol version;
- message type;
- request or event identifier;
- operation identifier;
- timestamp;
- payload schema version;
- correlation identifier.

Supported message families include:

```text
hello
capability_manifest
execute
progress
artifact
proposal
result
cancel
cancelled
heartbeat
error
shutdown
```

## Contract Definition

IPC schemas live in a language-neutral directory.

```text
schemas/ipc/
├── envelope.schema.json
├── capability-manifest.schema.json
├── execute.schema.json
├── progress.schema.json
├── result.schema.json
└── error.schema.json
```

Rust and Python types are generated or validated against the same schema version.

Contract tests run both implementations against a shared fixture suite.

## Payload Rules

IPC messages must not carry large document bodies or vector matrices unless no reference mechanism is available.

Preferred exchange:

```text
content object identifier
content version identifier
authorized temporary path
segment identifiers
Artifact identifier
Context Manifest identifier
```

The Rust runtime grants capability-scoped access and revokes temporary access after completion.

## Future Transport Trigger

Named pipes on Windows and Unix-domain sockets on macOS/Linux may replace standard streams if measurements demonstrate that the baseline cannot support:

- required concurrency;
- streaming throughput;
- cancellation reliability;
- sidecar multiplexing.

The logical protocol remains unchanged.

---

# Database Architecture

## SQLite Baseline

SQLite stores the initial physical representation of:

- canonical Domain records;
- canonical relationships;
- provenance;
- document metadata and versions;
- Commands and Proposals;
- approvals;
- Jobs and Process Instances;
- Domain Events;
- Outbox and Inbox records;
- audit records;
- configuration metadata;
- projection manifests;
- lexical-search projection;
- initial vector metadata and values.

## Ownership

Only Rust persistence adapters may open the canonical database for writes.

Python, React and external adapters do not receive a writable database connection.

Read access is also mediated through Rust contracts unless an explicitly read-only technical adapter is approved.

## Connection and Concurrency Model

The baseline uses:

- WAL mode;
- foreign keys enabled;
- short explicit transactions;
- a bounded connection pool;
- one logical write coordinator;
- optimistic concurrency through `record_version`;
- busy timeouts and bounded retries;
- no silent last-write-wins behavior.

Single-user does not mean single-operation. Multiple UI actions and background jobs may coexist, but canonical mutation remains serialized through declared transaction paths.

## Migrations

SQL migrations are owned by the Rust persistence package.

Each application release defines:

- minimum readable schema version;
- target schema version;
- forward migration;
- backup requirement;
- failure behavior;
- restoration procedure.

A migration failure prevents normal application startup and offers recovery or diagnostic export. It must not continue with a partially migrated canonical database.

## Local Data Location

The application resolves operating-system appropriate data directories.

Conceptually:

```text
ResearchOS/
├── data/
│   └── researchos.db
├── content/
├── projections/
├── cache/
├── logs/
├── backups/
└── runtime/
```

Paths are not hard-coded to one operating system.

## Database Encryption

Whole-database encryption is not assumed by default because it introduces cross-platform and recovery complexity.

Sensitive values are excluded from SQLite where practical and stored in the operating-system credential store.

Database encryption may be added after a threat-model and recovery ADR.

---

# Canonical Physical Mapping

## Aggregate Families

The Canonical Data Model permits separate tables for each aggregate family.

The baseline should prefer explicit tables and extension tables over one generic entity-property store.

Conceptual families include:

```text
projects
project_extensions

documents
document_versions
document_extensions

knowledge
knowledge_extensions

people
person_extensions

tasks
task_extensions

activities
activity_extensions

resources
resource_extensions
```

Exact fields and table names belong to physical data specifications.

## Derived Type Registry

The database stores a governed registry for:

- type identifier;
- base Core Entity kind;
- owning vertical;
- schema version;
- extension schema reference;
- lifecycle status.

Unversioned arbitrary JSON does not become canonical extension data.

## Relationships

Typed relationships remain canonical records.

They include, at minimum:

- source reference;
- relationship type;
- target reference;
- origin;
- validation status;
- confidence when applicable;
- provenance;
- temporal validity;
- record version.

Graph projections are derived from these records and never become an alternative source of truth.

---

# Content Storage

## Baseline

Retained bytes are stored outside SQLite in a local content-addressed store.

```text
content/<algorithm>/<prefix>/<digest>
```

SQLite stores:

- digest;
- size;
- media type;
- content version;
- original source metadata;
- storage state;
- integrity verification state.

## Write Protocol

```text
admit source
    ↓
write to temporary file
    ↓
calculate digest
    ↓
validate size and media type
    ↓
atomically move to content store
    ↓
commit metadata and Domain transition
```

If metadata commit fails, orphan reconciliation removes or adopts the unreferenced object according to policy.

## Access

The frontend never receives unrestricted filesystem paths.

The Rust runtime exposes bounded read streams, temporary handles or application URLs.

The Python sidecar receives only capability-authorized content references or temporary paths.

## Integrity

Content integrity is periodically checked against stored digests.

Corruption creates an operational incident and does not silently substitute another source.

---

# Document Processing

## Pipeline

```text
Document admitted
    ↓
content retained
    ↓
parser selected
    ↓
text and structure extracted
    ↓
quality assessed
    ↓
OCR fallback if justified
    ↓
normalized representation produced
    ↓
segments created
    ↓
lexical/vector projections scheduled
    ↓
Knowledge extraction optionally requested
```

## Parser Placement

Rust owns admission, content retention, job state and output commit.

Python owns initial scientific parsing and extraction adapters.

A parser may be replaced by a native Rust or C/C++ implementation without changing the pipeline contract.

## OCR

OCR is optional and expensive.

It is invoked only when:

- normal extraction fails;
- extracted text quality is below threshold;
- the user explicitly requests it;
- a document type requires it.

OCR binaries may be packaged as optional sidecars or downloaded through an explicit component-install workflow.

## Untrusted Files

Document parsers operate with minimal permissions.

Files are treated as untrusted input. Parsing limits include:

- maximum size;
- maximum page count where appropriate;
- bounded decompression;
- timeouts;
- temporary-directory isolation;
- denial of macro/script execution;
- explicit handling of malformed content.

---

# Retrieval Architecture

## Structured Retrieval

Rust repositories execute canonical filters and relationship queries.

This channel is authoritative for:

- identifiers;
- types;
- states;
- dates;
- ownership;
- relationships;
- provenance;
- permissions.

## Lexical Retrieval

SQLite FTS5 indexes normalized textual projections.

FTS records retain source and generation metadata and remain rebuildable.

## Vector Retrieval — Initial Stage

The initial vector projection stores:

- source identifier and version;
- optional segment identifier;
- model identifier;
- dimension;
- preprocessing version;
- embedding generation;
- vector bytes;
- access classification.

For the proving corpus, Python may load an authorized candidate set and calculate similarity using NumPy.

This baseline is intentionally simple and must be measured before introducing an approximate-nearest-neighbor engine.

## Graph Retrieval

Canonical relationships are queried with explicit SQL and bounded recursive CTEs.

For occasional algorithms, Rust `petgraph` or Python NetworkX may operate over an in-memory projection.

Neither library owns canonical relationships.

## Hybrid Retrieval

The Context Builder coordinates:

```text
structured filters
    ↓
lexical candidates
    ↓
vector candidates
    ↓
relationship expansion
    ↓
fusion and reranking
    ↓
Context Manifest
```

Every included item records retrieval channel, score, source version and inclusion reason.

## Dedicated Engine Triggers

A dedicated embedded or server engine requires evidence such as:

- corpus size exceeding acceptable brute-force latency;
- measured query latency above the interaction budget;
- memory pressure incompatible with local execution;
- graph traversals that cannot be maintained in SQLite;
- required algorithms unavailable in the baseline;
- unacceptable rebuild duration.

---

# Durable Jobs and Event Processing

## One Local Coordinator

The Rust runtime contains a durable Job Coordinator backed by SQLite.

The initial application does not run a separate worker service.

## Job Record

A durable Job records:

- identifier;
- job type and contract version;
- payload reference;
- state;
- priority;
- attempt count;
- availability time;
- lease owner and expiry when required;
- checkpoint reference;
- progress;
- cancellation state;
- result or failure reference;
- causal identifiers.

## Execution

- asynchronous I/O runs in Tokio tasks;
- blocking native libraries use bounded blocking pools;
- CPU-intensive work is delegated to the Python sidecar or an ephemeral process;
- canonical writes return through the Application path;
- jobs checkpoint at policy-defined boundaries.

## Shutdown and Resume

On application shutdown:

- new jobs stop being admitted;
- running work receives a bounded graceful-cancellation window;
- durable checkpoints are committed;
- sidecars are asked to stop;
- unresolved operations are marked recoverable.

On restart, the coordinator reconciles interrupted jobs before accepting new background work.

## Events

The Rust runtime implements:

```text
Command
    ↓
validated transaction
    ↓
Domain Event + Outbox record
    ↓
local dispatcher
    ↓
projection / process / notification reactions
```

Events report committed facts. They do not replace Commands or Application orchestration.

---

# Agent Runtime Technical Mapping

## Rust Control Plane

Rust implements the control plane defined by `AGENT_RUNTIME.md`:

- task admission;
- policy evaluation;
- budget allocation;
- operation state;
- execution graph;
- tool authorization;
- checkpointing;
- retries;
- cancellation;
- stopping conditions;
- human-approval gates;
- result acceptance;
- canonical proposal submission.

## Python Cognitive Plane

Python implements cognitive executors:

- Intent analysis when nondeterministic analysis is required;
- model-assisted planning;
- semantic extraction;
- model and embedding calls;
- reranking;
- critique and synthesis;
- output-schema validation helpers;
- evaluation functions.

## Verification Order

Technical verification proceeds from most independent and deterministic to least:

1. schema and type validation;
2. authorization and policy checks;
3. deterministic domain rules;
4. source-span and citation verification;
5. content and relationship consistency checks;
6. alternate retrieval or computation;
7. optional independent model critic;
8. human review.

A second answer from the same model is not treated as independent proof.

## Framework Policy

No general agent framework is mandatory.

LangGraph, Semantic Kernel or similar frameworks may be evaluated only behind an adapter and may not:

- own canonical operation state;
- hide checkpoints;
- bypass budgets;
- introduce undisclosed memory;
- control authorization;
- become the only representation of execution.

The default implementation should use ordinary Rust and Python components until a framework demonstrably reduces complexity.

---

# Model and Tool Integration

## Provider Adapters

Each provider adapter exposes a normalized contract for:

- model identity;
- supported modalities;
- structured output;
- token/context limits;
- streaming;
- cancellation behavior;
- usage and cost reporting;
- retry classification;
- data-retention configuration where available.

## Model Routing

Routing is based on operation requirements:

- deterministic extraction vs open synthesis;
- context size;
- modality;
- latency budget;
- cost budget;
- privacy classification;
- local or remote execution policy;
- observed evaluation performance.

Model prestige or size is not a routing criterion.

## Local Models

Local inference is a future adapter, not a baseline requirement.

A native engine such as llama.cpp or another runtime may be packaged as an optional sidecar when:

- hardware detection is available;
- model licenses are acceptable;
- package/download size is explicit;
- resource limits are enforced;
- quality is evaluated against the operation suite.

## Tools

Tools are capability adapters with:

- explicit input and output schemas;
- declared side effects;
- required permissions;
- timeout and resource budgets;
- audit metadata;
- deterministic or nondeterministic classification.

Unsafe code execution is deferred until a cross-platform isolation design is approved.

---

# Security Architecture

## Trust Boundaries

```text
React WebView
    → untrusted presentation input

Rust Runtime
    → authoritative trusted computing base

Python Sidecar
    → supervised, limited-trust cognitive executor

External Models and Tools
    → untrusted external systems

Imported Documents
    → untrusted content
```

## Secrets

Secrets are stored through an operating-system credential-store abstraction.

They are never stored in:

- frontend state;
- repository files;
- ordinary SQLite configuration rows;
- logs;
- prompt templates;
- diagnostic exports.

Python receives provider credentials only for the duration and scope required by an authorized invocation.

## IPC Security

- the sidecar is launched by the application;
- protocol handles are inherited, not publicly discoverable;
- every operation carries an unguessable identifier;
- capability authorization occurs before dispatch;
- messages are size-limited and schema-validated;
- no arbitrary command execution is accepted through IPC.

## Prompt and Retrieval Security

Retrieved or imported content is data, not instruction authority.

The runtime preserves:

- source trust classification;
- provenance;
- content boundaries;
- instruction hierarchy;
- detection of suspicious tool-use requests;
- separation of inferred and validated memory.

Repeated malicious content does not gain authority through frequency or embedding similarity.

## Platform Signing

Production installers should be code-signed.

macOS releases require application signing and notarization before general distribution.

Windows signing and reputation management are part of release readiness, not optional polish.

---

# Observability and Diagnostics

## Local Logging

Rust and Python emit structured records containing:

- timestamp;
- severity;
- component;
- operation identifier;
- causal/correlation identifiers;
- event or job identifier;
- duration;
- status;
- error category;
- model/provider metadata when permitted;
- token and cost metrics when available.

Prompts, document bodies, secrets and sensitive personal data are excluded by default.

## Log Storage

Logs are:

- local;
- size-bounded;
- rotated;
- retention-governed;
- exportable through user action.

## Diagnostic Export

The application can create a sanitized diagnostic package containing:

```text
application version
platform information
schema version
component manifest
redacted configuration summary
selected structured logs
failed-operation manifests
migration status
sidecar capability manifest
```

The user reviews and explicitly exports this package.

## OpenTelemetry

OpenTelemetry is not required in the local baseline.

Instrumentation may be added behind an adapter if development or optional remote diagnostics demonstrate a concrete need.

---

# Backup and Recovery

## Backup Scope

A complete backup includes:

- consistent SQLite snapshot;
- retained content objects;
- canonical configuration that is safe to export;
- type and schema registries;
- backup manifest;
- application and schema compatibility metadata.

Secrets are not exported in plaintext.

## Backup Process

```text
request backup
    ↓
quiesce or coordinate canonical writes
    ↓
SQLite online backup
    ↓
copy referenced content and manifests
    ↓
verify hashes
    ↓
create compressed backup package
    ↓
record backup result
```

Projection data may be excluded because it is rebuildable.

## Restore

Restore executes in a controlled maintenance mode and verifies:

- package integrity;
- schema compatibility;
- content hashes;
- migration path;
- available disk space;
- target-location safety.

The existing data directory is preserved until the restored application opens successfully.

## User Data on Uninstall

Uninstallation must not silently delete the ResearchOS data directory.

Data deletion is a separate explicit user action.

---

# Installation, Packaging and Updates

## Platform Packages

The build produces platform-native artifacts, for example:

```text
Windows
    installer executable / MSI as selected by release specification

macOS
    signed application bundle and DMG

Linux
    AppImage and/or distribution packages selected by release specification
```

One source repository supports all targets, but each production package is built and tested on its target operating system.

## Sidecar Bundling

Each target package includes a matching cognitive-runtime executable and native dependencies.

The Tauri configuration identifies sidecars per target triple.

## Build Reproducibility

Release provenance records:

- source commit;
- Rust lockfile;
- Python lockfile;
- frontend lockfile;
- toolchain versions;
- sidecar checksums;
- model/config schema versions;
- migration set;
- signing identity metadata;
- generated package hashes.

## Updates

Initial development releases may use manual installer replacement.

Automatic updates are enabled only after:

- signing is operational;
- migration rollback behavior is tested;
- update metadata is authenticated;
- interrupted-update recovery is validated;
- user data is proven independent from application binaries.

## First Run

First launch performs:

1. data-directory creation;
2. permission checks;
3. schema initialization or migration;
4. content-store verification;
5. sidecar protocol negotiation;
6. optional provider setup;
7. initial backup recommendation.

ResearchOS must open without an AI provider configured. Deterministic capabilities remain available.

---

# Repository Structure

```text
researchos/
├── apps/
│   └── desktop/
│       ├── frontend/                 # React + TypeScript + Vite
│       └── src-tauri/                # Tauri entry point and commands
│
├── crates/
│   ├── domain/
│   ├── application/
│   ├── canonical-data/
│   ├── persistence-sqlite/
│   ├── content-store/
│   ├── events/
│   ├── jobs/
│   ├── process-runtime/
│   ├── agent-control/
│   ├── cognitive-client/
│   ├── ipc-contracts/
│   ├── security/
│   └── diagnostics/
│
├── python/
│   └── cognitive_runtime/
│       ├── capabilities/
│       ├── providers/
│       ├── document_processing/
│       ├── retrieval/
│       ├── embeddings/
│       ├── verification/
│       ├── evaluation/
│       └── worker/
│
├── schemas/
│   ├── ipc/
│   ├── canonical/
│   ├── capabilities/
│   └── projections/
│
├── migrations/
│   └── sqlite/
│
├── tests/
│   ├── architecture/
│   ├── contract/
│   ├── integration/
│   ├── desktop/
│   ├── recovery/
│   └── evaluation/
│
├── packaging/
│   ├── windows/
│   ├── macos/
│   └── linux/
│
├── docs/
├── Cargo.toml
├── Cargo.lock
├── pyproject.toml
├── uv.lock
└── package.json / frontend lockfile
```

## Repository Rules

- Rust crates do not import Python implementation details.
- Python packages consume versioned IPC and capability schemas.
- frontend code imports generated TypeScript contracts, not Rust internals.
- canonical migrations are reviewed with their owning specification.
- provider SDKs remain inside provider adapters.
- native binaries are checksummed and declared in packaging manifests.
- no generated artifact becomes the source of a canonical schema.

---

# Development Environment

## Required Developer Tooling

Developers require:

- Rust toolchain;
- platform prerequisites for Tauri;
- Node.js for frontend builds;
- `uv` for Python environments;
- platform packaging tools when building installers.

The end user requires none of these.

## Local Development Modes

### Integrated Desktop Mode

Runs the Tauri host, frontend dev server and Python sidecar with development diagnostics.

### Rust Test Mode

Runs Domain, Application, persistence and runtime tests without the UI or Python where possible.

### Cognitive Test Mode

Runs Python capability tests against recorded fixtures and fake IPC hosts.

### Contract Test Mode

Runs Rust and Python against shared protocol fixtures.

### Packaging Mode

Builds the exact platform bundle and executes installation, launch, migration and uninstall tests in clean environments.

## Docker

Docker is optional for:

- CI experiments;
- reproducible external-service tests;
- security tooling;
- future isolated code execution.

Docker is not part of the end-user runtime and not required for ordinary local development.

---

# Testing Architecture

## Rust Tests

### Unit Tests

Cover:

- Domain invariants;
- Value Objects;
- policies;
- deterministic transformations;
- typed errors.

### Property Tests

Cover:

- identity and version behavior;
- relationship constraints;
- serialization round trips;
- job state machines;
- migration invariants;
- content hashing.

### Architecture Tests

Enforce:

- crate dependency direction;
- no persistence dependency in Domain;
- no provider dependency in Application;
- no direct frontend or Python canonical writes;
- adapter isolation.

### Integration Tests

Use temporary SQLite databases and content stores to verify:

- transactions;
- Outbox behavior;
- migrations;
- job recovery;
- backup/restore;
- projection rebuild;
- sidecar supervision.

## Python Tests

Cover:

- capability schemas;
- provider adapters;
- parsing;
- chunking;
- embedding consistency;
- retrieval;
- output validation;
- cancellation;
- bounded resource behavior.

## IPC Contract Tests

Verify:

- protocol negotiation;
- unknown-version rejection;
- message framing;
- progress streaming;
- cancellation;
- large-payload references;
- crash and restart behavior;
- malformed-message containment.

## Frontend Tests

Cover:

- interaction behavior;
- accessible navigation;
- typed command invocation;
- process progress;
- error recovery;
- offline behavior;
- no-provider setup.

## Installer Tests

Each target OS validates:

- clean installation;
- first launch;
- sidecar discovery;
- data-directory creation;
- upgrade over an older version;
- migration failure behavior;
- uninstall without data loss;
- package signature where configured.

## Evaluation Tests

The ResearchOS internal evaluation suite remains the release authority for cognitive behavior.

Public agent benchmarks are optional diagnostics and do not replace vertical acceptance tests.

---

# Quality Gates

## Rust

A change must pass:

- formatting;
- Clippy with project policy;
- compilation on supported targets;
- unit/property tests;
- architecture tests;
- dependency audit;
- unsafe-code review where applicable.

Unsafe Rust is forbidden in first-party code unless isolated, documented and approved through an ADR.

## Python

A change must pass:

- formatting and linting;
- static type checks for supported packages;
- pytest suites;
- dependency lock verification;
- provider contract tests;
- packaging smoke test.

## TypeScript

A change must pass:

- formatting and linting;
- strict type checking;
- unit/component tests;
- accessibility checks for affected views;
- generated-contract drift checks.

## Cross-Language

A release cannot pass if:

- IPC schemas and implementations disagree;
- sidecar capability manifests are incompatible;
- migration versions do not match the desktop build;
- installer contents differ from release provenance;
- a cognitive capability writes canonical state outside Rust.

---

# Proving-Slice Technical Mapping

## Reference Flow

```text
User imports a paper
        ↓
React requests file admission
        ↓
Rust validates and retains content
        ↓
Rust creates Document and durable processing Job
        ↓
Rust dispatches bounded parse task to Python
        ↓
Python returns normalized text and structure Artifact
        ↓
Rust validates and persists DocumentVersion/segments
        ↓
Rust updates FTS and schedules embedding task
        ↓
Python computes embeddings and candidate Knowledge
        ↓
Rust stores rebuildable vector projection and Proposals
        ↓
User reviews candidate Knowledge
        ↓
Rust commits accepted Knowledge and relationships
        ↓
Question triggers hybrid retrieval and Context Manifest
        ↓
Python produces evidence-grounded answer Artifact
        ↓
Rust records provenance, operation result and presentation data
```

## Technical Milestones

### T0 · Desktop Bootstrap

- Tauri application opens on all target development platforms;
- React invokes a typed Rust command;
- local data directory is created;
- Python sidecar negotiates protocol;
- installer smoke build succeeds on one platform.

### T1 · Canonical Local State

- SQLite migrations execute;
- one Core Entity command path works;
- canonical event and audit records are created;
- backup and restore work for the empty/minimal system.

### T2 · Durable Processing

- document content is retained atomically;
- a durable Job survives application restart;
- Python parsing is supervised and cancellable;
- normalized content is committed through Rust.

### T3 · Cognitive Proposal Path

- a bounded extraction operation runs;
- budgets and provider metadata are recorded;
- candidate Knowledge and provenance return as Proposals;
- canonical acceptance remains a human/Application action.

### T4 · Hybrid Retrieval

- structured, FTS and vector retrieval operate;
- canonical relationships expand context;
- Context Manifest records inclusion reasons;
- an answer cites retained source segments.

### T5 · Cross-Platform Release

- Windows, macOS and Linux packages build;
- at least one signed/notarized release path is validated;
- upgrade and migration tests pass;
- uninstall preserves data;
- diagnostic export works.

---

# Technologies Explicitly Removed from the Baseline

The following are not required by the local single-user product:

- PostgreSQL;
- Psycopg;
- pgvector;
- Caddy;
- Docker Compose as runtime;
- public FastAPI deployment;
- Node.js production server;
- external message broker;
- separate permanent worker service;
- Kubernetes;
- Redis;
- Neo4j or another graph server;
- OpenTelemetry backend;
- `pg_dump`;
- Restic as mandatory backup mechanism.

They may be reconsidered only if the product scope changes or measurements satisfy an extraction trigger.

---

# Technologies Explicitly Deferred

- automatic cloud synchronization;
- multi-user collaboration;
- public API server;
- dedicated vector engine;
- dedicated graph engine;
- workflow engine;
- multi-agent framework;
- sandboxed arbitrary code execution;
- bundled local foundation model;
- plugin marketplace;
- remote telemetry collection;
- mobile application;
- server deployment mode.

---

# Replacement and Extraction Triggers

## SQLite to Client–Server Database

Consider a server database only if:

- multi-user concurrent writes become a product requirement;
- remote access becomes canonical;
- one local writer cannot meet measured throughput;
- database size or maintenance exceeds acceptable desktop constraints;
- synchronization requires a server authority.

## Embedded Vector Projection

Introduce an embedded ANN engine when:

- measured vector search exceeds interaction budgets;
- corpus size makes NumPy search impractical;
- memory usage is unacceptable;
- required filtering/reranking cannot be maintained simply.

## Dedicated Graph Engine

Introduce a graph engine when:

- recurrent multi-hop queries exceed SQLite capabilities;
- graph algorithms become continuous product behavior;
- relationship volume and latency justify operational cost;
- the engine remains a projection, not canonical authority.

## Separate Background Service

Extract a worker process or service only when:

- tasks must continue after the desktop application fully exits;
- workload isolation cannot be achieved through the sidecar model;
- resource contention harms interaction;
- remote execution becomes a requirement.

## Alternative IPC

Replace standard streams with named pipes or domain sockets when measured concurrency or throughput requires it.

## Python Capability Replacement

A Python capability may move to Rust or native code when:

- startup or execution latency is material;
- memory use is unacceptable;
- packaging is unreliable;
- security requires a smaller trusted dependency surface;
- the algorithm is stable and no longer benefits from Python experimentation.

## Additional Language

Introduce Java, C++ or another runtime only through an ADR demonstrating a unique capability and acceptable packaging cost.

---

# Technical Risks and Mitigations

## Polyglot Complexity

**Risk:** Rust, Python and TypeScript increase build and debugging complexity.

**Mitigation:** strict ownership, one IPC contract, only two long-lived processes, generated schemas and shared contract tests.

## Rust Development Cost

**Risk:** implementing all Application and persistence behavior in Rust may slow early development.

**Mitigation:** keep the Domain small, use explicit crates, avoid premature abstractions and retain Python for rapidly changing cognitive algorithms.

## Python Packaging Variability

**Risk:** native Python dependencies may fail across targets or produce large packages.

**Mitigation:** platform-native CI builds, locked dependencies, smoke-tested sidecars, optional heavy components and capability-specific adapters.

## SQLite Write Contention

**Risk:** background jobs and UI commands compete for writes.

**Mitigation:** one logical write coordinator, short transactions, WAL, bounded queues, optimistic concurrency and measured extraction triggers.

## Sidecar Failure

**Risk:** the cognitive runtime crashes or hangs.

**Mitigation:** heartbeat, timeout, cancellation, process restart, durable checkpoints and no canonical authority in the sidecar.

## Installer Size

**Risk:** bundled Python and native libraries create large packages.

**Mitigation:** optional components, platform-specific pruning, shared library review, delayed local-model packaging and size budgets.

## Cross-Platform Differences

**Risk:** filesystem, WebView, signing and credential behavior differ by OS.

**Mitigation:** platform adapters, native CI runners, installer tests and explicit supported-platform matrix.

## Secret Leakage

**Risk:** credentials reach logs, IPC fixtures or diagnostics.

**Mitigation:** OS credential stores, short-lived credential injection, redaction, schema-level secret classification and export review.

## Local Data Loss

**Risk:** a single device contains the only canonical state.

**Mitigation:** first-class backup, restore testing, backup reminders, optional user-selected external backup location and no uninstall deletion.

## Cognitive Runtime Becomes a Second Application

**Risk:** Python accumulates canonical logic and hidden state.

**Mitigation:** no writable canonical DB access, proposals only, architecture tests, capability contracts and Rust-owned operation records.

---

# Technical Conformance Criteria

An implementation conforms to this baseline only if:

1. the product installs and launches without requiring external runtime installation;
2. Tauri is the desktop lifecycle and packaging host;
3. React communicates with Rust, never directly with Python;
4. Rust owns all canonical mutation and authorization;
5. SQLite is the initial canonical database;
6. Python executes only bounded cognitive/scientific capabilities;
7. Rust–Python communication uses a versioned, tested protocol;
8. the cognitive sidecar can be cancelled, restarted and recovered without corrupting canonical state;
9. durable Jobs and Process Instances survive application restart;
10. retained content uses integrity-checked local storage;
11. lexical, vector and graph structures remain rebuildable projections;
12. secrets use OS-backed secure storage;
13. the application remains usable without an AI provider configured;
14. installers preserve user data during update and uninstall;
15. backups can be restored in a clean environment;
16. no general agent framework owns operation state or hidden memory;
17. no additional language or server product is introduced without a measured need and ADR;
18. Windows, macOS and Linux builds derive from the same canonical source and contracts.

---

# Stabilization After Experiments

After the proving experiments and desktop package validation, each provisional choice receives one outcome:

```text
Retain
    evidence supports the baseline

Refine
    the mechanism remains but its contract or configuration changes

Replace
    another adapter better satisfies the same upstream contract

Extract
    the local component becomes an independently packaged runtime

Reject
    the capability or technology is unnecessary
```

The following decisions require explicit evidence before stabilization:

- SQLite write and retrieval performance;
- vector projection mechanism;
- Rust/Python IPC throughput and failure recovery;
- Python packaging reliability on all targets;
- installer size and startup latency;
- model-provider baseline;
- OCR packaging;
- automatic update mechanism;
- local-model support;
- need for a public or local integration API.

---

# Final Technical Statement

ResearchOS begins as a self-contained, local-first desktop application.

Its technical structure is:

```text
React + TypeScript
        ↓
Tauri desktop shell
        ↓
Rust authoritative runtime
        ↓
SQLite + local content store
        ↓
versioned IPC
        ↓
Python cognitive sidecar
        ↓
optional external or native capability adapters
```

Rust owns state, authority, execution and security.

Python supplies cognitive and scientific capabilities without becoming a second source of truth.

SQLite and the filesystem keep the product installable and operable by one user without infrastructure administration.

Every specialized engine, additional language or distributed service remains optional until measured product needs justify its cost.
