# Research Workbench Domain

**Status: v0.2 execution baseline, adopted on 1 October 2026.** This document defines the baseline's ubiquitous language, boundaries, and invariants for comparison with `ARCHITECTURE.md`, `DATA.md`, `CONTRACTS.md`, and the SPECs. Rules marked as proposals resolve decisions left open by the specification.

## Scope and Versions

- **Pilot 0.0.1:** local library, PDF import and reading, metadata, resume, archive/restore, and desktop lifecycle. It does not demonstrate PRE–P2 and is not equivalent to v0.1.
- **v0.1:** complete manual PRE–P2 workflow, typed capture, global concepts, relations and provenance, gates, search, export, and verified backup/restore.
- **Later:** P3, P4, automated rules, and AI assistance; these do not retroactively change the meaning of saved data.

Entity and state names in English are stable domain/wire keys. The UI currently presents Spanish labels; the user's 8 October 2026 decision changes the presentation language to English without changing these keys. SQLite is authoritative; PDFs and other binaries live in the library; indexes and exports are derived.

## Ubiquitous Language

| Term | Meaning |
|---|---|
| Paper | Bibliographic source and unit of reading; owns metadata, documents, and lifecycle. |
| Document | A specific version of a managed source file, identified by UUID and SHA-256. Anchors point to the Document, never only to the Paper. |
| Phase | A versioned work contract (PRE, P1, P2, P3, P4) associated with a Paper. It is not the Paper's editorial lifecycle. |
| PhaseAnswer | Typed answer to a question or required output of a Phase. UNKNOWN and NOT_APPLICABLE are explicit decisions, not empty values. |
| KnowledgeItem | Semantic unit with UUID, type, content, origin, lifecycle, and confidence. |
| Concept | A KnowledgeItem shared across Papers and reusable globally; it is not automatically duplicated per source. |
| Relation | Directed, typed link between KnowledgeItems, with its own origin and context. |
| Provenance | Trail to a Document and a concrete or explicitly pending locator. It does not imply that a claim is true. |
| Gate | Reproducible evaluation of a phase's outputs. It measures declared completeness, not scientific truth. |
| Revision | Optimistic counter for the affected aggregate, incremented on every committed mutation. |
| Archive | Reversible removal from active views while preserving identity, content, and links. |
| Trash | Recoverable removal state before possible future deletion. Physical deletion is not part of the pilot or v0.1 contract. |

## Aggregate Boundaries and Consistency

These are not microservices; the boundaries organize validation inside the monolith. Operational proposal: a dedicated actor serializes commands over one SQLite write connection. Commands and queries are logically separate, but cross-module changes use the same `UnitOfWork` and a single transaction/connection; do not open a transaction per module.

| Aggregate | Root and ownership | Invariants and transaction |
|---|---|---|
| Paper | Paper | Metadata, ordered authors, venue, lifecycle, and references to documents/processing; `PhaseRecord` owns its answers and rules. Creating/importing a Paper together with authors/document is coordinated; success is not visible until commit. Archive/restore preserves IDs and dependencies. |
| Document | Document | Hash is immutable per version. The relative path stays within the managed root. Replacing a file creates a new version; existing anchors are not silently changed. |
| PhaseRecord | (`paper_id`, `phase_code`) | Definition version is pinned when processing is initialized; answers and history have a revision. The advance gate is recalculated in the same transaction as the advance. Editing an earlier phase preserves later phases and marks them NEEDS_REVIEW if a changed output affects their gate. |
| KnowledgeItem | KnowledgeItem | Globally unique UUID. Subtype/extension agrees with `type_code`. Multi-table capture and hyperlinks are committed together. Archiving a shared item does not delete it or destroy provenance. |
| Concept | KnowledgeItem of type `concept` | Preferred name and normalized aliases help suggest searches; they do not prove equivalence. A person always chooses reuse; renaming preserves the UUID. |
| Relation | Relation | Endpoints exist and types are compatible; no self-link. Semantic uniqueness preserves distinct contexts. Relation provenance is stored with the relation. |
| Provenance | Provenance | Belongs to a Document and stores the hash observed at capture. Literal quotation and interpretive text remain separate. A different hash makes the locator `STALE`; it is not rewritten. |

