# Research Workbench — functional requirements

Status: specification of execution baseline v0.2, adopted on 1 October 2026. Defines expected behavior; it does not certify implemented code, an installer or delivered features. The architecture and normative signatures belong to `ARCHITECTURE.md`, `DOMAIN.md` and `CONTRACTS.md`. No UI control may simulate success: it performs an available operation or is clearly presented as a future feature.

## Scope by version

- **Pilot 0.0.1:** installed Windows 11 x64 product; local library, PDF import/copy, metadata, PDF reader, position, continuity, editing and paper archive/restore; protected, verified migration.
- **v0.1.0:** complete PRE/P1/P2, traceable global knowledge, metadata/knowledge search, JSONL/Markdown export, restorable backup and library switching. Adds archive/restore for papers, concepts and items. Excludes P3/P4, LLM, merging, an exposed trash bin and permanent deletion.
- Gherkin scenarios are required criteria, not implementation evidence. Completed phases represent editorial progress, not scientific validation. English is the current presentation language, following ADR-0014 in the repository root; stable identifiers and immutable historical definitions remain unchanged.

## SPEC-001 — Desktop and pilot installation

**REQ-001-01** Distribute a per-user x64 NSIS `setup.exe`, with an Installed Apps entry, Start shortcut, optional desktop shortcut, its own window and an uninstaller. **REQ-001-02** Package the UI and PDF.js/worker, include offline WebView2 and operate without Node, Rust, Git, Codex, a specific cwd, a development server or network. **REQ-001-03** Keep the library separate from binaries/code. First launch displays and confirms `%LOCALAPPDATA%\ResearchWorkbench\library` and explains local PDF copies. **REQ-001-04** One writer instance per library, with controlled closure. Updates create a consistent copy before migration; reject an incompatible future schema/downgrade without writing. Uninstallation preserves data and backups.

```gherkin
Given Windows 11 x64 without WebView2, network or development tools
When I install setup.exe offline and launch from Start
Then its own window opens without a development server and the installer supplies WebView2

Given a library with committed changes
When I perform a compatible update, uninstall and reinstall
Then migration and reinstallation preserve UUIDs, documents and position; uninstall removes binaries but retains data
```

**Errors:** runtime/installation lacks space or permissions, busy library, incompatible schema, backup/migration failure, closure with a pending write. Explain recovery and do not claim saving/updating without a commit. **Outputs:** actual installer/checksum, brief guide and clean-machine/offline evidence; distinguish build, mock and installed QA evidence.

## SPEC-002 — Library, import and reader

**REQ-002-01** Native picker; copy the PDF into the library with stable UUID/hash, independent of the original. **REQ-002-02** Manual metadata: title, optional authors, nullable year, venue, DOI, review type/domain; validate without inventing unknown values. **REQ-002-03** Detect duplicates by normalized DOI/hash; offer open existing/cancel without silently merging or overwriting. **REQ-002-04** List, metadata search, basic filters, detail, editing, archive/restore, empty and archived views. **REQ-002-05** Internal reader preserves the physical page numbered from one and zoom; resumes the committed paper/context/position. **REQ-002-06** Display “Saved” only after a persisted response; errors keep pending edits visible.

```gherkin
Given a readable PDF and reviewed metadata
When I confirm, move the original, close the app and reopen the paper
Then the managed copy opens with the same UUID and committed position

Given a candidate with a matching normalized DOI or hash
When I import the file
Then I can open the existing paper or cancel; nothing is created or merged without a decision
```

**Errors:** picker cancellation, invalid/protected PDF, full disk, invalid DOI, revision conflict, missing document, rendering/saving failure. Recoverable errors, without arbitrary path access. Promise neither OCR nor searchable PDF text. **Outputs:** registered paper/document, import recovery and loading/empty/error/pending/success states in English.

## SPEC-003 — Versioned PRE/P1 and gates

