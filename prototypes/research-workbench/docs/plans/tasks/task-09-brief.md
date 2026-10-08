# Tarea 09 — UI integrada, accesibilidad y verificación v0.1

**Owner:** Luna integración UI/QA. **Depende:** T01–T08 revisadas. **Riesgo:** alto por composición y evidencia. **Report:** `docs/reports/task-09-report.md`.

Leer IMPLEMENTATION, SPECS completo, QUALITY completo y APIs Search/Portability/Settings CONTRACTS. No estás solo; App/adapters/capabilities/ShellView/tokens/lifecycle compartidos Sol. Edición de panels T03/T06/T07 requiere transferencia ownership anotada antes, nunca concurrente.

**Posee:** `src/features/search/{SearchPage.tsx,SearchResults.tsx,useSearch.ts,search.test.tsx}`; `src/features/portability/{ExportDialog.tsx,BackupPanel.tsx,RestoreDialog.tsx,OperationProgress.tsx,portability.test.tsx}`; `src/features/settings/{SettingsPage.tsx,LibrarySwitchDialog.tsx,FirstRunLibrary.tsx,settings.test.tsx}`; `tests/ui/full_workflow.test.tsx`; `src-tauri/tests/full_workflow.rs`; `tests/fixtures/{research_fixture.json,fixture_manifest.json}`; `scripts/seed-quality-corpus.ps1`; `docs/verification/{functional-v01.md,accessibility.md,performance.md}`. Sol README/STATUS/public release notes.

**Consume:** todos APIs reales T01–T08, generated wire, feature panels. **Produce:** Settings/Search/Portability UI sin botones ficticios, props APIs; integration test fixture dos papers/concept/claim/evidence/relations/provenance/candidates, corpus QUALITY y mediciones, requests concretos composition roots a Sol.

- [ ] UI tests `search_filters_open_correct_entity_locator`, `search_index_error_not_empty`, `export_pdf_choice_explicit`, `backup_destination_cancel_safe`, `restore_prepare_then_confirm_switch`, `switch_error_shows_previous_active_library`, `operation_cancel_status_truthful`, `all_capabilities_match_real_registry`, `no_p3_p4_merge_trash_delete_controls`, `saved_only_after_commit`, `draft_survives_conflict`. Real backend test `full_pre_p1_p2_shared_concept_search_export_backup_restore_reopen` con expected IDs/hashes/answers/associations/candidates y definición fijada, además branches P1 light_read/archive.
- [ ] UI español Home/Library/PaperWorkspace/Knowledge/Settings; resumen biblioteca activa local/versión/diagnóstico readable; first-run confirma copia administrada. Snapshot jobs progreso/cancel/status consultable después restart; selector external backup/restore/target y confirmación switch muestra destino real. Recordatorio tras siete días de uso sin backup exitoso, sin scheduler residente. No activar restore automáticamente, no filesystem path libre.
- [ ] Sol integra panels/api/capability en shell y respuestas mutación recargan dependent views; no localStorage/IndexedDB canon. Search resultado navega item/concept/paper/locator, rank nunca confianza. Knowledge origin/confidence y PENDING/STALE/MIXED/lifecycle siempre legibles. Gate labels “P2 completada”, no validada ni Paper COMPLETED.
- [ ] Verificar teclado completo, foco inicial/visible/retorno, labels/errores/status live region, contraste/no color único, resizing/DPI100/150/200%; documentar dispositivos/tamaño/resultado real. UI simulada separada desktop real.
- [ ] Corpus determinista QUALITY:1000 papers/2000authors/10000items/15000relations/12000provenances/100000associations/500MB PDFs sintéticos; seed/hardware/time/corpus. Medir cada NFR exacto QUALITY (startup/query/open/save/UI stall/RAM/backuprestore) sin declarar propuesto como cumplido por intuición. No dataset privado; si medición no viable queda pendiente explícito.
- [ ] Execute scripts/check.ps1 y dependency boundary checks/registry-contract parity, DTO regenerate no diff, exe nativo sin bundle real offline (scope report), faults regression. Revisión Sol general/Rust/TS/security, devolver hallazgos propietarios y repetir checks afectados. Sol congela release después sin cambios concurrentes.

**Aceptación:** journey manual PRE→P1→P2/search/export/backup/restore/switch/reopen completo con SQLite real y UI confirmaciones honestas; FKs/hashes/state intactos; accesibilidad y NFR evidenciados o pendientes exactos. Gate final instalado T10 aún requerido.


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

