# Tarea 01 — Base ejecutable, persistencia y contratos

**Responsable:** worker Sol de scaffold por complejidad; coordinador Sol revisa/integración. **Complejidad/riesgo:** alta, fronteras IPC/persistencia/distribución. **Depende de:** baseline documental y entorno comprobado por Sol. No instalar SDKs/frameworks globales ni cambiar configuración global. No hacer commits hasta que Sol indique el protocolo del worktree.

## Autoridad y ownership

Leer `docs/architecture/{ARCHITECTURE,DOMAIN,CONTRACTS,DATA,SPECS,QUALITY,ADRS}.md`, `AGENTS.md` cuando exista y `docs/plans/IMPLEMENTATION.md`. La arquitectura aceptada manda sobre este brief. No estás solo: no revertir cambios ajenos. Ante conflicto de contrato/ownership, informar a Sol y continuar archivos independientes.

Posees inicialmente `package.json`, `package-lock.json`, `index.html`, `vite.config.ts`, `vitest.config.ts`, `tsconfig.json`, `tsconfig.app.json`, `tsconfig.node.json`, `src/vite-env.d.ts`, `src/main.tsx`, `src/app/{App.tsx,ShellView.ts,tokens.css}`, `src/shared/contracts/{generated/,wire.ts,ports.ts}`, `src/shared/adapters/tauri/{client.ts,index.ts}`, `contracts/{fixtures/,manifest.json}`, `src-tauri/{Cargo.toml,Cargo.lock,build.rs,tauri.conf.json,capabilities/main-local.json,permissions/,icons/}`, `src-tauri/src/{main.rs,lib.rs,domain/mod.rs,application/mod.rs,modules/mod.rs}`, `src-tauri/src/application/{ports.rs,unit_of_work.rs}`, `src-tauri/src/transport/{mod.rs,dto.rs,error.rs,commands/mod.rs,commands/settings.rs}`, `src-tauri/src/adapters/{mod.rs,sqlite/mod.rs,sqlite/actor.rs,sqlite/migrations.rs,sqlite/receipts.rs,sqlite/migrations/0001_library.sql,windows/mod.rs,windows/paths.rs,windows/lock.rs}`, `src-tauri/src/desktop/{mod.rs,lifecycle.rs,maintenance.rs}`, `src-tauri/tests/{scaffold_contracts.rs,db_actor.rs,desktop_bootstrap.rs}`, `src/shared/contracts/wire.test.ts`, `scripts/{check.ps1,generate-contracts.ps1}`, `docs/reports/task-01-report.md`.

Tras integrar T01, Sol reasume responsabilidad sobre composition roots, manifiestos/lockfiles, DTOs generados, migrator registry y capabilities y **asigna explícitamente su código a un worker** para cada integración posterior; el coordinador no escribe código de producto. Ninguna tarea posterior los modifica sin transferencia explícita. Crear carpetas de features vacías no requiere fabricar funcionalidades.

## Consume / produce

Consume: esquema lógico DATA 0001; todos los enums/envelopes/DTOs/API de CONTRACTS v1; selección de frameworks ARCHITECTURE; identidad y rutas SPEC-001. No hay código previo del que depender.

Produce:

- Proyecto Tauri 2 + React/TypeScript strict + Vite; Tailwind y componentes shadcn/Radix incorporados; RHF/Zod disponibles; PDF.js y worker empaquetables para T03. Versiones reales compatibles fijadas por lockfiles y licencias anotadas, sin inventar parches.
- `src/app/ShellView.ts`: `type ShellView = {kind:'Home'} | {kind:'Library'} | {kind:'PaperWorkspace';paperId:string} | {kind:'Knowledge';conceptId:string|null} | {kind:'Settings'}`. Solo vistas realmente disponibles activas.
- `src/shared/contracts/ports.ts`: `LibraryApi`, `ReaderApi`, `WorkflowApi`, `KnowledgeApi`, `ConceptApi`, `RelationApi`, `ProvenanceApi`, `SearchApi`, `PortabilityApi`, `SettingsApi` con firmas exactas CONTRACTS. Rust Serde camelCase + ts-rs genera DTOs; fixtures wire ejercitan round-trip y Zod valida el sobre y payload antes de salir del adaptador. Los DTOs de futuras capacidades son tipos, no implementaciones exitosas.
- `DbActor::start(root: LibraryRoot) -> Result<DbActor, AppError>` y `DbActor::submit<T: Send + 'static>(&self, job: impl FnOnce(&mut rusqlite::Connection) -> Result<T, AppError> + Send + 'static) -> impl Future<Output=Result<T,AppError>>`; una conexión poseída por un hilo, cola acotada 64, envío no bloqueante devuelve `Busy` si llena. Nunca exponer Connection a UI/transporte. Sol puede ajustar firma interna antes de congelarla manteniendo garantías y notificando consumidores.
- `with_transaction<T>(connection: &mut rusqlite::Connection, action: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<T,AppError>) -> Result<T,AppError>` en UoW: BEGIN IMMEDIATE, commit solo después de todas las escrituras. Repositorios reciben la transacción; sin commits ocultos.
- `LibraryRoot` y resolución OS en `windows/paths.rs`, bloqueo por biblioteca, estados `ready|readOnlyDiagnostic`, arranque/cierre finito, coordinator de mantenimiento compartido, ledger/migración/checksum y snapshot pre-migración mediante SQLite backup. No copiar DB con WAL como backup.
- `settings_get_app_info`, `settings_get_library_info`, `settings_get_library_status` reales; `settings_select_library`/`settings_switch_library` devuelven `UnsupportedCapability` durante piloto. Registry cerrado de CONTRACTS: ninguna API genérica filesystem/SQL/shell.
- Scripts `npm run dev`, `test` (no watch), `typecheck`, `build`, `tauri:dev`, `tauri:build`; `scripts/check.ps1` ejecuta UI y Cargo, `scripts/generate-contracts.ps1` verifica generación sin drift.

## Requisitos exactos

- `contractVersion=1`, UUID string canónico, UTC RFC3339, revisión entero desde cero. `IpcResult<T>` siempre contiene requestId, ok y data o error estructurado. Códigos exactos: `InvalidInput|NotFound|Conflict|DuplicateDecisionRequired|SourceUnreadable|InvalidPdf|StorageUnavailable|ImportRecoveryRequired|GateBlocked|UnsupportedCapability|PathNotAllowed|OperationCancelled|Busy|IntegrityFailure|SchemaTooNew|MigrationFailed|BackupFailed|ExportFailed`.
- Tipos/enums completos se copian desde CONTRACTS, no se crean variantes. `plain_text`, bodyJson null. Deserialización rechaza campos/enums desconocidos y números no finitos; no truncar entradas. Límite title 1000, answer/body 20000, snippet/quote 10000, contexto relación 5000, 100 autores, PDF 500 MiB, query 1000, excerpt 500, paginación default 100/max 500.
- Migración 0001 crea papers/authors/venues/paper_authors/documents/import_operations/reading_positions/app_session/schema_migrations/audit_events/app_settings/operation_receipts/maintenance_operations y revisiones/estado previo pertinentes; DOI único normalizado, FKs, authors ordenados, rutas relativas, states import `STAGING|PROMOTED|COMMITTED|FAILED`. `foreign_keys=ON`, WAL, synchronous FULL, busy_timeout 5000 ms. Verificar FTS5 de SQLite bundled, no del sistema.
- Receipts persistentes requestId+command+hash canónico+resultado; mismo requestId/payload devuelve resultado previo, distinto payload `Conflict`. Guardar receipt y auditoría con mutación, sin purga automática.
- `%LOCALAPPDATA%/ResearchWorkbench/library` con library.json/research.sqlite/documents/staging/recovery; backups y logs fuera de binarios. Primera pantalla confirma biblioteca y explica copia administrada. Ningún cwd hardcoded. Esquema futuro diagnóstico sin escrituras; migración fallida rollback y backup conservado.
- Tauri identidad estable `local.researchworkbench.desktop` fijada por ARCHITECTURE; ventana `main`, capability `main-local`, grupos propios `research-read`, `research-write`, `research-maintenance` derivados del registry (picker nativo dentro de comando aplicación Rust, sin permiso frontend genérico), CSP recursos empaquetados + protocolo research limitado. Sin shell/SQL/lectura genérica, red o telemetría. Logs rotados sin textos/citas/paths originales.
- NSIS x64 currentUser, WebView2 `offlineInstaller`, acceso Inicio y escritorio opcional, datos no borrados por uninstall, ejecutable release sin consola. Configuración inicial no significa instalación ya demostrada.
- No P3/P4, IA, OCR, merge, TRASHED visible, hard delete, rich text, stores globales, servidor HTTP, updater remoto, nuevos paquetes no acordados.

## Pasos y aceptación

