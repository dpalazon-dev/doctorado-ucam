# SPEC-001 · Repository and Build System

## Document Contract

| Concern | Contract |
|---|---|
| **Level** | Level 3 — Development Specification |
| **Normative status** | Development Specification · Review |
| **Authoritative for** | The exact repository topology, developer toolchain pinning, dependency and lockfile policy, root build-command interface, cross-platform CI matrix, repository-level architecture checks and completion evidence for the ResearchOS foundation. |
| **Not authoritative for** | Desktop lifecycle, domain-module design, canonical persistence, content storage, jobs, events, IPC semantics, cognitive-sidecar behavior, Workspace behavior, installer release policy or product features owned by later Specs. |
| **Required reading** | `SPEC_CATALOG.md`, `SOFTWARE_ARCHITECTURE.md`, `TECHNICAL_ARCHITECTURE.md`, `DOCUMENTATION_ARCHITECTURE.md`, `IMPLEMENTATION_CONTEXTS.md`, `CLAUDE.md`, ADR-0002, ADR-0007, ADR-0008 and ADR-0009. |
| **Downstream documents** | `SPEC_CATALOG.md`, `ROADMAP.md`, `CLAUDE.md`, SPEC-002 through SPEC-013, implementation branches, CI workflows and repository runbooks. |

> This Spec translates the approved local-first polyglot architecture into a reproducible repository and build contract. It does not authorize product behavior.

---

## Specification Metadata

| Field | Value |
|---|---|
| **Identifier** | SPEC-001 |
| **Title** | Repository and Build System |
| **Category** | Foundation |
| **Delivery wave** | Wave 1 — Desktop and authoritative foundation |
| **Status** | Review |
| **Prerequisites** | None |
| **Primary owners translated** | `TECHNICAL_ARCHITECTURE.md`, `SOFTWARE_ARCHITECTURE.md` |
| **Supporting owners** | `SYSTEM_ARCHITECTURE.md`, `COMPONENT_MODEL.md`, `DOCUMENTATION_ARCHITECTURE.md` |
| **Initial review date** | 2026-07-09 |

---

# 1. Purpose

ResearchOS requires one repository that can host and verify three implementation languages without allowing build convenience to erase architectural boundaries.

This Spec establishes the smallest reproducible foundation required by every later development Spec:

- one cross-platform source repository;
- deterministic Rust, TypeScript and Python dependency resolution;
- a stable repository topology;
- a single root command surface for developers and CI;
- build and test isolation between language ecosystems;
- architecture checks that reject forbidden dependency directions;
- native CI execution on Windows, macOS and Linux;
- auditable build provenance;
- explicit deferral of all product behavior.

The repository created under this Spec is a **buildable structural skeleton**, not an executable product milestone.

---

# 2. Imported Architectural Constraints

This Spec adopts, without redefining, the following upstream decisions.

## 2.1 Product topology

ResearchOS is a single-user, local-first desktop application. The initial implementation consists of:

```text
Tauri 2 + React + TypeScript
        ↓
authoritative Rust runtime
        ↓ controlled boundary
subordinate Python cognitive runtime
```

The repository MUST support this topology without requiring an end user or ordinary developer workflow to operate PostgreSQL, Docker, a broker, a reverse proxy, a graph server or a vector database server.

## 2.2 Authority boundary

Rust owns canonical authority, state-changing application behavior, durable execution, filesystem authority, process supervision and security mediation.

Python owns bounded cognitive and scientific computation. It MUST NOT gain canonical write authority through repository layout, generated code, test fixtures or build scripts.

TypeScript owns presentation. It MUST NOT import Rust implementation details, open canonical storage or invoke Python directly.

## 2.3 Modular-monolith baseline

Logical boundaries MUST be enforceable inside one repository before any physical service extraction is considered.

Repository packages are not microservices. A directory or package boundary MUST NOT imply independent state authority.

## 2.4 Exact-version policy

Technology families are selected upstream. Exact compiler, runtime, package-manager and dependency versions are recorded in repository manifests, toolchain files and lockfiles.

CI MUST NOT resolve floating `latest` toolchains or dependencies.

---

# 3. Scope

SPEC-001 authorizes implementation of the following concerns only.

## 3.1 Repository bootstrap

- root repository configuration;
- source, schema, migration, test and packaging directory reservation;
- language-workspace manifests;
- minimal compile-only package fixtures;
- cross-platform path and line-ending rules;
- ignore rules for generated and local-only files.

## 3.2 Toolchain management

