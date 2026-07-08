# Technical Architecture

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 2 — Specialized Implementation Model |
| **Normative status** | Provisional canonical technical baseline · v0.2 |
| **Authoritative for** | Current implementation technologies, repository layout, runtime packaging, physical data mapping, development toolchain, deployment baseline, security mechanisms, observability, backup and technical acceptance constraints. |
| **Not authoritative for** | Product vision, domain meaning, system authority, software/component boundaries, logical data ownership, agent-runtime semantics or experiment conclusions. |
| **Required reading** | `SYSTEM_ARCHITECTURE.md`, `SOFTWARE_ARCHITECTURE.md`, `COMPONENT_MODEL.md`, `DATA_ARCHITECTURE.md`, `AGENT_RUNTIME.md`, `IMPLEMENTATION_PLAN.md`, `DECISIONS.md` (especially ADR-0002 and ADR-0006). |
| **Downstream documents** | Repository bootstrap, development specifications, dependency manifests, Compose files, CI workflows, database migrations, deployment/runbook documentation and later technology ADRs. |

> This document selects a reversible Experimental Technical Baseline. Evidence from the proving experiments may replace any technology without changing upstream contracts.

---

## Status

**Baseline date:** 2026-07-08  
**Target:** single-user ResearchOS proving environment and first personal VPS deployment  
**Architecture stage:** modular monolith with separated interactive and worker runtime roles  
**Stability:** provisional until Experiments 0–3 produce evidence

This is not a claim that every selected tool is a permanent production standard.

It is the smallest coherent stack capable of implementing and testing the reference architecture.

---

## Purpose

The prior architecture documents intentionally deferred product and framework decisions.

The project now needs a technical baseline to:

- create the repository and package structure;
- implement Domain and Application contracts;
- persist canonical and operational state;
- process real documents durably;
- execute bounded cognitive operations;
- expose a usable Workspace/API;
- deploy safely to one VPS;
- collect experiment evidence;
- recover from failure;
- replace components without architectural rewrite.

This document turns logical components into a concrete but reversible implementation platform.

---

# Position in the Architecture

```text
Canonical models and architecture
        ↓ constrain
Component Model
        ↓ defines executable responsibility
Data Architecture
        ↓ defines logical state and ownership
Agent Runtime
        ↓ defines cognitive execution semantics
Technical Architecture
        ↓ selects mechanisms and products
Development Specifications
        ↓ define exact endpoints, schemas and code tasks
Implementation
```

A technical convenience never overrides an upstream authority or invariant.

---

# Baseline Selection Principles

## 1. One Deployable System Before Distribution

The first implementation is one codebase and one deployment stack with distinct runtime processes.

No microservices, service mesh or distributed database are introduced.

## 2. PostgreSQL-Centred Until Proven Insufficient

Canonical state, operational state, Outbox/Inbox, initial projections, lexical search and vector search begin in one PostgreSQL instance under logically separate schemas.

This minimizes dual-write and operations complexity while preserving replaceable ports.

## 3. Python for Domain, Application and Cognitive Work

Python is selected because the proving slice combines:

- document processing;
- data modeling;
- HTTP APIs;
- background processing;
- AI/model integrations;
- scientific evaluation.

The Domain remains plain Python and independent of frameworks.

## 4. Browser Workspace, Not Chat-Only UI

A typed React workspace provides documents, candidates, evidence and process status.

Conversation may be added as one interface but does not define the frontend architecture.

## 5. Durable Work Without an External Broker

The first queue, scheduler, Outbox and Inbox use PostgreSQL records and leases.

A broker or workflow engine is introduced only after measured need.

## 6. Provider Adapters, Not Provider Architecture

One reasoning-model adapter is implemented first, but contracts remain provider-neutral.

Embeddings, parsing and model calls are independently replaceable.

## 7. Operational Simplicity on One VPS

Docker Compose, Caddy, PostgreSQL and a small number of application containers form the deployment baseline.

Kubernetes is explicitly out of scope.

## 8. Exact Versions Live in Lockfiles

This document chooses supported major/minor lines.

The repository pins exact versions and image digests in lockfiles/manifests for reproducibility.

## 9. Progressive Cognitive Complexity

The implementation follows the Agent Runtime complexity ladder. Deterministic code and bounded single-model calls are preferred over planners, loops, parallel branches or multi-agent patterns when they satisfy the same contract.

No framework or infrastructure is introduced merely to imitate benchmark-leading agent systems.

## 10. Domain Evaluation Before Benchmark Optimization

The technical stack is optimized first against versioned ResearchOS tasks and proving-slice evidence. Public agent benchmarks may diagnose capabilities, but they do not determine the architecture or justify infrastructure by themselves.

---

# Technology Baseline

| Concern | Selected baseline | Role |
|---|---|---|
| Backend language | Python 3.13.x | Domain, Application, workers, cognitive runtime, CLI |
| Python dependency/tooling | uv + `pyproject.toml` + lockfile | environments, dependency resolution, commands |
| HTTP framework | FastAPI | API adapter and OpenAPI contract |
| Contract validation | Pydantic v2 | transport/runtime schemas; not Domain entities |
| Persistence mapping | SQLAlchemy 2.x | repository and projection adapters |
| PostgreSQL driver | Psycopg 3 | synchronous/asynchronous database access |
| Migrations | Alembic | reviewed schema migrations |
| Canonical database | PostgreSQL 17.x | canonical, operational, event and initial projection state |
| Vector extension | pgvector | initial semantic projection for Experiment 2 |
| Content storage | content-addressed local filesystem volume | PDFs, normalized text and persisted Artifacts |
| Background runtime | ResearchOS Python worker using PostgreSQL jobs/leases | durable jobs, processes, event reactions |
| Scheduler/dispatcher | PostgreSQL-backed scheduler + Outbox dispatcher | timers, retries, dispatch |
| Frontend language | TypeScript on Node.js 24 LTS | Workspace build/tooling |
| Frontend | React 19 + Vite | browser Workspace |
| API client | generated/typed client from OpenAPI or equivalent schema | frontend/backend contract |
| PDF parsing | PyMuPDF | initial text/metadata extraction |
| OCR fallback | Tesseract adapter, disabled until required by corpus | image-based documents |
| Reasoning provider | Anthropic adapter first | extraction, synthesis and answer generation |
| Embedding provider | configurable embedding adapter; first implementation may use an OpenAI-compatible API | vector projection |
| Observability | structured JSON logs + OpenTelemetry instrumentation | logs, traces and metrics |
| Testing | pytest, Hypothesis, architecture tests, Playwright/Vitest | unit, property, integration and UI tests |
| Packaging/deployment | Docker images + Docker Compose | local and VPS runtime |
| Edge/TLS | Caddy | HTTPS, static UI and reverse proxy |
| Backups | `pg_dump` + restic encrypted repository | database/content/config recovery |
| CI | GitHub Actions | lint, type, test, build, migration and security checks |

