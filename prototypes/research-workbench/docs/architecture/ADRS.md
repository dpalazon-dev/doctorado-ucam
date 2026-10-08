# ADRs — Research Workbench

**Architecture decision record for v0.1**
**Date:** 1 October 2026
**Status of all ADRs:** Accepted (baseline 2026-10-01). Adoption for implementation is recorded in INTENT.md by the user's instruction to create the environment and fully implement it in phases. Acceptance of the design does not prove product execution.
**Basis:** `work/spec.md` and `outputs/Research_Workbench_Plan_Multiagente.md`.

This record defines the target architecture for v0.1; the plan divides delivery into increments, and the first pilot covers only the library, import, and reading. Phase and knowledge decisions are implemented in later increments before v0.1 is declared complete. Frameworks and versions must be fixed when the repository is created: the technical references in this document justify capabilities, but do not prove installation or compatibility in this environment.

## ADR-001 — Local application for one researcher

**Status:** Accepted (baseline 2026-10-01)
**Context:** The product serves a researcher who needs to read, capture, and retrieve knowledge without relying on accounts, a network, or services. Collaboration and cloud synchronization are outside the initial scope.

**Alternatives:** Web application with a backend; local application with optional remote storage from the start; single-user local desktop application.

**Decision:** Build a local desktop application with no login or server whose main workflow can be completed offline. Store the library in a user data folder, separate from the program directory. Allow only one writing instance per library.

**Consequences:** This reduces operational and external dependencies and makes it easier to keep private documents on the device. Synchronization, collaboration, and access from several devices would require new contracts and conflict resolution.

**Risks:** Loss of the device or disk; a library copied while in use may be incomplete; a network-synchronized folder may not meet SQLite requirements. Verified backups and restoration are part of v0.1.

**Validation:** Complete import, reading, close, and reopen without a network; test that a second writing instance is blocked and restore from a verified backup.

**Revisit if:** Collaboration, synchronization, multi-user access, or network library storage is approved.

## ADR-002 — Tauri 2, React/TypeScript, Rust, and SQLite

**Status:** Accepted (baseline 2026-10-01)
**Context:** The product needs a conventional desktop experience, a local core that controls files and rules, and portable relational storage. The plan calls for Tauri 2, React/TypeScript, Rust, and SQLite; the development runtime is not complete, so dependencies must not be described as installed.

**Alternatives:** Local web application with a server; Electron with a TypeScript backend; fully native desktop application in Rust; Tauri 2 with a React/TypeScript frontend and Rust backend.

