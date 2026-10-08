# Tarea 04 — PRE/P1 versionadas y gates

**Owner:** Luna workflow PRE/P1. **Depende:** T02/T03 integradas. **Riesgo:** alto por transacciones/gates/migración. **Report:** `docs/reports/task-04-report.md`.

Leer IMPLEMENTATION, DOMAIN fases/invalidation, CONTRACTS Workflow, DATA 0002, SPEC-003/QUALITY y WORKFLOW_GATES/ADR-017 con JSON canónicos v1. No estás solo: shared registry/DTO/import service/lifecycle pertenecen a Sol. No redefinir firmas. T06 añade P2 mediante archivos propios después.

**Posee:** `src-tauri/src/domain/{workflow.rs,gates.rs}`; `src-tauri/src/modules/workflow/{mod.rs,service.rs,definitions.rs,invalidation.rs}`; `src-tauri/src/adapters/sqlite/{workflow_repository.rs,migrations/0002_workflow.sql}`; `src-tauri/src/transport/commands/workflow.rs`; `src/features/workflow/{PhaseWorkspace.tsx,PhaseAnswerForm.tsx,GateIssues.tsx,useWorkflow.ts,workflow.test.tsx}`; `src-tauri/tests/{workflow_gates.rs,workflow_migration.rs}`. Sol registry de migración/composición, adaptación import T02 y futuras ediciones invalidation.

**Consume:** Paper/Document/T02 UoW/receipts/DTO T01. **Produce:** WorkflowApi getPhase/getPhaseAnswers/getPhaseDefinition/savePhaseAnswer/evaluateGate/advancePhase/goBackToPhase/touchPhase exactos anexo; helpers `initialize_processing` y `invalidate_effective_change` con Transaction prestada para T05/T06/T07; definiciones inmutables PRE/P1/P2 fijadas por version/hash. set/getP3Candidate T06 permanece capability unavailable hasta implementado.

- [ ] Tests `unknown_requires_explanation`, `not_applicable_requires_explanation`, `pending_blocks_gate`, `definition_version_pinned`, `gate_preview_no_mutation`, `stale_advance_reevaluated`, `p1_structured_decision_not_parsed_from_text`, `p1_continue_opens_p2`, `p1_light_read_no_next_phase`, `p1_archive_atomic`, `save_decision_never_archives`, `answer_edit_invalidates_completed_and_started_later`, `noop_keeps_completion`, `go_back_preserves_answers`, `touch_never_completes`, `pilot_upgrade_preserves_ids`. Red/green; fixtures compare persisted snapshots/history after reopen.
- [ ] 0002 definitions/phases/answers y FK versión/revisions/snapshots. PRE required keys visibles `purpose,uncertainty_target,baseline,expected_outcome,desired_depth,review_type` + título/documento vigente ACTIVE disponible. review_type confirma `{reviewType: ReviewType}` contra metadato actual; unknown exige UNKNOWN+explanation, default import no es procesado. P1 required `scope,out_of_scope,review_type,literature_cutoff,core_message,relevance_decision`; field_organization opcional. PhaseAnswer UNKNOWN/NOT_APPLICABLE requieren explicación no vacía; no mínimo de calidad científico.
- [ ] Declarative completionRules solo `answerProcessed|paperHasActiveDocument|p1DecisionProcessed|p2ArtifactPresent|p2NoCandidatesJustified`; no código/templates ejecutables. IMPORT v0.1 inicializa PRE IN_PROGRESS, P1/P2 NOT_STARTED, active PRE mismo commit, processingInitialized true. Upgrade papers piloto inicializa sin alterar UUID/metadatos/archivo. Sol adapta el import llamando helper dentro de su tx.
- [ ] Gate hash canónico incluye definitionHash/respuestas normalizadas/revisions/artifacts UUID ordenados; evaluate lectura, advance BEGIN IMMEDIATE reevalúa y guarda snapshot/state/active/revision/history/lifecycle. P1 structuredValue `{relevance:sufficient|use_with_caution|weak_for_my_purpose,readingDecision:continue|light_read|archive}`. Continue P2, light_read ACTIVE/active P1/next null, archive archiva/completa P1 atómico conserva lifecycle previo/active P1/next null. P2 nunca iniciada queda NOT_STARTED; P2 previa conserva NEEDS_REVIEW y datos. Gate reconoce las tres ramas, no exige continue para completar P1.
- [ ] goBack/touch comparan revisión fase actualmente activa, no target, no borran ni descompletan por navegar. Cambio efectivo respuesta marca fase COMPLETED y posteriores iniciadas NEEDS_REVIEW; NOT_STARTED intacta; no-op no invalida. Exponer helper para artifact changes posteriores bajo misma UoW.
- [ ] UI español hidrata respuestas y explicación/structuredValue, conserva borradores en conflict/error, salidas gate y progreso editorial explícito; P3/P4 sin controles activos. Tests UI accesibilidad. Checks UI/Rust + migration backup/reopen y reporte con helpers congelados a Sol.