- Rust toolchain declaration;
- Node.js and npm declaration;
- Python and `uv` declaration;
- exact version capture;
- deterministic dependency installation;
- one lockfile per ecosystem.

## 3.3 Build command surface

- root commands for doctor, bootstrap, formatting, linting, type checking, tests, architecture checks, build and clean;
- non-interactive behavior suitable for CI;
- consistent exit codes;
- no shell-specific command assumptions.

## 3.4 Quality and architecture enforcement

- formatting and lint policies;
- strict static checking;
- repository-layout validation;
- forbidden cross-language and cross-layer dependency checks;
- lockfile and generated-file drift detection;
- license and dependency metadata checks where supported.

## 3.5 Continuous integration

- pull-request and main-branch validation;
- Windows, macOS and Linux native runners;
- deterministic installation from lockfiles;
- cross-platform build smoke tests;
- repository-policy checks;
- test and build evidence publication.

---

# 4. Non-Goals

The following work is explicitly outside SPEC-001.

| Concern | Owning follow-up |
|---|---|
| Application startup, single-instance enforcement, shutdown and recovery | SPEC-002 |
| Final Rust crate responsibilities, public ports and dependency graph | SPEC-003 |
| SQLite connections, migrations and transaction behavior | SPEC-004 |
| Canonical entity and relationship storage | SPEC-005 |
| Content-addressed retained-content implementation | SPEC-006 |
| Durable jobs, outbox, inbox, scheduling and recovery | SPEC-007 |
| Rust–Python message protocol and capability negotiation | SPEC-008 |
| Functional Python sidecar process and packaging | SPEC-009 |
| Document parsing or ingestion | SPEC-010 |
| Retrieval engines and projections | SPEC-011 |
| Proposal and approval workflow | SPEC-012 |
| Product Workspace navigation and interaction surfaces | SPEC-013 |
| Signed installers, notarization, automatic updates and release promotion | Product hardening Specs |

SPEC-001 MUST NOT introduce placeholder implementations that silently decide any of these contracts.

A minimal source file MAY exist only when a compiler or framework requires it to verify the repository build. Such a file MUST be labelled as a build fixture and MUST contain no product behavior, domain rule, persistence access, IPC contract or provider integration.

---

# 5. Concern Ownership and Implementing Boundary

## 5.1 Concern owner

The bounded concern is **Repository Foundation**.

It owns:

- source-tree shape;
- workspace membership;
- toolchain declarations;
- dependency-lock policy;
- root developer commands;
- repository-level conformance checks;
- CI workflow structure;
- build provenance emitted by those workflows.

It owns no runtime state and no domain state.

## 5.2 Implementing component

The implementation is repository infrastructure rather than an application component. Its physical boundary consists of:

```text
root manifests
root configuration
scripts/
.github/workflows/
repository-level tests
compile-only workspace fixtures
```

## 5.3 State ownership

SPEC-001 introduces no canonical database, event stream, process state, memory, context or application configuration.

Its only retained state is source-controlled build metadata:

- toolchain declarations;
- dependency manifests;
- lockfiles;
- CI definitions;
- build scripts;
- repository-policy configuration.

Generated build outputs remain non-authoritative and MUST be reproducible from the source commit plus locked inputs.

---

# 6. Proving Objective and Hypothesis

No user-facing use case is implemented by this Spec.

The proving objective is:

> A clean checkout can deterministically bootstrap, validate and build the structural ResearchOS workspace on Windows, macOS and Linux without undeclared machine state or architecture-breaking imports.

The hypothesis is falsified when any of the following is true:

- one supported OS requires an undocumented manual repository edit;
- a normal build mutates committed lockfiles;
- CI and local commands execute materially different validation paths;
- language packages can bypass declared boundaries without a failing architecture check;
- a clean checkout depends on a globally installed project-specific tool not declared by the repository;
- generated files or caches are required to understand the source of truth;
- later Specs cannot add their packages without restructuring the repository foundation.

---

# 7. Canonical Repository Topology

The implementation MUST establish the following top-level structure.

```text
researchos/
├── .github/
│   └── workflows/
│       └── ci.yml
│
├── apps/
│   └── desktop/
│       ├── frontend/                 # compile-only React/Vite fixture
│       └── src-tauri/                # compile-only Tauri host fixture
│
├── crates/                           # Rust workspace extension point; SPEC-003 owns crate semantics
│   └── README.md
│
├── python/
│   └── cognitive_runtime/            # package skeleton; SPEC-009 owns runtime behavior
│       ├── src/
│       └── tests/
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
├── scripts/
│   └── repo.mjs
│
├── specs/
│   ├── foundation/
│   ├── platform/
│   ├── cognitive/
│   ├── workspace/
│   └── verticals/
│
├── Cargo.toml
├── Cargo.lock
├── package.json
├── package-lock.json
├── pyproject.toml
├── uv.lock
├── rust-toolchain.toml
├── .node-version
├── .python-version
├── .editorconfig
├── .gitattributes
├── .gitignore
└── existing governed Markdown documents
```

