# Tarea 08 — FTS, export, backup/restore y cambio de biblioteca

**Owner:** Luna backend search/portability. **Depende:** T02/T04/T05/T06/T07 integradas. **Riesgo:** muy alto por snapshots/recuperación. **Report:** `docs/reports/task-08-report.md` (subcortes 08a/08b/08c dentro del reporte). No subagents; Sol puede asignar subcortes secuencialmente y revisar cada uno.

Leer IMPLEMENTATION, CONTRACTS Search/ExportBackup/Settings y layouts/whitelists completos, DATA0004/recovery, SPEC-005/006/007, QUALITY. No estás solo, Sol actor/maintenance/lifecycle/migrator/receipt/composition. Ningún root switch/path mediante UI libre, ninguna copia DB con WAL activo.

**Posee:** `src-tauri/src/domain/{search.rs,portability.rs}`; `src-tauri/src/modules/search/{mod.rs,service.rs,projection.rs}`; `src-tauri/src/modules/portability/{mod.rs,snapshot.rs,export.rs,closure.rs,backup.rs,restore.rs,operations.rs,switch.rs}`; `src-tauri/src/adapters/sqlite/{search_repository.rs,operation_repository.rs,migrations/0004_portability.sql}`; `src-tauri/src/adapters/documents/{snapshot.rs,export_store.rs,backup_store.rs,restore_store.rs}`; `src-tauri/src/adapters/windows/pickers.rs`; `src-tauri/src/transport/commands/{search.rs,portability.rs,settings_library.rs}`; `src-tauri/tests/{search.rs,export.rs,backup_restore.rs,library_switch.rs}`.

**Consume:** todos canónicos/snapshots T01–T07 y coordinator maintenance; SearchApi/PortabilityApi/Settings library exactas anexo. **Produce:** searchLibrary/searchKnowledge, todas 14 operaciones PortabilityApi, selectLibrary/switchLibrary reales; proyección FTS tx/rebuild, jobs durability/status/result/error completos; funciones switch que Sol conecta lifecycle. No modificar Settings getAppInfo archivo T01: Sol capabilities verdaderas.

### 08a FTS

- [ ] Tests `fts5_unicode_diacritics`, `fts_literal_and_no_operators`, `filters_archive_combinations`, `rank_tie_uuid_cursor_stable`, `excerpt_html_neutralized_max_500`, `fts_mutation_rollback_canonical_consistent`, `rebuild_matches_reference_canon`, `corrupt_index_not_empty_result`. Migración0004 papers_fts entity_id/title/authors_text/venue/domain y items_fts entity_id/type_code/title/body_text; tokenizer `unicode61 remove_diacritics 2`. Search concepts por proyección canónica concept item, sin FTS PDF/OCR.
- [ ] Queries input literal términos AND, query1000/limit100default500max; filtros parametrizados/lifecycle e includeArchived consistentes; excluir archived por default. Rank no calidad científica; ordenar empate UUID. Hook projection en la misma tx de cada mutación Library/Knowledge/Concept por Sol; rebuild bajo mantenimiento. Revisar y checks antes de08b.

### 08b Snapshot y export

- [ ] Tests `snapshot_wal_includes_committed_rows`, `export_all_record_types_parse`, `export_closure_cross_paper_provenance`, `export_no_dangling_edges`, `manifest_counts_hashes_match`, `pdf_option_omits_bytes_and_paths`, `malicious_title_cannot_escape_export_root`, `destination_unwritable_no_publish`, `cancel_after_publish_returns_terminal_result`. Snapshot detiene nuevas writes/drena accepted; SQLite backup + PDFs/ontology referenciados inmutables; no mantener tx larga ni renombrar final antes verificación.
- [ ] Layout/RecordType/manifest/whitelist exactos CONTRACTS (anexo completo). JSONL recordVersion1, manifest exportVersion1.0; preservar phase definitions y associations paperAuthor/paperItem(item.selectedForP3/priority/rationale)/itemConcept/itemProvenance/relationProvenance/conceptAlias. Sin app_session/settings/receipts/locks/logs/backups/paths originales. Validation export solo si existe.
- [ ] exportPaper closure Paper→authors/venue/docs/phases/answers/items→concepts usados→Relations solo ambos extremos incluidos→provenance→documents referidos y cada Paper padre adicional. Association solo si ambos extremos incluidos. IDs estables, Markdown derivado nombres por UUID. PDFs opcionales en files/documentId/source.pdf y manifest explícito, export texto nunca se anuncia backup. Revisar y checks antes08c.

### 08c Jobs, backup, restore y switch

- [ ] Tests `backup_integrity_fk_hash_verified`, `corrupt_missing_pdf_rejected`, `backup_manual_external_picker`, `backup_cancel_owned_staging_only`, `operation_terminal_result_survives_restart`, `restore_traversal_absolute_junction_rejected`, `restore_space_failure_active_unchanged`, `restore_prepares_new_root_no_auto_activation`, `switch_crash_before_after_db_close_recoverable`, `switch_target_busy_or_schema_future_keeps_previous`, `backup_before_migration`, `no_automatic_purge`, `seven_days_backup_reminder_no_resident_task`.
- [ ] Native destination/backup/restore/library tokens 24h ligados biblioteca/operación; selectBackup directory formato app (no zip); restore nueva raíz verifica manifest/formato/hashes/FKs/schema/space/integrity antes preparado token; nunca sobrescribir biblioteca activa. Backup SQLite+PDFs+ontology+library.json+manifest verificados. Manual/pre-migration, conservar todas verificadas; recordatorio tras siete días uso sin backup, no scheduler.
- [ ] Job `queued|running|completed|failed|cancelled`, operationId diferente a idempotencia requestId; maintenance_operations y receipts/result/error sobreviven restart. Cancel solicita punto seguro/limpia solo staging dueño; tras publicación/preparación entrega resultado terminal; abandonar Promise no rollback.
- [ ] Switch explícito solo target `{kind:'selected',targetToken}` o `{kind:'restored',preparedLibraryToken}`; coordinator bloquea nuevas writes, resuelve pendientes/cierra DB/adquiere lock destino/valida+migra backup/publica raíz/reabre. Journal durable **fuera** root cerrándose identifica destino/anterior/estado; crash recovery siempre raíz completa y anterior recuperable. Sin merge/move automático. Sol conecta lifecycle/readOnly diagnostics y capability registry.
- [ ] Checks Rust/contract fixtures/reopen en proceso/conexión nueva por todos fault points, informe con comandos/hash fixture. Pasa FKs e integrity_check, comparación IDs/content/source hashes/phase version/candidates entre backup y restaurada.

**Aceptación:** FTS coherente y reconstruible; export sin private paths ni dangling refs; backup verificable y restore/switch real tras reopen conserva PDFs/IDs y biblioteca previa recuperable. No llamar completado solo por manifest generado.


# Anexo de contratos exactos

Extracto literal de docs/architecture/CONTRACTS.md, contractVersion 1, tomado el 2026-10-01. Esta copia facilita ejecución autónoma; el documento normativo gobierna y Sol debe regenerarla si modifica el contrato. No simplificar firmas, enums, nullable, revisiones ni envelopes. No implementar servicios ajenos al ownership por aparecer sus tipos consumidores en el anexo.

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


