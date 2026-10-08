# Data design, persistence, and recovery

Status: v0.2 implementation baseline, adopted on 1 October 2026. Logical model v0.1; this is not an executed migration.

## 1. Authority and representation

SQLite stores entities, relations, provenance, answers, preferences, and history. PDFs are managed files identified through SQLite; their paths are stored relative to the root. Exports, FTS, concept views, and a possible graph are derived data.

UUIDs use canonical text form, dates use UTC RFC3339, and revision counters are integers starting at zero. Writers emit milliseconds and the `Z` suffix; the decoder also accepts `+00:00`, preserving existing strings and receipts, in accordance with ADR-019. Other offsets and local dates are rejected. DTOs use `camelCase`; columns use `snake_case`. Translating a display label does not change persisted enums. Missing bibliographic values are `NULL` or empty collections, never strings such as "unknown" that could look like real values. The initial body is `body_text`, with `plain_text` version 1; `body_json` remains `NULL`.

PDF blobs are not stored in SQLite. A remote download URL is not used as a substitute for a managed document. A document and a paper have separate identities so document versions and locators can be preserved.

## 2. Logical model and constraints

The original DOCX DDL is an antecedent to this model. Implemented migrations must incorporate the explicit adjustments below rather than copying it without review.

| Group | Tables and keys | Constraints and responsibilities |
|---|---|---|
| Bibliographic identity | papers(id), authors(id), venues(id), paper_authors(paper_id, author_order) | Normalized non-NULL DOI is unique; title is non-empty; authors are ordered; matching names do not prove that two records represent the same person |
| Documents | documents(id, paper_id) | Unique relative path; SHA-256; media type; size; current document selected explicitly; prior versions preserved when they have locators |
| Imports | import_operations(id) | Opaque token; STAGING/PROMOTED/COMMITTED/FAILED states; metadata and IDs reserved before promotion; commit result retained |
| Reading | reading_positions(document_id), app_session(singleton) | Physical page >=1; finite positive zoom; revision; nullable last-opened paper with FK |
| Phase definition | phase_definitions(code, version) | Validated JSON document, immutable per version; content hash and known declarative rules |
| Processing | paper_phases(paper_id, phase_code), phase_answers(paper_id, phase_code, question_key) | FK to exact version; phase and answer revisions; resolution, explanation, and validated structured value when required by the question; snapshot/hash of the last accepted gate |
| Objects | knowledge_items(id) | Valid type, explicit origin and confidence; revision; lifecycle; body text separate from citations |
| Concepts | concepts(item_id), concept_aliases(concept_id, normalized_alias) | Item subtype; normalized name for search, without global uniqueness that would force homonyms to merge |
| Specialization | evidence_details(item_id), question_details(item_id) | FK to item with type also checked by domain logic; specific states separate from lifecycle |
| Associations | paper_items(paper_id,item_id), item_concepts(item_id,concept_id) | Sharing an item/concept is not provenance; selected_for_p3, priority, and rationale are retained per paper/item and protected by a processing revision |
| Relations | relations(id) | Distinct, existing endpoints; type/context/origin; revision and archive; normalized symmetry; uniqueness of active semantic link according to DOMAIN |
| Provenance | provenance(id), item_provenance(item_id,provenance_id), relation_provenance(relation_id,provenance_id) | Document version, captured hash, page/section/quote, PENDING/LOCATED/STALE state; revision for editing a locator |
| Future evaluation | validations(id) | Reserved for P3; no complete human-validation commands in v0.1 |
| Catalog | ontology_types(code), relation_types(code) | Versioned core catalog; endpoint allowlist, direction, and rules; no executable code |
| Operations | schema_migrations(version), audit_events(id), app_settings(key), operation_receipts(request_id), maintenance_operations(id) | Migrations with checksums; events in the same transaction; validated preferences; idempotency; long-running jobs with durable state/result/error |
| Derived indexes | papers_fts, items_fts | Rebuildable FTS5; populated from canonical tables with lifecycle filtering after matching |

`papers` includes `revision` and `archived_from_lifecycle`. Items and relations only switch between ACTIVE/ARCHIVED: restore always returns them to ACTIVE, so no prior-state column is needed. Derived progress fields are not updated by the UI.

`papers` includes `current_phase` and `last_opened_at`; `app_session` references the last-opened paper. `paper_phases` retains `state`, `definition_version`, `revision`, and `completed_at`; `phase_answers` includes `revision`, `answer_text`, `resolution`, `explanation`, and `structured_value_json`, validated against the question type. Structured values such as the P1 decision are not inferred by searching prose for keywords. Provenance includes `revision` and `updated_at` when its locator can be edited. Every field whose contract uses `expectedRevision` has a persisted revision.