## 7.1 Documentation placement

Canonical architecture and process documents remain at the repository root according to `DOCUMENTATION_ARCHITECTURE.md` and `CLAUDE.md`.

SPEC-001 MUST NOT move them into `docs/`.

A future `docs/` directory MAY contain generated or implementation-facing material only when another approved Spec owns it.

## 7.2 Reserved directories

Directories owned by later Specs are represented by a short `README.md` where Git requires a tracked file.

A reservation file MUST state:

- the future owning Spec;
- that no contract is defined there yet;
- that implementations must not be added before the owner Spec is Approved.

Empty placeholder source modules that imply domain, persistence, IPC or runtime behavior are forbidden.

## 7.3 Path and naming rules

- repository paths use lower-case `snake_case` for Rust and Python package directories;
- frontend source naming follows TypeScript/React conventions inside its package;
- development Spec filenames follow `SPEC-NNN_UPPER_SNAKE_TITLE.md`;
- all text files are UTF-8;
- committed text uses LF line endings;
- executable scripts are avoided where a cross-platform Node entry point is sufficient;
- path comparison in repository checks accounts for Windows case-insensitivity without weakening canonical path spelling.

---

# 8. Workspace and Dependency Contract

## 8.1 One repository, three ecosystems

The repository contains three independently valid dependency ecosystems coordinated through one root command surface.

```text
Cargo workspace       → Rust
npm workspace         → TypeScript / frontend tooling
uv workspace          → Python
```

No ecosystem may become the semantic owner of another.

## 8.2 Lockfile invariants

The repository MUST contain exactly:

- one root `Cargo.lock`;
- one root `package-lock.json`;
- one root `uv.lock`.

Nested lockfiles are forbidden unless a later approved packaging Spec demonstrates that an independently released artifact requires one.

Normal bootstrap, check and build commands MUST consume lockfiles without rewriting them.

Lockfile updates occur only through an explicit maintenance command and a reviewed pull request.

## 8.3 Root package manager

The TypeScript ecosystem and root task entry point use **npm workspaces**.

Rationale within this bounded contract:

- npm ships with Node.js and adds no fourth package-manager prerequisite;
- `npm ci` provides deterministic lockfile consumption;
- root scripts can invoke Rust and Python commands through a platform-neutral Node program;
- the current repository does not require advanced monorepo scheduling.

Replacing npm as the root package manager requires a SPEC-001 revision or superseding Spec because downstream automation consumes this public command contract.

## 8.4 Toolchain declarations

The repository MUST declare:

| Ecosystem | Declaration | Rule |
|---|---|---|
| Rust | `rust-toolchain.toml` | Exact supported toolchain and required components are pinned. |
| Node.js | `.node-version` and `package.json#engines` | Exact development line is declared; CI uses the same version. |
| npm | `package.json#packageManager` | Exact package-manager version is recorded. |
| Python | `.python-version` and `pyproject.toml#requires-python` | One supported minor line is pinned for the initial build. |
| uv | CI installation declaration plus lock metadata | CI installs an immutable declared uv version. |

Version-selection policy:

1. choose a version supported by Tauri 2, Vite, PyInstaller and the selected lint/test tools at implementation time;
2. pin the exact version in the repository;
3. prove all three operating-system jobs;
4. change the pin only through a dependency-maintenance pull request;
5. never encode `latest`, `stable` without an exact resolved version, or an unbounded version range in CI.

The exact values live in the files above, not in duplicated prose.

## 8.5 Dependency declaration rules

- Rust dependencies are declared in Cargo manifests and resolved by the root lockfile.
- Shared Rust dependency versions SHOULD be declared in `[workspace.dependencies]` when more than one crate consumes them.
- Frontend dependencies are declared in the owning npm workspace.
- Python dependencies are declared in the owning `pyproject.toml` package and resolved by the root uv workspace.
- Provider SDKs are forbidden in SPEC-001 fixtures.
- Database, broker, graph-server and vector-server clients are forbidden in SPEC-001 fixtures.
- Git dependencies and direct URL archives require an explicit checksum, immutable revision and security rationale.
- Unbounded wildcard dependency versions are forbidden.

---

