# Research Workbench — Quality and Verification

Status: pre-implementation quality plan. The performance thresholds below are proposals to measure, not results or accepted criteria; hardware must be agreed and evidence obtained first. The cited tests are planned, not executed.

## Evidence Gates

- **Installed 0.0.1 pilot:** Windows 11 x64, actual NSIS installer, clean/offline machine, WebView2 from the package, library/reader journey, continuity, update/uninstall/reinstall. A build without bundling, browser tests, mock components, or opening from the checkout does not prove this gate.
- **v0.1:** verified PRE/P1/P2 workflow, global knowledge and provenance, search, export, backup/restore, and library switch; no P2 state implies P3/P4.
- Record every piece of evidence as `automated`, `manual`, `simulated`, or `pending`, with version/commit, fixture, environment, steps, and result. Missing evidence means pending, not accepted.

## Proposed NFRs

Proposed reference equipment: Windows 11 x64, 4 cores, 16 GB RAM, and local SSD; record the actual model and configuration. After warm-up, measure at least 30 repetitions for latency, median, and p95; report cold and warm results separately and use an installed/release build. Initial design objectives, not measured results:

| Metric | Proposed budget | Fixture/method | Status |
|---|---:|---|---|
| Warm start to Home | p95 ≤ 3 s | DB fixture: 1k papers, release app | Not measured |
| Cold start | p95 ≤ 8 s; excludes first installation | Clean VM without network; includes runtime | Not measured |
| Open PDF to first page | p95 ≤ 2 s | Local 20-page/10 MB PDF | Not measured |
| Confirmed small save | p95 ≤ 500 ms | Metadata and edits with 10k items | Not measured |
| Basic search | p95 ≤ 500 ms for 20 results | 1k papers + 10k items and filters | Not measured |
| Import | ≥20 MB/s sequential copy | 100 MB fixture file on SSD, hash verified | Not measured |
| Restore | ≤2× time to create backup of same corpus | 1k papers, 10k items, 500 MB PDF | Not measured |
| Stable memory | p95 <700 MB | 30 minutes, 100-page PDF; open/close 20 times | Not measured |
| Installer | Measure and record size | NSIS with offline WebView2 runtime | Not measured |

Also measure durability and memory growth; no threshold justifies weakening transactions, traceability, or recovery. If a threshold fails, record hardware/corpus and make an evidence-based decision before optimizing. Do not claim maximum supported size based on a fixture.

## Security, Privacy, and Integrity

1. **Paths and PDF protocol:** the UI selects a registered document through a dialog/token or UUID; it never reads an arbitrary path. The backend resolves the canonical path under the library and rejects traversal, unknown IDs, escaping symlinks/junctions, unregistered resources, and unexpected files. Verify CSP/local origin. Tests: `protocol_rejects_traversal`, `protocol_rejects_symlink_escape`, `unknown_document_not_found`, `unregistered_path_never_served`.
2. **IPC and database:** validate DTOs in Rust even when TypeScript also validates; validate enums, sizes, UUIDs, revisions, nullability, and states. Parameterize SQL. Do not expose queries, arbitrary paths, or general filesystem capabilities. Rust/TS contract tests cover camelCase, enums, errors, dates, and IDs; also `query_is_parameterized`, `stale_revision_rejected`.
3. **Untrusted content:** titles are never converted into paths; escape text for display; PDFs and ontologies are data. v0.1 `body` is `plain_text`; do not add a rich-text editor without design/sanitization. No remote CDN/worker. Opening external links requires explicit action.
4. **Local privacy:** no telemetry of papers/content, login, remote API, or automatic sending. Rotated logs omit bodies, excerpts, unnecessary original paths, and secrets. Shareable exports omit private paths by default. Do not claim SQLite/backups are encrypted.
5. **Concurrency/recovery:** one writer per library; release the lock when the process exits. Import staging/reconciliation must not delete files owned by an unknown process. Object and association writes are atomic. Under WAL, backup uses a consistent snapshot API and coordinates file changes.
6. **Backup and restore:** stage into a new root; verify hashes, SQLite/FKs, and resources before switching. On failure, keep the active library. Migration uses a prior copy, checksums, locking, and protection against downgrade/future schemas.
7. **Maintenance:** archive/restore does not cascade over global knowledge; tests `archive_paper_no_cascade`, `archive_restore_preserves_links`. Merge and permanent deletion are not v0.1 features.
8. **Installer:** per-user NSIS, offline WebView2, audited scripts, data outside the installable folder, and checksum. Do not promise Authenticode signing for the pilot.

Additional regression tests: `import_retry_is_idempotent`, `import_token_changed_payload_is_rejected`, `duplicate_doi_normalized`, `malicious_title_cannot_escape_export_root`, `plain_text_only_v01`, `logs_exclude_body_and_source_path`, `archive_restore_preserves_links`, `archive_paper_no_cascade`, `trash_merge_hard_delete_not_exposed_v01`. An offline network test covers main journeys and captures requests; document its limited scope, and do not present it as universal proof that no communications occur.

