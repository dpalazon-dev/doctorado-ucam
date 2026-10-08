# Tarea 07 — Relaciones y procedencia

**Owner:** Luna relations/provenance. **Depende:** T04/T05; T06 para UI P2 final. **Riesgo:** alto integridad/hash. **Report:** `docs/reports/task-07-report.md`.

Leer IMPLEMENTATION, DOMAIN matriz relations, CONTRACTS Relations/Provenance, DATA0003, SPEC-004. No estás solo; no modificar migration0003 ya publicada ni shared invalidation/DTO/composition de Sol. Única semántica locator con capture T05; si helper no cubre contrato se pide ajuste a Sol.

**Posee:** `src-tauri/src/domain/{relations.rs,provenance.rs}`; `src-tauri/src/modules/relations/{mod.rs,service.rs}`; `src-tauri/src/modules/provenance/{mod.rs,service.rs,hash_check.rs}`; `src-tauri/src/adapters/sqlite/{relation_repository.rs,provenance_repository.rs}`; `src-tauri/src/transport/commands/{relations.rs,provenance.rs}`; `src/features/relations/{RelationForm.tsx,RelationsPanel.tsx,relations.test.tsx}`; `src/features/provenance/{LocatorForm.tsx,ProvenancePanel.tsx,provenance.test.tsx}`; `src-tauri/tests/{relations.rs,provenance.rs}`.

**Consume:** catálogo/0003/items/document hashes T05; UoW/receipts/revision/gate invalidation T04; APIs exactos anexo. **Produce:** RelationApi create/update/list/archive/restore; ProvenanceApi attachLocator/updateLocator/getProvenance/checkDocumentHash; panel navegación Document/page por ReaderApi, DTO status PENDING/LOCATED/STALE/MIXED.

- [ ] Tests `all_13_endpoint_pairs_allowed`, `invalid_pair_unknown_type_self_link_rejected`, `symmetric_endpoints_normalized`, `distinct_context_not_deduplicated`, `relation_endpoints_type_immutable`, `relation_update_revision_conflict`, `archive_relation_keeps_parent`, `attach_exactly_one_parent`, `attach_increments_parent_revision`, `locator_edit_only_provenance_revision`, `captured_hash_from_backend`, `hash_change_marks_located_stale_and_revision`, `hash_check_receipt_retry`, `pending_and_mixed_visible`, `relation_locator_change_invalidates_p2_same_tx`. Red/green and reopen hash/source navigation.
- [ ] Matriz: supports evidence→claim; contradicts claim↔claim; extends method→method o claim→claim; causes claim→claim/concept; requires method→condition; depends_on method→concept/method; works_when/fails_when method→condition; compares_with method↔method; part_of concept→concept; similar_to concept↔concept; limits limitation→claim/method; improves method→method. Simétricas contradicts/compares_with/similar_to normalizan extremos; no self-links. Validar catálogo/matriz en create/read/import, no solo UI.
- [ ] Relation type/extremos inmutables; update solo context/justification/origin/confidence expectedRevision. Archive/restore no propagación. Presentar supports/contradicts como literales requiere procedencia/contexto pertinente; causas/mejora no prueba epistemológica.
- [ ] attach exactamente uno itemId/relationId y expectedParentRevision; crear locator/link e incrementar parent/audit/receipt/P2 revisión mismo tx. update expectedRevision Provenance incrementa solo ella, capturedDocumentHash conservado, no valida cita automáticamente. pageIndex null o entero>=1; página registrada con región opcional válida y hash registrado/capturado igual da LOCATED como ubicación registrada; no verifica página física ni cita, sin anclaje PENDING.
- [ ] checkDocumentHash hashing background y modificación idempotente receipt; hash distinto LOCATED→STALE/revision/updatedAt por provenance afectada, no editar contenido/hash capturado. Agregada item PENDING/LOCATED/STALE/MIXED visible. Sol conecta workflow invalidación conservando parent revision regla de updateLocator.
- [ ] UI quote literal separado de interpretación, editor localizador/errors, source abre documento/página registrada; origin/confidence explícitos; arquivar/restaurar relaciones y referencias archivadas navegables. Keyboard/focus/status tests; checks Rust/UI/reporte.

**Aceptación:** 13 tipos exactos y semántica validada, relaciones/provenance persistentes sin huérfanos, cambios externos STALE visibles, origen/cita no mezclados y P2 invalidado atómicamente.

**ADR-022:** reutilizar domain/provenance de T05. updateLocator conserva documentId/capturedHash y STALE; Document distinto InvalidInput, hash registrado distinto deja STALE, checkHash no reescribe hashes ni revive. Navegación Reader comprueba página física. Relations activas únicas por extremos/tipo/contexto canónico: create/update/restore colisionando da Conflict sin efectos. Probar modificación de contexto hacia otro activo, STALE sticky, ausencia de rebind y alcance/inversa P2 sin expansión por relaciones. No editar0003 ni atribuir a locator registrado una verificación científica.


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

## Activación P2 tras resolutores (ADR-023)

T06 entrega captura/candidatos y policy con fixtures; ningún save/evaluate/advance P2 de producción se habilita parcialmente. Tras conectar y revisar todos los resolutores T07, habilitar el conjunto según TASK06_DECISIONS y verificar el journey P2 completo. Transferencias de archivos Workflow/UI decididas por Sol antes del despacho; no cambiar wire ni JSON v1.