# 9. Language Workspace Contracts

## 9.1 Rust workspace

The root `Cargo.toml` MUST define a Cargo workspace that can include:

```text
apps/desktop/src-tauri
crates/*
```

SPEC-001 MAY create only the Tauri compile fixture required to validate the workspace.

The semantic crate map under `crates/` is deferred to SPEC-003.

The workspace MUST establish:

- resolver version appropriate to the pinned Rust edition;
- shared package metadata;
- release and development profile defaults;
- warnings policy compatible with `clippy -D warnings`;
- no first-party `unsafe` code in the fixture;
- deterministic locked builds.

## 9.2 TypeScript workspace

The root `package.json` MUST register `apps/desktop/frontend` as an npm workspace.

The frontend fixture MUST:

- compile with strict TypeScript;
- build through Vite;
- contain the minimum React surface required to validate the toolchain;
- be labelled as a build fixture rather than the Workspace implementation;
- contain no domain model, persistence client, Python invocation or product navigation.

The TypeScript baseline MUST include:

- ESLint;
- Prettier;
- strict `tsconfig` settings;
- Vitest for the fixture smoke test;
- an import rule preventing source imports from `crates/`, `python/`, `migrations/` and private Tauri implementation paths.

## 9.3 Python workspace

The root `pyproject.toml` MUST define a uv workspace containing `python/cognitive_runtime`.

The package skeleton MUST use a `src/` layout and MAY contain only:

- package metadata;
- an importable empty or version-reporting module;
- test fixtures proving package discovery;
- no long-lived process;
- no model provider;
- no parser;
- no filesystem authority;
- no database access;
- no IPC behavior.

The Python baseline MUST include:

- Ruff formatting and linting;
- mypy strict checking for first-party package code;
- pytest;
- build metadata sufficient to create a wheel or equivalent package artifact;
- frozen dependency synchronization through uv.

SPEC-009 may revise package internals without changing the root workspace contract.

---

# 10. Root Developer Command Interface

The root `package.json` exposes the public developer command surface. Commands are implemented by `scripts/repo.mjs` or direct platform-neutral invocations.

Shell-specific pipelines such as Bash-only command chains MUST NOT define the canonical local workflow.

| Command | Required behavior | Mutates tracked files? |
|---|---|---:|
| `npm run doctor` | Report declared and detected tool versions, required platform prerequisites and actionable mismatches. | No |
| `npm run bootstrap` | Install/synchronize all locked dependencies required for ordinary development. | No |
| `npm run format` | Apply configured formatters to first-party source and configuration. | Yes |
| `npm run format:check` | Verify formatting without modification. | No |
| `npm run lint` | Run Rust Clippy, ESLint and Ruff under repository policy. | No |
| `npm run typecheck` | Run Rust compilation checks, TypeScript strict checking and mypy. | No |
| `npm run test` | Run workspace unit/smoke tests in all three ecosystems. | No |
| `npm run architecture` | Run repository topology, lockfile and forbidden-dependency tests. | No |
| `npm run check` | Execute the complete merge gate: format check, lint, typecheck, tests and architecture checks. | No |
| `npm run build` | Produce non-installer build artifacts for the Rust workspace, frontend fixture and Python package. | No tracked files |
| `npm run clean` | Remove only declared generated outputs and caches inside the repository. | No tracked files |
| `npm run lock:update` | Intentionally refresh all ecosystem lockfiles under explicit maintainer action. | Yes |

## 10.1 Exit and logging semantics

- success returns exit code `0`;
- policy, compile or test failure returns a non-zero exit code;
- a missing required tool is reported by `doctor` with the exact expected and detected values;
- command output identifies the failing ecosystem and subcommand;
- commands MUST NOT suppress stderr from Cargo, npm, uv or test tools;
- secrets and environment-variable values MUST NOT be printed;
- `check` stops after a failed stage unless a CI diagnostic mode explicitly requests aggregate reporting.

## 10.2 Bootstrap semantics

`npm run bootstrap` MUST be idempotent for an unchanged checkout.

It performs the equivalent of:

```text
npm ci
cargo fetch --locked
uv sync --frozen --all-groups
```

Platform prerequisite installation is not performed silently. `doctor` reports missing native prerequisites and links to the maintained development instructions.

## 10.3 Build semantics

`npm run build` proves source compatibility and artifact creation. It MUST NOT claim that ResearchOS is installable or that the desktop lifecycle works.

It produces, at minimum:

- frontend static build output;
- compiled Rust workspace artifacts or successful release-profile compilation;
- a Python package artifact;
- a machine-readable build manifest containing the source commit when available and the declared toolchain versions.

