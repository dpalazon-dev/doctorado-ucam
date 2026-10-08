# Contratos IPC de Research Workbench

**Estado: baseline de ejecución v0.2, adoptada el 1 de octubre de 2026.** Este documento define el contrato interno Tauri IPC para `CONTRACTS.md` del proyecto. No es una API HTTP ni pública. DTOs neutrales, serialización JSON `camelCase`, SQLite fuente canónica. Debe mantenerse consistente con `DOMAIN.md`, `DATA.md` y los SPECs.

## Versión, compatibilidad y disponibilidad

- `contractVersion = 1` es la versión del sobre IPC, independiente de `schemaVersion`, `ontologyVersion` y versión de app.
- UUID se transmite como string canónico; timestamps como UTC RFC3339; enums usan exactamente las claves de `DOMAIN.md` (mayúsculas donde se especifica). Conforme ADR-019, escritores de fechas canónicos con milisegundos y sufijo `Z`; lectores admiten también `+00:00` con validación completa y preservan el string original. Se rechazan fechas locales, otros offsets y `-00:00`; no se reescriben receipts previos.
- **Piloto 0.0.1** implementa Library, Reader y Desktop lifecycle. **v0.1** añade Workflow, Knowledge, Concepts, Relations, Provenance, Search, Export, Backup y Settings. Esto no rebautiza el piloto como v0.1.
- DTOs de módulos futuros pueden definirse aquí desde ahora, pero comandos no implementados no se simulan como éxito; capability registry los marca `available: false` hasta la versión que los implemente.
- Cambios incompatibles de JSON requieren incrementar `contractVersion`; cambios aditivos tolerables conservan versión y no reutilizan un campo con semántica distinta.

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

## Módulo Library

Mantiene `LibraryApi` resumido del plan, con correcciones explícitas para duplicados y apertura/reanudación. La UI solo solicita selección de archivo vía picker nativo; no obtiene permiso de lectura arbitraria de rutas.

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

`selectPdf` devuelve `null` si el diálogo se cancela. DOI se normaliza: trim, quitar prefijo `doi:` o host `https://doi.org/`/`http://doi.org/`, lowercase y validar `10.<registrante>/<sufijo>`; no se resuelve en red. Si hay candidatos por DOI/hash, `confirmImport` exige decisión `reuseExisting` o se cancela con `cancelImport`; crear otro Paper con mismo DOI o SHA-256 no se admite en v0.1. Reutilizar devuelve Paper existente y no altera metadatos ni documento. El mismo importToken confirmado con mismo payload devuelve PaperDto previo; payload incompatible da Conflict. Token válido 24 h; recovery puede renovar token de la intención. Un DOI normalizado nunca crea Paper duplicado. La coincidencia no fusiona semánticamente conceptos.
Cuando confirmImport devuelve DuplicateDecisionRequired, error.details.candidates contiene una lista no vacía de DuplicateCandidateDto desde la transacción que detecta el duplicado, también en la comprobación final tras promoción. IDs únicos, orden por paperId; reasons sin repeticiones en orden doi, sha256; sin rutas ni contenido PDF. El error no confirma la importación ni permite cambiar el payload de una intención ya ligada. No hay nuevo IPC/DTO ni cambio de contractVersion. La interfaz ofrece abrir el candidato mediante cancelImport confirmado y después Reader.openPaper; no modifica silenciosamente duplicateResolution ni busca candidatos recorriendo listPapers. Fallo de cancelación conserva el diálogo/borrador sin abrir; fallo de apertura posterior permite reintentar solo Reader. reuseExisting conserva su semántica backend para peticiones compatibles. Interacción exacta en TASK03_BOUNDARIES.

Precondiciones/postcondiciones: token existe, no expirado y pertenece a proceso/biblioteca; source ya copiado a staging. En éxito, Paper+authors+Document y estado de import se confirman consistentemente. En error recuperable, staging queda ligado a intención; no se comunica éxito. Cancelar solo afecta el token indicado. Archive/restore conserva UUID, documento y relaciones y exige expectedRevision.

## Módulo Reader

Abrir el Paper actualiza en una transacción el último abierto, actividad y fase contextual; cubre la ausencia de setter de last-opened del resumen anterior.

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

