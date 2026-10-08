# Research Workbench IPC contracts

**Status: execution baseline v0.2, adopted on 1 October 2026.** This document defines the project's internal Tauri IPC contract. It is neither an HTTP nor a public API. Neutral DTOs, JSON `camelCase` serialization, SQLite canonical authority. It must remain consistent with `DOMAIN.md`, `DATA.md` and the SPECs.

## Version, compatibility and availability

- `contractVersion = 1` is the IPC envelope version, independent of `schemaVersion`, `ontologyVersion` and the app version.
- UUIDs are transmitted as canonical strings; timestamps as UTC RFC3339; enums use exactly the keys in `DOMAIN.md` (uppercase where specified). Under ADR-019, canonical date writers use milliseconds and the `Z` suffix; readers also accept `+00:00` with full validation and preserve the original string. Reject local dates, other offsets and `-00:00`; do not rewrite earlier receipts.
- **Pilot 0.0.1** implements Library, Reader and Desktop lifecycle. **v0.1** adds Workflow, Knowledge, Concepts, Relations, Provenance, Search, Export, Backup and Settings. This does not relabel the pilot as v0.1.
- DTOs for future modules may be defined here, but unimplemented commands must not simulate success; the capability registry marks them `available: false` until an implementing version.
- Incompatible JSON changes require incrementing `contractVersion`; tolerable additive changes retain the version and never reuse a field with different semantics.

## Envelope and error

Every command returns this success envelope. Tauri errors must also use the envelope, not ambiguous serialized text; the TypeScript adapter normalizes invoke errors into this same form.

```ts
type IpcSuccess<T> = {
  contractVersion: 1;
  requestId: UUID;
  ok: true;
  data: T;
};
type IpcFailure = {
  contractVersion: 1;
  requestId: UUID;
  ok: false;
  error: {
    code: IpcErrorCode;
    message: string;          // safe for the UI, English
    retryable: boolean;
    details?: JsonObject;     // bounded structure, no paths/sensitive content
  };
};
type IpcResult<T> = IpcSuccess<T> | IpcFailure;
type IpcErrorCode =
  | 'InvalidInput' | 'NotFound' | 'Conflict' | 'DuplicateDecisionRequired'
  | 'SourceUnreadable' | 'InvalidPdf' | 'StorageUnavailable'
  | 'ImportRecoveryRequired' | 'GateBlocked' | 'UnsupportedCapability'
  | 'PathNotAllowed' | 'OperationCancelled' | 'Busy' | 'IntegrityFailure'
  | 'SchemaTooNew' | 'MigrationFailed' | 'BackupFailed' | 'ExportFailed';
```

Every call carries a client-generated UUID `requestId`. `confirmImport`, cancellation and all persistent mutations receive a durable receipt; idempotent/destructive commands persist `requestId`, payload hash and result in the table described in DATA.md. Retrying the same requestId/payload returns the prior result; a different payload returns `Conflict`. Reads store no receipt. Updates require `expectedRevision`; conflicts include `currentRevision` and never overwrite. List pagination uses an opaque cursor, with a proposed limit of 100 (maximum 500).

## Shared DTOs

```ts
type UUID = string;
type ReviewType = 'survey' | 'topical_review' | 'slr' | 'mapping_study'
  | 'tutorial' | 'other' | 'unknown';
type PaperLifecycle = 'NEW' | 'ACTIVE' | 'COMPLETED' | 'ARCHIVED' | 'TRASHED';
type PhaseCode = 'PRE' | 'P1' | 'P2' | 'P3' | 'P4';
type PhaseState = 'NOT_STARTED' | 'IN_PROGRESS' | 'COMPLETED' | 'NEEDS_REVIEW';
type AnswerResolution = 'PENDING' | 'ANSWERED' | 'UNKNOWN' | 'NOT_APPLICABLE';
type Origin = 'literature' | 'researcher_interpretation' | 'researcher_hypothesis';
type Confidence = 'sufficiently_supported' | 'context_dependent' | 'uncertain' | 'requires_validation';
type LocatorState = 'PENDING' | 'LOCATED' | 'STALE';
type RelevanceRating = 'sufficient'|'use_with_caution'|'weak_for_my_purpose';
type ReadingDecision = 'continue'|'light_read'|'archive';
type RelevanceDecisionValue = { relevance: RelevanceRating; readingDecision: ReadingDecision };

type PaperMetadataInput = {
  title: string;                       // trim; 1..1000 characters (proposed limit)
  authors: string[];                   // bibliographic order; each name trimmed/non-empty
  year: number | null;                 // null or a four-digit year
  doi: string | null;                  // normalized before uniqueness checking
  venue: string | null;
  reviewType: ReviewType;
  domain: string | null;
};
type PaperDto = PaperMetadataInput & {
  id: UUID;
  documentId: UUID;
  lifecycle: PaperLifecycle;
  archivedFromLifecycle: Exclude<PaperLifecycle, 'ARCHIVED'|'TRASHED'> | null;
  activePhaseCode: PhaseCode | null;
  processingInitialized: boolean;
  revision: number;
  createdAt: string;
  updatedAt: string;
  lastOpenedAt: string | null;
};
type DocumentDto = {
  id: UUID; paperId: UUID; originalFilename: string; sha256: string;
  importedAt: string; status: 'ACTIVE'|'SUPERSEDED'|'MISSING';
};
type DuplicateCandidateDto = {
  paperId: UUID; title: string; reasons: Array<'doi'|'sha256'>;
};
type ImportPreviewDto = {
  importToken: UUID; originalFilename: string; sizeBytes: number;
  sha256: string; candidates: DuplicateCandidateDto[];
  expiresAt: string;                  // token valid for 24 h; recovery may renew it
};
type DuplicateResolution = { action: 'reuseExisting'; paperId: UUID };
type PaperFilterDto = {
  lifecycle: 'ACTIVE'|'ARCHIVED'|'ALL'; // ACTIVE includes NEW/ACTIVE/COMPLETED
  query: string; yearFrom: number|null; yearTo: number|null;
  reviewTypes: ReviewType[]; domain: string|null; phase: PhaseCode|null;
  cursor: string|null; limit: number;
};
type PageDto<T> = { items: T[]; nextCursor: string|null; total?: number };
type RevisionDto = { revision: number; updatedAt: string };
```

v1 limits: title 1000 chars; answer/body 20,000; snippet/quote 10,000; relation/context 5,000; maximum 100 authors per paper; PDF maximum 500 MiB. Search query 1000 chars, excerpt 500 chars, default 100 results per page, maximum 500. The backend validates all limits. Reject `NaN`, infinities, unselected paths and unknown/oversized JSON; never truncate. Pilot fixtures must include Unicode, long names and files near the limit to check handling without changing these maxima.

## Library module

Retains the plan's summarized `LibraryApi`, with explicit corrections for duplicates and opening/resuming. The UI requests file selection only through the native picker; it receives no arbitrary path-read permission.

