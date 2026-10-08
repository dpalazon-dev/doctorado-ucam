# Tarea 06 — P2, captura y cola P3

**Owner:** Luna P2/knowledge UI. **Depende:** T04/T05. **Riesgo:** alto por snapshot/candidatos. **Report:** `docs/reports/task-06-report.md`.

Leer IMPLEMENTATION, DOMAIN P2, CONTRACTS Workflow/Knowledge/Concept, SPEC-004, WORKFLOW_GATES/ADR-017 y JSON P2.v1 canónico. No estás solo, shared workflow service/commands/invalidation pertenecen ahora a Sol; enviar patch de integración, no editar archivos T04 mientras otro los use. T07 añade relation/provenance panels separados.

**Posee:** `src-tauri/src/modules/workflow/{p2.rs,candidates.rs}`; `src-tauri/src/adapters/sqlite/candidate_repository.rs`; `src-tauri/src/transport/commands/p2.rs`; `src/features/workflow/{P2Workspace.tsx,P3Candidates.tsx,p2.test.tsx}`; `src/features/knowledge/{KnowledgeWorkspace.tsx,KnowledgeItemForm.tsx,KnowledgeItemDetails.tsx,ConceptPicker.tsx,ConceptDetails.tsx,knowledge.test.tsx}`; `src-tauri/tests/p2_candidates.rs`.

**Consume:** WorkflowApi/KnowledgeApi/ConceptApi exactos anexo, candidate associations 0003, helpers invalidación y gate T04, DTO provenance T05. **Produce:** WorkflowApi setP3Candidate/getP3CandidateSummary reales; P2 policy reevaluado dentro de advancePhase existente por Sol; formularios todos doce tipos core/plain text, conceptos globales compartidos. No workspace P3 operativo.

- [ ] Tests `p2_required_outputs_exact`, `p2_no_arbitrary_counts`, `zero_candidates_requires_justification`, `selected_candidate_requires_rationale`, `only_linked_claim_question_candidate`, `priority_optional_integer_1_to_5`, `candidate_workflow_revision_is_p2_revision`, `candidate_change_updates_output_and_revision_same_tx`, `p2_complete_keeps_paper_active_next_null`, `p2_artifact_edit_needs_review`, `p2_noop_not_invalidated`; UI `all_types_have_form`, `concept_reuse_vs_new_sense`, `origin_confidence_defaults_visible_editable`, `pending_provenance_not_labelled_verified`, `candidate_controls_persist_after_reopen`, `conflict_keeps_draft`.
- [ ] Required outputs exact `main_questions_processed,field_synthesis,relevant_concepts_reviewed,meaningful_relations_reviewed,contradictions_reviewed,references_classified,p3_candidates_or_justification`. Entidades/references/relations/synthesis consultables, no checkbox que suplante artefactos. UNKNOWN explicado válido donde permitido; ausencia contradicciones/candidatos justificada, sin conteos mínimos.
- [ ] setP3Candidate expectedWorkflowRevision **revision paper_phases(P2)**. Solo claim/question enlazado al paper; selected requiere rationale 1..5000, priority null o integer1..5; cero candidatos justification no vacía, con candidatos justification null. Asociación es fuente de selección/priority/rationale; única justification en PhaseAnswer P2 estructurada. Cambio real de proyección incrementa answer.revision/updatedAt y clock P2 en misma UoW. Save de justificación cero sin item ficticio conserva candidates/count canónicos y no cambia selección. Resumen devuelve IDs/count/justification exactos gate. No-op canónico no invalida.
- [ ] Integrar P2 gate/advance: completar P2 COMPLETED/Paper ACTIVE/nextPhase null, jamás invocar P3 ni marcar Paper COMPLETED. Reconfirmar tras artifact changes. T07 habilita relations/provenance artifacts reales; hasta disponible capability/botones honestos y aceptación P2 final espera T07.
- [ ] UI captura origin/confidence explícitos, quote separado, required attributes por subtype, item/concept details/list filters/archive/restore, connect PRE questions, estado gate y borradores preservados; panels reciben APIs por props, no invoke. Archivar global explica alcance y conserva links.
- [ ] Tests Rust/UI/typecheck, reopen real candidates y Gate hash; reporte con integración requerida. Sol registra commands y completa composición de T04/T06/T07.