Tauri installer bundling, sidecar embedding, signing and notarization are deferred.

---

# 11. Repository-Level Architecture Enforcement

SPEC-001 establishes the mechanism for architecture checks while avoiding semantic decisions owned by SPEC-003 and later Specs.

## 11.1 Checks owned by SPEC-001

`npm run architecture` MUST fail when:

1. a required top-level path is absent;
2. an unapproved top-level source root appears;
3. a nested ecosystem lockfile appears;
4. a workspace package is not declared by its root workspace;
5. generated output or a local environment directory is tracked;
6. frontend source imports files from Rust, Python, migrations or packaging internals;
7. Python first-party code opens or declares direct access to the future canonical SQLite path in the SPEC-001 fixture;
8. a build fixture introduces provider, database-server, broker, graph-server or vector-server dependencies;
9. canonical documentation is moved away from its governed root location;
10. a CI workflow invokes a validation path unavailable through the root command interface;
11. build scripts rely on an undeclared shell or global project-specific executable;
12. committed toolchain declarations and CI setup disagree.

## 11.2 Checks delegated to later Specs

The following checks are not defined semantically by SPEC-001, but the architecture-test harness MUST be extensible to host them:

- Rust crate dependency direction — SPEC-003;
- persistence ownership and migration rules — SPEC-004;
- canonical schema conformance — SPEC-005;
- content-store integrity — SPEC-006;
- job/event idempotency — SPEC-007;
- cross-language protocol compatibility — SPEC-008;
- sidecar authority and capability isolation — SPEC-009;
- ingestion and retrieval conformance — SPEC-010 and SPEC-011;
- approval-path enforcement — SPEC-012;
- Workspace boundary rules — SPEC-013.

## 11.3 No mandatory local Git hook

Repository correctness is enforced by repeatable commands and CI, not by an unversioned local hook.

Optional hooks MAY call the canonical commands, but merging MUST NOT depend on a developer having installed them.

---

# 12. Continuous Integration Contract

## 12.1 Triggers

The primary workflow runs on:

- pull requests targeting `main`;
- pushes to `main`;
- manual dispatch for diagnosis.

Release, signing, scheduled dependency updates and installer publication are not part of SPEC-001.

## 12.2 Workflow jobs

The workflow MUST define at least the following logical jobs.

### A. Repository policy

Runs on Ubuntu and performs:

- checkout with immutable action versions;
- toolchain setup from repository declarations;
- `npm run bootstrap`;
- `npm run format:check`;
- `npm run lint`;
- `npm run typecheck`;
- `npm run test`;
- `npm run architecture`;
- lockfile and generated-drift verification;
- source-tree dirty check after all non-mutating commands.

### B. Cross-platform build smoke

Runs as a native matrix on:

```text
ubuntu-latest
windows-latest
macos-latest
```

Each matrix entry performs:

- platform prerequisite setup required for compile-time Tauri validation;
- toolchain setup from repository declarations;
- frozen dependency bootstrap;
- `npm run doctor`;
- `npm run build`;
- minimal package/import smoke tests;
- artifact-manifest upload.

### C. Documentation and configuration integrity

May be a separate job or part of repository policy. It verifies:

- relative Markdown links for governed repository documents;
- balanced fenced code blocks;
- valid JSON, TOML and YAML syntax;
- Spec catalogue links and registered paths;
- no duplicate Spec identifier.

## 12.3 CI security

- pull-request CI MUST require no repository secret;
- workflows use least-privilege permissions, normally `contents: read`;
- third-party and GitHub actions MUST be pinned to immutable commit SHAs;
- CI MUST NOT execute scripts downloaded through unauthenticated pipes;
- caches MUST be keyed by OS, toolchain declaration and relevant lockfile hashes;
- cache hits MUST NOT replace lockfile validation;
- fork or untrusted-branch code MUST NOT receive signing, release or provider credentials;
- generated artifacts are retained for a bounded diagnostic period.

## 12.4 Required merge checks

The protected-branch configuration should require:

- repository policy;
- Linux build smoke;
- Windows build smoke;
- macOS build smoke.

Branch protection configuration may be applied manually if the connector or repository plan cannot provision it, but the required check names MUST be documented.

---

# 13. Commands, Queries, Events, Jobs, Processes, Proposals and Effects

This Spec introduces no ResearchOS Application Command, Query, Domain Event, Job, Process Instance, Proposal or external Effect.

Repository commands defined in Section 10 are developer-tool operations, not domain or application messages.