```ts
interface LibraryApi {
  selectPdf(requestId: UUID): Promise<IpcResult<ImportPreviewDto|null>>;
  confirmImport(args: { requestId: UUID; importToken: UUID;
    metadata: PaperMetadataInput; duplicateResolution?: DuplicateResolution }): Promise<IpcResult<PaperDto>>;
  cancelImport(args: { requestId: UUID; importToken: UUID }): Promise<IpcResult<null>>;
  listPapers(args: { requestId: UUID; filter: PaperFilterDto }): Promise<IpcResult<PageDto<PaperDto>>>;
  getPaper(args: { requestId: UUID; paperId: UUID }): Promise<IpcResult<PaperDto>>;
  updateMetadata(args: { requestId: UUID; paperId: UUID;
    expectedRevision: number; metadata: PaperMetadataInput }): Promise<IpcResult<PaperDto>>;
  archivePaper(args: { requestId: UUID; paperId: UUID;
    expectedRevision: number }): Promise<IpcResult<PaperDto>>;
  restorePaper(args: { requestId: UUID; paperId: UUID;
    expectedRevision: number }): Promise<IpcResult<PaperDto>>;
}
```

`selectPdf` returns `null` when the dialog is cancelled. Normalize DOI: trim, remove `doi:` or the `https://doi.org/`/`http://doi.org/` host, lowercase and validate `10.<registrant>/<suffix>`; do not resolve it over the network. When DOI/hash candidates exist, `confirmImport` requires `reuseExisting`, or cancellation through `cancelImport`; creating another Paper with the same DOI or SHA-256 is disallowed in v0.1. Reuse returns the existing Paper without altering metadata or document. Confirming the same importToken with the same payload returns the prior PaperDto; an incompatible payload returns Conflict. Tokens are valid for 24 h; recovery may renew an intent's token. A normalized DOI never creates a duplicate Paper. Matching does not semantically merge concepts.
When confirmImport returns DuplicateDecisionRequired, error.details.candidates contains a non-empty list of DuplicateCandidateDto from the transaction detecting the duplicate, including the final post-promotion check. IDs are unique and sorted by paperId; reasons are unique in doi, sha256 order; no paths or PDF content. The error neither confirms import nor allows changing an already bound intent's payload. No new IPC/DTO or contractVersion change. The UI offers opening the candidate through confirmed cancelImport followed by Reader.openPaper; it does not silently change duplicateResolution or search for candidates by scanning listPapers. Cancellation failure retains the dialog/draft without opening; subsequent opening failure permits retrying Reader only. reuseExisting retains backend semantics for compatible requests. Exact interaction is in TASK03_BOUNDARIES.

Preconditions/postconditions: the token exists, has not expired and belongs to the process/library; the source is already copied into staging. On success, Paper+authors+Document and import state commit consistently. On recoverable error, staging remains bound to the intent; report no success. Cancellation affects only the specified token. Archive/restore preserves UUID, document and relations and requires expectedRevision.

## Reader module

Opening a Paper updates the last-opened paper, activity and contextual phase in one transaction; this fills the missing last-opened setter in the earlier summary.

```ts
type ReadingPositionDto = { documentId: UUID; pageIndex: number; zoom: number;
  revision: number; updatedAt: string };
type OpenPaperDto = { paper: PaperDto; document: DocumentDto;
  readingPosition: ReadingPositionDto; documentUrl: string };
interface ReaderApi {
  openPaper(args: { requestId: UUID; paperId: UUID }): Promise<IpcResult<OpenPaperDto>>;
  getLastOpenedPaper(args: { requestId: UUID }): Promise<IpcResult<PaperDto|null>>;
  getReadingPosition(args: { requestId: UUID; documentId: UUID }): Promise<IpcResult<ReadingPositionDto>>;
  saveReadingPosition(args: { requestId: UUID; documentId: UUID;
    expectedRevision: number; pageIndex: number; zoom: number }): Promise<IpcResult<ReadingPositionDto>>;
}
```

If no row exists, `getReadingPosition` returns deterministic defaults `pageIndex=1, zoom=1.0, revision=0, updatedAt=document.importedAt` without creating a row. Page index is an integer numbered from one; zoom is finite with proposed range 0.25–5.0. The backend does not know PDF page count; the frontend bounds the page and the backend validates structural range. Every document retains a separate position. Saves are serialized per document with optimistic revision; the client does not treat an old response as current state. Only the service emits `documentUrl`, after verifying a registered Document, UUID and canonical path inside the root; the protocol rejects traversal and unregistered files. It serves only that Document's PDF, never an arbitrary path. Render cancellation does not cancel already committed persistence.

openPaper records lastOpenedAt, session context, audit and receipt in one transaction; it preserves bibliographic Paper.revision and Paper.updatedAt and changes neither metadata nor phase. Replay does not record activity again. Before emitting documentUrl, check current access, including during replay: a prior receipt does not guarantee file availability. Reading profile and local resources: ADR-015 and docs/plans/TASK03_BOUNDARIES.md.

## Workflow module

`PhaseDefinitionDto` is a declarative immutable snapshot: `code`, `version`, `name`, `objective`, `keyQuestions`, `doItems`, `dontItems`, `considerations`, `requiredOutputs`, `completionRules`, `definitionHash`. Each output is `{key,label,prompt,required,allowedResolutions,ruleKey}`; it contains no scripts, evaluable expressions or code. `ruleKey` belongs to the closed enum `answerProcessed | paperHasActiveDocument | p1DecisionProcessed | p2ArtifactPresent | p2NoCandidatesJustified`. A processed P1 decision satisfies P1 completion for all three values; `continue`/`light_read`/`archive` branching occurs in `advancePhase`, not as a gate requirement excluding two options. The definition pinned when processing is created does not change when a new version is installed; preserve the prior snapshot in DB and export. `PhaseAnswerDto`: canonical resolution is in [WORKFLOW_GATES.md](WORKFLOW_GATES.md); [PRE/P1/P2 v1 JSON](phase-definitions/) fixes immutable payloads embeddable at compilation. completionRules lists handlers per output with a closed candidates/absence alternative, not a global conjunction.