Si no existe fila, `getReadingPosition` devuelve valor por defecto determinista `pageIndex=1, zoom=1.0, revision=0, updatedAt=document.importedAt` sin crearla. Page index es entero desde uno; zoom finito y rango propuesto 0.25–5.0. El backend no conoce conteo PDF; frontend limita página, y backend valida rango estructural. Cada documento conserva posición separada. Guardados serializados por documento y optimistic revision; el cliente no interpreta respuesta antigua como estado actual. `documentUrl` solo lo emite el servicio tras verificar Document registrado, UUID, path canónico dentro de raíz; el protocolo rechaza traversal y archivos no registrados. El protocolo sirve solo el PDF de ese Document, no ruta arbitraria. Cancelación de render no cancela una persistencia ya confirmada.

openPaper registra lastOpenedAt, contexto de sesión, auditoría y receipt en una transacción; conserva Paper.revision y Paper.updatedAt bibliográficos y no cambia metadatos/fase. El replay no vuelve a registrar actividad. Antes de emitir documentUrl se verifica el acceso actual, también en replay: el receipt previo no garantiza que el archivo siga disponible. Perfil de lectura y recursos locales: ADR-015 y docs/plans/TASK03_BOUNDARIES.md.

## Módulo Workflow

`PhaseDefinitionDto` es snapshot declarativo e inmutable: `code`, `version`, `name`, `objective`, `keyQuestions`, `doItems`, `dontItems`, `considerations`, `requiredOutputs`, `completionRules`, `definitionHash`. Cada salida es `{key,label,prompt,required,allowedResolutions,ruleKey}`; no contiene scripts, expresiones evaluables ni código. `ruleKey` pertenece al enum cerrado `answerProcessed | paperHasActiveDocument | p1DecisionProcessed | p2ArtifactPresent | p2NoCandidatesJustified`. La decisión P1 procesada satisface completitud de P1 en sus tres valores; la rama `continue`/`light_read`/`archive` se aplica en `advancePhase`, no es un requisito de gate que excluya dos opciones. La definición fijada al crear processing no cambia si se instala una versión nueva; el snapshot previo se preserva en DB y export. `PhaseAnswerDto`: La resolución canónica es [WORKFLOW_GATES.md](WORKFLOW_GATES.md); los JSON [PRE/P1/P2 v1](phase-definitions/) fijan payloads inmutables embebibles al compilar. completionRules enumera handlers por salida con alternativa cerrada candidatos/ausencia, no una conjunción global.

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

`relevance_decision` ANSWERED exige `structuredValue: RelevanceDecisionValue` completo por enums; PENDING/null guarda borrador sin decidir rama, UNKNOWN/NA no admitidos. Nunca interpretar answerText. PRE.review_type confirma Paper.reviewType con estructura y resolución según [WORKFLOW_GATES.md](WORKFLOW_GATES.md); P1.review_type caracteriza textualmente la fuente. UNKNOWN/NA textual explicado no exige repetir explicación como answerText. savePhaseAnswer.expectedRevision protege la respuesta: primera escritura expected=0/revision=1; replay previo CAS, no-op posterior CAS conserva revisions/snapshots. save nunca archiva.

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

## Módulo Search

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

Search usa FTS5 con tokenizer `unicode61 remove_diacritics 2`; consulta trata input como términos literales unidos por AND, sin exponer sintaxis avanzada FTS. Proyección FTS interna sin IDs/rowid como contrato público, mantenida en la misma transacción que los registros canónicos. Filtros parametrizados; excerpt límite 500 chars y neutralización de HTML. No busca texto integral del PDF/OCR en v0.1. Excluir archivados por defecto; no aplicar rank como puntuación científica. Índice reconstruible desde registros canónicos.

## Módulos Export y Backup

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

Tokens de destino/backup proceden de diálogo nativo; no aceptar paths libres. Tokens expiran en 24 h y quedan ligados a operación/biblioteca. Export JSONL/Markdown usa snapshot consistente, omite rutas privadas e incluye PDF solo por opción. Layout exacto:

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
  files/<documentId>/source.pdf       # solo con includePdfs