## Version Policy

- Python: pin the latest approved patch in the 3.13 line; review 3.14 after dependency compatibility is demonstrated.
- PostgreSQL: pin the current approved 17.x minor and update minors promptly after backup/restore testing.
- Node.js: use the active 24 LTS line for frontend tooling.
- Framework/library major versions are pinned in lockfiles and upgraded through reviewed changes.
- Container images use immutable tags or digests in deployment.
- Model identifiers and provider configuration are recorded per experiment and invocation; no floating “latest” model identifier is permitted in reproducible runs.

---

# Runtime Topology

## Stage 0 — Development and Initial Proving Slice

```text
Developer machine / CI

┌───────────────────────────────────────────────────────────┐
│ api process                                               │
│ FastAPI · Commands · Queries · Domain · Application       │
└──────────────────────┬────────────────────────────────────┘
                       │
┌──────────────────────▼────────────────────────────────────┐
│ PostgreSQL                                                │
│ domain · ops · eventing · projection · audit schemas      │
│ pgvector                                                  │
└──────────────────────┬────────────────────────────────────┘
                       │
┌──────────────────────▼────────────────────────────────────┐
│ worker process                                            │
│ Jobs · Process Managers · Events · Cognitive Runtime      │
└──────────────────────┬────────────────────────────────────┘
                       │
             ┌─────────▼─────────┐
             │ content volume    │
             └───────────────────┘

web dev server or built static Workspace calls `/api/v1`.
```

Development MAY run API and worker as local processes while PostgreSQL runs in a container.

## Stage 1 — Personal VPS