Operations spanning SQLite and the filesystem use a recoverable local saga, not a pretend atomic transaction: durable intent, staging on the same volume, verified hash, promotion, commit, and idempotent reconciliation at startup. An ambiguous file is not automatically deleted.

## Canonical Enums

The following exact values are persisted; presentation and translation do not change these keys:

```text
PaperLifecycle = NEW | ACTIVE | COMPLETED | ARCHIVED | TRASHED
PhaseCode = PRE | P1 | P2 | P3 | P4
PhaseState = NOT_STARTED | IN_PROGRESS | COMPLETED | NEEDS_REVIEW
AnswerResolution = PENDING | ANSWERED | UNKNOWN | NOT_APPLICABLE
QuestionState = OPEN | ANSWERED | DEFERRED | DISMISSED
Origin = literature | researcher_interpretation | researcher_hypothesis
Confidence = sufficiently_supported | context_dependent | uncertain | requires_validation
LocatorState = PENDING | LOCATED | STALE
ReviewType = survey | topical_review | slr | mapping_study | tutorial | other | unknown
Relevance = sufficient | use_with_caution | weak_for_my_purpose
ReadingDecision = continue | light_read | archive
```

**Editorial status:** `COMPLETED` is assigned only when the complete workflow through P4 is implemented and satisfied. In v0.1, completing P2 does not change the Paper to COMPLETED; the UI describes completed P2 or the complete initial review. This does not represent scientific certainty. When a Paper is archived from NEW/ACTIVE/COMPLETED, save `archived_from_lifecycle`; restore returns it to that state. `TRASHED` is reserved for the future and must not be exposed in the pilot or v0.1.

## Operational Phases and Gates

Definitions include `code`, `version`, `objective`, `key_questions`, `do_items`, `dont_items`, `considerations`, `required_outputs`, and `completion_rules`. The canonical v1 payloads are in [phase-definitions/](phase-definitions/); each Paper pins PRE/P1/P2 when processing is initialized. They are immutable and may be embedded at compile time. [WORKFLOW_GATES.md](WORKFLOW_GATES.md), ADR-017, specifies resolutions, per-output rules, scoring, projections, snapshots, and review without changing the wire contract.

General rule: a required output is processed according to `allowedResolutions`. ANSWERED text requires content; UNKNOWN/NOT_APPLICABLE requires a non-empty explanation and must not duplicate it in `answerText`. PRE `review_type`, P1 `relevance_decision`, and P2 `p3_candidates_or_justification` are processed through valid structure without redundant prose. PENDING/null may be saved as an outstanding response but never closes a gate. There is no epistemic score or quota. Exact form behavior, justified absence, and the closed P2 alternative for each key are defined in WORKFLOW_GATES.

**Deterministic invalidation:** a real change to `answerText`, `structuredValue`, `resolution`, or `explanation` in a COMPLETED phase changes it to NEEDS_REVIEW, even if its gate still passes; `advancePhase` completes it again. A change to a NOT_STARTED phase also moves it to NEEDS_REVIEW while preserving its data. An edit to an item, concept association, relation, provenance, or P3 candidacy that changes the completed P2 snapshot moves P2 to NEEDS_REVIEW. A no-op with the same canonical value does not invalidate. The accepted completion snapshot contains the versioned definition, normalized answers, and stable projection of required outputs/entities; its hash uses ordered canonical serialization.

### PRE — Prior Context

PRE's required visible outputs are `purpose`, `uncertainty_target`, `baseline`, `expected_outcome`, `desired_depth`, and `review_type`. The sixth response confirms the Paper's `reviewType` metadata using `{reviewType: ReviewType}`: known types resolve to ANSWERED; unknown resolves to UNKNOWN plus an explanation. The gate compares the current capture with metadata; an import default is not processing, and the answer does not edit the bibliography. The gate also requires a non-empty title and an existing ACTIVE Document. The five preceding questions retain their meaning; do not infer conclusions from the title.