**Decision:** Use Tauri 2 as the shell and IPC layer, React with TypeScript and Vite for the interface, Rust for services and domain logic, and SQLite through `rusqlite` as the local database. The UI starts with shadcn/ui on Radix UI and Tailwind, with React Hook Form and Zod for forms. Package PDF.js and its worker locally for reading. Compile `rusqlite` with bundled SQLite and backup support; verify creation of a real FTS5 table before relying on FTS5 in the artifact. Audit package versions, licenses, APIs, and lockfiles together before implementation; this ADR does not select unverified patches. [shadcn/ui manual installation](https://ui.shadcn.com/docs/installation/manual), [React Hook Form](https://www.react-hook-form.com/), [Zod](https://zod.dev/), [PDF.js Getting Started](https://mozilla.github.io/pdf.js/getting_started/?lang=en), [rusqlite bundled](https://docs.rs/crate/rusqlite/latest/source/README.md), [rusqlite features](https://docs.rs/crate/rusqlite/latest/features).

**Consequences:** This separates the UI from local capabilities and allows a compact Windows application compared with bundling a Chromium runtime. PDF.js also requires packaging and serving its resources/worker from the bundle. `rusqlite` exposes SQLite transactions and types, but connection and serialization strategies remain the application's responsibility. Tauri documents IPC commands; PDF.js separates its display API from its worker; rusqlite documents explicit transactions. [Tauri IPC](https://v2.tauri.app/concept/inter-process-communication/), [PDF.js Getting Started](https://mozilla.github.io/pdf.js/getting_started/?lang=en), [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/).

**Risks:** Interoperability and resource packaging must be tested in Windows WebView2. Dependency updates can change APIs; Rust MSVC and Visual Studio C++ Build Tools are Windows build requirements. The current audit found VS 2022 and WebView2, but Rust is not installed in PATH or in the observed Cargo directory.

**Validation:** Create an installed Tauri window, open a fixture PDF with local resources, invoke a Rust command, and open a temporary SQLite database; check the Windows build with lockfiles. Do not claim these checks until they have been run.

**Revisit if:** The actual installer fails the document's size, accessibility, performance, or support requirements, or compatibility with pinned versions cannot be maintained.

## ADR-003 — SQLite as the single structured authority

**Status:** Accepted (baseline 2026-10-01)
**Context:** Paper and concept views must represent the same entities and retain integrity when the program is reopened. Markdown and JSONL are for exchange and human reading, not parallel canonical databases.

**Alternatives:** Markdown/JSONL as the source; SQLite plus a graph store; canonical SQLite with managed binary files and rebuildable indexes/exports.

**Decision:** SQLite is the source of truth for entities, relations, provenance, workflow, revisions, and file metadata. Store PDFs and other binaries in the library; FTS5 indexes and projections are derived. Markdown/JSONL exports document data and versions and are never silently reimported to overwrite the database.

**Consequences:** Transactions can keep relational entities and associations together, and the search index can be rebuilt. Foreign keys must be enabled and checked per connection because SQLite requires enforcement to be enabled explicitly. WAL can improve reader concurrency but permits only one writer and does not work on network filesystems. [Foreign keys](https://www.sqlite.org/foreignkeys.html), [WAL](https://www.sqlite.org/wal.html), [transactions](https://www.sqlite.org/lang_transaction.html).

**Risks:** A single authority makes migrations, backups, and verifiable exports essential; arbitrary SQL written manually from the UI would break domain boundaries.

**Validation:** Check foreign_keys when opening each connection; run `foreign_key_check` and `integrity_check` during validation/restoration; execute `CREATE VIRTUAL TABLE ... USING fts5(...)` in a binary capability test and rebuild the index from canonical rows; export and reimport into a temporary library while preserving UUIDs, links, and provenance. Close the Cargo feature choice only after auditing the pinned `rusqlite`/`libsqlite3-sys`, its FTS5 build configuration, and the result of this test; the word `bundled` alone is not a functional check.

**Revisit if:** Measured volume or query patterns demonstrably exceed SQLite's limits, or there is a real requirement for a multi-user backend.

## ADR-004 — Global concepts, links, and locatable provenance

**Status:** Accepted (baseline 2026-10-01)
**Context:** Capture happens in the context of a paper, but a concept can recur across many sources. Each KnowledgeItem must retain its type, scope, attribution, and locator; an inference made by the user must not become an assertion attributed to the source.

**Alternatives:** Duplicate every concept per paper; share concepts and mix all their claims; keep globally identified concepts and associate claims/evidence with their sources and contexts.

**Decision:** Concept has a stable UUID and may be reused across papers; its name, alias, and definition are editable. KnowledgeItem, Evidence, Question, and Relation retain their own source and provenance links. Reuse or a suggested match is explicit; textual equality does not merge meanings.

**Consequences:** The paper screen preserves the reading sequence, while the concept view adds links to the same records. Contradictions remain separate claims with their own scope and sources. A locator points to a specific documentId/version; if it does not yet exist, it is marked pending.

**Risks:** The same term can have different meanings; an aggregate view can hide conditions if it summarizes without showing scope; removing a source can invalidate locators but must not automatically delete shared knowledge.

**Validation:** Link two fixture papers to the same Concept UUID; rename the concept and verify that links do not change; check that each claim opens its paper and locator; confirm that the view retains incompatible claims without synthesizing them.

**Revisit if:** New domains require context-dependent concept identity or a richer semantic-equivalence policy.

## ADR-005 — Workflow gates evaluated in the domain

**Status:** Accepted (baseline 2026-10-01)
**Context:** PRE, P1, and P2 guide work in v0.1; P3 and P4 are enabled later. Completing a phase means its requirements were processed, not that scientific truth or certainty was established.

**Alternatives:** Allow the frontend alone to decide advancement; hard-code rules in screen components; version phase definitions and validate them repeatedly in Rust domain services.

**Decision:** Each PhaseDefinition stores its version, objective, questions, allowed resolutions, and requirements. The Rust service evaluates and reevaluates the gate within the advance operation. Empty fields do not count as processed; explained `unknown` may satisfy completeness when the definition permits it. A gate result indicates operational completeness, not epistemic validation.

**Consequences:** Updating a template does not retrospectively change the meaning of completed phases. The UI can explain failures with concrete requirements, and the backend prevents inconsistent jumps.

**Risks:** Overly rigid criteria force artificial content; vague criteria make transitions opaque. Rules must not require a correct conclusion or automatically approve claims.

**Validation:** Test empty answers, explained unknown, captured answers, and incomplete outputs; call advance with a stale gate and verify that the backend rejects it; verify that the phase and its version are recorded.

**Revisit if:** Use shows that requirements are insufficient, differ materially between domains, or definitions need to evolve without changing historical work.

## ADR-006 — Managed documents with stable identity

**Status:** Accepted (baseline 2026-10-01)
**Context:** PDFs must remain available if the user moves the original file or removes its source code. Original absolute paths can disclose private information and are not document identity.

**Alternatives:** Open a PDF from its original path; store all binaries in SQLite; copy it into a managed library and keep a relational record keyed by UUID, hash, and relative path.

**Decision:** Import through staging inside the library: record durable intent, copy, calculate SHA-256, and verify readability; promote to an internal path derived from identifiers; then commit Paper/Document in SQLite. A startup reconciler repairs interrupted operations without deleting files owned by an unknown process. A locator refers to `documentId` and version, not merely a filename or original path.

**Consequences:** Binaries remain separate from relational data but under a portable root. SQLite retains the original name, date/hash, and, when appropriate, the original path for diagnostics only. SQLite and the filesystem do not share an atomic commit, so import and deletion require an explicit intent and recovery protocol.

**Risks:** A crash or lack of space between steps can leave staging files or unregistered files; moving a library outside the application can break relative paths; SHA-256 identifies byte changes, not scientific validity.

**Validation:** Simulate interruption at every import boundary, reopen, and verify reconciliation; move the original file after import; compare SHA-256 and internal path; ensure recovery does not delete files owned by others.

**Revisit if:** Remote sources, OCR, external storage, visible multiple versions, or changes to locator format are introduced.

## ADR-007 — Manual capture before automated assistance

**Status:** Accepted (baseline 2026-10-01)
**Context:** The initial test is to process a real survey manually from PRE through P2 and assess usefulness, retrieval, and traceability before adding models. A system inference is not evidence.

**Alternatives:** Add extraction/LLM to the first workflow; defer all models until a traceable manual process exists; allow automated suggestions without an acceptance control.

**Decision:** v0.1 works without AI, agents, RAG, embeddings, or a network connection. Typed capture and locators must be sufficient. Any future assistance produces separate proposals with model, date, and provenance; the researcher explicitly accepts or rejects each proposal.

**Consequences:** First validate whether the data model and workflow are useful without cost, text exposure, or network dependencies. A later integration may be an optional adapter, not the authority for asserting knowledge.

**Risks:** Manual entry may be slow; creating integration interfaces before observing real tasks adds abstraction without evidence.

**Validation:** Complete a test survey offline, reopen it, and export knowledge items with their sources; confirm no remote requests in the core workflow.

**Revisit if:** Usage identifies a repeated, bounded task that can be assisted without removing control, provenance, or offline operation.

## ADR-008 — Small, typed ontology that can be extended with review

**Status:** Accepted (baseline 2026-10-01)
**Context:** Classification supports capturing claims, evidence, questions, interpretations, and relations without turning the application into an ontology editor. Types and relations must retain domain constraints.

**Alternatives:** Free-form notes without types; a large user-editable vocabulary; a small core with stable enums and versioned schema changes.

**Decision:** Include a short set of KnowledgeItem and Relation types, global concepts, aliases/domain, and provenance links. Creating a relation validates endpoints, direction, and typed compatibility. Extensions use migrations/versions; free text cannot replace a core type.

**Consequences:** Forms can guide users who do not know ontology theory; exports retain a documented vocabulary and aggregate queries are reproducible.

**Risks:** The initial taxonomy may not cover a specific field; enum changes affect import/export and migrations; normalizing labels does not prove equivalence.

**Validation:** Check valid and invalid relations in the domain service and constraints; round-trip types with UUIDs and version-compatible unknown values; verify that relation creation does not allow self-relations/cycles where the type prohibits them.

**Revisit if:** Real use produces repeated categories that cannot be expressed, or vocabulary needs domain-specific extension through a versioned contract.
## ADR-009 — Modular monolith with pragmatic hexagonal boundaries

**Status:** Accepted (baseline 2026-10-01)
**Context:** The application is a local executable with one database and one researcher. Separating rules from UI and persistence helps protect consistency; microservices, mediators, or a complete architecture framework would not provide a useful operational boundary here.

**Alternatives:** Screen modules mixing logic and SQL; strict hexagonal architecture with a framework, DI container, and generic adapters for everything; modular monolith with use cases, a small domain, and repositories where they isolate persistence or improve testing.

**Decision:** Use a Rust monolith organized around domain modules/use cases. Tauri commands are input adapters; services validate and orchestrate; domain entities/values contain invariants; repositories encapsulate queries when that improves clarity or enables testing/change. Boundaries must be concrete and low-cost. In React, use local `useReducer` for transient state when needed and derive computable values. Initial navigation is a discriminated `ShellView`, without React Router; structured forms use React Hook Form/Zod; plain-text bodies use `<textarea>` in v1, without TipTap. Defer Zustand, TanStack Query, and a global cache until observed need justifies them. Do not add a global state/routing framework, DI, event bus, ORM, CQRS, or DAO by default. React recommends keeping state minimal and deriving computable values. [Thinking in React](https://react.dev/learn/thinking-in-react), [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/), [Vitest](https://vitest.dev/guide/).

**Consequences:** Few steps are needed to follow an operation from IPC to transaction; modules can be tested with temporary SQLite or minimal adapters. The architecture does not require separating reads from writes in infrastructure: commands/use cases may organize queries and mutations in the same process and database. React recommends keeping UI state minimal and computing derived values. [Thinking in React](https://react.dev/learn/thinking-in-react).

**Risks:** Oversized modules may recouple rules; interfaces added without a real substitute make changes more expensive; avoiding patterns on principle is also a mistake when a concrete need emerges.

**Validation:** Check that gates, phase changes, archive, import, and merge do not depend on React components; each command has an identifiable use case; and the UI neither executes SQL nor accesses the general filesystem.

**Revisit if:** Several real adapters, different backends, repeated transaction complexity, or team size require formal boundaries.

## ADR-010 — Typed and versioned Tauri IPC, without REST API

**Status:** Accepted (baseline 2026-10-01)
**Context:** UI and backend ship together as a local application. There is no remote client or need for an HTTP server; commands must limit capabilities and return errors with a stable structure.

**Alternatives:** Local REST server; frontend access to files/SQL; Tauri commands with explicit DTOs and a versioned contract.

**Decision:** Use Tauri commands for UI requests to Rust with canonical serde DTOs, stable UUIDs, RFC3339 UTC dates, expected revisions, and typed results/errors. Generate TypeScript types from Rust DTOs to avoid two independent definitions; `ts-rs` is the selected tool. Check type/serde representations when pinning the toolchain, before implementing features. If that check reveals incompatibility, revise this ADR before implementation. The final frontend API (`LibraryApi`, etc.) declares agreed operations at bootstrap and wraps `invoke`; do not leave fictitious contracts as placeholders. Domain modules do not know Tauri. Reserve events for one-way lifecycle/progress notifications, not mutations requiring confirmation. Version the exportable contract and any schema compatibility. Do not deploy REST or accept arbitrary file/SQL paths from the UI. [Tauri IPC](https://v2.tauri.app/concept/inter-process-communication/), [Runtime Authority](https://v2.tauri.app/security/runtime-authority/), [ts-rs](https://docs.rs/ts-rs/latest/ts_rs/).

**Consequences:** Bounded commands are easy to authorize through capabilities/scopes and test with an adapter. Tauri documents that `invoke` serializes arguments/results using a JSON-RPC-like protocol and Events are one-way messages; the product must still define its own DTOs and errors. [Tauri IPC](https://v2.tauri.app/concept/inter-process-communication/), [Runtime Authority](https://v2.tauri.app/security/runtime-authority/).

**Risks:** Concurrent changes to Rust DTOs and TypeScript can desynchronize them; string errors without codes make the UI brittle; an overly general command becomes an excessive capability API.

**Validation:** Cross-language Rust/TypeScript serialization test; known and unknown error tests; rejection of stale `expectedRevision`; audit capabilities for the minimum permitted commands.

**Revisit if:** An external client, independent frontend/backend updates, or long-lived bidirectional communication is required.

## ADR-011 — Stable desktop identity and offline Windows installation

**Status:** Accepted (baseline 2026-10-01)
**Context:** Acceptance requires users to open the application from Start without Node, Rust, Git, Codex, a terminal, or a checkout, and work offline. Installation must not own or delete the data directory.

**Alternatives:** Web application/local server distribution; MSI or portable installation without stable identity; per-user NSIS installer with a fixed bundle identity and offline runtime.

**Decision:** Before the first distributable artifact, fix the bundle identifier, name, version, Windows x64 architecture, and stable icons. Produce a per-user NSIS installer; separate binaries/shortcuts from the persistent library; bundle PDF.js locally and configure `offlineInstaller` for WebView2 while offline installation remains a requirement. The uninstaller removes the program and shortcuts but preserves the library and backups. The current working directory does not determine data paths. The Tauri guide covers NSIS and `offlineInstaller`, noting an approximate 127 MB increase and offline WebView2 installation. [Tauri Windows installer](https://v2.tauri.app/distribute/windows-installer/), [Tauri configuration](https://v2.tauri.app/reference/config/).

**Consequences:** Updates can identify the same installation and open the same library; the offline runtime materially increases download size. Installation, update, uninstall, and reinstall must be tested in a clean Windows environment. WebView2 on the development machine does not prove runtime independence.

**Risks:** Changing identity can create duplicate installations; a poorly scoped NSIS script could delete data; external resources/CDNs would break offline mode.

**Validation:** Install on clean Windows 11 x64 without development tools, WebView2, or network; launch from Start, import/read/resume; update while preserving UUIDs and PDFs; uninstall and verify the library remains available for reinstall. Check bundled resources and no downloads in the offline workflow.

**Revisit if:** Windows is no longer the primary platform, distribution moves to the Store, an online runtime is chosen, or data ownership/location changes.

## ADR-012 — Coordinated single writer, protected migrations, and filesystem intents

**Status:** Accepted (baseline 2026-10-01)
**Context:** SQLite supports parallel reads but serializes writers; WAL does not enable multiple writers and is unsuitable for network filesystems. Importing/moving a file and committing SQL are not one transaction. A failure must not show “Saved” or leave the library partially updated.

**Alternatives:** Concurrent writes from every command/window; shared connection without a policy; local actor/worker that serializes SQLite writes and coordinates file operations through durable intents, revisions, and recovery.

**Decision:** Allow one writing instance per library and use an actor/worker on its own thread as the sole owner of the synchronous `rusqlite` connection. Asynchronous IPC awaits that worker; no blocking SQLite call runs on the UI/runtime thread. Serialize all mutations; reads use the same connection/owner. Select WAL, synchronous=FULL, foreign_keys=ON, busy_timeout=5000 ms, and a queue capped at 64 jobs. Verify these decisions with durability, backup, and shutdown tests before delivery; they do not enable multiple writers. Do not use the library on a network drive. Long filesystem jobs copy/hash on a background worker and coordinate results through durable intent and a short SQL unit of work; do not hold a transaction during PDF/backup copies or while waiting for a user. Every entity update uses `expectedRevision` to reject stale edits without deleting pending UI text. Multi-step filesystem operations (import, restore, future deletion) record intent/idempotency key and recoverable state, execute filesystem and SQL in a defined order, and reconcile at startup without deleting items of unknown ownership. Versioned/checksummed migrations run under exclusive write access, with a consistent snapshot of SQLite and relevant documents, verification, and rejection of a future schema; failure leaves the previous library recoverable. Backup uses SQLite Online Backup API or an equivalent consistent method; never blindly copy the database while WAL is active. [SQLite transactions](https://www.sqlite.org/lang_transaction.html), [WAL](https://www.sqlite.org/wal.html), [Online Backup API](https://www.sqlite.org/backup.html), [rusqlite Transaction](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html), [rusqlite backup](https://docs.rs/rusqlite/latest/rusqlite/backup/).

**Consequences:** There is a clear order for saving and signaling confirmation; an operation can recover after unexpected shutdown without pretending the database and directories are atomic. This adds operation tables, revisions, manifests, and failure tests; UI access waits for a commit response before showing Saved.

**Risks:** The worker can become a bottleneck if long jobs run inside a transaction; forced shutdowns, full disks, and permissions need test scenarios. When enabled, WAL creates `-wal`/`-shm` files that are part of the state while the database is open. The actor needs orderly shutdown, cancellation, and errors returned to the requester; it must not become a generic bus.

**Validation:** Concurrent changes with the same revision; forced shutdown after each import step; full disk/backup/migration failure; restore into a new folder with hash/integrity checks; reject incompatible downgrade; only one process can write. Verify the selected WAL/FULL settings and pass backup/recovery checks before accepting implementation.

**Maintenance scope and sequence:** The product document considers concept merging and permanent deletion with dependency review. The first pilot plan defers those, along with typed knowledge, gates, export, and user backup. Therefore the pilot implements reversible archive/restore, expected revisions, and import/migration intents; it offers neither merge nor permanent deletion. In a later maintenance increment, any merge/deletion requires a dependency screen, explicit target/scope, recoverable backup, confirmation, and audit record; it is not a side effect of deleting a paper.

**Revisit if:** Multiple writing instances, synced folders, network storage, bulk operations, permanent deletion, or migrations that change document locators are authorized.

## ADR-013 — Library ports and durable import intent

**Status:** Accepted (implementation decision, 2026-10-01).

**Context:** The library service must coordinate selection, files, and SQL while keeping a single confirmation transaction. T01 review showed that hiding SQL behind a function is insufficient if the application still imports the concrete adapter.

**Decision:** Use three specific ports: native PDF selection, DocumentStore, and LibraryPersistence. SQLite implements persistence over DbActor; application helpers receive the transactional repository and the explicitly permitted UoW Transaction. The service does not import the concrete adapter. Composition lives in lib/desktop. Do not add a generic executor or DI container.

Persist confirmed intent in metadata_json using an internally versioned format that separates original input from normalized metadata; cancellation uses FAILED with an OperationCancelled intent and cleanup state. Binding details, tests, and ownership are in [TASK02_PORTS](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/plans/TASK02_PORTS.md). Recovery completes only unambiguous confirmed intents; it preserves pending/ambiguous cases and reports recoveryRequired. Do not add an implicit API to resume drafts from earlier sessions.

Implement the picker with tauri-plugin-dialog=2.8.1 from Rust, without a JavaScript package or generic dialog/filesystem permissions. Version compatibility and sources were checked in [NATIVE_PICKER](../development/NATIVE_PICKER.md); compilation and tests remain pending. The 63 owned commands and CONTRACTS v1 remain unchanged.

**Consequences:** Persistence wrappers and validated internal JSON provide a testable boundary and recovery without modifying 0001. Copies without confirmation that cannot be cancelled unambiguously remain explicit incidents. T04 extends the same transactional confirmation to initialize Workflow, preserving one COMMIT.

## ADR-014 — Structural PDF probe and protected documents

**Status:** Accepted (implementation decision, 2026-10-02).

**Decision:** Validate in the backend with pdf=0.10.0 and no default features, against managed staging, with input limited to 500 MiB and one active validation per library. Check structure and the first page without decoding graphics or changing bytes. Reject encryption in v0.1; do not add passwords or persisted decryption. Details, limits, and sources are in [PDF_VALIDATION](../development/PDF_VALIDATION.md). No IPC contract changes.

**Consequences:** Up to 500 MiB of buffer plus parser structures; this is not a total RAM bound or validation of every page. The pilot rejects some protected PDFs that other readers can open. This avoids a hand-built parser, unsafe mmap, and changing NSIS for a Windows API requiring package identity. Compatibility, fixtures, and actual costs are verified in T02; this ADR does not claim they are proven.

## ADR-015 — Opening and locally serving PDFs to the reader

**Status:** Accepted (implementation decision, 2026-10-02).

**Decision:** Use a specific ReaderPersistence port and reader access through the existing DocumentStore, with explicit transactional helpers. openPaper records activity/context/receipt without changing bibliographic revision or updatedAt. The asynchronous research protocol is authorized by UUID, window, and origin; it uses a complete 200 profile without claiming Range/streaming. Bundle PDF.js and all local resources, use useWasm=true with a specific wasm-unsafe-eval CSP and no JavaScript unsafe-eval. Details and sources are in [TASK03_BOUNDARIES](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/plans/TASK03_BOUNDARIES.md). [TASK03_PORTS](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/plans/TASK03_PORTS.md) fixes internal signatures, the owning handle retained during commit/response, and the single RequestRegistry injected into Library/Reader. Contextual phase comes from Paper; do not add a column to app_session or another repository layer in modules.

**Consequences:** Opening does not create artificial conflicts in metadata drafts. Full reads require memory proportional to PDF size and copies; measure this before the pilot rather than lowering 500 MiB for convenience. WASM adds resources and a restricted CSP directive, requiring offline bundle/render verification. Consumer cancellation does not abort the Rust job. Freeze internal signatures/shared ownership after T02 and before T03; do not implement early.

## ADR-016 — Handle-bound files and ownership of admitted sagas

**Status:** Accepted (T02 correction, 2026-10-02).

**Decision:** Managed Windows operations retain verified handles and parent guards; deletion operates on the verified object and promotion is atomic without replacement. Authorize windows-sys0.61.2, already present in the lockfile, behind a private adapter. Each admitted saga owns its permissions/exclusions until completion even if its caller disappears; coordinate requestId before filesystem effects. Recovery retains incidents/errors for Settings and never reports cleanup DONE without evidence. Details, limits, and primary references are in [MANAGED_FILES](../development/MANAGED_FILES.md). No new public API or migration change.

**Consequences:** A small Windows FFI implementation requires security/Rust review, retains more handles, and may report Busy during external concurrent access. Admitted jobs can delay shutdown and continue to a safe point even if the UI discards the response. A global recovery failure prevents new mutations until reconciliation succeeds; isolated pending cases leave unrelated papers usable. Compilation or process-restart tests do not certify power-loss tolerance.

**F10 precision, 2026-10-02:** Sharing does not prevent FILE_WRITE_ATTRIBUTES/reparse. Adopt [WINDOWS_DIRECTORY_GUARDS](../development/WINDOWS_DIRECTORY_GUARDS.md): local NTFS volume; relative NtCreateFile acquisition; for non-empty directories, retain a child with READ_DATA/no-delete and later revalidate the same directory. An existing DB acts as a read pin without writes before version diagnosis. Create a new DB atomically with FILE_CREATE only after LibraryLock; a collision aborts without opening SQLite. `.rw-directory-pin` protects empty managed directories, is omitted from export/backup, and is regenerated on restore. Additional minimal Windows features are documented; handle-based Win32 promotion remains. Costs include more retained handles/pins, library storage limited to local NTFS, and coordinated release during close/switch. An approved, executed synthetic probe supports the choice; Rust implementation and regressions still require review.
## ADR-017 — Canonical gates, explicit acceptance, and per-Paper phase clock

**Status:** Accepted (Sol, 2026-10-02).

**Context:** PRE did not distinguish imported unknown type from explicit review; P2 payloads/resolutions and rule composition were not fixed. A sufficient gate without acceptance allowed touch to move forward while skipping NEW→ACTIVE/the prior snapshot. Historical hashes did not prove current PDF access. Equal local revisions could accept expectations from another active context.

**Decision:** [WORKFLOW_GATES.md](WORKFLOW_GATES.md) fixes semantics without changing DTOs/signatures or contractVersion=1. [Version 1 definitions](phase-definitions/) are canonical and immutable, and may be embedded at build time. The sixth PRE output, review_type, confirms metadata through the existing structure; P1 characterizes the source. UNKNOWN/NA require explanations without duplicated text; the P1 decision is structured ANSWERED, and PENDING/null remains pending. P2 links artifacts through outputs (Insight synthesis), without quotas; candidate choice uses a closed OR by key and one PhaseAnswer justification; changing its projection renews answer.revision and the phase clock.

Forward movement/completion requires a COMPLETED chain, accepted snapshots, current gates, and accepted P1 continue. Querying started data does not implicitly accept it. PRE/P1/P2 project the live document. Proof/handle acquisition occurs outside the TX and the DB reference is revalidated inside; fix the ABI after T03, and do not read/hash a 500 MiB file under DbActor. A fresh clock uses max(revision)+1 per affected Paper/phase/context under UoW, replay before CAS and no-op afterward; snapshots exclude clock/context/timestamps. NEW→ACTIVE occurs only on PRE advance; P1 light/archive preserves prior active P1/P2 state. Reserved COMPLETED preserves upgrade/read/Library archive/restore and blocks incompatible transitions without a special migration. Actual P2 acceptance waits for T07; a missing capability never simulates complete artifacts.

**Positive consequences:** Observable human confirmation, stable rules/payloads, rejection of stale acceptance, preserved history, and one UoW/candidate authority. **Costs/limits:** Six PRE answers; explicit clock propagation for shared mutations; proof/handle requires T03 composition; definition edits require a new version. Verified access does not prove detection of every external modification or hardware tolerance. This decision does not prove implementation/installation.

**Rejected alternatives:** Treat default unknown as confirmed; require redundant text or scientific minimum counts; global AND for candidates and absence; an extra UI hash in the wire contract; a clock table/service; forward movement based only on preview; check only the historical PDF; read/hash large files in SQLite; normalize reserved lifecycle during upgrade.

## ADR-018 — Workflow composition over the integrated pilot

**Status:** Accepted by Sol, 2026-10-03. **Scope:** T04, after T10 pilot; does not change CONTRACTS v1.

**Context:** T03 provides document access with a retained handle, but its public error does not distinguish an inaccessible file from internal failures. Workflow needs availability in the gate without hiding integrity errors; it also needs to recover prior receipts if the PDF has disappeared. The pilot still does not initialize processing or offer an entry point independent of the reader.

**Decision:** [TASK04_PORTS](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/plans/TASK04_PORTS.md) fixes ports and composition. Migration 0002 is self-contained, with seed and backfill in the existing transaction; 0001 remains immutable. Application helpers with a borrowed repository initialize an import before its durable result and invalidate effective inputs in the same UoW. Domain remains pure and SQL stays exclusively in adapters.

The document probe uses a typed internal ABI: either an available handle or a known OS open cause (missing, access denied, sharing violation). Guard failures, generic failures, and panics remain errors. Reader retains its contract through a shared open core. Early and repeated replay occurs inside TX; the reference is revalidated even when access failed; permissions and handle survive commit/rollback. No TX is held during filesystem work, and the full PDF is not read for the gate.

Library provides access to Processing by paperId without going through Reader. Querying archived Papers or inaccessible documents does not require restoration or phase acceptance. Eight PRE/P1 commands are enabled, while P2 and candidates remain bounded by their actual capabilities.

**Costs and alternatives:** Historical seed duplication with parity tests; one extra internal open-cause classification and a short read for replay. Reject unnecessary migration callbacks, indiscriminate translation of StorageUnavailable to missing PDF, a second actor, and form access conditioned on openPaper. Open failures do not prove guards for a handle that was never acquired. Rust/security review and Windows guard regressions remain mandatory before integration.

**Conformance clarification, 2026-10-03:** T04b preflight found TASK04_PORTS summarized invalidation by preserving IN_PROGRESS in later phases, contrary to DOMAIN/SPEC-003. Correct the summary: directly changed COMPLETED inputs become NEEDS_REVIEW; every later phase already started becomes NEEDS_REVIEW even if IN_PROGRESS. A directly changed IN_PROGRESS input is preserved only if it is not also later than another changed input. NOT_STARTED, content, and last historical acceptance are preserved; one clock value per phase/UoW. This does not change the normative domain or wire contract; tests cover both cases and union of inputs.

## ADR-019 — Canonical UTC dates and reading prior Reader results

**Status:** Accepted by Sol, 2026-10-03. **Trigger:** T03 native QA; diagnosis in [native-open-diagnosis](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/reviews/task-03/native-open-diagnosis.md).

**Context:** DATA requires UTC RFC3339. Reader produced valid UTC dates with `+00:00`, but the UI validator accepted only `Z`. A confirmed SQLite open was rejected by the client. Fixing only the writer would leave earlier Paper/Position records and receipts inaccessible.

**Decision:** Both Reader writers use the existing canonical format with milliseconds and `Z` suffix. The wire UTC decoder accepts both verified UTC representations, `Z` and `+00:00`, while fully validating date/time. It rejects local dates, non-zero offsets, and `-00:00` (unknown offset). It preserves the received string and does not rewrite DB records, receipts, precision, or historical results. This specifies representation for existing UTC strings without new fields, enums, or IPC version.

**Verification:** Test real Reader results against client schemas; open, save/query, and replay, including a fixture for the previous receipt. Invalid dates, non-UTC offsets, and non-string values must still fail. Keep strict validation for other envelope and DTO properties.

**Rejected alternatives:** Accept any offset; ignore contract errors; mutate confirmed receipts; delete the synthetic library; add a data migration for equivalent UTC representation. Read compatibility does not prove the native viewer works; repeat that workflow on corrected code.

## Technical references consulted

Primary sources cited alongside the decisions they support:

- Tauri: [IPC](https://v2.tauri.app/concept/inter-process-communication/), [Runtime Authority and capabilities](https://v2.tauri.app/security/runtime-authority/), [Windows installer](https://v2.tauri.app/distribute/windows-installer/), [configuration](https://v2.tauri.app/reference/config/).
- SQLite: [transactions](https://www.sqlite.org/lang_transaction.html), [WAL](https://www.sqlite.org/wal.html), [Online Backup API](https://www.sqlite.org/backup.html), [foreign keys](https://www.sqlite.org/foreignkeys.html).
- Rust/SQLite: [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/), [features](https://docs.rs/crate/rusqlite/latest/features), [rusqlite Online Backup API](https://docs.rs/rusqlite/latest/rusqlite/backup/), and [Transaction](https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html).
- PDF.js: [Getting Started](https://mozilla.github.io/pdf.js/getting_started/?lang=en) and [examples](https://mozilla.github.io/pdf.js/examples/index.html).
- React: [Thinking in React](https://react.dev/learn/thinking-in-react).
- UI tests: [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/); the plan selects [Vitest](https://vitest.dev/guide/) as the runner.
- Contracts: [ts-rs](https://docs.rs/ts-rs/latest/ts_rs/) is selected to generate TS bindings from Rust types; verify enum, UUID, date, error, and required serde attribute representations.

Package availability, exact versions, crate compatibility, and concrete Tauri configuration still need review when initializing the repository and must be tested in an installed build. This ADR does not claim they have been installed or run.

## ADR-020 — Saving answers does not start phases

**Status:** Accepted by Sol, 2026-10-03, before implementing save in T04b. Independent comparison with DOMAIN/WORKFLOW_GATES/CONTRACTS found no earlier explicit rule for save in NOT_STARTED.

**Context:** touch/advance require a currently accepted chain to enable phases, but it was not specified whether save could write into a NOT_STARTED phase. Allowing it without starting the phase would create answers at revision 0 that invalidation leaves untouched; starting it from save would create another enablement path. Requiring every edit to target the active phase is also absent from the contract and would make it harder to correct earlier started phases.

**Decision:** After durable lookup/replay and applicable guards, a new savePhaseAnswer request against NOT_STARTED fails with GateBlocked, without an answer, clock/context change, success audit, or new receipt. Save never starts a phase. An already-started phase may be edited even when inactive, using answer CAS and without changing context. Effective changes invalidate the completed phase and later started phases; no-ops preserve tokens. Do not require reaccepting the chain before editing: forward enablement and acceptance remain subject to R1. Archived, reserved COMPLETED, and missing-capability restrictions remain. A confirmed replay still precedes this guard.

**Verification:** Saving P1 NOT_STARTED leaves no durable effects; saving a started PRE/P1 outside active context preserves activePhaseCode and invalidates/advances clocks according to D1; replay and stale no-op preserve precedence. UI permits phase queries without starting them and explains explicit activation before saving to NOT_STARTED. Native tests/installation are not implied by this decision.

**Costs and alternatives:** Persistent prefill of a not-yet-enabled phase is not allowed. Reject implicit start, persisted data under NOT_STARTED/revision0, and the extra restriction to only the active phase. No change to wire, DTOs, schema, or canonical definitions. WORKFLOW_GATES and CONTRACTS include the clarification; DOMAIN/SPEC retain and reference their rules.

## ADR-021 — Transactional Workflow application use cases

**Status:** Accepted, 2026-10-03. **Scope:** T04b correction before integration; no new product capability.

**Context:** Review found an ignored pin, generic history without change identity, and use-case coordination inside the SQLite adapter. Extracting only some pure functions does not meet the accepted application architecture with a borrowed transaction.

**Decision:** Adopt [TASK04_APPLICATION_CASES](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/plans/TASK04_APPLICATION_CASES.md). Five application functions receive TX, WorkflowRepository, and LibraryRepository. Add four persistence primitives and one borrowed struct; reuse paper/update_lifecycle/audit_change and archive_paper_in_tx without expanding LibraryRepository. A concrete pure plan decides destination/preserved state and NEW→ACTIVE; application applies clocks/pins/CAS and specific events. An unsupported pin fails explicitly; never reinterpret it as v1. Generic legacy request audit remains separate from effective changes; no-op/replay invents no modification.

**Boundaries:** Do not change WorkflowPersistence, wire/DTOs, SQLite schema, actor, UnitOfWork, Reader, Windows, or receipts. The adapter retains SQL/mapping, actor work, replay, document revalidation, and proof/permission ownership around all commit/rollback. Do not move SQL into application or create a generic effects engine.

**Verification:** Pure plan for first acceptance/reconfirmation; persisted and unsupported pin; before/after audit with identity; no-op/CAS/replay; late rollback during event/receipt; and ownership regressions. Accepting this appendix does not prove implementation or gate behavior.

**Cost:** Four repository methods, one concrete plan, and extraction of five existing use cases. Reject duplicating Library services or adding another harness/actor because current interfaces support the needed composition.

## ADR-022 — Capture and concepts with closed transactional contracts

**Status:** Accepted by Sol, 2026-10-03, before implementing T05. Historical documentary review R1–R5 is recorded; final review PASS, with no important findings open. This decision does not implement the capability.

**Decision:** Adopt TASK05_PORTS with fourteen existing wire commands and explicit application cases for capture/create/update/lifecycle/link. Borrowed TX, replay before CAS, no-op, and consistent P2 scope/inverse; effective events include requestId and before/after. 0003 has one authority per attribute, core catalogue 1.0.0, and consumer schema for T07/T08 without those services. T05a delivers schema/domain/capture; T05b edits/queries/IPC, sequentially, with a new author and review.

**Semantics:** NONE preserves absence; parent/provenance starts at revision0; anchoring is closed to page_region and LOCATED is a record. Source/hash is immutable and STALE remains on edit; no implicit revalidation. ConceptApi owns canonical fields; Knowledge owns only Concept confidence; Insight affected is an explicit subset of links. ID sets are canonical, homonyms are allowed, and aliases are unique within a concept. New writes to an archived parent require restore; an existing link from an active parent remains a no-op if its destination is archived. Active Relations are unique by endpoint/type/canonical context; create/update/restore collision is atomic Conflict.

**Verification:** Capture rollback, twelve attributes round-trip/reopen, migration/backup, CAS/replay/no-op, specific audit, transitive closure/cycles without expansion through relations, archived link, candidate-answer independence, pending/stale locator, fourteen handlers, and permission ownership. DDL/Rust/SQL still need later executable review.

**Costs and alternatives:** Bounded, verified name mirror; no global deduplication or semantic merge. Normalized aliases do not prove equivalence. v0.1 does not remove STALE or silently reassign a source. Reject a new API/CAS for set-add link, duplicate application logic in adapters, or fictitious provenance storage. No new wire/DTO/dependencies or change to integrated migrations.

## ADR-023 — Persistent candidates and complete P2 enablement

**Status:** Accepted by Sol, 2026-10-03; independent documentary review PASS. Exact semantics in [TASK06_DECISIONS](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/plans/TASK06_DECISIONS.md). This decision does not implement the feature; the T06 ABI still needs comparison with integrated T05.

**Decision:** Setting a candidate requires P2 to be started, preserves context, and uses existing P2 CAS/replay. Selecting requires an active Claim/Question with a direct association; deselecting an archived item is allowed, and priority/rationale must be null. A final zero-candidate decision requires canonical justification of 1..5000 characters, with candidates null; it produces ANSWERED. Archive/restore preserves selection/count; summary includes archived candidates, while the gate treats them as invalid artifacts. Save does not change selection and permits PENDING/null drafts; a present object must always be complete/verified. PhaseAnswer is the sole justification store; no second store.

**Delivery:** T06a candidates/backend/policy, T06b capture/queue/UI; all production P2 save/evaluate/advance wait for T07. No per-key enablement or new wire flag. In T06, zero-candidate absence cannot yet be saved through save or a fictitious item. T07 enables the set when complete resolvers are connected; fixtures do not prove capabilities. PRE questions provide manual context, without NLP or new fragment association.

**Verification:** CAS/no-op/replay, last candidate deselection/justification, PENDING/null, archived selected candidate visible but invalid, rollback of association/answer/clock/audit/receipt, no implicit activation or acceptance without resolvers. Compare ABI/decoders/closure with T05 before the T06 author starts.

**Costs:** The queue retains flagged invalid candidates; candidates cannot be preselected before P2; formal forms wait one more milestone. This avoids divergence from filtering archived candidates or partially enabling behavior that is difficult to explain. No new schema or rewrite of integrated migrations.
