# Tarea 05 — Conocimiento tipado y conceptos globales

**Owner:** Luna backend knowledge/concepts. **Depende:** T04 helpers invalidación y T02 DocumentStore. **Riesgo:** alto por UoW compartida. **Report:** `docs/reports/task-05-report.md`.

Leer IMPLEMENTATION, DOMAIN, CONTRACTS Knowledge/Concept/Provenance, DATA 0003, SPEC-004. No estás solo, no revertir ajenos. Sol shared DTOs/registry/invalidación; T07 no edita migration0003 en paralelo.

**Posee:** `src-tauri/src/domain/{knowledge.rs,concepts.rs,ontology.rs}`; `src-tauri/src/modules/knowledge/{mod.rs,service.rs,concept_service.rs}`; `src-tauri/src/adapters/sqlite/{knowledge_repository.rs,concept_repository.rs,migrations/0003_knowledge.sql}`; `src-tauri/src/transport/commands/{knowledge.rs,concept.rs}`; `src-tauri/tests/{knowledge_capture.rs,concepts.rs,ontology.rs}`. 0003 incluye **todo** esquema knowledge/concept/associations/relations/provenance/validations catálogo; T07 implementa sus servicios sin editar SQL liberado. Helpers inicial locator son primitivas de captura y se coordinan con T07 para única semántica.

**Consume:** KnowledgeInput/Patch discriminados y APIs exactos anexo, Origin/Confidence, Document hash y helpers workflow Transaction prestada. **Produce:** KnowledgeApi seis métodos y ConceptApi ocho métodos reales exactos; catálogo versionado core doce tipos y matrix relaciones 13 para T07; `capture` atómico item/details/paper links/concept links/provenance/audit/receipts/phase revisions. DTO nunca omite estado procedencia pendiente.

- [ ] Tests `all_12_type_attributes_roundtrip`, `mismatched_patch_type_rejected`, `origin_and_type_immutable`, `plain_text_only_v01`, `capture_rollback_no_orphans`, `capture_with_pending_provenance_visible`, `explicit_origin_confidence_required`, `shared_concept_two_papers_same_uuid`, `alias_unique_within_concept`, `homonyms_not_merged`, `concept_rename_keeps_uuid`, `archive_restore_preserves_links`, `archived_concept_reference_resolves`, `item_change_invalidates_p2_same_tx`, `noop_preserves_phase`, `item_edit_preserves_unchanged_candidate_projection`. Red/green usando SQLite y source hashes reales.
- [ ] Implementar `concept|claim|evidence|question|gap|assumption|condition|limitation|method|example|insight|reference` y attributes exactos anexo. bodyFormat plain_text/bodyJson null, literal quote separado. Create Concept solo ConceptApi con origin/confidence/provenance explícitos; UI defaults futuros visibles/editables, backend nunca atribuye silenciosamente.
- [ ] Capture conserva source document real y crea padre/procedencias en revision0 dentro de tx; literatura sin fuente es NONE pendiente de atribuir, con fuente sin anclaje es PENDING visible, no afirmar trazable/validada. Hash capturado del Document backend. Confidence humana explícita, no conteos. Estado Questions `OPEN|ANSWERED|DEFERRED|DISMISSED`, Reference `TO_REVIEW|INSPECTED|DISMISSED`.
- [ ] Suggest conceptos lexically por nombre/alias/domain; seleccionar existente o crear otro sentido, sin dedup global/merge. list/get filtros lifecycle/includeArchived consistentes: ACTIVE exige false, ARCHIVED/ALL exige true; archived refs siguen accesibles. expectedRevision update/archive/restore, enlaces no cascadan.
- [ ] Toda mutación efectiva de artefacto actualiza P2 revision/invalida COMPLETED vía helper workflow misma UoW; repetir payload/valor no cambia snapshot/revision indebida. Sol asigna al worker invalidation hooks y commands/permissions, y revisa. Checks Rust y fixture export canonical rows para T08, report con esquema/exports.

**Aceptación:** 12 tipos validables, dos papers reutilizan UUID Concept, captura sin huérfanos y pending explícito, archive/reopen preserva referencias y edits invalidan P2 sin perder datos.

**ADR-022 / TASK05_PORTS:** contratos internos aceptados antes de código; T05a esquema/dominio/captura, T05b edición/lifecycle/queries/IPC, secuenciales con autor nuevo. Se añade ownership application/{knowledge,knowledge_ports}.rs y domain/provenance.rs para regla pura única; no modules/knowledge/capture.rs redundante. Migrations/registries/composition se transfieren sólo con BASE/despacho. Casos application, auditoría before/after y alcance/inversa exactos según TASK05_PORTS. Las pruebas fixture P2 no acreditan gates/candidatos futuros.


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