- [ ] Antes de tocar fuentes, registrar toolchain real/dependencias/licencias y comunicar carencias a Sol. Instalar dependencias locales de proyecto solo si entorno permite; no instalar herramientas globales.
- [ ] Escribir pruebas `ipc_envelope_camel_case_roundtrip`, `unknown_enum_and_payload_rejected`, `db_actor_capacity_64_returns_busy`, `db_actor_owns_one_connection`, `receipt_retry_returns_previous_result`, `receipt_payload_mismatch_conflict`, `transaction_failure_rolls_back`, `schema_future_read_only_no_write`, `migration_failure_keeps_backup`, `data_root_independent_of_cwd`, `second_writer_rejected`, `command_outside_permission_group_rejected`, `other_window_command_rejected`, `logs_exclude_body_and_source_path` y ejecutar para documentar fallos reales previos.
- [ ] Implementar scaffold, actores/puertos, migración, DTOs/adaptador y Settings mínimos. Añadir prueba `fts5_available_in_bundled_sqlite`; `foreign_key_check` vacío y `quick_check='ok'` tras reopen. Saturación usa barrier/latch determinista, no sleeps frágiles.
- [ ] Ejecutar `npm run typecheck`, `npm run test`, `npm run build`; `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml`; generación DTO sin diff. Todos exit 0 o bloqueo exacto reportado, nunca éxito supuesto.
- [ ] Ejecutar build Tauri sin bundle y abrir ventana real cuando entorno permita; registrar proceso/versión y qué se observó. NSIS final lo verifica T10. Revisar capacidades/configuración antes de integración.
- [ ] Entregar reporte con archivos, firmas internas congeladas, comandos/exit codes, tests y límites, sin commits no autorizados. Sol revisa Rust/TS y seguridad IPC, integra y solo entonces habilita T02/T03.

**Criterio medible:** una ventana real que abre y cierra, DB temporal reabrible con mismas identidades, actor acotado, errors/DTOs exactos y checks verdes. Ausencia de entorno nativo deja evidencia pendiente, no modifica el criterio.


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

Si no existe fila, `getReadingPosition` devuelve valor por defecto determinista `pageIndex=1, zoom=1.0, revision=0, updatedAt=document.importedAt` sin crearla. Page index es entero desde uno; zoom finito y rango propuesto 0.25–5.0. El backend no conoce conteo PDF; frontend limita página, y backend valida rango estructural. Cada documento conserva posición separada. Guardados serializados por documento y optimistic revision; el cliente no interpreta respuesta antigua como estado actual. `documentUrl` solo lo emite el servicio tras verificar Document registrado, UUID, path canónico dentro de raíz; el protocolo rechaza traversal y archivos no registrados. El stream sirve solo el PDF de ese Document, no ruta arbitraria. Cancelación de render no cancela una persistencia ya confirmada.


## Módulo Workflow

`PhaseDefinitionDto` es snapshot declarativo e inmutable: `code`, `version`, `name`, `objective`, `keyQuestions`, `doItems`, `dontItems`, `considerations`, `requiredOutputs`, `completionRules`, `definitionHash`. Cada salida es `{key,label,prompt,required,allowedResolutions,ruleKey}`; no contiene scripts, expresiones evaluables ni código. `ruleKey` pertenece al enum cerrado `answerProcessed | paperHasActiveDocument | p1DecisionProcessed | p2ArtifactPresent | p2NoCandidatesJustified`. La decisión P1 procesada satisface completitud de P1 en sus tres valores; la rama `continue`/`light_read`/`archive` se aplica en `advancePhase`, no es un requisito de gate que excluya dos opciones. La definición fijada al crear processing no cambia si se instala una versión nueva; el snapshot previo se preserva en DB y export. `PhaseAnswerDto`:

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