**REQ-003-01** Pin canonical PRE/P1/P2 version=1 per Paper; new templates do not rewrite history. **REQ-003-02** Pending/answered/unknown/not-applicable responses follow the JSON matrices; explain UNKNOWN/NA without redundant prose, use structured P1 decisions with ANSWERED, and draft PENDING/null. **REQ-003-03 PRE** Six visible outputs: `purpose`, `uncertainty_target`, `baseline`, `expected_outcome`, `desired_depth`, `review_type`; the sixth confirms the canonical type without editing bibliography; also require a title and an available current PDF. An unknown default is not explained confirmation. **REQ-003-04 P1** `scope`, `out_of_scope`, `review_type`, `literature_cutoff`, `core_message`, `relevance_decision`; textual review_type characterizes the source/methodology; accepted continue enables P2. **REQ-003-05** Reevaluate during advance without trusting the UI; forward requires a COMPLETED prerequisite, accepted snapshot and current gate. Touch does not implicitly accept. **REQ-003-06** An effective edit preserves history and requires reconfirmation of a COMPLETED phase and started later phases even when its content gate still passes. Details are closed in [WORKFLOW_GATES.md](WORKFLOW_GATES.md), ADR-017.

```gherkin
Given accepted current PRE and an unknown P1 text field with a reason
When I confirm P1 through advancePhase with the continue decision
Then it counts as processed and P2 is enabled

Given PRE has a pending output and the UI attempts to advance with stale state
When the service processes advancePhase
Then it rejects the transition, returns pending requirements and retains the active phase

Given saved P2 and a P1 edit invalidating its prerequisite
When the edit is committed
Then P2 retains its data but is marked for review
```

Invalidation preserves data, the accepted snapshot and historical completedAt; NEEDS_REVIEW requires reconfirmation. Unstarted NOT_STARTED and no-op preserve revisions. Saving a P1 decision does not archive. Continue accepts P1/activates P2; light_read completes P1 and retains active P1/Paper ACTIVE; archive completes P1/archives atomically with active P1. Both terminal decisions return nextPhase=null; prior P2 retains NEEDS_REVIEW and is not reset. NEW→ACTIVE occurs only on the first PRE advance, not import/upgrade/reader/draft save. goBack/touch preserves content/completion and renews the destination token only when context changes. A live PDF is an input to all phases: proof/handle outside the transaction and DB reference revalidated inside, without large hashing in DbActor. Exact R1/R2 cases appear in WORKFLOW_GATES.

**Errors:** unavailable definition/version, incomplete gate, invalid value, stale revision or persistence failure. Distinguish UI states and preserve a draft on error. **Outputs:** versioned definition, answers, reproducible gate, active phase and review flags; infer no scientific truth.

Additional required scenarios: touch(P1) blocked even when IN_PROGRESS PRE has sufficient content; touch(P2)/advance(P2) blocked when P1 is unaccepted/NEEDS_REVIEW; preserve queries of started previous phases; a PDF becoming inaccessible after accepting PRE/P1 blocks later evaluate/advance without mutation or a success receipt, available changes the current hash while the accepted snapshot remains intact. Restored access permits reevaluation; a DB Document change renews CAS/invalidates inside the UoW. Reserved COMPLETED and ARCHIVED from COMPLETED survive upgrade/read/Library archive/restore; incompatible Workflow transitions return UnsupportedCapability without effects.

## SPEC-004 — P2, global concepts, provenance and relations