```

Cada JSONL línea es `{recordType: RecordType, recordVersion: 1, data: object}`; `RecordType` es el enum cerrado `paper | author | venue | document | phaseDefinition | paperPhase | phaseAnswer | knowledgeItem | concept | relation | provenance | validation | paperAuthor | paperItem | itemConcept | itemProvenance | relationProvenance | conceptAlias`. Manifest cerrado: `{exportVersion:'1.0',schemaVersion:number,ontologyVersions:Record<string,string>,createdAt:string,entityCounts:Record<string,number>,files:Array<{path:string,sizeBytes:number,sha256:string}>,includedPdfs:boolean}`. No exportar `app_session`, `app_settings`, receipts, locks, backups, logs o rutas originales. Los `data` preservan UUID y FKs como IDs; cada tipo usa whitelist cerrada basada en columnas canónicas: Paper (`id,title,doi,year,reviewType,domain,url,venueId,lifecycle,createdAt,updatedAt`); Author (`id,displayName,orcid`); Venue (`id,name,kind,identifier`); Document (`id,paperId,originalFilename,sha256,mediaType,sizeBytes,importedAt,status`); PhaseDefinition (code/version/definitionHash/definition JSON declarativo); PaperPhase (paperId/phaseCode/definitionVersion/state/revision/acceptedGateSnapshotHash/completedAt); PhaseAnswer (paperId/phaseCode/questionKey/answerText/structuredValue/resolution/explanation/revision/updatedAt); KnowledgeItem (id/typeCode/title/bodyText/bodyFormat/bodyJson/origin/lifecycle/confidence/attributes/revision/timestamps); Concept (itemId/preferredName/normalizedName/domain/mergedIntoId); Relation (id/sourceItemId/targetItemId/typeCode/contextText/justificationText/origin/confidence/lifecycle/revision/timestamps); Provenance (id,documentId,pageIndex,pageLabel,section,quoteText,locator,capturedDocumentHash,locatorState,revision,timestamps). Association records son `paperAuthor(paperId,authorId,position)`, `paperItem(paperId,itemId,phaseCode,selectedForP3,priority,rationale)`, `itemConcept(itemId,conceptId)`, `itemProvenance(itemId,provenanceId)`, `relationProvenance(relationId,provenanceId)`, `conceptAlias(conceptId,alias,normalizedAlias)`. Estos registros preservan los edges semánticos completos; una asociación se emite únicamente cuando todos sus extremos están incluidos. Validation rows se incluyen cuando existan. `phase_definitions.jsonl` preserva la definición inmutable necesaria para interpretar históricamente cada `definitionVersion`.

Para `exportPaper`, closure inicia en Paper y agrega autores/venue/documents, fases/respuestas y KnowledgeItems asociados; Concepts usados y Relations solo si ambos extremos entran en el closure. Después agrega provenance de cada item/relation incluido y cualquier Document referenciado por esas provenance, incluyendo siempre el Paper padre de cada Document, aunque no sea el Paper inicial. Así no se emite relación con extremo ausente ni se omite procedencia de un objeto incluido. Asociaciones se exportan solo si sus extremos están incluidos. Si no se incluyen PDFs, se conserva Document metadata/hash y se omiten binario y ruta.

Backup incluye snapshot SQLite + PDFs referenciados + ontología/manifest, verificado por hashes e `integrity_check`. `selectBackup` selecciona un backup directory creado por esta aplicación. Restore escribe a raíz nueva, valida contenido y devuelve `PreparedLibraryDto`; nunca activa automáticamente. Usuario revisa destino y activa mediante `switchLibrary`. Si se pierde conexión/proceso, `getOperationStatus` devuelve result/error terminal completo; job y receipt sobreviven restart. Backup v0.1 manual y pre-migration; conservar todas las copias verificadas; ninguna purga automática.

Cancelación de export/backup/restore solo marca cancelación y limpia staging cuyo operationId/intención coincide; tras publicar export/backup o preparar restore, devuelve el resultado terminal. Restore cancelado deja la biblioteca activa intacta. Mutaciones SQL cortas no se interrumpen a mitad; cancelar después del commit recupera el receipt y resultado.

## Módulo Settings y Desktop lifecycle

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

`LibraryInfoDto = { libraryId: UUID; displayName: string; rootLabel: string; schemaVersion: number; writable: boolean }`. `switchLibrary` pertenece a v0.1, no al piloto 0.0.1. Añadir `selectLibrary` para que un diálogo nativo emita `targetToken` y `switchLibrary` valide la biblioteca antes de activar: cerrar/flush/soltar lock de la anterior, validar y migrar destino con backup y adquirir su lock de forma ordenada. En piloto ambos devuelven `UnsupportedCapability`; no aceptar path libre. Si seleccionar/cambiar falla, la biblioteca activa anterior sigue abierta o recuperable y el usuario ve cuál continúa activa.

Root de biblioteca se determina en backend; solo se expone etiqueta amigable, no path absoluto, salvo pantalla local diagnóstica explícita. Configuración no acepta arbitrary filesystem path. La elección/cambio de biblioteca se incorpora en v0.1 mediante tokens de picker y validación de destino.

Arranque toma single-instance/library lock antes de recuperar operaciones o escribir, verifica schema compatibility, migra solo tras backup y reconcilia staging. Cierre bloquea nuevas mutaciones, resuelve persistencias pendientes con timeout finito, cierra DB y suelta lock; no reporta Saved previo al commit. Segunda instancia enfoca primera o informa Busy. WebView2/instalador no son IPC de dominio. `getLastOpenedPaper` es lectura; `openPaper` de Reader es el único setter y debe persistir last-open en commit. El backend serializa operaciones con actor dedicado y una conexión SQLite; cualquier `UnitOfWork` cruzado conserva transacción en esa misma conexión.

## Atomicidad, cancelación y postcondiciones

| Operación | Atomicidad y postcondición |
|---|---|
| Import | Saga filesystem/SQLite con `import_operations`; resultado solo tras commit; reconciliación reiniciable por token. No borrar destino/archivo de propietario ambiguo. |
| Metadata/archive/restore | Una transacción DB incrementa revision y auditoría. Stale revision no cambia nada. |
| Save answer | Respuesta+revision+historial en transacción. |
| Advance phase | Releer todas las respuestas/salidas/gates bajo write transaction; falla sin efectos o completa la fase e inicializa la siguiente cuando esa rama la habilita, en un único commit; light_read/archive en P1 y cierre P2 devuelven nextPhase=null. |
| Create item/relation/provenance | Objeto, asociaciones, extensiones y audit event en una transacción; sin entidad huérfana por fallo parcial. |
| Save reading position | Last-write por revision bajo transacción; los clientes serializan; stale revision da Conflict y cliente vuelve a leer/mezcla con usuario. |
| Export/backup | Snapshot consistente coordinado con imports/DB; solo publicar carpeta final tras verificar; temporal cancelable antes del rename final. |
| Restore | Nunca muta biblioteca activa durante validación; activa solo tras éxito y elección explícita fuera del contrato de restore. |

`requestId` persistido hace retry seguro de las mutaciones designadas. `operationId` identifica trabajo largo de export/backup y permite consultar/cancelar progreso; no es clave idempotente. Cancelación no significa rollback de un commit ya confirmado. Si proceso cae, el siguiente arranque reanuda o deja issue recuperable según estado durable. Mensajes no incluyen texto científico, rutas privadas, stack trace ni bytes de PDF.

## Seguridad y recursos

- Solo comandos explícitos; React no puede ejecutar SQL, leer archivos arbitrarios o invocar shell.
- Picker nativo produce token; token de import expira y se vincula a sesión/biblioteca. PDF protocol recibe Document UUID, resuelve path desde SQLite y verifica canonicalización/raíz.
- Consultas SQL parametrizadas; `limit` acotado; validar UUID, enums, resolución de IDs y tamaño antes de escribir.
- PDFs, JSON de ontología, body y quote son datos no confiables; no ejecutar contenido ni convertir a HTML sin sanitizar. CSP restringida a recursos empaquetados y protocolo de documentos.
- No telemetría ni red en núcleo local. Logs rotados de error/códigos/UUID técnicos; no body, quotes, PDF, paths originales o metadatos completos.
- SQLite single writer por biblioteca; foreign keys on en cada conexión. Operaciones destructivas no disponibles en piloto/v0.1.

## Registro de comandos Tauri

Las funciones internas se registran con prefijo de feature y este registry es cerrado en contractVersion 1: `library_select_pdf`, `library_confirm_import`, `library_cancel_import`, `library_list_papers`, `library_get_paper`, `library_update_metadata`, `library_archive_paper`, `library_restore_paper`; `reader_open_paper`, `reader_get_last_opened_paper`, `reader_get_reading_position`, `reader_save_reading_position`; `workflow_get_phase`, `workflow_get_phase_answers`, `workflow_get_phase_definition`, `workflow_save_phase_answer`, `workflow_evaluate_gate`, `workflow_advance_phase`, `workflow_go_back_to_phase`, `workflow_touch_phase`, `workflow_set_p3_candidate`, `workflow_get_p3_candidate_summary`; `knowledge_create_item`, `knowledge_update_item`, `knowledge_archive_item`, `knowledge_restore_item`, `knowledge_get_item`, `knowledge_list_items`; `concept_suggest`, `concept_get`, `concept_list_items`, `concept_create`, `concept_update`, `concept_link`, `concept_archive`, `concept_restore`; `relation_create`, `relation_update`, `relation_list`, `relation_archive`, `relation_restore`; `provenance_attach_locator`, `provenance_update_locator`, `provenance_get`, `provenance_check_document_hash`; `search_library`, `search_knowledge`; `export_choose_destination`, `export_library`, `export_paper`, `backup_choose_destination`, `backup_create`, `backup_select`, `backup_choose_restore_target`, `backup_verify`, `backup_restore`, `operation_get_status`, `operation_cancel`; `settings_get_app_info`, `settings_get_library_info`, `settings_select_library`, `settings_switch_library`, `settings_get_library_status`. Los comandos no implementados permanecen capability-gated; no se aceptan aliases libres. No se registra una API genérica de filesystem, SQL o ejecución.

## Decisiones propuestas fijadas y validación antes de aceptación

Las decisiones de esta sección están cerradas como propuesta coherente para revisión; no equivalen a aprobación. La aceptación debe comprobarlas contra la implementación planeada y pruebas de frontera: límites declarados (title 1000, answer/body 20.000, snippet/quote 10.000, contexto/relación 5.000, 100 autores, PDF 500 MiB); DOI normalizado sin resolución en red y sin duplicar DOI/hash; allowlist completa de 13 relaciones con matriz de extremos de DOMAIN; `plain_text` con `bodyJson=null`; protocolo de documento por UUID con capability local restringida; conflictos por revision optimista, no-op canónico sin invalidación y cambio real invalidando snapshots dependientes según DOMAIN. Hard delete y merge quedan fuera de v0.1 y requieren ADR posterior. La aceptación requiere pruebas de límites/Unicode, round-trip Rust↔TypeScript de DTO/envelope, rechazo de payload/enum desconocido, reintento idempotente, conflicto de revision, reconciliación import filesystem/SQLite, reconstrucción export/backup con asociaciones y definiciones de fase, y compatibilidad de migraciones. El protocolo y las capabilities concretas se fijan en el documento de arquitectura para la versión elegida; no amplían el conjunto de comandos de este contrato.

Los filtros con lifecycle e includeArchived tienen una única interpretación: lifecycle=ACTIVE exige includeArchived=false; lifecycle=ARCHIVED o ALL exige includeArchived=true. Combinaciones incoherentes devuelven InvalidInput. El estado de lifecycle gobierna el filtrado y el booleano expresa la elección explícita de consultar archivados.

## Precisión de candidatos y entrega P2 (ADR-023)

Rigen las siete decisiones de [TASK06_DECISIONS](../plans/TASK06_DECISIONS.md). setP3Candidate exige P2 iniciada sin activarla; selected=false exige priority/rationale=null y permite deseleccionar item archivado. Summary conserva seleccionados archivados; el gate exige artefactos activos. Cero candidatos tras set exige justificación1..5000 recibidos y canónica no vacía, con candidatos null. Save candidata permite PENDING/null, nunca cambia selección; objeto presente completo y coincidente con DB. Ese save, al igual que todo save/evaluate/advance P2, se habilita en producción sólo tras T07. T06 entrega captura/candidatos reales; no existe guardado de justificación cero sin items hasta habilitar save. La ABI interna se fijará tras verificar T05; wire existente intacto.