`relevance_decision` exige `structuredValue: RelevanceDecisionValue`, validado por enums; nunca analizar `answerText` para decidir gate. `savePhaseAnswer` guarda decisión pero no archiva: archive ocurre solo en `advancePhase` desde P1. `getPhaseAnswers` hidrata respuestas/resoluciones/explicaciones/structuredValues. `evaluateGate` calcula hash del snapshot canónico de definitionHash, respuestas y artefactos requeridos, sin mutar. `advancePhase` en BEGIN IMMEDIATE vuelve a leer/recalcular gate y persiste snapshot, fase, activePhase, revision, lifecycle e historial en un commit. Desde P1 archive, completa P1 + Paper ARCHIVED + archivedFromLifecycle + audit; `nextPhase=null`. Desde P1 light_read, completa P1, Paper sigue ACTIVE, P2 permanece NOT_STARTED y `nextPhase=null`. Desde P1 continue, avanza a P2. Desde P2 gate completo, completa P2, mantiene Paper ACTIVE y `nextPhase=null`; no crea/invoca P3. Si cambió input snapshot/revisión, devuelve Conflict/GateBlocked sin efecto. `goBackToPhase` compara la revisión de la fase actualmente activa y solo cambia contexto activo; no descompleta ni borra. `touchPhase` usa esa misma revisión de fase activa, inicializa/activa una fase habilitada y nunca la completa. `setP3Candidate.expectedWorkflowRevision` es exactamente revision de `paper_phases(P2)`; mutación efectiva de candidato y de cualquier artefacto que altere snapshot de P2 incrementa esa revisión dentro de la misma transacción. Solo Claims y Questions enlazadas al Paper pueden seleccionarse como candidato. Se persisten selección, `priority` nullable (si presente integer 1..5), `rationale` (texto obligatorio 1..5000 al seleccionar) y `noCandidatesJustification`; la salida estructurada P2 se actualiza en la misma UoW. `getP3CandidateSummary` devuelve esos mismos IDs/count/justification usados por gate. Si queda cero, justification no vacío requerido; con candidatos, justification de ausencia debe ser null. Tras un cambio real que altera snapshot de fase COMPLETED, fase y fases posteriores instanciadas no NOT_STARTED pasan NEEDS_REVIEW, con datos preservados. Repetir valores canónicos es no-op. `advancePhase` reevalúa y requiere reconfirmación. Captura/edición de item, asociación de concepto, relación, provenance o candidatura P3 invalida P2 si cambia snapshot completado.

En import del piloto 0.0.1 `processingInitialized=false`: PRE/P1/P2 no están habilitadas. Al habilitar Workflow en v0.1, la importación nueva inicializa PRE `IN_PROGRESS`, P1/P2 `NOT_STARTED`, fija PhaseDefinition version y activePhase `PRE` en el mismo commit. Migrar biblioteca piloto inicializa el procesamiento de papers existentes sin cambiar UUID/metadatos. P1 solo se habilita si PRE gate pasa; P2 solo si P1 gate pasa con `readingDecision=continue`. Completar P2 pone P2 `COMPLETED`, mantiene Paper `ACTIVE`, y devuelve `nextPhase=null`; no crea ni invoca P3, que queda reservada. Paper `COMPLETED` no se asigna en v0.1.


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


## Registro de comandos Tauri

Las funciones internas se registran con prefijo de feature y este registry es cerrado en contractVersion 1: `library_select_pdf`, `library_confirm_import`, `library_cancel_import`, `library_list_papers`, `library_get_paper`, `library_update_metadata`, `library_archive_paper`, `library_restore_paper`; `reader_open_paper`, `reader_get_last_opened_paper`, `reader_get_reading_position`, `reader_save_reading_position`; `workflow_get_phase`, `workflow_get_phase_answers`, `workflow_get_phase_definition`, `workflow_save_phase_answer`, `workflow_evaluate_gate`, `workflow_advance_phase`, `workflow_go_back_to_phase`, `workflow_touch_phase`, `workflow_set_p3_candidate`, `workflow_get_p3_candidate_summary`; `knowledge_create_item`, `knowledge_update_item`, `knowledge_archive_item`, `knowledge_restore_item`, `knowledge_get_item`, `knowledge_list_items`; `concept_suggest`, `concept_get`, `concept_list_items`, `concept_create`, `concept_update`, `concept_link`, `concept_archive`, `concept_restore`; `relation_create`, `relation_update`, `relation_list`, `relation_archive`, `relation_restore`; `provenance_attach_locator`, `provenance_update_locator`, `provenance_get`, `provenance_check_document_hash`; `search_library`, `search_knowledge`; `export_choose_destination`, `export_library`, `export_paper`, `backup_choose_destination`, `backup_create`, `backup_select`, `backup_choose_restore_target`, `backup_verify`, `backup_restore`, `operation_get_status`, `operation_cancel`; `settings_get_app_info`, `settings_get_library_info`, `settings_select_library`, `settings_switch_library`, `settings_get_library_status`. Los comandos no implementados permanecen capability-gated; no se aceptan aliases libres. No se registra una API genérica de filesystem, SQL o ejecución.