The CI workflow is a build process, not the ResearchOS durable Process Runtime.

No emitted CI event may be represented as a Domain Event.

---

# 14. Ports and Adapters

SPEC-001 defines build-time interfaces only.

## 14.1 Public build port

The root npm command interface is the public entry point consumed by:

- developers;
- CI;
- future packaging workflows;
- coding agents;
- local automation.

Downstream automation MUST call this interface or an explicitly documented language-specific subcommand. It MUST NOT depend on private implementation details of `repo.mjs`.

## 14.2 Tool adapters

The build system adapts to:

- Cargo;
- npm;
- uv;
- Rust format/lint/test tools;
- TypeScript format/lint/test tools;
- Python format/lint/test tools;
- GitHub Actions native runners.

These adapters have no runtime authority.

## 14.3 Future extension port

Later Specs add checks through one of:

- a new root command;
- a registered subcommand invoked by `npm run check`;
- a test suite under the owning ecosystem;
- a repository architecture test under `tests/architecture`.

A later Spec MUST NOT replace the root command interface silently.

---

# 15. Logical and Physical Data Schema

SPEC-001 introduces no product data schema and no canonical migration.

Its configuration schema consists of version-controlled manifests and lockfiles.

## 15.1 Required configuration records

The repository MUST make the following information machine-readable:

- workspace members;
- toolchain versions;
- package-manager version;
- dependency versions and integrity data;
- root commands;
- supported CI operating systems;
- build profile;
- artifact-manifest fields.

## 15.2 Build manifest

Each CI build-smoke job emits a JSON build manifest with at least:

```json
{
  "schema_version": 1,
  "source_revision": "<git-sha>",
  "operating_system": "<runner-os>",
  "architecture": "<runner-arch>",
  "rust_toolchain": "<exact>",
  "node_version": "<exact>",
  "npm_version": "<exact>",
  "python_version": "<exact>",
  "uv_version": "<exact>",
  "cargo_lock_sha256": "<digest>",
  "npm_lock_sha256": "<digest>",
  "uv_lock_sha256": "<digest>",
  "result": "success"
}
```

This manifest is build evidence, not canonical ResearchOS state.

## 15.3 Migration behavior

No database migration exists under this Spec.

Changes to repository layout or command names after approval require:

1. compatibility analysis for downstream Specs and CI consumers;
2. a documented transition in SPEC-001's change log;
3. synchronized updates to root commands and workflows;
4. a deprecation window when a command is already consumed by implementation automation.

---

# 16. Workspace, IPC, Runtime-Role and External-Interface Contract

## 16.1 Workspace

The React package is a compile fixture only. It defines no product interaction contract.

## 16.2 Desktop runtime

The Tauri package is a compile fixture only. Startup, shutdown, single-instance, paths and recovery are deferred to SPEC-002.

## 16.3 IPC

No Rust–Python IPC transport, schema or handshake is introduced. The `schemas/ipc/` path is reserved for SPEC-008.

## 16.4 Python runtime

No long-lived Python process is started. Package import and build are the only accepted Python runtime evidence.

## 16.5 External interfaces

The build may access public package registries during dependency installation. It exposes no ResearchOS server, local port or external API.

---

# 17. Authorization, Approval and Data Classification

## 17.1 Repository authority

Only reviewed source changes may modify:

- toolchain declarations;
- lockfiles;
- CI workflows;
- root build commands;
- repository architecture rules.

## 17.2 Secrets

SPEC-001 requires no application secret.

The repository MUST NOT contain:

- provider keys;
- signing certificates;
- production tokens;
- local absolute paths;
- personal datasets;
- generated credential files.

Environment example files, when introduced later, MUST contain names and safe placeholders only.

## 17.3 Data classification

All SPEC-001 fixtures are synthetic and non-sensitive.

CI logs and artifacts are classified as project-internal build metadata. They MUST exclude environment dumps, home-directory listings and secret values.

## 17.4 Approval

Changing a lockfile is an explicit reviewed action. CI-generated dependency changes MUST arrive through a pull request and never push directly to `main`.

---

# 18. Failure, Idempotency, Retry and Recovery

## 18.1 Failure model

Build failures are classified as:

- toolchain mismatch;
- dependency-resolution failure;
- lockfile drift;
- formatting or lint failure;
- type or compile failure;
- test failure;
- architecture-policy failure;
- platform-prerequisite failure;
- artifact-generation failure;
- CI infrastructure failure.

The root command reports the category and failing subcommand.

## 18.2 Idempotency

For an unchanged checkout:

- `doctor`, `format:check`, `lint`, `typecheck`, `test`, `architecture`, `check` and `build` are non-mutating with respect to tracked files;
- `bootstrap` may update local caches or environments but not tracked manifests or lockfiles;
- `clean` may remove only declared generated paths;
- rerunning a successful command yields the same policy result.

## 18.3 Retry

CI jobs MAY be manually retried for infrastructure failures.

Tool or test failures MUST NOT be hidden by unconditional retries.

Network package download retries are bounded by the underlying package manager and must ultimately preserve lockfile integrity.

## 18.4 Recovery

A failed bootstrap or build must be recoverable by:

1. correcting the reported prerequisite or source issue;
2. running `npm run clean` when generated output may be inconsistent;
3. rerunning `npm run bootstrap`;
4. rerunning the failed canonical command.

Deleting lockfiles is not an accepted recovery procedure.

---

# 19. Observability and Audit Evidence

Each CI run preserves:

- source revision;
- workflow revision;
- runner OS and architecture;
- declared and detected tool versions;
- lockfile digests;
- command start, result and duration;
- failing stage and exit code;
- build manifest;
- bounded build artifacts when useful for diagnosis.

Local commands SHOULD print the same stage names used by CI so failures are reproducible outside GitHub Actions.

No prompt, document content, personal data or provider metadata exists in this Spec's observability surface.

---

# 20. Test Specification

## 20.1 Unit tests

The repository orchestrator MUST have tests for:

- command registration;
- platform-specific executable resolution;
- exit-code propagation;
- version comparison;
- generated-path cleanup allowlist;
- build-manifest serialization.

## 20.2 Property tests

Where practical, repository-policy tests cover:

- arbitrary nested lockfile placement is rejected;
- clean never targets a path outside approved generated directories;
- path normalization produces the same canonical repository-relative path across supported OS separators;
- build-manifest field order or whitespace does not affect semantic validation.

## 20.3 Architecture tests

Architecture tests MUST prove all Section 11.1 rules.

They MUST be runnable locally through `npm run architecture` without GitHub-specific APIs.

## 20.4 Integration tests

A temporary clean checkout test verifies:

```text
checkout
  ↓
doctor
  ↓
bootstrap
  ↓
check
  ↓
build
  ↓
verify git working tree is clean
```

The test runs with no pre-existing project cache as part of at least one CI path.

## 20.5 Cross-platform acceptance tests

On each target runner:

- declared tool versions are installed;
- frozen dependency installation succeeds;
- path handling succeeds;
- frontend build succeeds;
- Rust workspace compilation succeeds;
- Python package build and import succeed;
- build manifest validates;
- the source tree remains clean.

## 20.6 Negative acceptance fixtures

Repository tests include fixtures proving failure for:

- an extra nested `Cargo.lock`;
- an undeclared npm workspace;
- a forbidden frontend import;
- a fixture dependency on a provider SDK;
- a CI toolchain version that differs from the declaration;
- a clean target outside the generated-output allowlist.

---

# 21. Implementation Tasks and Dependency Order

Implementation proceeds in the following order.

## Task 1 — Root repository policy

Create:

- `.editorconfig`;
- `.gitattributes`;
- `.gitignore`;
- root naming and generated-path policy;
- reserved-directory ownership READMEs.

## Task 2 — Toolchain declarations

Create and pin:

- `rust-toolchain.toml`;
- `.node-version`;
- `.python-version`;
- npm package-manager declaration;
- documented native Tauri prerequisites.

## Task 3 — npm root workspace and orchestrator

Create:

- root `package.json`;
- npm workspace declaration;
- `scripts/repo.mjs`;
- orchestrator tests;
- root command interface.

## Task 4 — TypeScript compile fixture

Create the minimal React, TypeScript, Vite and Vitest package under `apps/desktop/frontend`.

Apply strict type checking, linting and forbidden-import policy.

## Task 5 — Cargo workspace and Tauri compile fixture

Create:

- root Cargo workspace;
- compile-only `apps/desktop/src-tauri` package;
- workspace formatting and lint policy;
- no semantic crates under `crates/` before SPEC-003.

## Task 6 — uv workspace and Python package fixture

Create:

- root Python workspace metadata;
- `python/cognitive_runtime` package skeleton;
- Ruff, mypy and pytest configuration;
- package build/import smoke test.

## Task 7 — Lockfiles

Generate and commit:

- `Cargo.lock`;
- `package-lock.json`;
- `uv.lock`.

Verify normal bootstrap and build do not modify them.

## Task 8 — Repository architecture tests

Implement Section 11 checks and negative fixtures.

