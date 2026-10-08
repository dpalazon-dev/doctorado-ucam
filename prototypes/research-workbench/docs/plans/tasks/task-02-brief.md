# Tarea 02 — Biblioteca e importación recuperable

**Owner:** Luna backend library. **Depende:** T01 revisada e integrada. **Complejidad/riesgo:** alta por filesystem/SQL. **Report:** `docs/reports/task-02-report.md`.

Leer arquitectura DOMAIN/CONTRACTS/DATA/SPEC-002/QUALITY y global constraints de IMPLEMENTATION. No estás solo: no revertir ajenos; no editar compartidos de Sol ni migración 0001 liberada. Trabajar en worktree asignado, no commits sin protocolo Sol.

**Posee:** `src-tauri/src/domain/library.rs`; `src-tauri/src/modules/library/{mod.rs,service.rs,repository.rs,recovery.rs,doi.rs}`; `src-tauri/src/adapters/sqlite/library_repository.rs`; `src-tauri/src/adapters/documents/{mod.rs,store.rs,imports.rs}`; `src-tauri/src/transport/commands/library.rs`; `src-tauri/tests/library_integration.rs`. Entregar a Sol registro/permissions/module exports requeridos.

**Consume:** DbActor/UoW/LibraryRoot/receipts T01, DTO `PaperMetadataInput|PaperDto|ImportPreviewDto|PaperFilterDto|PageDto`, tablas 0001. **Produce:** todos los ocho métodos LibraryApi exactos (anexo); servicios de aplicación usados por Reader y Workflow bajo la misma transacción, sin commits de repositorio. `reconcile_imports` devuelve informe de operaciones recuperadas/issues seguro, no éxito ambiguo. LibraryService asíncrono consume NativePdfSelection, DocumentStore y LibraryPersistence; el adaptador SQLite usa DbActor/UoW y llama helpers transaccionales de aplicación. Repositorios/helpers reciben Transaction como excepción pragmática explícita. Responsabilidades, formatos internos y recuperación están fijados en ../TASK02_PORTS.md, que debe leerse antes de implementar.

- [ ] Escribir tests `import_survives_original_move`, `duplicate_doi_normalized`, `duplicate_hash_requires_decision`, `reuse_existing_keeps_metadata`, `import_token_retry_payload_conflict`, `expired_token_rejected`, `cancel_removes_only_owned_staging`, `unicode_path_import`, `invalid_pdf_no_success`, `size_limit_500_mib`, `source_unreadable`, `recovery_after_staging`, `recovery_after_promote_before_commit`, `ambiguous_target_never_deleted`, `stale_revision_rejected`, `archive_restore_preserves_document_and_links`, `authors_order_survives_reopen`; confirmar fallos previos y PASS posterior.
- [ ] Implementar selector nativo con token 24h/biblioteca/sesión; copiar a staging y registrar STAGING, hash/verificación tamaño/legibilidad. PDF mínimo real, no inferir metadatos ni OCR. No paths UI. Normalizar DOI trim/quitar doi: y doi.org/lowercase/validar `10.<registrante>/<sufijo>` sin red.
- [ ] Antes de promoción guardar UUIDs reservados, metadata normalizada, relative path/hash; promover mismo volumen → PROMOTED → transacción Paper/authors/Document/COMMITTED/receipt/audit. Duplicado DOI/hash solo reuseExisting o cancel; reutilizar preserva existente. Recovery verifica propiedad/hash e idempotencia tras proceso nuevo; archivos ambiguos permanecen como issue.
- [ ] Implementar list/get/update/archive/restore con filters/cursor/limits, ACTIVE incluye NEW/ACTIVE/COMPLETED, no TRASHED. Optimistic revision y archiveFromLifecycle conservan datos/enlaces; errores SQL/files preservan intención recuperable. Processing piloto false; T04 modifica integración para inicializar workflow, no anticipar PRE falso.
- [ ] Verificar FKs/quick_check/reopen y fallo rollback (sin Paper exitoso), escribir reporte sin contenido sensible. Ejecutar `cargo test --manifest-path src-tauri/Cargo.toml --test library_integration` y checks Rust completos. Sol asigna registro commands al worker y revisa seguridad/receipt/import antes de integrar.

**Ownership adicional:** application/library_ports.rs y tipos internos requeridos; adapters/windows/pdf_picker.rs; integración explícitamente transferida de lib.rs, desktop/lifecycle.rs, modules/mod.rs, adapters/mod.rs, adapters/sqlite/mod.rs, adapters/windows/mod.rs, transport/commands/mod.rs, contracts/manifest.json, Cargo.toml/Cargo.lock y avisos de licencias. No ampliar capacidades frontend genéricas. Mantener firmas públicas; los cambios de wire/DTO que no sean correcciones ya revisadas requieren ruling antes de editar.

**Validación PDF:** aplicar `docs/development/PDF_VALIDATION.md` y ADR-014 antes de publicar preview. Se autoriza `adapters/documents/pdf_probe.rs`, pdf=0.10.0 sin default-features, lockfile/avisos y fixtures correspondientes. Buffer acotado, una validación activa, estructura/primera página, rechazo de cifrado; no certificar render completo ni una cota total de RAM. No modificar bytes del documento.

**Aceptación:** todas operaciones LibraryApi reales, UUID/authors/hash persistidos tras reopen y mover original; cero duplicados DOI/hash, cero pérdida/archivo ajeno eliminado y ningún Saved prematuro. UI posterior integra contra servicios confirmados.


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
