# T03 — propuesta de contratos internos

Estado: DONE_WITH_CONCERNS — propuesta para resolución/promoción por Sol, 2026-10-02. No modifica IPC v1, DTOs, esquema 0001 ni decisiones aprobadas. No es un segundo plan de implementación.

Base inspeccionada: `21fd14ed20ae391518a7dabaaece3dc551a92fa2`, exclusivamente mediante `git show`; T02 fix3 está activo y no se ha leído su WIP. Antes del dispatch hay que contrastar esta propuesta con el HEAD T02 finalmente revisado. Autoridad: INTENT, STATUS, TASK03_BOUNDARIES, brief real `docs/plans/tasks/task-03-brief.md`, TASK02_PORTS resolución 7/8, MANAGED_FILES, CONTRACTS Reader, DATA y SPEC-002. Consulta de memoria local sin coincidencias Research-Workbench; OpenViking no consultado y peer no verificado.

## Ubicación y responsabilidades

| Archivo | Responsabilidad |
|---|---|
| `src-tauri/src/application/reader_ports.rs` nuevo | ReaderPersistence, ReaderRepository, comandos y registros internos, acceso/handle lector |
| `src-tauri/src/application/reader.rs` nuevo | Helpers Reader sobre Transaction prestada: apertura, posición/default y validaciones de asociación |
| `src-tauri/src/application/request_registry.rs` nuevo | Únicamente exclusión compartida por requestId y permiso RAII |
| `src-tauri/src/application/library_ports.rs` existente | Una primitiva de actividad añadida a LibraryRepository |
| `src-tauri/src/application/library.rs` existente | Helper público Library para actividad/auditoría en la TX prestada |
| `src-tauri/src/adapters/sqlite/reader_repository.rs` nuevo | SqliteReaderPersistence con DbActor/UoW/receipt y SqliteReaderRepository con SQL |
| `src-tauri/src/adapters/sqlite/library_repository.rs` existente | Implementación SQL de la primitiva de actividad |
| `src-tauri/src/adapters/documents/store.rs` existente | LocalDocumentStore implementa también DocumentReadAccess; produce handle concreto privado |
| `src-tauri/src/modules/reader/{service.rs,protocol.rs,mod.rs}` nuevos | Coordinación de puertos/trabajos; autorización/transporte del protocolo en su frontera |
| `src-tauri/src/modules/library/service.rs`, `application/mod.rs`, composición `lib.rs`/desktop y exports pertinentes | Extraer/injectar RequestRegistry y componer Reader con el mismo actor/store/mantenimiento/RecoveryStatus |

Suprimir del ownership propuesto `modules/reader/repository.rs`: el trait vive en aplicación y su implementación SQL en adapters, igual que Library tras resolución 7. No se necesita un reexport sin consumidor. No mover los puertos Library existentes a otro archivo. La interfaz lectora se declara en reader_ports y se implementa sobre la misma instancia LocalDocumentStore, sin duplicar raíz ni autoridad.

## Firmas mínimas de aplicación

Pseudofirmas Rust de ABI fuente, no código compilado. `ReaderFuture<'a,T>` sigue exactamente la convención local `Pin<Box<dyn Future<Output=Result<T,AppError>> + Send + 'a>>`; no introduce runtime ni dependencia. UUID, DTO y errores son los tipos existentes; reutilizarlos internamente no cambia wire.

```rust
struct OpenPaperCommand { request_id: UUID, paper_id: UUID }
struct SaveReadingPositionCommand {
    request_id: UUID, document_id: UUID,
    expected_revision: i64, page_index: i64, zoom: f64,
}

struct RegisteredDocument {
    library_id: UUID,
    document: DocumentDto, // id, paperId, status, sha256, importedAt, filename
    relative_path: String, // solo persistencia; namespace administrado exacto
    size_bytes: i64,
}

trait ReaderPersistence: Send + Sync {
    fn writable(&self) -> bool;
    fn prepare_open(&self, command: OpenPaperCommand)
        -> ReaderFuture<'static, RegisteredDocument>;
    fn confirm_open(&self, command: OpenPaperCommand, verified: VerifiedDocument)
        -> ReaderFuture<'static, OpenPaperDto>;
    fn last_opened_paper(&self) -> ReaderFuture<'static, Option<PaperDto>>;
    fn registered_document(&self, document_id: UUID)
        -> ReaderFuture<'static, RegisteredDocument>;
    fn reading_position(&self, document_id: UUID)
        -> ReaderFuture<'static, ReadingPositionDto>;
    fn save_reading_position(&self, command: SaveReadingPositionCommand)
        -> ReaderFuture<'static, ReadingPositionDto>;
}

type VerifiedDocument = Box<dyn DocumentReadHandle + Send>;
trait DocumentReadAccess: Send + Sync {
    fn open_verified(&self, registered: RegisteredDocument)
        -> ReaderFuture<'static, VerifiedDocument>;
}
trait DocumentReadHandle: Send {
    fn registered(&self) -> &RegisteredDocument;
    fn read_all(self: Box<Self>) -> ReaderFuture<'static, DocumentBody>;
}
struct DocumentBody {
    bytes: Vec<u8>,
    lease: VerifiedDocument,
}
```