## Task 9 — CI workflow

Implement native policy and three-OS build-smoke jobs with immutable action pins, least privilege and build-manifest artifacts.

## Task 10 — Developer documentation

Document:

- prerequisites;
- first checkout;
- canonical commands;
- platform-specific Tauri packages;
- failure recovery;
- required merge-check names.

Update `README.md` only with concise navigation to the implementation instructions; do not duplicate this Spec.

## Task 11 — Acceptance execution

Run the complete clean-checkout cycle on Windows, macOS and Linux and attach evidence to the implementing pull request.

No later task may begin before its required upstream task produces a working public contract.

---

# 22. Completion Criteria

SPEC-001 reaches **Implemented** when:

1. every required repository path and manifest exists;
2. all three lockfiles are committed and consumed in frozen mode;
3. the root command interface is implemented and documented;
4. `npm run check` passes locally on a supported developer environment;
5. repository architecture tests and negative fixtures pass;
6. native Windows, macOS and Linux CI build-smoke jobs pass;
7. all non-mutating commands leave the source tree clean;
8. no product behavior from SPEC-002 or later has been introduced;
9. build manifests are uploaded for all target runners;
10. downstream Specs can reference stable workspace paths and command names.

SPEC-001 reaches **Validated** when:

1. a clean checkout succeeds on documented clean target environments for all three operating systems;
2. a second maintainer or independent clean environment reproduces the bootstrap and build contract;
3. no undocumented global tool or manual repository edit is required;
4. the repository remains structurally extensible for SPEC-002 and SPEC-003 without moving existing foundations;
5. `SPEC_CATALOG.md` and `ROADMAP.md` record the accepted evidence and unlock dependent Specs according to their gates.

---

# 23. Approval Review Checklist

Before moving this Spec to Approved, review must confirm:

## Authority review

- no rule contradicts `SOFTWARE_ARCHITECTURE.md` or `TECHNICAL_ARCHITECTURE.md`;
- npm selection is bounded to repository orchestration and does not create runtime authority;
- no placeholder encodes a later Spec's semantics.

## Dependency review

- SPEC-002 and SPEC-003 receive stable paths and commands;
- no later Spec is required to implement SPEC-001;
- reserved directories state their future owners.

## Data and migration review

- no product schema or migration is introduced;
- build metadata is clearly non-canonical;
- lockfile transition rules are explicit.

## Security review

- CI requires no secrets;
- actions and tools are immutably pinned;
- fixtures contain no sensitive data;
- clean and script execution cannot escape the repository allowlist.

## Testability review

- every merge gate runs locally;
- negative architecture fixtures exist;
- all three target operating systems have an acceptance path.

## Local-first and packaging review

- no server or container is required;
- build success is not confused with installer readiness;
- future sidecar and installer packaging remain possible without repository restructuring.

---

# 24. Explicit Deferrals

The following are intentionally deferred and MUST NOT be treated as missing SPEC-001 implementation:

- functional desktop startup and shutdown;
- application data directories;
- SQLite and migrations;
- canonical records;
- document storage;
- durable job runtime;
- IPC schemas and sidecar process startup;
- PyInstaller or Nuitka output;
- Tauri installer bundling;
- code signing and notarization;
- automatic updates;
- provider credentials;
- model, parsing, embedding or retrieval dependencies;
- production Workspace design;
- benchmark and evaluation datasets;
- Docker-based development as a mandatory path;
- remote caches or build farms;
- release channels and semantic-version policy.

Each deferral is owned by the later Spec or roadmap phase named in Section 4.

---

# 25. Downstream Public Contract

After approval and implementation, downstream Specs may rely on the following stable contract:

1. repository paths declared in Section 7 exist;
2. root toolchain and lockfiles are authoritative for development builds;
3. `npm run doctor`, `bootstrap`, `check`, `build`, `clean` and `architecture` are available on all supported developer platforms;
4. CI runs repository policy plus native Windows, macOS and Linux build smoke;
5. new Rust crates may be added under `crates/` by SPEC-003;
6. SPEC-002 may replace the Tauri compile fixture with governed lifecycle behavior without changing the root workspace;
7. SPEC-009 may replace the Python package fixture with the governed sidecar without changing the root uv workspace;
8. later Specs may extend architecture checks without bypassing the root command interface;
9. no later Spec may add a second lockfile or alternative root task runner without revising this contract.

---

# 26. Change Log

| Date | Status | Change |
|---|---|---|
| 2026-07-09 | Review | Initial complete specification authored from the approved Software and Technical Architecture baselines. |