```ts
type PhaseRuleKey = 'answerProcessed' | 'paperHasActiveDocument' | 'p1DecisionProcessed'
  | 'p2ArtifactPresent' | 'p2NoCandidatesJustified';
type PhaseRequiredOutputDto = { key: string; label: string; prompt: string;
  required: boolean; allowedResolutions: AnswerResolution[]; ruleKey: PhaseRuleKey };
type PhaseDefinitionDto = { code: PhaseCode; version: number; name: string;
  objective: string; keyQuestions: string[]; doItems: string[]; dontItems: string[];
  considerations: string[]; requiredOutputs: PhaseRequiredOutputDto[];
  completionRules: PhaseRuleKey[]; definitionHash: string };
type PhaseAnswerDto = { paperId: UUID; phaseCode: PhaseCode; questionKey: string;
  answerText: string; structuredValue: JsonValue|null; resolution: AnswerResolution; explanation: string|null;
  revision: number; updatedAt: string };
type GateIssueDto = { requirementKey: string; message: string;
  status: 'missing'|'pending'|'invalid' };
type GateEvaluationDto = { paperId: UUID; phaseCode: PhaseCode; definitionVersion: number;
  complete: boolean; issues: GateIssueDto[]; phaseRevision: number;
  inputSnapshotHash: string; evaluatedAt: string };
type P3CandidateDto = { itemId: UUID; priority: number|null; rationale: string|null };
type P3CandidateSummaryDto = { paperId: UUID; candidates: P3CandidateDto[];
  candidateCount: number; noCandidatesJustification: string|null; workflowRevision: number };
type PhaseDto = { paperId: UUID; code: PhaseCode; definitionVersion: number;
  state: PhaseState; revision: number; completedAt: string|null };
```

```ts
interface WorkflowApi {
  getPhase(args: { requestId: UUID; paperId: UUID; phaseCode: PhaseCode }): Promise<IpcResult<PhaseDto>>;
  getPhaseAnswers(args: { requestId: UUID; paperId: UUID; phaseCode: PhaseCode }): Promise<IpcResult<PhaseAnswerDto[]>>;
  getPhaseDefinition(args: { requestId: UUID; phaseCode: PhaseCode; version?: number }): Promise<IpcResult<PhaseDefinitionDto>>;
  savePhaseAnswer(args: { requestId: UUID; paperId: UUID;
    phaseCode: PhaseCode; questionKey: string; expectedRevision: number;
    answerText: string; structuredValue: JsonValue|null;
    resolution: AnswerResolution; explanation: string|null }): Promise<IpcResult<PhaseAnswerDto>>;
  evaluateGate(args: { requestId: UUID; paperId: UUID; phaseCode: PhaseCode }): Promise<IpcResult<GateEvaluationDto>>;
  advancePhase(args: { requestId: UUID; paperId: UUID;
    fromPhase: PhaseCode; expectedPhaseRevision: number }): Promise<IpcResult<{ phase: PhaseDto;
      gate: GateEvaluationDto; nextPhase: PhaseCode|null; paperLifecycle: PaperLifecycle }>>;
  goBackToPhase(args: { requestId: UUID; paperId: UUID; targetPhase: PhaseCode;
    expectedActivePhaseRevision: number }): Promise<IpcResult<{ activePhaseCode: PhaseCode; activePhaseRevision: number }>>;
  touchPhase(args: { requestId: UUID; paperId: UUID; phaseCode: PhaseCode;
    expectedActivePhaseRevision: number }): Promise<IpcResult<PhaseDto>>;
  setP3Candidate(args: { requestId: UUID; paperId: UUID; itemId: UUID; selected: boolean;
    priority: number|null; rationale: string|null; noCandidatesJustification: string|null;
    expectedWorkflowRevision: number }): Promise<IpcResult<P3CandidateSummaryDto>>;
  getP3CandidateSummary(args: { requestId: UUID; paperId: UUID }): Promise<IpcResult<P3CandidateSummaryDto>>;
}
```

**Phase editing (ADR-020):** for a new request, savePhaseAnswer on NOT_STARTED returns GateBlocked without an answer, clock/context change, success audit or new receipt. Saving does not start a phase. An already started phase may be edited while inactive without reaccepting its prerequisite chain or changing activePhaseCode: answer CAS, no-op and invalidation retain their rules. Resolve durable replay before this guard; lifecycle and capability restrictions remain applicable (P2 is not enabled in T04).

ANSWERED `relevance_decision` requires complete enum-defined `structuredValue: RelevanceDecisionValue`; PENDING/null saves a draft without branch selection, and UNKNOWN/NA are disallowed. Never interpret answerText. PRE.review_type confirms Paper.reviewType using the structure and resolution in [WORKFLOW_GATES.md](WORKFLOW_GATES.md); P1.review_type characterizes the source textually. Explained textual UNKNOWN/NA does not require repeating the explanation as answerText. savePhaseAnswer.expectedRevision protects the answer: first write expected=0/revision=1; replay before CAS, no-op after CAS preserves revisions/snapshots. Saving never archives.

evaluateGate is a read with canonical snapshot/hash. advance compares expectedPhaseRevision of active fromPhase and reevaluates under IMMEDIATE; snapshot/phase/context/revisions/lifecycle/audit/receipt commit together. P1 continue activates P2; light_read completes P1 with Paper ACTIVE, active P1, nextPhase=null; archive completes P1+Paper ARCHIVED+archivedFromLifecycle with active P1, nextPhase=null. P2 never started remains NOT_STARTED on terminal branches; previous P2 retains data/NEEDS_REVIEW. Closing P2 retains active P2 and Paper ACTIVE, nextPhase=null, without P3. Stale inputs or a blocked gate produce Conflict/GateBlocked without effects.

goBack/touch compares the currently active phase, not the destination. Navigation preserves answers/state/completedAt/snapshot; a real context change renews the destination revision with a fresh max(revision)+1 clock per Paper under the UoW, without a new table/wire. Forward enablement—initialization or activation—and completion require a COMPLETED prerequisite, accepted snapshot and current gate, plus accepted continue for P2. Touch does not accept prerequisites; reading/navigating already started data does not authorize advance. Exact definitions/rules/clocks are in WORKFLOW_GATES.

PRE/P1/P2 includes the current live Document {id,status,sha256,available} in the snapshot. Obtain access proof/retained handle before the transaction; revalidate the DB reference inside it. Finalize document ABI after T03, without large PDF reads/hashes under DbActor. An inaccessible file blocks without mutation/success receipt; external disappearance does not increment a fictitious clock. A DB Document change invalidates/renews revisions in its UoW.

setP3Candidate.expectedWorkflowRevision is the paper_phases(P2) revision. Only associated Claims/Questions; priority is null or integer1..5, rationale is 1..5000 when selecting. Associations are selection authority; the sole zero-candidate justification lives in the existing structured PhaseAnswer and must be null when candidates exist. Saving that key permits absence without a fictitious item and never changes selection. Summary and gate use the same IDs/count/justification. An effective projection change renews the answer and phase clock in the same UoW. Six P2 outputs link artifacts per key; p2ArtifactPresent OR p2NoCandidatesJustified applies only to candidates, never a global AND or declarative code. Missing required capability returns UnsupportedCapability; final P2 acceptance waits for T07.

After an effective change, a COMPLETED phase and already started later phases become NEEDS_REVIEW with data/historical snapshot preserved; NOT_STARTED remains unstarted. Canonical no-op does not invalidate; advance requires reconfirmation. Normalization, artifacts, absence and snapshot details are closed in WORKFLOW_GATES.