`RegisteredDocument` es un snapshot de la asociación de DB, no prueba de acceso. No se serializa ni se acepta desde IPC. library_id procede del actor activo; document contiene los IDs y estado reales, y relative_path solo puede designar `documents/<documentId>/original.pdf`. La construcción y validación se limitan a persistencia y adaptador; nunca una ruta arbitraria del caller.

El handle concreto conserva RegisteredDocument, ManagedFile y sus padres/pins de T02. Su API no entrega File/PathBuf/raw handle ni permite promoción o borrado. `open_verified` comprueba biblioteca, IDs, namespace, estado/asociación autorizados, raíz/identidad/reparse y tamaño usando la infraestructura T02; cualquier lectura/hash que haga ocurre fuera del actor/TX y sobre ese handle. No se prescribe aquí una política de rehash nueva. Ausencia o discrepancia produce el error seguro existente.

Se usa propiedad `Box + Send`, sin exigir Clone/Sync a las guardas Windows ni préstamos a través del actor. `read_all` mueve el handle al trabajo bloqueante; devuelve bytes y la misma lease para conservar las guardas hasta entregar la respuesta del protocolo. La implementación maneja posición del cursor internamente, verifica longitud y límite existentes; no vuelve a abrir por nombre. Es la lectura del handle ya abierto, no otro servicio de archivos.

## Repositorios y helpers transaccionales

```rust
trait ReaderRepository {
    fn registered_document(&self, tx: &Transaction<'_>, document_id: &str)
        -> Result<RegisteredDocument, AppError>;
    fn position(&self, tx: &Transaction<'_>, document_id: &str)
        -> Result<Option<ReadingPositionDto>, AppError>;
    fn write_position(&self, tx: &Transaction<'_>, expected_revision: i64,
                      next: &ReadingPositionDto) -> Result<usize, AppError>;
    fn last_opened_paper_id(&self, tx: &Transaction<'_>)
        -> Result<Option<UUID>, AppError>;
    fn set_last_opened_paper(&self, tx: &Transaction<'_>, paper_id: &str)
        -> Result<(), AppError>;
    fn audit_position(&self, tx: &Transaction<'_>, request_id: &str,
                      position: &ReadingPositionDto) -> Result<(), AppError>;
}

// Adición a LibraryRepository, no un segundo repositorio bibliográfico:
fn set_last_opened_at(&self, tx: &Transaction<'_>, paper_id: &str, now: &str)
    -> Result<usize, AppError>;

// application/library.rs; reutiliza LibraryRepository.paper y audit_change.
fn record_open_activity_in_tx(tx: &Transaction<'_>, repo: &dyn LibraryRepository,
    request_id: &str, paper_id: &str, now: &str) -> Result<PaperDto, AppError>;

// application/reader.rs; nunca abre/commitea TX ni toca filesystem.
fn open_paper_in_tx(tx: &Transaction<'_>, reader: &dyn ReaderRepository,
    library: &dyn LibraryRepository, command: &OpenPaperCommand,
    verified: &RegisteredDocument, now: &str) -> Result<OpenPaperDto, AppError>;
fn reading_position_in_tx(tx: &Transaction<'_>, reader: &dyn ReaderRepository,
    document_id: &str) -> Result<ReadingPositionDto, AppError>;
fn save_reading_position_in_tx(tx: &Transaction<'_>, reader: &dyn ReaderRepository,
    command: &SaveReadingPositionCommand, now: &str)
    -> Result<ReadingPositionDto, AppError>;
```

El adaptador fija library_id del repositorio desde el mismo actor; no lo toma de parámetros de UI. La lectura del Paper usa `LibraryRepository.paper`, también para comprobar que paper.documentId apunta al documento presentado. El helper de apertura vuelve a obtener la asociación, biblioteca, referencia, estado, hash y tamaño actuales y los compara con el snapshot verificado antes de escribir. Un snapshot obsoleto no autoriza otro recurso.

El helper Library cambia solo last_opened_at y emite auditoría de entidad con requestId dentro de la TX prestada; devuelve PaperDto actualizado. No incrementa revision, no toca updated_at, metadatos, lifecycle ni current_phase; un archivado sigue archivado. Reader escribe app_session.last_opened_paper_id en la misma TX. La fase contextual se obtiene del Paper devuelto (`activePhaseCode` / papers.current_phase); no hay columna de fase en app_session.

`write_position` devuelve filas afectadas y no decide reglas: el helper valida pageIndex/zoom, resuelve default, compara expectedRevision, calcula la revisión siguiente y audita. Para ausencia, expectedRevision=0 permite primera inserción con revision=1; una fila revision=0 existente se trata como fila existente. Conflicto conserva el resultado durable previo y devuelve currentRevision. La lectura ausente da página 1, zoom 1, revisión 0 y document.importedAt, sin insertar ni auditar.

## Apertura, replay y propiedad del trabajo