**REQ-004-01** Enable forms for Concept, Claim, Evidence, Question, Gap, Assumption, Condition, Limitation, Method, Example, Insight and Reference according to the contract; future types never appear as active controls. v0.1 `item.body` is `plain_text`, without a rich-text editor. **REQ-004-02** Global Concept UUID with preferred name, definition, aliases/domain; allow reuse or creation of another sense, without automatic merging. Paper and concept views query shared entities. **REQ-004-03** Every item specifies origin (`literature`, `researcher_interpretation`, `researcher_hypothesis`); literature capture requires a source and locator or is displayed as pending. Quotation and interpreted reading are separate fields. **REQ-004-04** Provenance supports document, physical page, section, quotation/snippet, hash and stale/pending/located status. **REQ-004-05** Typed Relation validates endpoints and semantics and retains context/provenance; connect P2 with PRE questions. **REQ-004-06** The P2 gate uses the versioned required-output list: `main_questions_processed`, `field_synthesis`, `relevant_concepts_reviewed`, `meaningful_relations_reviewed`, `contradictions_reviewed`, `references_classified`, `p3_candidates_or_justification`. Resolution matrices and per-key p2ArtifactPresent belong to WORKFLOW_GATES: answers link persisted entities/relations, synthesis is an Insight, absence of artifacts is explained without quotas; the candidate output has a closed alternative of artifacts OR justified absence. The sole zero-candidate justification lives in structured PhaseAnswer; selection lives in paper_items with a checked projection. A real projection change increments answer.revision and the P2 clock in the same UoW. **REQ-004-07** Edits preserve history and mark later progress for review if required outputs become invalid; never delete data. P2 does not imply P3/P4.

```gherkin
Given two papers and a concept with a shared sense
When I link a claim from the second paper to the concept
Then the global UUID is reused and the claim retains its own source, locator and origin

Given literature capture without a resolved locator
When I save it as a draft
Then provenance is pending, not traceable/validated

Given a P2 gate without relevant P3 candidates
When I save the reasoned absence decision
Then the requirement is processed without invented candidates or a minimum count
```

**Errors:** invalid/stale locator, changed document, missing endpoint, disallowed relation, conflict and rollback. Destroy no shared knowledge and present no incompatibility as consensus. **Outputs:** paper/concept records, provenance, relations, persistable P3 queue and gate; no claim of epistemological validation.

Completing P2 requires an accepted/current chain, live PDF and real T06/T07 resolvers; missing required capability returns UnsupportedCapability, and final acceptance waits for T07. Confirm its gate and return nextPhase=null in v0.1; do not call a P3 command that would prevent closing P2. Paper remains ACTIVE and displays completed P2; global COMPLETED is reserved for the later complete workflow. The P3 queue uses the paper/item association with selection, priority and rationale, queryable even when the P3 workspace is disabled. Effective edits to artifacts used by P2 invalidate its completion without losing them.

## SPEC-005 — FTS5 search

**REQ-005-01** Index paper metadata and KnowledgeItem title/text body; do not index PDF/OCR content. **REQ-005-02** Contract-defined filters by type, concept, paper, domain, confidence and state; exclude archived records by default and allow explicit queries. Do not expose TRASHED in v0.1. **REQ-005-03** Results identify type, context and origin and open the object/locator. **REQ-005-04** The derived index is rebuildable and consistent with committed changes, never canonical authority.

```gherkin
Given paper and captured knowledge in a valid index
When I search text with paper/type filters
Then I receive only applicable matches and each result navigates to the correct object

Given interruption during an FTS update
When I rebuild from SQLite
Then results match the source and exclude archived records by default
```

**Errors:** distinguish a corrupt index or failed rebuild from zero results and offer retry; a valid query without matches is empty. **Outputs:** filterable results and index state. No search inside PDFs.

## SPEC-006 — JSONL/Markdown export

**REQ-006-01** Versioned export includes a manifest (schema/ontology versions, UTC timestamp, counts, hashes), UUIDs, entities, relations, phases/answers, associations and provenance. **REQ-006-02** JSONL has one entity per line; Markdown has paper/concept views. **REQ-006-03** Explicit option to include PDFs; shareable exports exclude private paths/logs by default. **REQ-006-04** Validate destinations against traversal and overwriting; export does not mutate SQLite; editing Markdown does not change the canonical store.