In pilot 0.0.1, processingInitialized=false. Workflow v0.1 imports/initializes PRE IN_PROGRESS, P1/P2 NOT_STARTED, active PRE and pins all three v1 definitions in the same commit. Upgrade preserves UUID/metadata/document/lifecycle/archivedFrom/bibliographic revisions; it creates no PRE.review_type confirmation and does not restore. Only the first successful PRE advance changes NEW→ACTIVE. Accepted/current enablement conditions in WORKFLOW_GATES govern P1/P2. Completed P2 retains Paper ACTIVE, not global COMPLETED. Reserved COMPLETED survives read/upgrade/Library archive/restore; incompatible Workflow transitions return UnsupportedCapability without effects. T04 may read/pin/activate P2 through accepted continue, but saves/gates/advance/candidates remain capability-gated until real T06/T07 resolvers.

## Knowledge, Concepts and Relations modules

```ts
type KnowledgeType = 'concept'|'claim'|'evidence'|'question'|'gap'|'assumption'|'condition'
  |'limitation'|'method'|'example'|'insight'|'reference';
type JsonValue = null | boolean | number | string | JsonValue[] | JsonObject;
type JsonObject = { [key: string]: JsonValue };
// Numbers must be finite. Objects contain data only: no paths, code, prototypes or SQL.
type KnowledgeAttributes =
  | { typeCode: 'claim'; scope: string|null; conditions: string|null; limitations: string|null }
  | { typeCode: 'evidence'; evidenceKind: string; observedResult: string|null; conditionsText: string|null; limitationsText: string|null }
  | { typeCode: 'question'; state: 'OPEN'|'ANSWERED'|'DEFERRED'|'DISMISSED'; answerText: string|null; nextAction: string|null }
  | { typeCode: 'gap'; scope: string; justification: string; searchPending: string|null }
  | { typeCode: 'assumption'; context: string|null }
  | { typeCode: 'condition'; applicabilityScope: string|null }
  | { typeCode: 'limitation'; effect: string|null; applicabilityScope: string|null }
  | { typeCode: 'method'; family: string|null; context: string|null }
  | { typeCode: 'example'; description: string|null }
  | { typeCode: 'insight'; baseline: string; interpretation: string; affectedConceptIds: UUID[] }
  | { typeCode: 'reference'; identifier: string|null; reason: string; state: 'TO_REVIEW'|'INSPECTED'|'DISMISSED' }
  | { typeCode: 'concept'; preferredName: string; definition: string; aliases: string[]; domain: string|null };
type KnowledgeItemInput = { typeCode: Exclude<KnowledgeType,'concept'>; title: string;
  bodyText: string; origin: Origin; confidence: Confidence;
  attributes: Exclude<KnowledgeAttributes,{typeCode:'concept'}> };
type KnowledgeAttributesPatch =
  | { typeCode: 'claim'; scope: string|null; conditions: string|null; limitations: string|null }
  | { typeCode: 'evidence'; evidenceKind: string; observedResult: string|null; conditionsText: string|null; limitationsText: string|null }
  | { typeCode: 'question'; state: 'OPEN'|'ANSWERED'|'DEFERRED'|'DISMISSED'; answerText: string|null; nextAction: string|null }
  | { typeCode: 'gap'; scope: string; justification: string; searchPending: string|null }
  | { typeCode: 'assumption'; context: string|null }
  | { typeCode: 'condition'; applicabilityScope: string|null }
  | { typeCode: 'limitation'; effect: string|null; applicabilityScope: string|null }
  | { typeCode: 'method'; family: string|null; context: string|null }
  | { typeCode: 'example'; description: string|null }
  | { typeCode: 'insight'; baseline: string; interpretation: string; affectedConceptIds: UUID[] }
  | { typeCode: 'reference'; identifier: string|null; reason: string; state: 'TO_REVIEW'|'INSPECTED'|'DISMISSED' };
type KnowledgeItemPatch = { title?: string; bodyText?: string; confidence?: Confidence;
  attributes?: Exclude<KnowledgeAttributesPatch,{typeCode:'concept'}> }; // service requires patch typeCode = persisted typeCode
type KnowledgeFilter = { paperId: UUID|null; conceptId: UUID|null; typeCodes: KnowledgeType[];
  lifecycle: 'ACTIVE'|'ARCHIVED'|'ALL'; includeArchived: boolean;
  confidence: Confidence|null; origin: Origin|null; domain: string|null };
type ProvenanceSummaryDto = { id: UUID; documentId: UUID; pageIndex: number|null;
  pageLabel: string|null; locatorState: LocatorState };
type KnowledgeItemDto = { id: UUID; typeCode: KnowledgeType; title: string; bodyText: string;
  bodyFormat: 'plain_text'; bodyJson: null; origin: Origin; lifecycle: 'ACTIVE'|'ARCHIVED';
  confidence: Confidence; attributes: KnowledgeAttributes; paperIds: UUID[]; conceptIds: UUID[];
  provenance: ProvenanceSummaryDto[]; provenanceStatus: 'NONE'|'PENDING'|'LOCATED'|'STALE'|'MIXED';
  revision: number; createdAt: string; updatedAt: string };
type ConceptDto = Omit<KnowledgeItemDto,'typeCode'|'attributes'> & { typeCode: 'concept';
  attributes: Extract<KnowledgeAttributes,{typeCode:'concept'}>; preferredName: string;
  normalizedName: string; domain: string|null; aliases: string[]; mergedIntoId: null };
type ProvenanceDto = { id: UUID; documentId: UUID; pageIndex: number|null; pageLabel: string|null;
  section: string|null; quoteText: string|null; locator: JsonObject|null;
  capturedDocumentHash: string; locatorState: LocatorState; revision: number;
  createdAt: string; updatedAt: string };
type RelationType = 'supports'|'contradicts'|'extends'|'causes'|'requires'|'depends_on'
  |'works_when'|'fails_when'|'compares_with'|'part_of'|'similar_to'|'limits'|'improves';
type RelationDto = { id: UUID; sourceItemId: UUID; targetItemId: UUID; typeCode: RelationType;
  contextText: string; justificationText: string; origin: Origin; confidence: Confidence;
  lifecycle: 'ACTIVE'|'ARCHIVED'; provenanceIds: UUID[];
  revision: number; createdAt: string; updatedAt: string };
```

