# Tarea 03 — Biblioteca y lector completos

**Owner:** Luna library/reader UI + Reader backend. **Depende:** T01; T02 para pruebas reales. **Riesgo:** alto por protocolo/guardados tardíos. **Report:** `docs/reports/task-03-report.md`.

Leer IMPLEMENTATION global constraints, CONTRACTS Library/Reader, DATA, SPEC-001/002, QUALITY, TASK03_BOUNDARIES.md y TASK03_PORTS.md. No estás solo ni revertir ajenos. Sol asigna los archivos shared concretos al dispatch después de congelar T02; el worker integra y Sol revisa, sin código de producto escrito por el orquestador.

**Posee:** `src/features/library/{LibraryHome.tsx,LibraryPage.tsx,ImportPaperDialog.tsx,PaperDetails.tsx,useLibrary.ts,library.test.tsx}`; `src/features/reader/{PaperReader.tsx,pdfLoader.ts,useReadingPosition.ts,reader.test.tsx}`; `src-tauri/src/modules/reader/{mod.rs,service.rs,protocol.rs}`; `src-tauri/src/adapters/sqlite/reader_repository.rs`; `src-tauri/src/transport/commands/reader.rs`; `src-tauri/tests/{reader_integration.rs,document_protocol.rs}`; `tests/fixtures/{sample.pdf,scanned.pdf,invalid.pdf}`. Fixtures sintéticos, no documentos privados.

**Consume:** LibraryApi T02, ReaderApi exacta anexo, generated DTO/wire T01, DocumentStore T02. **Produce:** `LibraryPage({api,onOpenPaper})`, `LibraryHome({libraryApi,readerApi,onOpenPaper})`, `PaperReader({opened,api,onClose})` con props tipadas; ReaderApi real, protocolo research registrado por el worker en la composición asignada y revisado por Sol, PDF.js worker/fonts/cMaps empaquetados. No import invoke en componentes.

- [ ] Tests UI `empty_library_shows_import`, `picker_cancel_no_form`, `metadata_error_accessible`, `duplicate_opens_existing`, `archive_filter_and_restore`, `conflict_preserves_draft`, `reader_saved_page_restored`, `scanned_pdf_renders`, `corrupt_protected_pdf_recoverable`, `render_old_result_discarded`, `close_cancels_render`, `save_error_never_shows_saved`. Backend `open_paper_sets_last_opened_transactionally`, `get_paper_no_last_open_effect`, `position_default_no_insert`, `position_per_document_revision`, `protocol_unknown_uuid_rejected`, `protocol_traversal_rejected`, `protocol_junction_escape_rejected`. Failing tests → minimal code → PASS.
- [ ] Biblioteca español con Home último abierto, list/filtros/metadatos/edit/archive/restore; title requerido, authors/año opcionales sin inventar. Estados loading/empty/no-results/error/pending/saved claros; foco y teclado en diálogos. Mutación confirmada actualiza/recarga dependientes.
- [ ] Reader openPaper único setter last-opened/actividad/contexto en commit; default posición pageIndex=1 zoom=1 revision=0 updatedAt=importedAt sin insertar. Save expectedRevision; backend pageIndex entero>=1 zoom finito 0.25..5.0, frontend limita a conteo PDF. Guardados serializados por documento; respuesta antigua no se muestra como estado actual.
- [ ] Protocol resuelve solo documentId registrado y archivo propio canónico, valida reparse points/raíz; no acceso paths libres. Render PDF.js offline, worker y assets locales, cancelar render anterior y liberar documento/canvas al cambiar/cerrar; errores no destruyen biblioteca. Página física desde uno y zoom/atajos/jump accesibles.
- [ ] Integrar recorrido real import→mover original→abrir→page2→cerrar→nuevo proceso→reanudar→archive→restore con backend T02; mocks UI etiquetados separados. Checks UI/Rust y reporte. El worker conecta App/IPC/capabilities en los archivos asignados; Sol revisa integración y protocolo; T10-piloto verifica instalador antes de declarar 0.0.1.

**Decisiones previas:** ADR-015/TASK03_BOUNDARIES fija puertos, actividad sin alterar revisión bibliográfica, protocolo200completo sin anunciar Range, recursos PDF.js locales y WASM/CSP limitado. Las firmas internas y paths compartidos concretos se asignan con la base T02 revisada, antes de empezar T03. No interpretar async como streaming ni cancelación UI como aborto Rust.

**Aceptación:** reanudación y archive/restore conservan IDs/doc/hash/posición, render offline sin CDN, ningún path no autorizado ni Saved antes del commit.


# Anexo de contratos exactos

Extracto literal de docs/architecture/CONTRACTS.md, contractVersion 1, tomado el 2026-10-01 y aclarado para Reader el 2026-10-02. Esta copia facilita ejecución autónoma; el documento normativo gobierna y Sol debe regenerarla si modifica el contrato. No simplificar firmas, enums, nullable, revisiones ni envelopes. No implementar servicios ajenos al ownership por aparecer sus tipos consumidores en el anexo.

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