```text
Internet
    ↓ HTTPS
Caddy
    ├── `/`       → static Workspace
    └── `/api/*`  → API container

Docker Compose network
    ├── api
    ├── worker
    ├── postgres
    └── optional isolated-runner (disabled by default)

Persistent volumes
    ├── postgres-data
    ├── researchos-content
    └── backup-staging
```

The Scheduler and Outbox Dispatcher initially execute inside the worker process as independently supervised loops.

## Runtime Separation

### API Process

May host:

- Experience adapters;
- Command/Query/Application components;
- Domain components;
- authentication/session;
- synchronous proposal/approval operations.

Must not host:

- long model calls;
- full PDF processing;
- vector rebuild;
- unbounded event cascades;
- browser/code execution.

### Worker Process

Hosts:

- Job Coordinator;
- Process Manager Host;
- Event Reaction Host;
- Outbox Dispatcher;
- Scheduler;
- Cognitive Operation Manager;
- Context Builder;
- Agent Runtime;
- projection builders;
- model/tool adapters.

### Isolated Runner

Not required for initial paper extraction.

When introduced, it executes as a separate locked-down container/process with:

- no canonical database credentials;
- no unrestricted host filesystem;
- scoped network policy;
- one-time task credentials;
- resource/time limits;
- normalized result return.

---

# Repository Structure

```text
researchos/
├── pyproject.toml
├── uv.lock
├── .python-version
├── package.json                 # root scripts/workspace metadata if needed
├── compose.yaml
├── Caddyfile
├── .env.example
├── README.md
├── docs/                        # architecture and future specifications
│
├── apps/
│   ├── api/
│   │   └── main.py
│   ├── worker/
│   │   └── main.py
│   └── cli/
│       └── main.py
│
├── src/researchos/
│   ├── shared_kernel/
│   ├── domain/
│   │   ├── knowledge/
│   │   ├── documents/
│   │   ├── projects/
│   │   ├── people/
│   │   ├── tasks/
│   │   ├── activities/
│   │   └── resources/
│   ├── application/
│   │   ├── commands/
│   │   ├── queries/
│   │   ├── proposals/
│   │   ├── policies/
│   │   └── effects/
│   ├── cognitive/
│   │   ├── operations/
│   │   ├── context/
│   │   ├── agent_runtime/
│   │   ├── verification/
│   │   └── artifacts/
│   ├── processes/
│   ├── eventing/
│   ├── contracts/
│   │   ├── driving/
│   │   └── driven/
│   ├── adapters/
│   │   ├── persistence/
│   │   ├── content/
│   │   ├── projections/
│   │   ├── models/
│   │   ├── tools/
│   │   ├── integrations/
│   │   └── notifications/
│   ├── interfaces/
│   │   ├── http/
│   │   └── cli/
│   └── operations/
│       ├── config/
│       ├── identity/
│       ├── telemetry/
│       ├── audit/
│       └── health/
│
├── migrations/
│   ├── env.py
│   └── versions/
│
├── web/
│   ├── package.json
│   ├── src/
│   │   ├── app/
│   │   ├── features/
│   │   ├── entities/
│   │   ├── api/
│   │   └── components/
│   └── tests/
│
├── tests/
│   ├── unit/
│   ├── property/
│   ├── architecture/
│   ├── integration/
│   ├── contract/
│   ├── end_to_end/
│   └── fixtures/
│
├── experiments/
│   ├── datasets/
│   ├── manifests/
│   ├── evaluations/
│   └── reports/
│
├── infra/
│   ├── docker/
│   ├── backup/
│   └── scripts/
│
└── scripts/
    ├── dev
    ├── test
    ├── migrate
    ├── backup
    └── restore-test
```

## Package Rules

- `domain` imports only `shared_kernel` and standard-library/domain-safe dependencies.
- `application` imports Domain and port contracts, never concrete adapters.
- `cognitive` imports cognitive/application ports and read contracts, never canonical write adapters.
- `processes` issue Commands through application ports.
- `adapters` implement driven ports.
- `interfaces` call driving ports.
- runtime bootstrap under `apps/` composes concrete implementations.
- provider SDK imports are restricted to their adapter packages.

These rules are enforced through architecture tests.

---

# Backend Architecture

## Python Runtime

Python 3.13.x is the baseline.

Reasons:

- active bugfix line at baseline creation;
- mature library compatibility;
- modern typing and runtime behavior;
- long enough security horizon for the proving phase.

The exact patch is pinned in `.python-version` and CI/container images.

## Dependency Management

`uv` owns:

- Python installation for development/CI where practical;
- virtual environment and dependency resolution;
- reproducible lockfile;
- task execution through repository scripts.

Production images install from the lockfile, not unconstrained requirements.

Dependency groups:

- runtime;
- development;
- test;
- experiment;
- optional adapters.

Provider SDKs remain optional extras where possible.

## FastAPI Boundary

FastAPI implements only the HTTP adapter.

It provides:

- routing;
- authentication/session integration;
- transport validation;
- OpenAPI generation;
- streaming/status endpoints where useful;
- mapping of application results to HTTP.

FastAPI types do not enter Domain modules.

## Pydantic Usage

Pydantic v2 is used for:

- Command/Query transport models;
- event envelopes;
- model structured outputs;
- capability/tool schemas;
- configuration validation;
- OpenAPI/JSON Schema generation.

Pydantic models are not the canonical Domain model by default.

Domain entities and Value Objects remain framework-independent dataclasses/classes with explicit constructors and invariants.

## Persistence Adapter

SQLAlchemy 2.x implements:

- repository mappings;
- Unit of Work;
- projection queries;
- migrations metadata;
- explicit transaction boundaries.

Psycopg 3 is the PostgreSQL driver.

The initial implementation may use async database access in API/worker adapters while keeping Domain behavior synchronous.

No lazy ORM object graph crosses repository boundaries.

Repositories load one aggregate boundary explicitly.

## Migrations

Alembic manages physical schema migrations.

Rules:

- migrations are reviewed code;
- autogeneration is advisory;
- every migration declares affected logical owner;
- destructive changes use expand/backfill/switch/contract where possible;
- semantic data migrations have tests and rollback/recovery plan;
- production migration runs before application cutover;
- CI checks for model/migration drift.

---

# Database Architecture

## PostgreSQL Version

PostgreSQL 17.x is the conservative experimental baseline.

The current minor is pinned and updated through normal maintenance.

PostgreSQL 18 may be evaluated later, but no experiment depends on an 18-only feature.

## Logical Schemas

```text
researchos database
├── domain
│   ├── knowledge_*
│   ├── document_*
│   ├── project_*
│   ├── person_*
│   ├── task_*
│   ├── activity_*
│   └── resource_*
├── ops
│   ├── command_receipt
│   ├── proposal / approval
│   ├── job / process_instance
│   ├── cognitive_operation / checkpoint
│   ├── model_invocation / capability_invocation
│   └── effect_intent / effect_result
├── eventing
│   ├── outbox
│   ├── inbox
│   └── dead_letter
├── projection
│   ├── workspace views
│   ├── lexical search records
│   ├── vector records
│   ├── graph nodes/edges
│   └── projection checkpoints
├── audit
│   └── audit_entry
└── public
    └── extension metadata only; application tables avoided
```

Exact table names are defined by data specifications.

## Database Roles

At minimum:

- migration owner;
- application read/write role;
- worker role;
- projection/rebuild role;
- backup role;
- read-only diagnostic role.

The proving deployment may initially share some credentials, but production configuration must preserve least privilege and schema ownership.

## Transactions

- command transaction: aggregate + Outbox + idempotency result;
- operational transitions: separate short transactions;
- no model/network/tool call while a database transaction is open;
- optimistic aggregate version check;
- explicit isolation level per operation, defaulting to ordinary read committed unless stronger semantics are justified;
- advisory locks only for scheduler/maintenance coordination, not Domain correctness.

## Job Queue

The first durable queue is an `ops.job` table.

Workers claim ready rows using:

- short transaction;
- ordered selection;
- `FOR UPDATE SKIP LOCKED` or equivalent;
- lease owner and expiry;
- attempt counter;
- committed claim before work begins.

A worker:

1. claims a job;
2. executes outside the claim transaction;
3. checkpoints operation/process state;
4. marks success, retry or terminal failure;
5. releases/renews lease explicitly.

## Wake-Up Mechanism

Polling is the correctness mechanism.

PostgreSQL `LISTEN/NOTIFY` may reduce wake-up latency but is not the durable queue and may be lost without affecting correctness.

## Outbox/Inbox

- Outbox inserts atomically with canonical mutation.
- Dispatcher leases undispatched rows and publishes to the internal consumer registry.
- For the modular monolith, “publish” initially means durable routing to registered event consumers/jobs through database-backed records.
- Inbox uniqueness is `(consumer_id, event_id)`.
- External brokers may later implement the same contracts.

## Lexical Search

Initial retrieval uses PostgreSQL full-text search and structured filters.

It indexes:

- Document title/metadata;
- normalized text segments;
- validated Knowledge;
- selected Person/Project/Task labels.

Language configuration and stemming are explicit per content language.

## Vector Search

pgvector implements the first vector projection.

Rules:

- vectors remain in `projection` schema;
- each row carries source/version/model/preprocessing generation;
- exact search is acceptable for small experiment corpora;
- approximate indexes are introduced after measured dataset/query need;
- changing embedding model creates a new generation and rebuild;
- vector results never bypass canonical access policy.

## Graph Projection

The first graph representation is relational:

- canonical relationship records projected as typed edges;
- candidate/inferred edges explicitly classified;
- recursive SQL for bounded traversal;
- no dedicated graph database in the baseline.

A graph database requires measured query/scale evidence and an ADR.

## Hybrid Retrieval Assembly

The Context Builder combines retrieval channels at the application level rather than delegating authority to one database product.

The initial channel set is:

1. canonical references and structured SQL filters;
2. PostgreSQL full-text search;
3. pgvector semantic similarity;
4. relational graph traversal;
5. authorized external retrieval through a capability adapter.

Implementation rules:

- every result carries source identity, version, retrieval channel and projection generation;
- access policy is applied before ranking and fusion;
- lexical, vector and graph scores are not assumed to be directly comparable;
- fusion/reranking is implemented as a replaceable application strategy and recorded in the Context Manifest;
- a simple deterministic fusion method is preferred before model-based reranking;
- external results remain raw evidence until ingested and governed;
- deletion or reclassification of canonical data invalidates dependent projection entries.

This provides hybrid retrieval without requiring separate vector and graph products during the proving phase.

---

# Content Storage

## Baseline

Content is stored on a persistent filesystem volume using content-addressed paths.

```text
/content/
└── sha256/
    └── ab/
        └── cd/
            └── <full-digest>
```

Metadata remains in PostgreSQL.

## Write Protocol

1. stream upload to temporary restricted path;
2. enforce size/media policy;
3. calculate SHA-256 digest;
4. optionally scan/inspect;
5. atomically move into content-addressed path;
6. commit Content Object metadata and Document reference;
7. clean temporary file on failure.

## Rules

- content files are immutable;
- application paths are never canonical identifiers;
- generated/normalized content has its own digest and provenance;
- content access occurs through the Content Repository port;
- filenames from users are metadata, never trusted paths;
- the API does not expose raw host paths;
- backup includes content and database metadata consistently enough for reconciliation.

## Future S3 Compatibility

An S3-compatible adapter may replace the filesystem when:

- multi-host access is required;
- content volume exceeds practical VPS storage;
- off-host durability or lifecycle policies justify it.

The logical Content Object contract remains unchanged.

---

# Document Processing

## Initial Parser

PyMuPDF is the baseline PDF parser for:

- page count and metadata;
- text blocks/words;
- page coordinates;
- basic images/tables where available;
- text extraction for ordinary digital PDFs.

## Parsing Pipeline

```text
original content
    ↓ media validation
PyMuPDF extraction
    ↓
page/block records + normalized text
    ↓ quality checks
optional OCR fallback
    ↓
versioned derived content + provenance
```

## OCR

Tesseract is an optional capability adapter.

It is activated only when deterministic extraction quality indicates image-based text.

OCR output is always marked as derived and lower-confidence until reviewed or cross-checked.

## Scientific Structure

GROBID or another scientific-document service is deferred until ordinary extraction proves insufficient for citations, sections or bibliography.

Its introduction must be evaluated against the same parser contract.

## Malicious Documents

Initial parsing occurs in the worker with strict library/resource limits for trusted personal inputs.

Before accepting arbitrary external uploads, parsing moves to the Isolated Runner.

---

# Agent and Model Technology

## Agent Framework Policy

ResearchOS implements the Agent Runtime contracts directly for the proving slice. A general orchestration framework is neither prohibited nor assumed.

A framework such as a graph executor may be evaluated later only behind a ResearchOS runtime adapter and only if it demonstrates that it can:

- persist authoritative execution state in ResearchOS stores;
- expose typed Plans, Steps, Observations and checkpoints;
- obey ResearchOS budgets, stopping conditions and approval gates;
- avoid hidden provider or framework memory as a correctness dependency;
- resume after process failure without replaying an opaque conversation;
- emit complete traces and normalized model/tool records;
- preserve Capability Registry and Command/Proposal boundaries;
- reduce measured implementation or maintenance cost.

Framework-owned domain state, implicit autonomous loops and uninspectable memory are disqualifying.

## Runtime Implementation

The Agent Runtime is implemented inside `src/researchos/cognitive/agent_runtime` using ResearchOS contracts.

No general agent framework is required for the proving slice.

Reasons:

- the runtime semantics are already defined;
- durable state belongs in ResearchOS operational records;
- a framework must not become a second process/state authority;
- the initial operations are bounded pipelines, not open-ended autonomous agents.

A library may be adopted later for specific graph execution or provider abstraction only if it fits the contracts and reduces measured implementation cost.

## Reasoning Model Adapter

The first Model Gateway adapter targets Anthropic through its official Python SDK. One provider is sufficient for the proving slice; multi-provider execution is an evaluation and resilience option, not a baseline requirement.

The adapter supports:

- one-shot Messages-style requests;
- structured output parsing through ResearchOS schemas;
- tool-call proposals normalized into Capability Requests;
- streaming only for user-visible Artifact generation, not as durable state;
- rate-limit and transient-error classification;
- provider/model identity and usage recording.

The exact model is configuration, not hard-coded architecture.

Model routing begins with explicit task-class configuration. Dynamic routing, ensembles or provider fallback are introduced only when evaluation shows a material benefit. A more prestigious or larger model is not automatically preferred.

Each experiment manifest pins:

- model identifier;
- provider API/SDK version;
- instruction version;
- relevant sampling parameters;
- budget;
- date and dataset.

## Embedding Adapter

The first embedding adapter may target an OpenAI-compatible hosted API.

The contract remains independent of provider and records:

- model identifier;
- dimensions;
- preprocessing version;
- usage/cost;
- source segment digest.

A local embedding model may be evaluated later for privacy/cost, but it is not required to prove the architecture.

## Prompt and Instruction Storage

Prompts/instructions are versioned repository resources, for example:

```text
src/researchos/cognitive/instructions/
├── extract_knowledge/
│   ├── v1.md
│   └── schema_v1.json
└── answer_with_evidence/
    ├── v1.md
    └── schema_v1.json
```

Rules:

- no critical prompt exists only in provider dashboards;
- every invocation records instruction version;
- prompts do not contain secrets;
- domain policy is implemented outside prompts;
- evaluation changes accompany prompt changes.

## Structured Output

Pydantic/JSON Schema validates model output.

Repair attempts are bounded.

After repeated invalid output, the operation fails/replans rather than silently accepting text.

## Verification Implementation

Verification is implemented as a composable pipeline:

1. schema and contract checks;
2. deterministic calculations and policy rules;
3. source-span and citation verification;
4. cross-source checks when required;
5. optional bounded model critic;
6. human review for governed cases.

The same provider may be used for production and critique during early experiments, but this is not considered independent verification. Multi-run voting or a second provider may be tested for high-risk tasks only after comparison with deterministic and evidence-based checks.

## Runtime Experience Storage

Operational experience candidates are stored in PostgreSQL operational/audit records with references to the originating operation, task class, runtime version and evaluation status.

They are not injected into future operations by default. Approved experience becomes a versioned routing rule, prompt/instruction change, recovery policy or code change through the normal review path. The runtime never edits these resources autonomously.

## Model Privacy

Provider eligibility is configured by data classification.

Restricted content is never sent to an external provider unless policy explicitly permits it.

Provider retention/training controls are configuration and operational policy, not assumed.

---

# Frontend Architecture

## Baseline

- Node.js 24 LTS;
- TypeScript strict mode;
- React 19;
- Vite;
- browser-based Workspace;
- generated typed API client or schema-validated equivalent.

## Workspace Scope for Proving Slice

The initial UI contains:

1. Document import;
2. Document processing status;
3. extracted text/metadata preview;
4. Knowledge candidate review;
5. approve/reject/edit actions;
6. evidence/source viewer;
7. question and evidence-backed answer view;
8. operation trace summary and failure/retry state.

It does not require:

- full dashboard customization;
- general chat history;
- calendar/Kanban;
- graph visualization;
- rich text collaborative editor;
- mobile client.

## State Management

- server state is queried through API resources;
- local UI state remains non-canonical;
- optimistic updates are used only when conflict/rejection can be represented;
- long operations return operation identifiers and update by polling or server-sent events;
- WebSocket infrastructure is not required initially.

## Accessibility and Interaction

- semantic HTML;
- keyboard navigation;
- visible loading/error states;
- evidence and AI-generated content clearly labelled;
- approval actions display exact subject and consequences;
- conversation is not the only route to any essential function.

---

# API Architecture

## Versioning

Base path:

```text
/api/v1
```

The OpenAPI document is generated and checked into/used by CI for client compatibility where useful.

## Resource and Operation Style

The API exposes intention-revealing operations, for example:

```text
POST /api/v1/documents:register
POST /api/v1/documents/{id}:process
POST /api/v1/knowledge-candidates/{id}:approve
POST /api/v1/knowledge-candidates/{id}:reject
POST /api/v1/questions:answer
GET  /api/v1/operations/{id}
GET  /api/v1/documents/{id}
GET  /api/v1/knowledge/{id}
```

Exact routes belong to development specs.

Generic CRUD endpoints are not the default when an intention-revealing command exists.

## Error Contract

The HTTP adapter maps stable application errors:

- validation error;
- unauthorized/forbidden;
- not found;
- domain rejection;
- concurrency conflict;
- idempotency conflict;
- accepted asynchronous operation;
- rate/budget limit;
- dependency unavailable;
- internal correlation reference.

Provider exceptions are never returned directly.

## Long Operations

Long operations return `202 Accepted` plus operation reference.

Progress is queried through an Operation resource and optionally delivered through server-sent events.

---

# Identity and Security

## Deployment Assumption

The baseline is a single-user personal system exposed through a private or authenticated public endpoint.

Single-user does not mean unauthenticated.

## Authentication

Initial authentication:

- one local user account;
- Argon2id password hashing through a vetted library;
- server-side or signed session identifier;
- Secure, HttpOnly, SameSite cookies;
- CSRF protection for state-changing browser requests;
- session rotation after authentication;
- configurable inactivity and absolute expiry.

OAuth/OIDC is deferred until an external identity provider or multi-user need exists.

## Authorization

Application policy checks still receive actor identity and autonomy scope.

The single user may hold all human permissions, but AI/process actors remain distinct and restricted.

## Transport

Caddy terminates HTTPS and redirects HTTP.

Only Caddy exposes public ports.

PostgreSQL and worker ports remain on the internal Compose network.

## Secrets

Production secrets are injected through mounted secret files or another restricted mechanism, not committed `.env` files.

`.env` is allowed only for non-production local development and `.env.example` contains placeholders.

Secrets include:

- database password;
- session signing/encryption key;
- model/embedding API keys;
- backup repository credentials;
- external integration tokens.

## File Security

- upload size limits;
- media-type detection rather than extension trust;
- path normalization;
- no user-controlled filesystem paths;
- restrictive file permissions;
- parser time/memory limits;
- future malware scanning/isolation before broader ingestion.

## Prompt, Retrieval and Memory Security

- retrieved documents, emails and webpages are always delimited as untrusted data;
- capability authorization occurs outside model output;
- projection rows retain provenance, trust class and source generation;
- candidate Knowledge and inferred edges are excluded from validated retrieval unless explicitly requested;
- source deletion/reclassification triggers projection invalidation or rebuild;
- suspicious instruction-like content is observable and may force isolated processing;
- raw provider output cannot enter approved runtime experience or canonical Knowledge directly.

## Dependency and Image Security

CI performs:

- dependency vulnerability scan;
- secret scan;
- container image scan;
- lockfile verification;
- reproducible image build where practical.

No automatic dependency update merges without tests.

---

# Background Processing Implementation

## Worker Loop

The worker supervises independent loops:

- job claimant/executor;
- Outbox dispatcher;
- event consumer runner;
- scheduler/timer activation;
- projection builder;
- stale lease recovery;
- health heartbeat.

A failure in one loop is isolated and restarts with bounded backoff.

## Concurrency

Initial defaults are conservative:

- low worker concurrency;
- per-job-type concurrency limit;
- one active cognitive step per operation unless plan explicitly fans out;
- provider rate-limit budget;
- parser/model work not executed in API event loop;
- database connection pool sized below server limits.

Exact values are configuration and measured during experiments.

## Process Supervision

Containers use restart policies and health checks.

The application itself persists work state; container restart is not the recovery mechanism by itself.

## Cancellation

Cancellation is cooperative:

- operation/process marked cancellation requested;
- worker checks between Steps and before effects;
- current safe atomic step completes or times out;
- external calls may remain ambiguous and require reconciliation;
- result records cancelled/partial state.

---

# Observability

## Logging

Structured JSON logs include:

- timestamp;
- severity;
- service/runtime role;
- component;
- operation/command/event/job/process IDs;
- actor;
- correlation/causation;
- error class;
- duration;
- no secret or unrestricted prompt/content body.

Development may use human-readable formatting.

## Tracing

OpenTelemetry traces cover:

- HTTP request;
- Command handler and Unit of Work;
- Outbox dispatch and event reaction;
- Job/Process step;
- Context build;
- model/tool invocation;
- verification;
- Proposal/Approval;
- resulting Command/Event.

The system remains operational if no external telemetry collector is available.

## Metrics

Initial metrics:

- HTTP latency/error;
- command rejection/conflict;
- queue depth and oldest job age;
- job duration/retry/failure;
- Outbox/Inbox lag;
- projection lag;
- cognitive operation outcome;
- model tokens/cost/latency;
- context size/build time;
- verification failures;
- candidate approval/edit/rejection;
- database pool and storage usage;
- backup freshness.

## Collector

The proving baseline may export to logs/local OTLP endpoint.

A full Prometheus/Grafana/Loki stack is deferred until operations justify it.

---

# Testing Architecture

## Backend Tests

### Unit

- Value Objects and aggregate invariants;
- domain lifecycle transitions;
- Application handlers with fake ports;
- planning/verification deterministic logic.

### Property-Based

Hypothesis tests:

- identity/version invariants;
- lifecycle invalid transitions;
- idempotency;
- parser normalization;
- relationship cardinality;
- retry/backoff bounds.

### Architecture

Import/dependency tests enforce Component Model rules.

A dedicated tool such as `import-linter`, custom AST checks or equivalent is selected during bootstrap.

### Integration

Run against real PostgreSQL and content filesystem.

Cover:

- Unit of Work + Outbox atomicity;
- optimistic concurrency;
- Job leasing/reclaim;
- Inbox deduplication;
- migrations;
- vector/lexical query;
- backup/restore smoke tests.

### Contract

- model adapter with recorded/fake provider responses;
- tool/capability schemas;
- OpenAPI compatibility;
- event schema versioning.

### End-to-End

One real PDF through import → processing → candidate approval → query answer.

## Frontend Tests

- Vitest for component/state logic;
- React Testing Library for user-visible behavior;
- Playwright for proving-slice flows;
- accessibility checks in critical review/approval screens.

## Non-Deterministic Tests

Model-dependent evaluation is separated from deterministic CI.

CI uses:

- fakes/fixtures for contracts;
- optional scheduled/provider-enabled evaluation jobs;
- versioned experiment manifests;
- threshold reports, not brittle exact string assertions.

## Evaluation Harness

The repository contains a provider-neutral evaluation harness with versioned cases and rubrics for:

- document extraction and source-span accuracy;
- evidence-backed question answering;
- retrieval precision, recall, attribution and freshness;
- policy compliance and forbidden-action tests;
- restart/checkpoint recovery;
- cost, latency and budget behavior;
- repeated-run stability;
- comparison against deterministic, single-call and simple-RAG baselines.

Public benchmarks such as GAIA, AgentBench or policy-compliance suites may run as optional diagnostic jobs. Their results are recorded with model/tool versions but do not gate a ResearchOS release unless a development specification explicitly adopts them.

Sensitive evaluation corpora store references/digests and controlled fixtures rather than committing confidential source material.

---

# Development Quality Gates

Every pull request must pass, as applicable:

1. formatting and lint;
2. type checking;
3. unit/property tests;
4. architecture dependency tests;
5. migration consistency check;
6. integration tests for changed adapters;
7. frontend tests/build;
8. OpenAPI/client compatibility;
9. security/secret scan;
10. documentation/spec impact check.

## Python Tooling

Baseline:

- Ruff for formatting and lint;
- Pyright or mypy for strict-enough static checks, selected at bootstrap and used consistently;
- pytest;
- Hypothesis;
- coverage reporting;
- import-boundary tests.

## TypeScript Tooling

Baseline:

- TypeScript strict mode;
- ESLint where needed beyond TypeScript/Vite defaults;
- formatter policy consistent with repository;
- Vitest;
- Playwright.

Exact configuration belongs to the repository bootstrap spec.

---

# Environment Strategy

## Local Development

- Python/API/worker may run on host through `uv`;
- PostgreSQL runs through Compose;
- Vite dev server runs locally;
- content uses a local managed directory;
- model calls disabled or use explicit developer credentials;
- fake adapters available for deterministic tests.

## Test

- ephemeral PostgreSQL database/schema;
- temporary content directory;
- fake model/tool adapters by default;
- migrations applied from zero;
- fixed clock/identifier adapters where needed.

## VPS Production-Like

- built images;
- Compose-managed services;
- Caddy HTTPS;
- persistent volumes;
- production secrets;
- off-host backups;
- provider adapters enabled by policy;
- migration and restore runbooks.

No separate staging environment is required initially; reproducible local/CI integration and a protected VPS deployment are sufficient for the personal project.

---

# Container Architecture

## Images

### `researchos-api`

Contains backend package and API entrypoint.

### `researchos-worker`

May use the same image with a different entrypoint to reduce drift.

### `researchos-web`

Build stage produces static assets; Caddy may serve them directly.

### `postgres`

Pinned PostgreSQL 17 image with pgvector extension available.

### `caddy`

Pinned Caddy image with mounted configuration and persistent certificate data.

### `isolated-runner`

Deferred and not started by default.

## Build Rules

- multi-stage builds;
- non-root application user;
- minimal runtime dependencies;
- no build credentials in final image;
- locked dependencies;
- image labels include source revision and build date;
- health checks;
- read-only root filesystem where practical;
- writable paths explicit.

---

# Deployment and Release

## Deployment Flow

```text
merge to main
    ↓ CI quality gates
build immutable images
    ↓
backup + migration precheck
    ↓
pull images on VPS
    ↓
run migrations
    ↓
restart API/worker
    ↓
readiness + smoke test
    ↓
record deployed revision
```

For the personal baseline, deployment may be manually approved and executed through a scripted SSH workflow.

## Database Migration Order

1. verify backup freshness;
2. run migration in controlled job/container;
3. validate schema version;
4. start compatible application version;
5. run smoke/integrity checks;
6. enable workers;
7. monitor queues/errors.

Backward-compatible expand/switch migrations are preferred when downtime becomes material.

## Rollback

Application images may roll back only when compatible with the migrated schema.

A database rollback is not assumed. Recovery may require a forward fix or restore according to runbook.

---

# Backup and Disaster Recovery

## Baseline Objectives

For the experimental/personal deployment:

- target RPO: 24 hours or better;
- target RTO: 4 hours or better;
- no claim of high availability;
- recovery correctness prioritized over automatic failover.

These values are reviewed after real usage and data criticality are known.

## Backup Plan

### Database

- nightly logical dump;
- retained migration revision and deployment metadata;
- encrypted off-host transfer through restic;
- more frequent dumps before migrations or major experiments.

### Content

- content volume backed up through restic snapshots;
- digest verification during periodic integrity checks.

### Configuration

- non-secret deployment configuration in source control;
- encrypted backup of required secret material or documented secret recreation;
- Caddy certificate state may be backed up but remains recreatable.

## Retention Baseline

Example initial policy:

- daily: 14;
- weekly: 8;
- monthly: 6;
- pre-migration snapshots: retain until migration proven and normal retention covers it.

Exact values are configuration.

## Restore Test

At least monthly during active development:

1. restore database into isolated environment;
2. restore content;
3. run migrations if required;
4. run integrity checks;
5. rebuild projections;
6. inspect pending jobs/processes/effects;
7. execute proving-slice read path;
8. record result and duration.

A backup that has not been restored is unverified.

## Future PITR

PostgreSQL WAL archiving and point-in-time recovery are introduced when a 24-hour RPO becomes unacceptable.

---

# Experimental Configuration and Reproducibility

Every proving experiment produces a manifest containing:

- source revision;
- database schema/migration revision;
- Python/Node/runtime versions;
- dependency lockfile digest;
- container image digests;
- model and embedding identifiers;
- instruction/prompt versions;
- parser and preprocessing versions;
- dataset/corpus digest;
- configuration and budgets;
- start/end time;
- result metrics and report reference.

Experiment output never relies on an unrecorded provider “latest” model or mutable prompt dashboard.

---

# Proving-Slice Technical Mapping

## Flow

```text
React Workspace
    → FastAPI RegisterDocument Command
    → PostgreSQL domain.document + eventing.outbox
    → content-addressed filesystem
    → worker Outbox/Event reaction
    → PostgreSQL ops.job/process
    → PyMuPDF extraction
    → normalized content
    → Agent Runtime + Anthropic adapter
    → Knowledge candidate Proposals
    → React review/approval
    → CreateKnowledge Command
    → canonical Knowledge + events
    → PostgreSQL lexical/pgvector projections
    → Answer request
    → Context Builder
    → Agent Runtime
    → evidence-backed Answer Artifact
```

## First Technical Milestones

### T0 · Repository Bootstrap

- Python and frontend workspaces;
- package boundaries;
- lint/type/test/CI;
- Compose PostgreSQL;
- configuration and telemetry skeleton.

### T1 · Canonical Command Path

- Documents aggregate;
- repository/Unit of Work;
- Outbox atomicity;
- HTTP/CLI command;
- architecture tests.

### T2 · Durable Processing

- Jobs, leases, worker;
- event reaction;
- content storage;
- PyMuPDF extraction;
- operation status UI/API.

### T3 · Cognitive Candidate Path

- Cognitive Operation records;
- Context Manifest;
- Anthropic model adapter;
- structured candidate extraction;
- verification;
- Proposal/Approval.

### T4 · Retrieval and Answer

- lexical baseline;
- pgvector generation;
- context comparison;
- answer Artifact with citations;
- Experiment 2 metrics.

### T5 · Recovery and Deployment

- restart recovery;
- failure injection;
- backup/restore;
- Docker Compose VPS;
- Caddy HTTPS;
- end-to-end smoke test.

---

# Technologies Explicitly Deferred

| Technology/class | Why deferred | Trigger for reconsideration |
|---|---|---|
| Microservices | no team/scale evidence; adds network consistency cost | measured isolation/scaling/release need |
| Kubernetes | single VPS and few processes | multi-node operations requiring orchestration |
| Kafka/RabbitMQ/NATS | PostgreSQL delivery sufficient for baseline | throughput, cross-service routing or retention needs exceed it |
| Redis | no proven cache/queue need | measured latency/ephemeral coordination requirement |
| Celery/Dramatiq/Temporal | internal jobs/process contracts are simple enough initially | process complexity/reliability cost exceeds custom runtime |
| Neo4j/other graph DB | relational graph sufficient for experiment | graph query performance/algorithms justify separate store |
| Qdrant/Weaviate/Pinecone | pgvector sufficient for initial corpus | vector scale/features/operations justify dedicated store |
| MinIO/S3 service | local content volume simplest on one host | multi-host/off-host object requirements |
| Elasticsearch/OpenSearch | PostgreSQL FTS sufficient initially | lexical scale/ranking/analytics requirements |
| Local LLM serving/GPU | operational cost and hardware complexity | privacy/cost/latency evidence supports it |
| General agent framework | runtime semantics already defined; avoid duplicate state | proven reduction in code without authority conflict |
| WebSockets | polling/SSE sufficient for operation status | bidirectional real-time interaction need |
| OIDC/SSO | single-user baseline | multi-user/external identity requirement |
| Event sourcing | aggregate state is canonical | evidence that full event reconstruction is required |

Deferred technologies must not be scaffolded “for later.”

---

# Replacement and Extraction Triggers

## External Message Broker

Consider only when:

- multiple independently deployed consumers exist;
- database dispatch load is measured as problematic;
- retention/replay across services is required;
- operational ownership justifies another stateful system.

## Dedicated Vector Store

Consider when:

- corpus/query volume exceeds pgvector objectives;
- required filtering/hybrid features are inadequate;
- independent scaling materially reduces cost/latency;
- benchmark evidence is reproducible.

## Dedicated Graph Store

Consider when:

- required multi-hop/graph algorithms are awkward or too slow relationally;
- graph projection has stable semantics and rebuild pipeline;
- operational cost is justified.

## Workflow Engine

Consider when:

- process definitions become numerous and complex;
- timers/compensation/recovery code becomes a dominant maintenance burden;
- engine state can remain operational and subordinate to ResearchOS authority;
- integration preserves Command/Event contracts.

## Separate Services

Use the extraction criteria in ADR-0002 and the Component Model.

The first likely candidates, if ever justified, are isolated execution, document processing or model gateway—not Domain truth ownership by default.

---

# Technical Risks and Mitigations

## PostgreSQL Becomes Too Central

**Risk:** canonical, jobs and projections compete for one instance.

**Mitigation:** logical schemas, query limits, separate pools, measured indexes, projection rebuilds, later extraction through ports.

## Custom Job Runtime Costs More Than Expected

**Risk:** retries, leases and process state become complex.

**Mitigation:** implement only proving-slice requirements; evaluate a workflow engine after metrics; preserve runtime contracts.

## Python Boundary Erosion

**Risk:** ORM/Pydantic/framework types enter Domain.

**Mitigation:** architecture tests, explicit mapping, plain Domain types, bootstrap-only composition.

## Model Provider Lock-In

**Risk:** prompts/tool semantics depend on one provider.

**Mitigation:** Model Gateway, normalized records, structured schemas, adapter contract tests, provider-independent operation specs.

## Frontend Scope Expansion

**Risk:** dashboard work delays architecture validation.

**Mitigation:** proving-slice screens only; no generalized productivity suite before experiments.

## Personal VPS Data Loss

**Risk:** single host failure.

**Mitigation:** encrypted off-host backups, restore tests, content digests, migration discipline.

## Sensitive Research Data Sent Externally

**Risk:** confidentiality/privacy breach.

**Mitigation:** classification policy, provider eligibility, local deterministic processing, explicit configuration, audit and future local adapters.

---

# Technical Conformance Criteria

The baseline implementation conforms only if:

1. one repository implements a modular monolith with enforceable package boundaries;
2. Domain code has no FastAPI, SQLAlchemy, Pydantic-provider or model SDK dependency;
3. API and worker are separate runtime entrypoints;
4. canonical mutation and Outbox insertion are atomic in PostgreSQL;
5. long work is represented by durable PostgreSQL Jobs/Processes rather than in-memory tasks;
6. model and parsing calls occur outside canonical transactions;
7. PostgreSQL schemas preserve logical data classes and ownership;
8. content is immutable, digest-addressed and accessed through a port;
9. lexical, vector and graph data are projection schema records and rebuildable;
10. provider SDK code exists only in adapters;
11. Agent Runtime state is recoverable without provider thread state;
12. approval is enforced by Application code;
13. HTTPS and authenticated sessions protect the VPS interface;
14. secrets are not committed or included in prompts/logs;
15. structured logs and correlated traces cover the proving path;
16. exact dependency/model versions are recorded for experiments;
17. CI enforces format, type, tests, architecture and migration checks;
18. backup and restore are scripted and tested;
19. no deferred infrastructure is required to run the proving slice;
20. every selected technology can be replaced through an upstream port or component boundary.

---

# Stabilization After Experiments

After Experiments 0–3:

1. compare observed needs with selected mechanisms;
2. document failures, bottlenecks and workarounds;
3. retain, replace or simplify each provisional choice;
4. record significant stabilized decisions as new ADRs;
5. update this document to v1.0;
6. only then treat the technical stack as the production reference.

Possible outcomes include:

- keep PostgreSQL-only architecture;
- remove pgvector if domain-grounded lexical retrieval is sufficient;
- introduce a dedicated projection store if evidence supports it;
- adopt a workflow library if durable-process code is excessive;
- change model/embedding providers;
- simplify the frontend;
- revise the Domain Model if Experiment 0 falsifies it.

Architecture stability is earned through evidence, not document completion.

---

# Official Reference Basis

The baseline was checked against official project documentation current at the baseline date:

- [Python releases](https://www.python.org/downloads/)
- [uv documentation](https://docs.astral.sh/uv/)
- [FastAPI documentation](https://fastapi.tiangolo.com/)
- [Pydantic documentation](https://pydantic.dev/docs/)
- [SQLAlchemy 2.0 documentation](https://docs.sqlalchemy.org/en/20/)
- [Alembic documentation](https://alembic.sqlalchemy.org/en/latest/)
- [Psycopg 3 documentation](https://www.psycopg.org/psycopg3/docs/)
- [PostgreSQL version support](https://www.postgresql.org/support/versioning/)
- [pgvector](https://github.com/pgvector/pgvector)
- [Node.js releases](https://nodejs.org/en/about/previous-releases)
- [React documentation](https://react.dev/)
- [Vite documentation](https://vite.dev/guide/)
- [Docker Compose documentation](https://docs.docker.com/compose/)
- [Caddy documentation](https://caddyserver.com/docs/)
- [OpenTelemetry documentation](https://opentelemetry.io/docs/)
- [Anthropic client SDK documentation](https://docs.anthropic.com/en/api/client-sdks)
- [PyMuPDF documentation](https://pymupdf.readthedocs.io/)
- [restic documentation](https://restic.readthedocs.io/)

These references support product capability and maintenance assumptions. ResearchOS contracts remain authoritative for system behavior.

---

# Final Technical Statement

ResearchOS begins as a Python and PostgreSQL modular monolith with a React workspace, durable PostgreSQL-backed background processing, content-addressed file storage and provider-neutral cognitive adapters.

FastAPI exposes application contracts.

Plain Python Domain modules own behavior and invariants.

SQLAlchemy, Psycopg and Alembic implement persistence without entering the Domain.

PostgreSQL stores canonical, operational, event and initial projection data under explicit logical separation.

pgvector, full-text search and relational graph projections are sufficient to test retrieval assumptions before adding specialized databases.

The Agent Runtime executes in a recoverable worker and invokes models/tools only through gateways.

Docker Compose and Caddy deploy the system safely to one VPS.

Every choice is pinned, observable, backed up and replaceable.

This baseline exists to produce evidence. The experiments, not architectural fashion, decide what becomes permanent.