```ts
interface KnowledgeApi {
  createItem(args: { requestId: UUID; item: KnowledgeItemInput;
    paperIds: UUID[]; conceptIds: UUID[]; provenance: ProvenanceInput[] }): Promise<IpcResult<KnowledgeItemDto>>;
  updateItem(args: { requestId: UUID; itemId: UUID;
    expectedRevision: number; patch: KnowledgeItemPatch }): Promise<IpcResult<KnowledgeItemDto>>;
  archiveItem(args: { requestId: UUID; itemId: UUID;
    expectedRevision: number }): Promise<IpcResult<KnowledgeItemDto>>;
  restoreItem(args: { requestId: UUID; itemId: UUID; expectedRevision: number }): Promise<IpcResult<KnowledgeItemDto>>;
  getKnowledgeItem(args: { requestId: UUID; itemId: UUID }): Promise<IpcResult<KnowledgeItemDto>>;
  listKnowledgeItems(args: { requestId: UUID; filter: KnowledgeFilter; cursor: string|null;
    limit: number }): Promise<IpcResult<PageDto<KnowledgeItemDto>>>;
}
interface ConceptApi {
  suggestConcepts(args: { requestId: UUID; query: string; domain: string|null; limit: number }): Promise<IpcResult<ConceptDto[]>>;
  getConcept(args: { requestId: UUID; conceptId: UUID }): Promise<IpcResult<ConceptDto>>;
  listConceptItems(args: { requestId: UUID; conceptId: UUID; filter: KnowledgeFilter;
    cursor: string|null; limit: number }): Promise<IpcResult<PageDto<KnowledgeItemDto>>>;
  createConcept(args: { requestId: UUID; preferredName: string;
    definition: string; aliases: string[]; domain: string|null; origin: Origin;
    confidence: Confidence; provenance: ProvenanceInput[] }): Promise<IpcResult<ConceptDto>>;
  updateConcept(args: { requestId: UUID; conceptId: UUID; expectedRevision: number;
    preferredName?: string; definition?: string; aliases?: string[]; domain?: string|null }): Promise<IpcResult<ConceptDto>>;
  linkConcept(args: { requestId: UUID; itemId: UUID;
    conceptId: UUID }): Promise<IpcResult<null>>;
  archiveConcept(args: { requestId: UUID; conceptId: UUID;
    expectedRevision: number }): Promise<IpcResult<ConceptDto>>;
  restoreConcept(args: { requestId: UUID; conceptId: UUID; expectedRevision: number }): Promise<IpcResult<ConceptDto>>;
}
interface RelationApi {
  createRelation(args: { requestId: UUID; sourceItemId: UUID;
    targetItemId: UUID; typeCode: RelationType; contextText: string; origin: Origin;
    justificationText: string; confidence: Confidence; provenanceIds: UUID[] }): Promise<IpcResult<RelationDto>>;
  updateRelation(args: { requestId: UUID; relationId: UUID; expectedRevision: number;
    contextText: string; justificationText: string; origin: Origin;
    confidence: Confidence }): Promise<IpcResult<RelationDto>>;
  listRelations(args: { requestId: UUID; itemId: UUID; includeArchived: boolean }): Promise<IpcResult<RelationDto[]>>;
  archiveRelation(args: { requestId: UUID; relationId: UUID; expectedRevision: number }): Promise<IpcResult<RelationDto>>;
  restoreRelation(args: { requestId: UUID; relationId: UUID; expectedRevision: number }): Promise<IpcResult<RelationDto>>;
}
```

In v1, `bodyFormat='plain_text'` and `bodyJson=null`; no TipTap or rich format. Attributes are discriminated unions `KnowledgeAttributes`/`KnowledgeAttributesPatch`, not free JSON schemas. `JsonValue` is finite data without code, paths, prototypes or executable content. `createItem` with detail, paper/concept links, provenance and audit is all-or-nothing in one UnitOfWork/SQLite commit. `literature` capture may be saved as a pending draft, but the DTO always includes `provenance` and `provenanceStatus`. Update changes neither origin nor typeCode; patch attribute typeCode must match the persisted value. All list/get operations allow archived state according to `includeArchived`/lifecycle; active-object references to archived concepts still resolve and label lifecycle.

Concept matching is a lexical suggestion, never merging. `createConcept` requires explicit `origin`, `confidence` and provenance parameters; if the UI preselects `researcher_interpretation`/`requires_validation`, show editable values and send them explicitly, never silently attribute them. Normalized aliases are unique within a concept; similar names may identify different concepts. Merge and hard delete are outside the pilot 0.0.1/v0.1 implementable contract and require a later ADR and versioned extension. Reversible archive/restore is in v0.1. Archive retains related entities, keeps references navigable and excludes them from active lists by default; `includeArchived` filters permit explicit queries.

Relations validate the DOMAIN.md allowlist + endpoint matrix during creation and read/import; UI validation is insufficient. In v0.1 endpoints and typeCode are immutable; correcting them requires archiving and creating a new Relation to preserve history. Update changes only context, justification, origin and confidence with expectedRevision. Archive/restore propagates neither to endpoints nor provenance. Supports/contradicts require relevant origin/context and provenance to be presented as literal source statements; researcher proposals require explicit origin.

### Capture and concept clarifications (ADR-022)

Keep the wire signatures above. Canonical text: CRLF/CR→LF and outer trim, interior content unchanged; validate existing maxima before accepting normalization, without truncation. normalizedName/alias/domain collapses Unicode whitespace and applies Unicode lowercase, preserving diacritics/punctuation without NFKC. paperIds/conceptIds/affectedConceptIds are sorted, exactly deduplicated sets of valid UUIDs; reject duplicate normalized aliases within a concept. Content canonicalization changes neither payload hash nor receipt identity.

Capture without provenance produces genuine NONE without a fictitious Document/row; literature NONE is shown as awaiting attribution. An explicit Document without an anchor produces PENDING. Parent and every initial Provenance have revision0; creation has no expectedParentRevision. New paper_items use P2 with selection=false and null priority/rationale; they neither start nor activate a phase.

Concept preferredName/title is one canonical value with an atomic mirror, and definition/bodyText has one authority. ConceptApi controls name/definition/aliases/domain; Knowledge.updateItem on Concept allows only confidence. Any supplied title/bodyText/attributes rejects the whole patch even if unchanged. Insight.affectedConceptIds must be a subset of conceptIds on create and existing links on update; removing an affected concept does not unlink it, and linkConcept does not mark it affected.

New updateItem/updateConcept/linkConcept requests require an ACTIVE parent; ARCHIVED returns InvalidInput and requires restore, including no-op. Reject new links/capture to archived Concepts. With an ACTIVE parent, an existing link is a no-op even if its destination is now ARCHIVED: create no association and alter no historical association. Durable replay precedes all these guards. An effective link increments only item.revision, never the destination Concept; wire adds no expectedRevision, and revision is read inside IMMEDIATE. No-op preserves clocks/revisions/times and creates no effective event.

listConceptItems accepts filter.conceptId null or equal to its conceptId argument; a different value returns InvalidInput. domain matches normalized equality on the Concept itself or any linked Concept (existential OR without duplicate rows); it does not inherit Paper.domain. Archived references remain readable/exportable. Equal aliases/preferredName across Concepts do not merge senses.