When a phase is completed, `paper_phases` stores `accepted_gate_snapshot_json/hash` and retains the historical snapshot and `completed_at` when invalidated; `state` determines whether it remains current. The canonical snapshot and exact projections are defined in [WORKFLOW_GATES.md](WORKFLOW_GATES.md), ADR-017: `definitionHash`, normalized answers/revisions, accepted dependencies, and the live Document in PRE/P1/P2, plus ordered artifacts/edges. Do not include `phaseRevision`, context, lifecycle, or timestamps in the hash. Do not reconstruct history from current state.

An effective answer change conservatively invalidates completion for that phase and marks later phases already started as NEEDS_REVIEW, without changing NOT_STARTED phases or deleting data. Effective changes to artifacts used by P2—items, relations, provenance, links, and P3 selection—invalidate P2 if it was complete. An idempotent request or one that preserves the same values does not invalidate anything. The rule and affected revisions are applied in the same Unit of Work; relevance is never inferred from free text.

Minimum indexes include DOI, documents by paper, papers by lifecycle/updated_at, items by type/lifecycle, relations by both endpoints, provenance by document, and associations by both sides. Catalog constraints that require joins are checked inside the use case, in addition to basic FKs and CHECK constraints.

All persistent links use verifiable FKs. Archiving a source does not cascade into deletion of global knowledge. Queries exclude archived records by default and may request them explicitly; an ACTIVE library filter means not archived, including NEW.

Initial FTS5 projections: `papers_fts` contains `entity_id UNINDEXED`, title, authors_text, venue, and domain; `items_fts` contains `entity_id UNINDEXED`, `type_code UNINDEXED`, title, and body_text. The tokenizer is `unicode61` with `remove_diacritics=2`. UUIDs are the public identity, never FTS's internal rowid. The use case updates the projection in the same transaction as the canonical mutation; a full rebuild runs under maintenance and must complete before the index is declared usable. Search treats input as literal tokenized text and requires every term; v0.1 offers neither SQL syntax nor advanced FTS operators. Relevance ties are resolved by UUID for stable pagination. The [FTS5 documentation](https://www.sqlite.org/fts5.html) is the adapter reference; weights and ordering are product decisions, not measures of scientific quality.

Answers: `expectedRevision=0` only for an absent row, first revision is 1; compare-and-swap precedes no-op detection, and receipt replay precedes compare-and-swap. Phase clocks are fresh per Paper: `max(revision)+1` under the UoW, distinct values for each affected phase, and destination renewal on context change; no new table. No-op preserves revisions/timestamps/snapshot; rollback does not consume a clock. A changed candidate projection increments `PhaseAnswer.revision` as well as the P2 clock. The only zero-candidate justification is `structured_value_json` of `PhaseAnswer(P2,p3_candidates_or_justification)`; selection/priority/rationale remain authoritative in `paper_items`. See WORKFLOW_GATES.

## 3. SQLite configuration and unit of work

Each connection sets `foreign_keys=ON`. The local library uses `journal_mode=WAL`, `synchronous=FULL`, and an initial `busy_timeout` of 5000 ms. These durability choices will be measured in tests; they do not guarantee protection against defective hardware or disks.

The same thread owns the connection and runs short transactions. Mutations use an IMMEDIATE transaction when they read and modify shared invariants; repositories receive the transaction and do not commit independently. An update with `expectedRevision` checks and increments the revision in the same transaction; updating zero rows is a conflict, not success.