1. ReaderService valida entrada, obtiene RequestPermit compartido y OperationPermit de mantenimiento, y transfiere ambos al trabajo admitido propietario, conforme al patrón T02. RecoveryStatus y writable se comprueban antes de mutar. Descartar el receptor IPC no cancela ese trabajo.
2. `prepare_open` comprueba primero command/payload/receipt: petición incompatible da Conflict antes de abrir archivo. Para apertura nueva obtiene el documento activo del Paper; para replay usa el documentId del resultado durable y exige su autorización/asociación actual. No sustituye el documento del receipt por otro actual. Esta fase no escribe actividad ni receipt.
3. Fuera de TX, `open_verified` adquiere la lease. El trabajo la mueve a `confirm_open`, que la captura por valor en `DbActor.submit` y la mantiene hasta que termine su commit/rollback. No basta retenerla en el future que espera al actor: el actor ejecuta aunque desaparezca ese future.
4. `confirm_open` usa with_receipt existente. Para resultado nuevo, el helper revalida asociación y confirma actividad Library, contexto, posición consultada y receipt atómicamente. La URL se deriva solo del UUID autorizado según la forma Windows ya fijada; no contiene ruta física.
5. En replay, with_receipt devuelve el OpenPaperDto original sin ejecutar el helper ni duplicar auditoría. Se revalida además el registro actual contra la lease antes de devolver la URL. Esa lectura cabe en el mismo job actor mediante una TX breve separada, sin anidar TX en with_receipt ni añadir un executor/receipt framework. No hay otro job actor intercalado. El archivo ya está abierto y protegido durante todo el job; no hay I/O de archivo dentro de ninguna TX.
6. Si falla el acceso posterior, se devuelve error y se conserva intacto el receipt exitoso original. Ni ese error ni un nuevo timestamp reemplazan el resultado durable. Un replay posterior vuelve a verificar acceso y devuelve el mismo resultado original cuando procede.

El protocolo obtiene el RegisteredDocument actual mediante ReaderPersistence después de sus validaciones de URI/ventana/origen/método; abre su propia lease con el mismo store. La URL no es una reserva permanente de un handle ni una autorización que omita esas comprobaciones. Su trabajo conserva OperationPermit hasta terminar lectura/respuesta, incluso si desaparece fetch/UI; no necesita RequestPermit porque no crea receipts ni recibe requestId. Perfil aprobado: bytes completos, 200, longitud exacta, sin anunciar Range. No se crean caches de handles, tokens de sesión nuevos ni canales de cancelación.

## RequestRegistry compartido

```rust
#[derive(Clone, Default)]
struct RequestRegistry { /* Arc<Mutex<HashMap<String, Weak<tokio::Mutex<()>>>>> */ }
struct RequestPermit { /* OwnedMutexGuard<()> */ }
impl RequestRegistry {
    async fn acquire(&self, request_id: &UUID) -> Result<RequestPermit, AppError>;
}
```

Extraer únicamente `requests_inflight`, `lock_request` y RequestPermit de LibraryService en 21fd14e, manteniendo el algoritmo de weak/pruning y la adquisición atómica de la entrada bajo el mutex del mapa. La composición crea una instancia por biblioteca activa, la inyecta obligatoriamente en LibraryService y ReaderService y comparte clones de esa instancia. No conservar un constructor Library que cree silenciosamente su registro privado. El registro no guarda payloads, no ejecuta comandos ni conoce receipts; persistencia sigue validando command/hash durable.

Orden: requestId antes de permisos/token y antes de filesystem; exclusión token sigue privada de Library. Los permisos request/maintenance se mantienen hasta resultado terminal del trabajo admitido, incluidos actor o filesystem aún activos. Lecturas admitidas del protocolo retienen mantenimiento en su propietario. No hace falta una abstracción general de ejecución para preservar este patrón.

## Verificación y límites de esta entrega

Comprobaciones que debe cubrir T03: actividad/receipt atómicos sin cambiar revisión bibliográfica; default sin insert; replay sin actividad y con archivo ausente/registro cambiado; lease retenida hasta commit; caller descartado durante lectura/commit mantiene mantenimiento ocupado; saga Library pausada y Reader con mismo requestId incompatible no toca segundo recurso ni se apropia del receipt; lectura real por handle protegido con 200/longitud correcta. Son las regresiones existentes aplicadas a estas firmas, no resultados ya obtenidos.

Se inspeccionaron con `git show 21fd14e:<path>` library_ports, library helpers, LibraryService, actor.submit, receipts, ambos repositorios/Store Windows pertinentes, maintenance, composición y migración 0001. Se leyeron documentos locales de autoridad y las skills Karpathy/writing-plans. No se ejecutaron tests ni se editó producto; no hay commit de esta propuesta. `git status --short` inicial ya mostraba AGENTS.md modificado por otro trabajo; se conserva intacto. Único archivo escrito por este agente: este informe.

Autorrevisión: seis operaciones Reader más writable, seis primitivas ReaderRepository, un helper/primitiva Library; todos los métodos tienen consumidor T03. Un solo actor, store y registro de requestId; sin DB/filesystem en UI, sin TX durante filesystem, sin schema/IPC nuevo. Pendiente únicamente decisión/promoción por Sol y contraste con T02 final. Las pseudofirmas no afirman compilación ni comportamiento nativo probado.