- [ ] Implementar mapping p2ArtifactPresent por key y alternativa cerrada candidatos/ausencia de [WORKFLOW_GATES](../../architecture/WORKFLOW_GATES.md), sin cambiar JSON v1 ni AND global. Las seis salidas enlazan artefactos tipados, síntesis Insight, ausencia explicada sin cuota; ningún resolutor ausente se considera ausencia justificada.
- [ ] Forward/advance exige cadena COMPLETED+snapshot aceptado+gate vigente; continue P1 aceptado. Documento vivo en inputs P2 y proof/handle fuera TX/reference revalidada dentro; sin IO/hash masivo bajo actor. T07/capacidades requeridas pendientes mantienen UnsupportedCapability y aceptación final pendiente.
- [ ] Tests `p2_requires_accepted_chain`, `p2_live_document_unavailable_blocks`, `zero_items_justification_without_fake_id`, `candidate_projection_bumps_answer_and_phase_revision`, `p2_artifact_ids_and_types_validated`, `synthesis_insight_persisted`, `empty_artifact_sets_require_explanation`, `shared_artifact_invalidates_all_reached_papers`, `missing_resolver_never_completes_gate`, `context_and_artifact_interleaving_conflicts`; snapshots históricos/reopen conservados.

**Aceptación:** captura 12 tipos, concepts compartidos, P2 completo sin números inventados y cola P3 persistente; artifact change invalida P2 y conserva contenido.

**ADR-022:** mostrar NONE como pendiente de atribuir si literature, sin crear fuente ficticia; PENDING/STALE no equivalen a cita verificada. ConceptApi edita nombre/definición/aliases/domain y Knowledge sólo confidence de Concept. Insight affected es subset explícito de conceptos vinculados. Cambio body de candidato conserva answer.revision si su proyección no cambia. Consultar contratos actualizados y TASK05_PORTS antes de ampliar T04/T05.


# Anexo de contratos exactos

Extracto literal de docs/architecture/CONTRACTS.md, contractVersion 1, actualizado el 2026-10-03 tras ADR-022. La norma gobierna; estos tipos consumidores no amplían ownership.

## Sobre y error

Todo comando devuelve este envelope exitoso. Error Tauri también debe usar el envelope, no texto serializado ambiguo; el adaptador TypeScript normaliza errores de invoke a esta misma forma.

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
    message: string;          // seguro para UI, español
    retryable: boolean;
    details?: JsonObject;     // estructura limitada, sin rutas/contenido sensible
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

Cada llamada lleva `requestId` UUID generado por cliente. `confirmImport`, cancelación y todas las mutaciones persistentes reciben receipt durable; para los comandos idempotentes/destructivos se persiste `requestId` + hash de payload y resultado en una tabla descrita en DATA.md. Reintento con mismo requestId/payload devuelve resultado previo; payload distinto da `Conflict`. Lecturas no guardan receipt. Actualizaciones requieren `expectedRevision`; conflicto incluye `currentRevision`, nunca sobreescribe. En listas la paginación es cursor opaco, límite propuesto 100 (máximo 500).