The [SQLite WAL documentation](https://www.sqlite.org/wal.html) describes its associated files and operating limits. This design restricts the active library to a local disk. A connection is not shared concurrently among tasks.

| Operation | Writes committed together |
|---|---|
| confirmImport | Paper, ordered authors, document, COMMITTED intent, receipt, and audit |
| Typed capture | Item and specialized detail, paper membership, concepts, locator and initial link, audit, and affected revisions |
| Save answer | Answer, processing revision, reevaluation of relevant invalidations, and event |
| Advance phase | Current gate snapshot, phase state, active phase, history, and revision; a P1 archive decision includes archiving the Paper in that same commit |
| Relation | Checked endpoints and types, relation/initial provenance, and audit |
| Archive/restore | Previous and new state, revision, dependent invalidations, and audit |

There is no real shared transaction between the filesystem and SQLite. Operations affecting both retain a verifiable intent and support explicit recovery.

Check document access/handle and guards outside the transaction; inside it, revalidate the Paper/Document reference and its agreement with the check. Do not read or hash PDFs up to 500 MiB under DbActor/transaction. External inaccessibility changes snapshot availability but does not invent a revision; DB Document changes do invalidate/renew revisions in the UoW. When enabling or completing a phase, revalidate the COMPLETED + accepted snapshot + current gate chain. ABI after T03; see WORKFLOW_GATES.

## 4. Idempotency, history, and mutation limits

`confirmImport` returns its PaperDto again if the same token was already confirmed, without creating another entity; a request with incompatible content returns a conflict. `operation_receipts` stores the request_id UUID, command, canonical request hash, result references, and timestamp for the idempotent commands defined in CONTRACTS. Repeating the same ID and hash retrieves the result; reusing the ID with different content is rejected.

Receipts are not purged automatically in v0.1. Edits still require `expectedRevision`; `requestId` does not replace protection against stale text. The transaction, including the receipt, is committed before success is reported.

`audit_events` records the action, affected identity, and changes needed to explain history. Sensitive content history stays inside the library and its backups; it is not copied into diagnostic logs. This is not called event sourcing and does not promise general undo. Merge and permanent purge are not exposed in v0.1.

`maintenance_operations` retains `operationId`, type, state, progress when known, result/error, and the manifest of owned resources. Activating a restored root or changing libraries uses a durable coordination journal outside the root being closed, so the change can be resolved even if the previous database is no longer open. The journal does not serialize secrets or paper contents.

## 5. Roots and managed files

```text
%LOCALAPPDATA%/ResearchWorkbench/
  library/
    library.json             # libraryId, format, and compatibility
    research.sqlite
    documents/<documentId>/original.pdf
    staging/<importOperationId>/source.pdf
    recovery/                # operation manifests where applicable
  backups/<backupId>/         # internal protection and locally selected copies
  logs/                      # rotated diagnostics without research content
```

The actual root is resolved through OS APIs and preferences; no operation depends on cwd. SQLite metadata and library.json must agree on libraryId; the manifest is not a second authority for knowledge. An open library retains its identity when moved.

DocumentStore checks relative paths, the canonical root, and reparse points/junctions before opening. The reader protocol takes a documentId, never a path. A file changed externally marks its locators STALE and triggers an integrity diagnostic; the captured hash is not rewritten automatically, and provenance is not falsely reported as verified.

The first pilot manages one current document per paper. The structure supports versions without promising a PDF replacement interface. Binary resources are retained while locators exist or a recoverable operation uses them.

## 6. Import and recovery

1. The native picker grants an operation for a chosen file. A STAGING intent is recorded and the file is copied to staging; cancellation removes only files owned by that attempt.
2. The format, size, and readability are minimally validated, the hash is calculated, and a decision is requested for a duplicate DOI/hash. Missing authors or year are not inferred.
3. Before promotion, reserved document/paper UUIDs, confirmed metadata, relative destination, and hash are persisted. Staging and destination are on the same volume.
4. The file is promoted and the attempt is marked PROMOTED. The next transaction inserts the entities and marks the attempt COMMITTED.
5. If interrupted between steps, startup compares intent, destination, and hash. An unambiguous match may be completed; an ambiguous case remains pending, and a file with an unknown owner is never deleted.

A COMMITTED record without a readable file is an integrity incident, not a reason to delete the record. Incomplete staging without a destination may be cancelled with a recorded result. Tests inject interruption before and after each durable boundary.

Internal `.rw-directory-pin` files may remain at the root/managed directories under [WINDOWS_DIRECTORY_GUARDS](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/development/WINDOWS_DIRECTORY_GUARDS.md), ADR-016. They are not documents or knowledge; cleanup does not remove them, and export/backup omit them from manifests. Restore of a supported format regenerates them when acquiring the root/directories and does not trust supplied pins as authorization. v0.1 libraries use a local NTFS volume.

## 7. Migrations and compatibility

| Planned sequence | Contents |
|---|---|
| 0001 pilot | Sources, documents, imports, reading, revision, settings, receipts, audit, and ledger |
| 0002 workflow | PRE/P1/P2 definitions, answers, processing, and versioned gates |
| 0003 knowledge | Core catalog, items, concepts, associations, relations, and provenance |
| 0004 portability | FTS projections and metadata needed for v0.1 export/backup/restore |

This is a design sequence, not SQL already released. A released migration is immutable and retains its checksum. appVersion, dbSchemaVersion, ipcVersion, ontologyVersion, phaseDefinitionVersion, and exportSchemaVersion are independent versions.

Before migrating: block writers, complete or pin file operations, verify schema and checksum, and create a consistent snapshot with the required files. The backup includes recoverable import state. Validate `foreign_key_check` and `quick_check`; any failure stops migration. Run the migration in a transaction and write the ledger in that same commit. A failure rolls back the DB and retains the backup; it does not restore automatically or start an editable UI on a partial schema.

If dbSchemaVersion is newer than the supported version, open only a diagnostic screen; do not modify the library or run a downgrade migration. The installer limits downgrades, while the backend enforces this protection separately.

## 8. Snapshot, export, and backup

SnapshotService stops new writes and obtains a consistent state of the DB and document references; it copies the required immutable resources while retaining maintenance permission. For the DB, use the [SQLite backup API](https://www.sqlite.org/backup.html), not an isolated copy of the file while WAL is active. The design also guarantees consistency of the set by excluding application mutations.

ExportLibrary creates documented JSONL and derived Markdown with IDs, relations, provenance, and versions. ExportPaper calculates reference closure: it includes shared entities and required endpoints so it never emits IDs without definitions. Omission of PDFs from a text export is declared in the manifest and is not confused with a restorable backup.

A restorable backup contains a snapshot DB, required PDFs, library.json, and manifest.json with backupId, libraryId, versions, date, relative file list, sizes, and hashes. The manifest is the verification contract; logs are not needed to recover research.

Proposed initial policy: protection is mandatory before migration and backups are manual from v0.1; remind after seven days of use without a successful backup, with no resident task or synchronization. Keep all copies by default; any future deletion must be explicit. A backup beside the library does not protect against disk loss, so the UI lets the user choose another local destination.

## 9. Restore and library switching

Restore verifies the manifest, supported format, hashes, references, schema, and available space **before** activating data. It extracts or copies to a new root beside the library, and rejects absolute paths, traversal, and links that escape. The initial backup format is a directory; packaging it as a compressed file requires specifying and testing safe extraction.

Prepare the root and verify it with SQLite; restore returns a prepared-library token without changing the active library. After an explicit choice, switchLibrary records a durable intent, closes the connection, changes the active root through recoverable configuration, and reopens it. The previous root remains available for explicit rollback; an open DB is never overwritten. A crash at any step is resolved on next startup using the manifest to identify which roots are complete.

SwitchLibrary uses the same coordinator: it blocks operations, preserves or cancels pending work, and verifies libraryId/compatibility/target lock. It does not merge libraries, change IDs, or move data accidentally. Choosing another root requires the native picker, not an arbitrary path from the UI.

## 10. Persistence checks before delivery

- Constraints and transactions: FKs, revisions, duplicates, rollback, and consistent history.
- Real reopen: same identity, content, last page, and provenance after restart.
- Filesystem: Unicode, spaces, escapes, full disk, corrupt PDF, and changed hash.
- Fault injection: interruptions during promotion/import, commit, snapshot, migration, and activation of a restored root.
- Portability: export without broken references; verifiable backup; full restore with PDFs in a temporary library.
- Installation cycle: upgrade, incompatible-version rejection, uninstall, and reinstall while preserving data.

These are planned tests; results will be recorded when implementation exists. Acceptance is governed by QUALITY and the SPECs, not by the presence of these tables in a document.

## 0003: accepted consumer manifest (ADR-022)

[TASK05_PORTS](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/plans/TASK05_PORTS.md) fixes columns, attribute authority, keys/indexes, the core1.0.0 catalog, and consumer mapping before implementation. Concept stores its definition only in body_text, with preferred_name/title as an atomic mirror; Concept/Evidence/Question attributes are reconstructed from their specific tables, with attributes_json=NULL. The other nine types use a fully validated discriminated object. There are no duplicate authorities or defaults in the event of corruption.

Initial parent/Provenance revision is 0. New paper_items use phase_code=P2, selected_for_p3=0, and priority/rationale NULL. Concept shares a revision with knowledge_items. Active Relations are UNIQUE by endpoint/type/canonical context; a create/update/restore collision returns Conflict without side effects. validations(id) is an empty reservation, with no rows or simulated services. No ON DELETE CASCADE or destructive triggers. 0001/0002/checksums/runner remain intact; DDL0003 and backup/upgrade/reopen are reviewed in T05a, with FTS0004 later.

An effective audit records requestId, entity, closed action, and relevant before/after values within the data/receipt/clocks TX. The generic legacy receipt event does not replace content history. P2 scope and inverse follow exactly the WORKFLOW_GATES closure; a no-op does not invent modifications.