```gherkin
Given two papers share a concept and there are relations/provenance records
When I export without binary documents
Then UUID references resolve, the manifest contains counts/versions/hashes and local paths are omitted

Given a malicious title or unwritable destination
When I export
Then the operation rejects or requests a destination and never writes outside it or changes SQLite
```

**Errors:** destination cancellation, space, inconsistent serialization/hash or conflict. Confirm success only after closing and verifying the package. **Outputs:** documented package and result; no Markdown round-trip.

## SPEC-007 — Backup, restore and library switching

**REQ-007-01** Consistent SQLite snapshot including WAL, referenced documents, ontology and a manifest with hashes/versions; v0.1 policy: manual backup and backup before migration. Offer no automatic schedule/configurable retention without a contract. Never remove the last verified copy before verifying another. **REQ-007-02** Verify integrity, FKs and hashes; select an external copy with the native picker and allow another disk as destination. **REQ-007-03** Restore to a new location; open/verify before switching and keep the previous active library recoverable until confirmation. **REQ-007-04** Switch to an existing/prepared library with the writer stopped, relative paths, full verification and a recoverable operation. Automatic relocation of an existing library requires a later contract; v0.1 can produce it in another destination through backup/restore. **REQ-007-05** The UI shows the active location/result; no simulated controls.

```gherkin
Given a verified backup and a different active library
When I restore into a new folder
Then I validate DB/FKs/hashes, open the restored library and activate it only after confirmation; the previous library remains recoverable

Given an active library writer or a destination running out of space
When I change its location
Then closure is awaited or failure is explained and no partial path becomes active
```

**Errors:** WAL/inconsistency, missing resource, wrong hash, version/permissions/space, occupied destination, cancellation/interruption. Explain which library remains active. **Outputs:** verified restorable backup, operation report and active location; merely creating a file is insufficient to claim successful backup.

## SPEC-008 — Reversible maintenance and progress

**REQ-008-01** Editorial lifecycle is separate from workflow. Paper archive/restore preserves UUID, documents, metadata, answers and knowledge. **REQ-008-02** Concept and KnowledgeItem archive/restore preserves relations and provenance and hides them from active views; restoring does not change UUID. **REQ-008-03** Renaming a concept preserves identity/links. **REQ-008-04** Editing an earlier phase preserves history/later data; mark progress for review when requirements cease to hold. **REQ-008-05** Removing a paper displays dependencies and never silently cascades over global concepts/items. `TRASHED` may exist as a reserved/legacy model value, but is not exposed in the pilot/v0.1; no trash/purge operation exists and physical deletion must not be inferred. A reversible trash bin requires a later ADR/contract before enabling it.

The source specification describes reviewed merging and reversible deletion; scope agreed with DOMAIN/CONTRACTS places **merging, exposed TRASHED and permanent deletion outside the pilot/v0.1**. Create no controls for them. A reserved `TRASHED` enum value does not imply implementation; any later trash bin requires explicit ADRs, dependencies, recovery and contracts.

```gherkin
Given an archived paper with position and linked KnowledgeItems
When I restore it
Then it retains UUID/document/data and returns to its previous lifecycle

Given an archived concept and existing relations
When I restore it
Then the same UUID and all its relations/provenance return to active views

Given a P1 edit invalidating a later required output
When I save the edit
Then history and later data remain and progress is flagged for review
```

**Errors:** stale revision, dependent resource, global reference, failed rollback or unavailable restoration. Atomic rejection with dependencies; never delete shared data. **Outputs:** distinct lifecycle/phase state, visible impact and history; implemented actions only.

## Proposal traceability

ADRs and contracts are normative references for this proposal. Listed tests are planned, not executed.