An active relation is unique by source/target normalized for symmetry, typeCode and canonical contextText. Colliding create, context update and restore return atomic Conflict: never replace/merge or leave content, revision, clock, audit or success-receipt changes. Translate only the known constraint into Conflict; do not hide other SQL errors.

## Provenance module

```ts
type ProvenanceInput = { documentId: UUID; pageIndex: number|null; pageLabel: string|null;
  section: string|null; quoteText: string|null; locator: JsonObject|null };
interface ProvenanceApi {
  attachLocator(args: { requestId: UUID; itemId?: UUID; relationId?: UUID;
    expectedParentRevision: number; input: ProvenanceInput }): Promise<IpcResult<ProvenanceDto>>;
  updateLocator(args: { requestId: UUID; provenanceId: UUID; expectedRevision: number;
    input: ProvenanceInput }): Promise<IpcResult<ProvenanceDto>>;
  getProvenance(args: { requestId: UUID; provenanceId: UUID }): Promise<IpcResult<ProvenanceDto>>;
  checkDocumentHash(args: { requestId: UUID; documentId: UUID }): Promise<IpcResult<{ currentHash: string; staleProvenanceIds: UUID[] }>>;
}
```

Require exactly one of `itemId`/`relationId`. `attachLocator` increments the parent revision in the same transaction that creates Provenance and the association. `updateLocator` checks Provenance revision, increments only that revision and preserves the captured hash; a locator change does not automatically validate a quotation. `checkDocumentHash` is an idempotent mutation with a `requestId` receipt; detecting a different current hash changes LOCATED→STALE and increments revision/updatedAt of each affected Provenance without altering content/anchor. `ProvenanceDto` exposes revision and updatedAt; `KnowledgeItemDto.provenance[]` and aggregate status reflect PENDING/LOCATED/STALE/MIXED so the UI cannot hide pending work. `pageIndex` is null or integer >=1. Creation captures `capturedDocumentHash` from the registered Document, not the UI. Locator becomes LOCATED only when coordinates/locator validate and hashes match; no explicit anchor means PENDING. Original paths leave neither IPC nor shareable export.

### Registered anchoring and source preservation (ADR-022)

locator is null or the closed object `{kind:'page_region',x,y,width,height}` with finite numbers: x/y>=0, width/height>0, each value<=1, x+width<=1, y+height<=1; a region requires integer pageIndex>=1. No extra keys, coercion or clamping. A page alone is a registered LOCATED anchor with registered hash equal to captured hash; pageLabel/section/quote alone are PENDING. LOCATED describes a registered location: T05 checks neither page count, physical PDF nor quotation truth. T07 uses Reader to validate/navigate the real page and presents errors without inventing navigation.

updateLocator preserves documentId and capturedDocumentHash; a different documentId returns InvalidInput. STALE remains STALE after editing. If capturedHash differs from the registered hash, a valid update retains STALE and both hashes. Outside STALE, adding/removing a valid page permits LOCATED/PENDING under the rule, without verifying the quotation. A different source requires a new explicit attach and retained history; no implicit rebind/revalidate in v0.1. checkDocumentHash never modifies Document.sha256 or capturedHash and never automatically revives PENDING/STALE.

## Search module

```ts
type SearchRequestDto = { query: string; scopes: Array<'papers'|'knowledge'|'concepts'>;
  filters: { paperId: UUID|null; conceptId: UUID|null; typeCodes: KnowledgeType[];
    lifecycle: 'ACTIVE'|'ARCHIVED'|'ALL'; includeArchived: boolean;
    domain: string|null; confidence: Confidence|null };
  cursor: string|null; limit: number };
type SearchHitDto = { entityType: 'paper'|'knowledgeItem'|'concept'; entityId: UUID;
  title: string; excerpt: string; paperId: UUID|null; pageIndex: number|null;
  rank: number; lifecycle: string };
interface SearchApi {
  searchLibrary(args: { requestId: UUID; request: SearchRequestDto }): Promise<IpcResult<PageDto<SearchHitDto>>>;
  searchKnowledge(args: { requestId: UUID; request: SearchRequestDto }): Promise<IpcResult<PageDto<SearchHitDto>>>;
}
```

Search uses FTS5 tokenizer `unicode61 remove_diacritics 2`; treat input as literal terms joined by AND, without exposing advanced FTS syntax. The internal FTS projection exposes no IDs/rowid as a public contract and is maintained in the same transaction as canonical records. Parameterized filters; excerpt maximum 500 chars with HTML neutralization. No full PDF/OCR text search in v0.1. Exclude archived records by default; never use rank as a scientific score. The index is rebuildable from canonical records.

## Export and Backup modules

```ts
type ExportRequestDto = { destinationToken: UUID; includePdfs: boolean;
  format: 'jsonl_markdown'; paperIds: UUID[]|null };
type ExportResultDto = { exportId: UUID; outputName: string; manifestSha256: string;
  fileCount: number; entityCounts: Record<string,number>; includedPdfs: boolean };
type BackupDto = { backupId: UUID; createdAt: string; schemaVersion: number;
  verified: boolean; manifestSha256: string; sizeBytes: number };
type LongOperationState = 'queued'|'running'|'completed'|'failed'|'cancelled';
type LongOperationDto = { operationId: UUID; state: LongOperationState;
  progress: number|null; result: { kind: 'export'|'backup'|'restore'; value: ExportResultDto|BackupDto|PreparedLibraryDto }|null;
  error: { code: IpcErrorCode; message: string; retryable: boolean }|null };
type ExportDestinationDto = { destinationToken: UUID; displayName: string };
type BackupDestinationDto = { destinationToken: UUID; displayName: string };
type RestoreTargetDto = { targetToken: UUID; displayName: string };
type PreparedLibraryDto = { preparedLibraryToken: UUID; libraryId: UUID; displayName: string; schemaVersion: number };
type LibrarySwitchTarget = { kind: 'selected'; targetToken: UUID }
  | { kind: 'restored'; preparedLibraryToken: UUID };
interface PortabilityApi {
  chooseExportDestination(args: { requestId: UUID }): Promise<IpcResult<ExportDestinationDto|null>>;
  exportLibrary(args: { requestId: UUID; operationId: UUID; request: ExportRequestDto }): Promise<IpcResult<LongOperationDto>>;
  exportPaper(args: { requestId: UUID; operationId: UUID; paperId: UUID;
    destinationToken: UUID; includePdf: boolean }): Promise<IpcResult<LongOperationDto>>;
  chooseBackupDestination(args: { requestId: UUID }): Promise<IpcResult<BackupDestinationDto|null>>;
  createBackup(args: { requestId: UUID; operationId: UUID; destinationToken: UUID }): Promise<IpcResult<LongOperationDto>>;
  selectBackup(args: { requestId: UUID }): Promise<IpcResult<{ backupToken: UUID; displayName: string }|null>>;
  chooseRestoreTarget(args: { requestId: UUID }): Promise<IpcResult<RestoreTargetDto|null>>;
  getOperationStatus(args: { requestId: UUID; operationId: UUID }): Promise<IpcResult<LongOperationDto>>;
  cancelOperation(args: { requestId: UUID; operationId: UUID }): Promise<IpcResult<{ cancellationRequested: boolean }>>;
  verifyBackup(args: { requestId: UUID; backupToken: UUID }): Promise<IpcResult<{ valid: boolean; issues: string[] }>>;
  restoreBackup(args: { requestId: UUID; operationId: UUID; backupToken: UUID;
    targetToken: UUID }): Promise<IpcResult<LongOperationDto>>;
}
```