- [ ] Semántica canónica [WORKFLOW_GATES](../../architecture/WORKFLOW_GATES.md) y JSON [v1](../../architecture/phase-definitions/): snapshot/normalización/no-op exactos, primera respuesta expected0→revision1, clock max+1 por Paper bajo UoW con tokens distintos por fase/contexto y replay antes CAS. Ningún cambio a wire ni payload v1.
- [ ] R1: PRE IN_PROGRESS con gate suficiente no permite touch(P1); cadena COMPLETED+snapshot aceptado+gate vigente para forward/completar, continue aceptado para P2; consulta fases iniciadas no acepta. NEW→ACTIVE sólo advance PRE; import/upgrade/lector/borradores conservan lifecycle/metadatos.
- [ ] R2: PDF vivo en inputs de PRE/P1/P2; proof/handle antes TX y referencia DB revalidada dentro; ABI tras T03, sin lectura/hash masivo en DbActor. PDF sintético inaccesible después de aceptación bloquea sin mutaciones/receipt exitoso, snapshot actual difiere y aceptado permanece; recuperación de acceso y cambio DB Document distinguidos.
- [ ] Tests `forward_touch_requires_accepted_prerequisite`, `continue_saved_does_not_enable_p2`, `unavailable_pdf_blocks_later_phase`, `document_proof_revalidated_in_tx`, `active_context_revision_never_collides`, `replay_precedes_cas`, `stale_noop_conflicts`, `completed_reserved_preserved_workflow_unavailable`, `light_read_preserves_started_p2`, `new_active_only_after_pre_advance`. Pin/lecturas/activación P2 disponible, saves/gates/advance/candidatos UnsupportedCapability hasta resolutores T06; aceptación P2 final espera T07.

**Aceptación:** PRE/P1 gates y las tres ramas exactas persistentes; cambio posterior conserva datos y requiere reconfirmación, ningún gate autorizado por UI stale.


# Anexo de contratos exactos

Extracto literal de docs/architecture/CONTRACTS.md, contractVersion 1, actualizado el 2026-10-02. Esta copia facilita ejecución autónoma; el documento normativo gobierna y Sol debe regenerarla si modifica el contrato. No simplificar firmas, enums, nullable, revisiones ni envelopes. No implementar servicios ajenos al ownership por aparecer sus tipos consumidores en el anexo.

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

`relevance_decision` ANSWERED exige `structuredValue: RelevanceDecisionValue` completo por enums; PENDING/null guarda borrador sin decidir rama, UNKNOWN/NA no admitidos. Nunca interpretar answerText. PRE.review_type confirma Paper.reviewType con estructura y resolución según [WORKFLOW_GATES.md](../../architecture/WORKFLOW_GATES.md); P1.review_type caracteriza textualmente la fuente. UNKNOWN/NA textual explicado no exige repetir explicación como answerText. savePhaseAnswer.expectedRevision protege la respuesta: primera escritura expected=0/revision=1; replay previo CAS, no-op posterior CAS conserva revisions/snapshots. save nunca archiva.