Only a successful PRE advance accepts its snapshot and changes NEW→ACTIVE in the same `UnitOfWork`. Enabling P1 requires PRE COMPLETED, an accepted snapshot, and a passing gate. Enabling P2 requires an accepted chain and P1 `readingDecision = continue`. Touching or saving does not implicitly accept or complete a phase. All phases observe live document availability as defined in WORKFLOW_GATES.

### P1 — Orientation

P1 required outputs: `scope`, `out_of_scope`, `review_type`, `literature_cutoff`, `core_message`, and `relevance_decision`; `field_organization` is optional. P1 `review_type` characterizes the approach/methodology according to the source; it is not a second editor for the metadata confirmed in PRE. Unstated literature/methodology is recorded as explained UNKNOWN where the definition allows; do not invent it.

`relevance_decision` combines an assessment (`sufficient`, `use_with_caution`, `weak_for_my_purpose`) and a decision (`continue`, `light_read`, `archive`). ANSWERED requires both enums; UNKNOWN/NOT_APPLICABLE do not invent a branch. Only `continue` enables P2. `light_read` completes P1, leaves P1 active, keeps the Paper ACTIVE, and sets `nextPhase = null`. `archive` is applied only by the full P1 advance and archives the Paper atomically, preserving its previous lifecycle, answers, and active P1; `nextPhase = null`. P2, if never started, remains NOT_STARTED; if previously started, its data is preserved and it becomes NEEDS_REVIEW. None of these decisions completes the entire workflow.

### P2 — Conceptual Model

Canonical P2 outputs: `main_questions_processed`, `field_synthesis`, `relevant_concepts_reviewed`, `meaningful_relations_reviewed`, `contradictions_reviewed`, `references_classified`, and `p3_candidates_or_justification`. Questions are related to PRE. Each output links to queryable entities/relations; synthesis is retained as an Insight. Empty arrays require an explicit justification of absence, without arbitrary minimum counts. UNKNOWN/NOT_APPLICABLE are allowed only where the definition permits. The exact key mapping and persistent projection are defined in WORKFLOW_GATES; a narrative checkbox does not replace artifacts.

Completing P2 processes the manual map and P3 queue; it does not validate claims or the priority/rationale of Claims/Questions. The only justification for zero candidates is `PhaseAnswer(P2, p3_candidates_or_justification).structuredValue.noCandidatesJustification`; candidates/count are a proven projection. `setP3Candidate` and saving an absence answer operate on that source in the same `UnitOfWork`; a changed projection renews the answer revision and P2 clock. The P3 workspace may be disabled. Final acceptance requires all T06/T07 capabilities; without a resolver, return UnsupportedCapability.

P3 and P4 have reserved codes but are not executable in v0.1. Do not mark a Paper COMPLETED after P2 while the product definition still requires other operational phases; within v0.1 the UI should describe completed P2 / complete initial review.

## Knowledge

Core type catalog: `concept`, `claim`, `evidence`, `question`, `gap`, `assumption`, `condition`, `limitation`, `method`, `example`, `insight`, `reference`. The catalog supports these types, but forms may be deployed gradually. Metric/Task/System and domain packs are deferred.

- `literature`: content describes what the source states. To present it as traceable, require Provenance `LOCATED`; draft captures may be saved without a source (`NONE`, to be attributed) or with a source but no anchor (`PENDING`), visibly marked as pending.
- `researcher_interpretation`: the researcher's interpretation; do not present it as a verbatim quotation. It may link to the source that motivated it.
- `researcher_hypothesis`: an explicit hypothesis attributed to the researcher; it may lack a citation, but its context/reason is recorded.
- Quote/snippet is literal and separate from body/interpretation. `page_index` is the physical page number starting at 1; `page_label` may differ. A matching captured/current document hash may mark a locator LOCATED; changing the file/hash marks it STALE.
- Initial Confidence is `requires_validation`. Only an explicit human decision may change it. The app does not calculate confidence by counting quotations, relations, text matches, or phase states.
- Evidence describes an inspected passage or result; its state is not confused with a Claim's confidence. A DOI or secondary citation alone is not evaluated evidence.

## Relations Allowed in v0.1

Every relation stores `source`, `target`, `type`, `context`, `origin`, `confidence`, dates, and separate provenance. Proposed initial allowlist for forms/validation:

| type | permitted source → target | Symmetric |
|---|---|---|
| supports | evidence → claim | no |
| contradicts | claim ↔ claim | yes, normalize endpoints |
| extends | method → method; claim → claim | no |
| causes | claim → claim or concept | no |
| requires | method → condition | no |
| depends_on | method → concept or method | no |
| works_when | method → condition | no |
| fails_when | method → condition | no |
| compares_with | method ↔ method | yes |
| part_of | concept → concept | no |
| similar_to | concept ↔ concept | yes |
| limits | limitation → claim or method | no |
| improves | method → method | no |

The v0.1 allowlist enables exactly these 13 types and combinations; the persisted catalog and `RelationType` use the same version. `causes`/`improves` require explicit context and origin; choosing them does not prove causation or improvement. Every relation allows context; different contexts are not deduplicated. Reject endpoints outside the matrix. Directed relations preserve endpoint order; symmetric relations normalize it.

## Archive, Restore, Delete, and Merge

- **Paper archive/restore:** reversible; preserves documents, metadata, phases, KnowledgeItems, and provenance. Archiving a Paper does not archive global content or delete shared dependencies.
- **Item/Concept archive/restore:** reversible; excluded from active lists by default, while historical links remain visible and the item can be restored.
- **Delete Paper/KnowledgeItem/Concept:** in pilot 0.0.1 and v0.1, no hard delete and no exposed TRASHED state. Paper and KnowledgeItem/Concept use reversible archive/restore; relations and provenance are retained. `TRASHED` is reserved for a possible future trash feature.
- **Merge Concept:** outside pilot 0.0.1 and v0.1; requires an ADR for post-v0.1 scope. If approved later, it is an explicit operation with a preview of affected links/aliases/relations, a chosen destination, and confirmation. The transaction reassigns safe references, deduplicates only equivalent relations while preserving context, marks the source as redirected to the destination, and records an audit. The source UUID remains; reject cycles, self-relations, or ambiguous collisions. Reject if semantics/provenance cannot be preserved.

## Outside the v0.1 Domain

Users/roles, synchronization, remote API, microservices, OCR, automatic analysis, LLMs/agents, RAG/embeddings, operational P3/P4, automatic confidence score, generic ontology editor, and complex visual graph. Future rules report only gaps in the record; they do not turn an individual's lack of coverage into a gap in all literature.

Reserved COMPLETED is preserved in upgrade/get/read and Library archive/restore, including ARCHIVED from COMPLETED. Incompatible Workflow transitions return UnsupportedCapability without side effects; there is no silent conversion or special migration. Upgrade does not change metadata/lifecycle or restore archived Papers; PRE NEW drafts remain editable, and only PRE advance activates them editorially.

ADR-020 editing clarification: saving does not start a phase; NOT_STARTED rejects response writes, while an initiated phase may be edited without moving the active context. Replay, CAS, clocks, and invalidation are governed by WORKFLOW_GATES.

## Shared Capture Clarifications (ADR-022)

Concept preserves its canonical name/definition and UUID; Knowledge modifies only its confidence, while ConceptApi controls the rest. `Insight.affectedConceptIds` classifies an explicit subset of linked concepts: removing that classification does not remove the link, and `linkConcept` does not add the classification. Homonyms are not merged. Editing/linking from an archived parent requires restore; new associations to an archived Concept are rejected, while historical associations still resolve. Repeating an existing association from an active parent is a no-op even when the destination has been archived.

The P2 projection starts from `paper_items` and follows `item_concepts` transitively, deduplicating by UUID to terminate cycles. It includes only relations whose two endpoints have already been reached; relations do not expand reachability. Archived items remain in history/projection. Reverse invalidation uses the same closure before and after a change. Editing an artifact renews the P2 clock when appropriate; a candidate `PhaseAnswer` changes only when selection/count/justification changes, not when the body is edited without changing that projection.

Provenance without a source is NONE, not a fabricated row. LOCATED means a locator is registered as specified in CONTRACTS; it is not a verified citation or proof of physical page. T07 validates navigation. Editing a locator preserves its Document/hash and does not clear STALE; changing source requires an explicit new attachment. Full signatures, constraints, and criteria are in CONTRACTS and TASK05_PORTS.