Destination/backup tokens come from native dialogs; accept no free paths. Tokens expire in 24 h and are bound to the operation/library. JSONL/Markdown export uses a consistent snapshot, omits private paths and includes PDFs only by explicit option. Exact layout:

```text
research-export/
  manifest.json
  papers.jsonl
  authors.jsonl
  venues.jsonl
  documents.jsonl
  phase_definitions.jsonl
  phases.jsonl
  phase_answers.jsonl
  knowledge_items.jsonl
  concepts.jsonl
  relations.jsonl
  provenance.jsonl
  validations.jsonl
  associations.jsonl
  ontology/
  markdown/papers/<paperId>.md
  markdown/concepts/<conceptId>.md
  files/<documentId>/source.pdf       # only with includePdfs
```

Each JSONL line is `{recordType: RecordType, recordVersion: 1, data: object}`; `RecordType` is the closed enum `paper | author | venue | document | phaseDefinition | paperPhase | phaseAnswer | knowledgeItem | concept | relation | provenance | validation | paperAuthor | paperItem | itemConcept | itemProvenance | relationProvenance | conceptAlias`. Closed manifest: `{exportVersion:'1.0',schemaVersion:number,ontologyVersions:Record<string,string>,createdAt:string,entityCounts:Record<string,number>,files:Array<{path:string,sizeBytes:number,sha256:string}>,includedPdfs:boolean}`. Do not export `app_session`, `app_settings`, receipts, locks, backups, logs or original paths. `data` preserves UUIDs and FKs as IDs; each type uses a closed allowlist based on canonical columns: Paper (`id,title,doi,year,reviewType,domain,url,venueId,lifecycle,createdAt,updatedAt`); Author (`id,displayName,orcid`); Venue (`id,name,kind,identifier`); Document (`id,paperId,originalFilename,sha256,mediaType,sizeBytes,importedAt,status`); PhaseDefinition (code/version/definitionHash/declarative definition JSON); PaperPhase (paperId/phaseCode/definitionVersion/state/revision/acceptedGateSnapshotHash/completedAt); PhaseAnswer (paperId/phaseCode/questionKey/answerText/structuredValue/resolution/explanation/revision/updatedAt); KnowledgeItem (id/typeCode/title/bodyText/bodyFormat/bodyJson/origin/lifecycle/confidence/attributes/revision/timestamps); Concept (itemId/preferredName/normalizedName/domain/mergedIntoId); Relation (id/sourceItemId/targetItemId/typeCode/contextText/justificationText/origin/confidence/lifecycle/revision/timestamps); Provenance (id,documentId,pageIndex,pageLabel,section,quoteText,locator,capturedDocumentHash,locatorState,revision,timestamps). Association records are `paperAuthor(paperId,authorId,position)`, `paperItem(paperId,itemId,phaseCode,selectedForP3,priority,rationale)`, `itemConcept(itemId,conceptId)`, `itemProvenance(itemId,provenanceId)`, `relationProvenance(relationId,provenanceId)`, `conceptAlias(conceptId,alias,normalizedAlias)`. These records retain complete semantic edges; emit an association only when all endpoints are included. Include Validation rows when present. `phase_definitions.jsonl` preserves the immutable definition required to interpret each historical `definitionVersion`.

For `exportPaper`, closure starts at Paper and adds authors/venue/documents, phases/answers and associated KnowledgeItems; include used Concepts and Relations only if both endpoints enter the closure. Then add provenance for every included item/relation and any Document referenced by that provenance, always including each Document's parent Paper even if it is not the initial Paper. Thus no relation lacks an endpoint and no included object's provenance is omitted. Export associations only when all endpoints are included. Without PDFs, retain Document metadata/hash and omit binary and path.

Backup includes SQLite snapshot + referenced PDFs + ontology/manifest, verified with hashes and `integrity_check`. `selectBackup` selects a backup directory created by this app. Restore writes into a new root, validates content and returns `PreparedLibraryDto`; never activate automatically. The user reviews the destination and activates through `switchLibrary`. If connection/process is lost, `getOperationStatus` returns a complete terminal result/error; job and receipt survive restart. v0.1 backups are manual and pre-migration; retain every verified copy, with no automatic purge.

Export/backup/restore cancellation only marks cancellation and cleans staging whose operationId/intent matches; after publishing export/backup or preparing restore, return the terminal result. Cancelled restore leaves the active library intact. Do not interrupt short SQL mutations halfway; cancellation after commit retrieves the receipt and result.

## Settings module and Desktop lifecycle

```ts
type AppInfoDto = { appVersion: string; contractVersion: 1; schemaVersion: number|null;
  libraryId: UUID|null; libraryRootLabel: string|null;
  state: 'ready'|'readOnlyDiagnostic'; capabilities: Record<string,boolean> };
interface SettingsApi {
  getAppInfo(args: { requestId: UUID }): Promise<IpcResult<AppInfoDto>>;
  getLibraryInfo(args: { requestId: UUID }): Promise<IpcResult<LibraryInfoDto>>;
  selectLibrary(args: { requestId: UUID }): Promise<IpcResult<{ targetToken: UUID; displayName: string }|null>>;
  switchLibrary(args: { requestId: UUID; target: LibrarySwitchTarget }): Promise<IpcResult<LibraryInfoDto>>;
  getLibraryStatus(args: { requestId: UUID }): Promise<IpcResult<{ writable: boolean;
    activeOperations: number; recoveryRequired: boolean }>>;
}
```

`LibraryInfoDto = { libraryId: UUID; displayName: string; rootLabel: string; schemaVersion: number; writable: boolean }`. `switchLibrary` belongs to v0.1, not pilot 0.0.1. Add `selectLibrary` so a native dialog emits `targetToken` and `switchLibrary` validates before activation: close/flush/release the previous lock, validate and migrate the destination with backup and acquire its lock in order. In the pilot both return `UnsupportedCapability`; accept no free path. Selection/switch failure leaves the previous library open or recoverable, and the user sees which library remains active.

The backend determines library root; expose only a friendly label, not an absolute path, except on an explicit local diagnostic screen. Settings accept no arbitrary filesystem path. v0.1 incorporates library selection/switching through picker tokens and destination validation.