| SPEC | ADR to reconcile | Semantic contracts/operations | Planned tests |
|---|---|---|---|
| 001 | ADR-001 local; ADR-002 Tauri; ADR-011 NSIS/offline; ADR-012 single writer/recovery | `getAppInfo`, `getLibraryInfo`, `DesktopLifecycle`, `MigrationGuard` | `desktop_clean_offline_install`, `upgrade_preserves_library`, `uninstall_retains_data`, `single_writer`, `migration_failure_recovery` |
| 002 | ADR-002; ADR-006 documents | `LibraryService`, `openPaper`, `ReadingPosition`, `ReaderProtocol` | `import_survives_original_move`, `duplicate_decision`, `reader_resume_after_restart`, `protocol_rejects_traversal` |
| 003 | ADR-005 gates | `evaluateGate`, `advancePhase`, `PhaseDefinition`, `PhaseAnswer`, `GateResult` | `unknown_requires_explanation`, `gate_rechecked_in_domain`, `invalidated_phase_marks_later_review`, `template_version_stable` |
| 004 | ADR-003 SQLite authority; ADR-004 concepts; ADR-005 gates; ADR-008 ontology; ADR-010 versioned IPC | `createItem/updateItem`, `createConcept/updateConcept`, `createRelation`, `attachLocator` | `concept_shared_uuid`, `literature_locator_or_pending`, `relation_endpoints_valid`, `p2_no_arbitrary_minimum`, `plain_text_roundtrip` |
| 005 | ADR-003 SQLite authority; ADR-009 modular monolith | `searchLibrary`, `searchKnowledge`, FTS projection | `fts_reference_match`, `fts_rebuild_matches_source`, `archived_excluded_by_default` |
| 006 | ADR-003; ADR-006 | `exportLibrary`, `exportPaper` | `export_references_resolve`, `private_paths_omitted`, `export_hashes_verify`, `failed_export_not_success` |
| 007 | ADR-003; ADR-006; ADR-012 | `createBackup`, `verifyBackup`, `restoreBackup`, `switchLibrary` | `backup_restore_fixture`, `wal_snapshot_consistent`, `restore_failure_keeps_active`, `move_reconciles_paths`, `retain_last_verified_backup` |
| 008 | ADR-003; ADR-004; ADR-005; ADR-012 | `archiveConcept/restoreConcept`, archive/restore paper/item; `evaluateGate` | `archive_restore_preserves_links`, `archive_paper_no_cascade`, `phase_edit_marks_review`, `trash_not_exposed_v01` |

## Definition of Ready before implementation

1. `ARCHITECTURE.md`, `DOMAIN.md` and `CONTRACTS.md` agree on canonical authority, limits, IDs, DTOs, errors, IPC version and transactions; ADR IDs match the reviewed architecture.
2. Every REQ has a service owner, persistence/DTO, observable result, error and test; pending cross-references are labeled, never invented as closed contracts.
3. Distinguish pilot 0.0.1 and v0.1.0 gates; agree labels for phases, unknowns, P1 decisions and invalidated progress.
4. PRE/P1/P2 have exact required fields, versioned `PhaseDefinition`, a reproducible gate, history policy and domain revalidation; P2 allows justified absence of candidates without arbitrary minimums.
5. Restore/switch and migrations specify WAL, files, manifest, locking, retention, failure/interruption and recovery. Archive/restore specifies dependencies. `TRASHED` is reserved and unexposed; no trash bin, merging or permanent deletion is enabled in v0.1.
6. Reader/protocol/path boundaries, Tauri/CSP capabilities, sanitization, private logs and export are specified; tests cover traversal/unknown documents.
7. Fixtures and tests are identified pending work. Distinguish mock tests, real integration, installed Windows QA and manual review where required.
8. NFR budgets remain proposals until measured; version/migration planning tests prior snapshots, checksums, downgrade and future schemas.

Record any open point in its owning document before implementing dependent behavior.

SPEC-003 / ADR-020 clarification: savePhaseAnswer in NOT_STARTED returns GateBlocked without effects or a new receipt (durable replay first). Editing a started phase even when inactive preserves activePhaseCode and applies CAS/invalidation; saving never starts or accepts phases. Cover both scenarios and stale no-op.