evaluateGate es lectura con snapshot/hash canónico. advance compara expectedPhaseRevision de fromPhase activa y reevalúa bajo IMMEDIATE; snapshot/fase/contexto/revisiones/lifecycle/audit/receipt se confirman juntos. P1 continue activa P2; light_read completa P1, Paper ACTIVE, active P1, nextPhase=null; archive completa P1+Paper ARCHIVED+archivedFromLifecycle, active P1, nextPhase=null. P2 nunca iniciada queda NOT_STARTED en las ramas terminales; P2 anterior conserva datos/NEEDS_REVIEW. Cerrar P2 mantiene active P2 y Paper ACTIVE, nextPhase=null, sin P3. Inputs obsoletos o gate bloqueado producen Conflict/GateBlocked sin efectos.

goBack/touch compara la fase actualmente activa, no destino. Navegar preserva respuestas/state/completedAt/snapshot; cambio real de contexto renueva revisión destino con clock fresco max(revision)+1 por Paper bajo UoW, sin tabla/wire nuevo. Habilitar fase hacia delante —inicializar o activar— y completar exige prerequisito COMPLETED, snapshot aceptado y gate vigente, más continue aceptado para P2. Touch no acepta prerequisitos; consulta/navegación de datos ya iniciados no autoriza avance. Definición/reglas/clock exactos en WORKFLOW_GATES.

PRE/P1/P2 incluye Document vigente vivo {id,status,sha256,available} en snapshot. Prueba de acceso/handle retenido antes TX; referencia DB revalidada dentro; ABI de documento se concreta tras T03, sin lectura/hash de PDF grande bajo DbActor. Archivo inaccesible bloquea sin mutar/receipt exitoso; desaparición externa no incrementa clock ficticio. Cambio DB Document invalida/renueva revisiones en su UoW.

setP3Candidate.expectedWorkflowRevision es revision paper_phases(P2). Sólo Claims/Questions asociados; priority null o integer1..5, rationale 1..5000 al seleccionar. Asociaciones son autoridad de selección; única justificación de cero candidatos en PhaseAnswer estructurada existente, con candidatos debe ser null. Save de esa key permite ausencia sin item ficticio y nunca cambia selección. Resumen y gate usan mismos IDs/count/justificación. Cambio efectivo de proyección renueva respuesta y phase clock en misma UoW. Seis salidas P2 enlazan artefactos por key; alternativa p2ArtifactPresent OR p2NoCandidatesJustified sólo para candidatos, no AND global ni código declarativo. Capacidad requerida ausente devuelve UnsupportedCapability; aceptación P2 final espera T07.

Tras cambio efectivo, fase COMPLETED y posteriores ya iniciadas pasan NEEDS_REVIEW con datos/snapshot histórico preservados; NOT_STARTED sin iniciar. No-op canónico no invalida; advance requiere reconfirmación. Detalle cerrado de normalización, artifacts, ausencia y snapshots en WORKFLOW_GATES.

En piloto 0.0.1 processingInitialized=false. Workflow v0.1 importa/inicializa PRE IN_PROGRESS, P1/P2 NOT_STARTED, active PRE y fija las tres definiciones v1 en el mismo commit. Upgrade preserva UUID/metadatos/documento/lifecycle/archivedFrom/revisiones bibliográficas; no genera confirmación PRE.review_type ni restaura. Sólo primer advance PRE exitoso cambia NEW→ACTIVE. Las condiciones de habilitación aceptada/vigente de WORKFLOW_GATES gobiernan P1/P2. P2 completada sigue Paper ACTIVE, no asigna COMPLETED global. COMPLETED reservado se conserva en lectura/upgrade/Library archive/restore; transiciones Workflow incompatibles UnsupportedCapability sin efectos. T04 puede leer/pinar/activar P2 por continue aceptado, pero sus saves/gates/advance/candidatos quedan capability-gated hasta resolutores reales T06/T07.