Startup acquires the single-instance/library lock before recovery or writing, verifies schema compatibility, migrates only after backup and reconciles staging. Closure blocks new mutations, resolves pending persistence with a finite timeout, closes DB and releases the lock; never report Saved before commit. A second instance focuses the first or reports Busy. WebView2/installer are not domain IPC. `getLastOpenedPaper` is a read; Reader `openPaper` is the sole setter and must persist last-open in the commit. The backend serializes operations through a dedicated actor and one SQLite connection; any cross-module `UnitOfWork` retains its transaction on that same connection.

## Atomicity, cancellation and postconditions

| Operation | Atomicity and postcondition |
|---|---|
| Import | Filesystem/SQLite saga with `import_operations`; result only after commit; restartable reconciliation by token. Never delete a destination/file with ambiguous ownership. |
| Metadata/archive/restore | One DB transaction increments revision and audit. Stale revision changes nothing. |
| Save answer | Answer+revision+history in one transaction. |
| Advance phase | Reread all answers/outputs/gates under a write transaction; fail without effects or complete the phase and initialize the next when that branch enables it, in one commit; P1 light_read/archive and closing P2 return nextPhase=null. |
| Create item/relation/provenance | Object, associations, extensions and audit event in one transaction; no orphan entity from partial failure. |
| Save reading position | Last-write by revision under a transaction; clients serialize; stale revision returns Conflict and the client rereads/merges with the user. |
| Export/backup | Consistent snapshot coordinated with imports/DB; publish the final folder only after verification; temporary work is cancellable before final rename. |
| Restore | Never mutate the active library during validation; activate only after success and explicit selection outside the restore contract. |

Persisted `requestId` makes designated mutation retries safe. `operationId` identifies long export/backup jobs and permits querying/cancelling progress; it is not an idempotency key. Cancellation does not roll back an already confirmed commit. After a crash, the next startup resumes or leaves a recoverable issue according to durable state. Messages include no scientific text, private paths, stack trace or PDF bytes.

## Security and resources

- Explicit commands only; React cannot run SQL, read arbitrary files or invoke a shell.
- Native picker emits a token; import tokens expire and are bound to session/library. PDF protocol receives Document UUID, resolves its path from SQLite and verifies canonicalization/root.
- Parameterized SQL queries; bounded `limit`; validate UUIDs, enums, resolved IDs and size before writing.
- PDFs, ontology JSON, body and quote are untrusted data; neither execute content nor convert it to HTML without sanitization. Restrict CSP to packaged resources and the document protocol.
- No telemetry or network in the local core. Rotated technical error/code/UUID logs; no body, quotations, PDF, original paths or complete metadata.
- One SQLite writer per library; foreign keys enabled on every connection. Destructive operations are unavailable in the pilot/v0.1.

## Tauri command registry

Internal functions register with a feature prefix; the registry is closed in contractVersion 1: `library_select_pdf`, `library_confirm_import`, `library_cancel_import`, `library_list_papers`, `library_get_paper`, `library_update_metadata`, `library_archive_paper`, `library_restore_paper`, `reader_open_paper`, `reader_get_last_opened_paper`, `reader_get_reading_position`, `reader_save_reading_position`, `workflow_get_phase`, `workflow_get_phase_answers`, `workflow_get_phase_definition`, `workflow_save_phase_answer`, `workflow_evaluate_gate`, `workflow_advance_phase`, `workflow_go_back_to_phase`, `workflow_touch_phase`, `workflow_set_p3_candidate`, `workflow_get_p3_candidate_summary`, `knowledge_create_item`, `knowledge_update_item`, `knowledge_archive_item`, `knowledge_restore_item`, `knowledge_get_item`, `knowledge_list_items`, `concept_suggest`, `concept_get`, `concept_list_items`, `concept_create`, `concept_update`, `concept_link`, `concept_archive`, `concept_restore`, `relation_create`, `relation_update`, `relation_list`, `relation_archive`, `relation_restore`, `provenance_attach_locator`, `provenance_update_locator`, `provenance_get`, `provenance_check_document_hash`, `search_library`, `search_knowledge`, `export_choose_destination`, `export_library`, `export_paper`, `backup_choose_destination`, `backup_create`, `backup_select`, `backup_choose_restore_target`, `backup_verify`, `backup_restore`, `operation_get_status`, `operation_cancel`, `settings_get_app_info`, `settings_get_library_info`, `settings_select_library`, `settings_switch_library`, `settings_get_library_status`. Unimplemented commands remain capability-gated; free aliases are not accepted. Register no generic filesystem, SQL or execution API.

## Fixed proposed decisions and validation before acceptance

This section's decisions are closed as a coherent proposal for review, not equivalent to approval. Acceptance must check them against planned implementation and boundary tests: declared limits (title 1000, answer/body 20,000, snippet/quote 10,000, context/relation 5,000, 100 authors, PDF 500 MiB); normalized DOI without network resolution or duplicate DOI/hash; complete 13-relation allowlist with DOMAIN endpoint matrix; `plain_text` with `bodyJson=null`; UUID document protocol with restricted local capability; optimistic revision conflicts, canonical no-op without invalidation and real changes invalidating dependent snapshots under DOMAIN. Hard delete and merge remain outside v0.1 and require a later ADR. Acceptance requires boundary/Unicode tests, Rust↔TypeScript DTO/envelope round-trip, unknown payload/enum rejection, idempotent retry, revision conflict, filesystem/SQLite import reconciliation, export/backup reconstruction with associations and phase definitions, and migration compatibility. Concrete protocol/capabilities are fixed in the architecture document for the selected version; they do not expand this contract's command set.

lifecycle and includeArchived filters have one interpretation: lifecycle=ACTIVE requires includeArchived=false; lifecycle=ARCHIVED or ALL requires includeArchived=true. Inconsistent combinations return InvalidInput. Lifecycle controls filtering and the boolean expresses an explicit choice to query archived records.

## Candidate and P2 delivery clarification (ADR-023)

The seven decisions in [TASK06_DECISIONS](https://github.com/dpalazon-dev/doctorado-ucam/blob/c985b079d39ee5915c017c38c1f50b7a94526843/prototypes/research-workbench/docs/plans/TASK06_DECISIONS.md) apply. setP3Candidate requires started P2 without activating it; selected=false requires priority/rationale=null and permits deselecting an archived item. Summary retains selected archived items; the gate requires active artifacts. Zero candidates after set requires a received justification of 1..5000 characters that is canonically non-empty, with candidates null. Candidate save permits PENDING/null and never changes selection; a present object must be complete and match DB. That save, like every P2 save/evaluate/advance, becomes available in production only after T07. T06 delivers real capture/candidates; no zero-item justification save exists until save is enabled. Finalize the internal ABI after verifying T05; existing wire remains unchanged.