## DTOs compartidos

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
  title: string;                       // trim; 1..1000 caracteres (límite propuesto)
  authors: string[];                   // orden bibliográfico; cada nombre trim/no vacío
  year: number | null;                 // null o año de cuatro cifras
  doi: string | null;                  // normalizado antes de unicidad
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
  expiresAt: string;                  // token válido 24 h; recovery puede renovar
};
type DuplicateResolution = { action: 'reuseExisting'; paperId: UUID };
type PaperFilterDto = {
  lifecycle: 'ACTIVE'|'ARCHIVED'|'ALL'; // ACTIVE incluye NEW/ACTIVE/COMPLETED
  query: string; yearFrom: number|null; yearTo: number|null;
  reviewTypes: ReviewType[]; domain: string|null; phase: PhaseCode|null;
  cursor: string|null; limit: number;
};
type PageDto<T> = { items: T[]; nextCursor: string|null; total?: number };
type RevisionDto = { revision: number; updatedAt: string };
```

Límites v1: title 1000 chars; answer/body 20,000; snippet/quote 10,000; relación/contexto 5,000; authors máximo 100 por paper; PDF máximo 500 MiB. Search query 1000 chars, excerpt 500 chars, 100 resultados por página por defecto, 500 máximo. Backend valida todos. Rechazar `NaN`, infinitos, paths no seleccionados y JSON desconocido/sobre límite; nunca truncar. La fixture del piloto debe incluir Unicode, nombres largos y archivos próximos al límite para comprobar manejo sin alterar estos máximos.

## Módulo Workflow

`PhaseDefinitionDto` es snapshot declarativo e inmutable: `code`, `version`, `name`, `objective`, `keyQuestions`, `doItems`, `dontItems`, `considerations`, `requiredOutputs`, `completionRules`, `definitionHash`. Cada salida es `{key,label,prompt,required,allowedResolutions,ruleKey}`; no contiene scripts, expresiones evaluables ni código. `ruleKey` pertenece al enum cerrado `answerProcessed | paperHasActiveDocument | p1DecisionProcessed | p2ArtifactPresent | p2NoCandidatesJustified`. La decisión P1 procesada satisface completitud de P1 en sus tres valores; la rama `continue`/`light_read`/`archive` se aplica en `advancePhase`, no es un requisito de gate que excluya dos opciones. La definición fijada al crear processing no cambia si se instala una versión nueva; el snapshot previo se preserva en DB y export. `PhaseAnswerDto`: La resolución canónica es [WORKFLOW_GATES.md](../../architecture/WORKFLOW_GATES.md); los JSON [PRE/P1/P2 v1](../../architecture/phase-definitions/) fijan payloads inmutables embebibles al compilar. completionRules enumera handlers por salida con alternativa cerrada candidatos/ausencia, no una conjunción global.

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

**Edición de fases (ADR-020):** para una petición nueva, savePhaseAnswer sobre NOT_STARTED devuelve GateBlocked sin respuesta, cambio de clock/contexto, auditoría de éxito ni receipt nuevo. Guardar no inicia una fase. Una fase ya iniciada puede editarse aunque no sea la activa, sin aceptar previamente de nuevo su cadena ni cambiar activePhaseCode: CAS de respuesta, no-op e invalidación conservan sus reglas. El replay durable se resuelve antes de esta guarda; siguen vigentes restricciones de lifecycle y capacidades (P2 no habilitada en T04).

`relevance_decision` ANSWERED exige `structuredValue: RelevanceDecisionValue` completo por enums; PENDING/null guarda borrador sin decidir rama, UNKNOWN/NA no admitidos. Nunca interpretar answerText. PRE.review_type confirma Paper.reviewType con estructura y resolución según [WORKFLOW_GATES.md](../../architecture/WORKFLOW_GATES.md); P1.review_type caracteriza textualmente la fuente. UNKNOWN/NA textual explicado no exige repetir explicación como answerText. savePhaseAnswer.expectedRevision protege la respuesta: primera escritura expected=0/revision=1; replay previo CAS, no-op posterior CAS conserva revisions/snapshots. save nunca archiva.

evaluateGate es lectura con snapshot/hash canónico. advance compara expectedPhaseRevision de fromPhase activa y reevalúa bajo IMMEDIATE; snapshot/fase/contexto/revisiones/lifecycle/audit/receipt se confirman juntos. P1 continue activa P2; light_read completa P1, Paper ACTIVE, active P1, nextPhase=null; archive completa P1+Paper ARCHIVED+archivedFromLifecycle, active P1, nextPhase=null. P2 nunca iniciada queda NOT_STARTED en las ramas terminales; P2 anterior conserva datos/NEEDS_REVIEW. Cerrar P2 mantiene active P2 y Paper ACTIVE, nextPhase=null, sin P3. Inputs obsoletos o gate bloqueado producen Conflict/GateBlocked sin efectos.

goBack/touch compara la fase actualmente activa, no destino. Navegar preserva respuestas/state/completedAt/snapshot; cambio real de contexto renueva revisión destino con clock fresco max(revision)+1 por Paper bajo UoW, sin tabla/wire nuevo. Habilitar fase hacia delante —inicializar o activar— y completar exige prerequisito COMPLETED, snapshot aceptado y gate vigente, más continue aceptado para P2. Touch no acepta prerequisitos; consulta/navegación de datos ya iniciados no autoriza avance. Definición/reglas/clock exactos en WORKFLOW_GATES.

PRE/P1/P2 incluye Document vigente vivo {id,status,sha256,available} en snapshot. Prueba de acceso/handle retenido antes TX; referencia DB revalidada dentro; ABI de documento se concreta tras T03, sin lectura/hash de PDF grande bajo DbActor. Archivo inaccesible bloquea sin mutar/receipt exitoso; desaparición externa no incrementa clock ficticio. Cambio DB Document invalida/renueva revisiones en su UoW.

setP3Candidate.expectedWorkflowRevision es revision paper_phases(P2). Sólo Claims/Questions asociados; priority null o integer1..5, rationale 1..5000 al seleccionar. Asociaciones son autoridad de selección; única justificación de cero candidatos en PhaseAnswer estructurada existente, con candidatos debe ser null. Save de esa key permite ausencia sin item ficticio y nunca cambia selección. Resumen y gate usan mismos IDs/count/justificación. Cambio efectivo de proyección renueva respuesta y phase clock en misma UoW. Seis salidas P2 enlazan artefactos por key; alternativa p2ArtifactPresent OR p2NoCandidatesJustified sólo para candidatos, no AND global ni código declarativo. Capacidad requerida ausente devuelve UnsupportedCapability; aceptación P2 final espera T07.

Tras cambio efectivo, fase COMPLETED y posteriores ya iniciadas pasan NEEDS_REVIEW con datos/snapshot histórico preservados; NOT_STARTED sin iniciar. No-op canónico no invalida; advance requiere reconfirmación. Detalle cerrado de normalización, artifacts, ausencia y snapshots en WORKFLOW_GATES.

En piloto 0.0.1 processingInitialized=false. Workflow v0.1 importa/inicializa PRE IN_PROGRESS, P1/P2 NOT_STARTED, active PRE y fija las tres definiciones v1 en el mismo commit. Upgrade preserva UUID/metadatos/documento/lifecycle/archivedFrom/revisiones bibliográficas; no genera confirmación PRE.review_type ni restaura. Sólo primer advance PRE exitoso cambia NEW→ACTIVE. Las condiciones de habilitación aceptada/vigente de WORKFLOW_GATES gobiernan P1/P2. P2 completada sigue Paper ACTIVE, no asigna COMPLETED global. COMPLETED reservado se conserva en lectura/upgrade/Library archive/restore; transiciones Workflow incompatibles UnsupportedCapability sin efectos. T04 puede leer/pinar/activar P2 por continue aceptado, pero sus saves/gates/advance/candidatos quedan capability-gated hasta resolutores reales T06/T07.

## Módulos Knowledge, Concepts y Relations

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

En v1 `bodyFormat='plain_text'` y `bodyJson=null`; no TipTap ni formato enriquecido. Los atributos son los unions discriminados `KnowledgeAttributes`/`KnowledgeAttributesPatch`; no esquemas JSON libres. `JsonValue` es dato finito sin código, paths, prototypes ni contenido ejecutable. `createItem` con detalle, paper/concept links, provenance y audit es todo o nada en un UnitOfWork/SQLite commit. Captura `literature` puede guardarse como borrador pendiente, pero el DTO siempre incluye `provenance` y `provenanceStatus`. Update no altera origin ni typeCode; el typeCode del atributo de patch debe coincidir con el persistido. Todos los list/get permiten estado archivado según `includeArchived`/lifecycle; las referencias desde objetos activos a conceptos archivados siguen resolviendo y etiquetan el lifecycle.

Concept matching es sugerencia léxica, nunca fusión. `createConcept` exige `origin`, `confidence` y provenance como parámetros explícitos; si la UI preselecciona `researcher_interpretation`/`requires_validation`, debe mostrarlos como valores editables y enviarlos expresamente, nunca atribuirlos silenciosamente. Alias normalizado único dentro del concepto; nombres parecidos pueden pertenecer a conceptos distintos. Merge y hard delete no están en el contrato implementable de piloto 0.0.1 ni v0.1; requieren ADR posterior y ampliación versionada. Archive/restore reversible sí está en v0.1. Archive conserva entidades relacionadas, mantiene sus referencias navegables y las excluye de listas activas por defecto; filtros `includeArchived` permiten consultar explícitamente.

Relaciones validan allowlist + endpoint matrix de DOMAIN.md tanto en create como al leer/importar; no basta validación UI. En v0.1 extremos y typeCode son inmutables; corregirlos exige archivar y crear una nueva Relation para preservar historia. Update cambia únicamente contexto, justificación, origin y confidence con expectedRevision. Archive/restore no propaga a extremos ni provenance. Supports/contradicts deben tener origen/contexto y provenance pertinente para presentarse como literales; propuestas del investigador requieren origin explícito.

### Precisiones de captura y conceptos (ADR-022)

Las firmas wire anteriores se conservan. Texto canónico: CRLF/CR→LF y trim exterior, contenido interior intacto; validar máximos existentes antes de aceptar normalización, sin truncar. normalizedName/alias/domain colapsa whitespace Unicode y aplica Unicode lowercase, conserva tildes/puntuación, sin NFKC. paperIds/conceptIds/affectedConceptIds son sets de UUID válidos ordenados/deduplicados exactos; alias normalizados duplicados intraconcepto se rechazan. La canonicalización de contenido no cambia el hash de payload ni la identidad de receipts.

Captura sin provenance produce NONE real, sin Document/fila ficticia; literatura NONE se muestra pendiente de atribuir. Document explícito sin anclaje produce PENDING. Padre y cada Provenance inicial revision0; no expectedParentRevision en creación. paper_items nuevos usan P2 con selección=false y prioridad/motivo null; no inician ni activan fase.

Concept preferredName/title es un canon con mirror atómico, definition/bodyText una sola fuente. ConceptApi controla nombre/definición/aliases/domain; Knowledge.updateItem sobre Concept sólo admite confidence. Cualquier title/bodyText/attributes presente rechaza todo el patch aunque sea igual. Insight.affectedConceptIds debe ser subset de conceptIds en create y de vínculos existentes en update; quitar un afectado no desvincula, linkConcept no lo marca afectado.

UpdateItem/updateConcept/linkConcept nuevos requieren padre ACTIVE; ARCHIVED devuelve InvalidInput y requiere restore, incluso no-op. Nuevo enlace/captura hacia Concept archivado se rechaza. Con padre ACTIVE, enlace ya existente es no-op aunque destino ahora esté ARCHIVED: no crea asociación ni altera la histórica. Replay durable precede todas estas guardas. Un link efectivo incrementa sólo item.revision, nunca Concept destino; wire no añade expectedRevision, se lee revisión dentro de IMMEDIATE. No-op conserva clocks/revisiones/tiempos y no evento efectivo.

listConceptItems acepta filter.conceptId null o igual al argumento conceptId; distinto InvalidInput. domain compara igualdad normalizada en Concept propio o cualquier Concept vinculado (OR existencial sin filas duplicadas); no hereda domain de Paper. Referencias archivadas siguen legibles/exportables. Alias/preferredName iguales entre Concepts no fusionan sentidos.

Relación activa única por source/target normalizados según simetría, typeCode y contextText canónico. Create, update de contexto y restore que colisionan devuelven Conflict atómico: no reemplazar/fusionar ni dejar cambios de contenido, revisions, clocks, audit o receipt de éxito. Sólo la constraint conocida se traduce a Conflict; no ocultar otros errores SQL.

## Módulo Provenance

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

Exactamente uno de `itemId`/`relationId` requerido. `attachLocator` incrementa revision del padre en la misma transacción que crea Provenance y asociación. `updateLocator` revisa revision de Provenance, incrementa solo ella y conserva el hash capturado; cambiar localizador no valida automáticamente la cita. `checkDocumentHash` es mutación idempotente y usa `requestId` receipt; al detectar hash actual distinto cambia estados LOCATED→STALE e incrementa revision/updatedAt de cada Provenance afectado, sin alterar contenido/anclaje. `ProvenanceDto` expone revision y updatedAt; `KnowledgeItemDto.provenance[]` y status agregada reflejan PENDING/LOCATED/STALE/MIXED para que UI no oculte pendientes. `pageIndex` null o entero >=1. Al crear captura `capturedDocumentHash` desde Document registrado, no UI. Locator cambia a LOCATED solo si coordenadas/localizador validan y hash coincide; sin anclaje explícito: PENDING. Ruta original no sale por IPC ni export compartible.

### Anclaje registrado y conservación de fuente (ADR-022)

locator es null o el objeto cerrado `{kind:'page_region',x,y,width,height}` con números finitos: x/y>=0, width/height>0, cada valor<=1, x+width<=1, y+height<=1; región exige pageIndex entero>=1. Sin keys extra, coerción ni clamping. Página sola es anclaje registrado LOCATED con hash registrado igual al capturado; pageLabel/section/quote solos son PENDING. LOCATED describe registro de ubicación: T05 no comprueba page count, PDF físico ni verdad de cita. T07 usa Reader para validar/navegar la página real y presenta errores sin inventar navegación.

updateLocator conserva documentId y capturedDocumentHash; documentId distinto devuelve InvalidInput. STALE permanece STALE al editar. Si capturedHash discrepa del registrado, actualización válida deja STALE conservando ambos. Fuera de STALE, añadir/quitar página válida permite LOCATED/PENDING conforme regla, sin verificar cita. Fuente distinta requiere attach explícito nuevo y conservación de historia; no rebind/revalidate implícito en v0.1. checkDocumentHash nunca modifica Document.sha256 ni capturedHash, ni revive estados PENDING/STALE automáticamente.

## Corte y semántica prevalentes ADR-023

Antes del despacho leer TASK06_DECISIONS: T06a backend/candidatos/policy, T06b captura y cola UI, autores nuevos secuenciales. **Todos** save/evaluate/advance P2 de producción esperan T07; las pruebas de policy son fixtures explícitas, la aceptación P2 completa del brief se verifica después T07. Cero items tampoco habilita justificación por save en T06, nunca item ficticio. Summary conserva candidatos archivados y UI los señala; set exige P2 iniciada y permite deseleccionar archivado. La ABI y transferencias concretas se confirmarán contra T05 integrada; los archivos y firmas propuestos en el preflight no son implementación ni contrato congelado.