## Planned Tests

### Domain, Contracts, and UI

- Unit tests without UI: justified unknown; incomplete and revalidated gates; versioned definitions; later edits and phases marked `NEEDS_REVIEW` while preserving data; types/origin/provenance; relation constraints; archive/restore; FTS filters; export version/manifest.
- Test the Tauri adapter contract against real commands when they exist; label mocks as simulated. Errors retain stable codes and English messages without private paths.
- UI by version: empty library/import/duplicate/error; position/reopen; PRE/P1 with blocking, unknown, and choices; P2 capture with/without selection, shared concept, and pending provenance; search, export, and restore. Review loading/empty/no-results/saving/saved/error/conflict states.
- Accessibility: full keyboard support, visible/initial/return focus, labels, associated errors, status announcements, contrast, and semantics that do not rely only on color. DPI at 100/150/200%, resizable window, and different scaling settings.

### SQLite/Filesystem and Failures

Every test uses an isolated temporary directory and synthetic data; never use a personal library.

- Valid import, moving the original, duplicate DOI/hash, Unicode, invalid input, permissions/space, interruption after staging and promotion; recover without recording false success.
- FK integrity, rollback of paper-item-relation-provenance; out-of-order writes; lock/second instance; optimistic revision.
- Migration before/during/after, wrong checksum, full disk, WAL, future schema; do not write during a partial migration.
- Backup with all resources, corrupt SQLite, broken FK/hash, missing resource, insufficient space, restore staging, failure before switch, and recovery of prior library.
- FTS interrupted between mutation and index update; rebuilding matches a reference query against the canonical source and excludes archived/trash items by default.
- Export counts/references, line-parseable JSONL, hashes, optional binaries, omitted private paths, malicious/unwritable destinations.

Use fault injection in filesystem/DB where possible. Show `Saved`, `Exported`, `Restored`, and the active location only after commit/verification. Interruption tests must check state from a new connection/process.

### Moderate Performance Corpus

Deterministic fixture without private content: **1,000 papers**, 2,000 authors, **10,000 KnowledgeItems**, 15,000 relations, 12,000 provenance records, 100,000 associations, and **500 MB of synthetic PDFs**. Record seed, types/sizes, filters, and hardware. Measure metadata/FTS separately from binaries. The corpus evaluates quality; it does not limit supported size.

### Installed Product QA

Use a Windows 11 x64 VM/machine without Node/npm/Rust/Cargo/Git/Codex and without WebView2; never remove runtimes from the development computer. Record OS/architecture, installer/version/checksum, WebView2, network, working directory, and whether a source checkout exists. Install offline, launch from the Start menu, use it without development tools and from a different working directory, and exercise import→read→page→close→reopen→archive/restore offline. Check second launch, process termination after close, update using a fixture, inject migration failure/reject downgrade, uninstall while retaining data, then reinstall and reopen the same library. A new profile on a development machine does not prove absence of tools.

## Architecture Fitness

- **Hexagonal modular monolith:** domain modules (`library`, `workflow`, `knowledge`, `relations`, `provenance`, `search`, `export`, `backup`) depend on ports/interfaces; SQLite/filesystem/Tauri adapters remain outside the domain. The domain does not import React, Tauri, SQLite, or UI types.
- **Dependency direction:** `app/composition → adapters → application/domain`; UI invokes use cases through typed/versioned IPC adapters; no component imports repositories or calls SQL. The domain contains no `invoke` or OS paths.
- **Canonical data and derivatives:** SQLite + managed files are authoritative; FTS, UI projections, and JSONL/Markdown are rebuildable/outputs. UUIDs remain stable across adapters/export/recovery.
- **Transactions:** multi-object mutations expose atomic use cases; do not split gates, merge, restore, or library switch into a sequence of React decisions.
- **Verification:** import boundaries/lints and contract tests check dependencies; a composition test creates a temporary SQLite adapter and evaluates the same domain as IPC; frontend must not contain a second set of business rules.

## Acceptance Gates

- **Installed pilot:** real installer tested on a clean/offline system, runtime included, Library/Reader journey, continuity, update, and uninstall/reinstall. Does not imply PRE/P1/P2.
- **v0.1:** PRE→P1→P2, concepts/provenance, gates and invalidation, search, export, backup/restore, and library switch; closing and reopening preserves state. Does not imply P3/P4.
- Acceptance blockers: loss of a confirmed change, broken UUID/links, access to paths outside the root, writable partial migration, restore without validation, export sharing paths by default, or a saved indicator before commit.
- Mark an NFR achieved only when measured on recorded hardware/corpus. Web tests do not replace manual/native QA where they do not exercise the installed product.
