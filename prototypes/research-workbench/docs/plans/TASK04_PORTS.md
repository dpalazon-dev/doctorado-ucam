# T04 — puertos internos y composición

Estado: **Accepted por Sol**, ADR-018, 2026-10-03. Corte: integración `f2d39a25feb81141b51ca6cf028777b38503818b`, según el preflight. No implementa producto ni cambia el wire v1. El brief T04 y WORKFLOW_GATES/ADR-017 gobiernan la semántica. Las firmas siguientes fijan nuevos puertos internos, salvo donde se indica API existente. La implementación aún está pendiente.

## 1. Migración SQL-only, seed y backfill atómicos

**Decisión:** conservar `Migration { version, sql }` y el runner existente. `0002_workflow.sql` crea tablas/constraints, inserta las tres definiciones v1 y hace el backfill de todos los Papers piloto dentro de la transacción que ya abre `migrate`. Registrar 0002 y subir `SCHEMA_VERSION=2`; no alterar 0001.

- Copiar literalmente los tres payloads canónicos aprobados al seed SQL, escapados como literales SQL, con su `definitionHash`. Las copias son datos de migración inmutables. `definitions.rs` embebe los JSON canónicos; una prueba comprueba igualdad completa seed/JSON y recalcula el hash. SQLite no necesita calcular SHA-256 ni ejecutar reglas.
- Por Paper con `processing_initialized=0`: insertar PRE IN_PROGRESS/revision1 y P1/P2 NOT_STARTED/revision0, fijar versión1 en las tres; `completed_at` y snapshots NULL; respuestas vacías. Actualizar únicamente `processing_initialized=1` y `current_phase='PRE'` en Paper. Preservar UUID, documento, metadata, lifecycle, archivedFrom, revision y updatedAt bibliográficos, también para ARCHIVED y COMPLETED reservado.
- Un conflicto estructural inesperado aborta toda la migración; no usar `INSERT OR REPLACE` ni ignorar incoherencias. Backup previo, ledger/checksum, user_version e integridad continúan bajo las garantías del runner actual. Ningún seed tardío al arrancar servicios.
- Import nuevo llama al helper siguiente dentro de `confirm_in_tx`, tras insertar Paper/Document y antes de leer/serializar el Paper para `mark_import_committed` y receipt. El recovery ya llega al mismo helper de confirmación. Replay y reuseExisting no reinicializan.

Firmas fijadas en `application/workflow.rs`, consumiendo `WorkflowRepository` de `application/workflow_ports.rs` (TX prestada, sin actor/receipt/commit propio):

```rust
pub fn initialize_processing(
    tx: &Transaction<'_>, repository: &dyn WorkflowRepository, paper_id: &str,
) -> Result<(), AppError>;

pub fn invalidate_effective_change(
    tx: &Transaction<'_>, repository: &dyn WorkflowRepository, paper_id: &str,
    input_phases: &[PhaseCode],
) -> Result<(), AppError>;
```

`initialize_processing` exige Paper existente y no inicializado, inserta las tres filas/pins y actualiza los dos campos de contexto mediante el repositorio. Ya inicializado sólo admite no-op si sus filas/pins son coherentes; nunca repara ni resetea silenciosamente. No necesita tiempo: la inicialización no cambia timestamps bibliográficos y no crea respuestas/completions. SQL reside exclusivamente en `adapters/sqlite/workflow_repository.rs`; dominio/gates son puros, independientes de SQLite/Tauri, y servicio no contiene SQL. `modules/workflow/invalidation.rs`, si se conserva del brief, no ofrece un segundo helper con TX ni duplica la implementación.

Ampliación interna aceptada por Sol durante esta preparación: añadir `workflow: &dyn WorkflowRepository` inmediatamente después de `repository: &dyn LibraryRepository` en `application/library.rs::{confirm_in_tx,update_metadata_in_tx}`. Sus restantes argumentos/retornos se conservan. `adapters/sqlite/library_repository.rs` pasa la única implementación `SqliteWorkflowRepository` al delegar en ambos helpers; importar/recovery siguen la misma confirmación. No añadir métodos puente Workflow a LibraryRepository ni una segunda implementación.

`invalidate_effective_change` se llama una sola vez por Paper/UoW con la unión de fases cuyos inputs cambiaron realmente. Deduplica y expande a posteriores ya iniciadas, distinguiendo inputs directos de dependencias posteriores. Una fase directamente modificada COMPLETED pasa a NEEDS_REVIEW; si está IN_PROGRESS conserva ese estado salvo que también sea posterior de otra fase cuyos inputs cambiaron. Toda fase posterior ya iniciada pasa a NEEDS_REVIEW, incluso si estaba IN_PROGRESS. NOT_STARTED permanece intacta. Conserva datos, completedAt y snapshots históricos. Esta distinción aplica también a la unión de varias fases de entrada: ser input directo no exime la invalidación por una entrada anterior. Renueva clock de cada fase afectada una sola vez, en orden PRE/P1/P2 desde max+1; NOT_STARTED queda intacta. El llamador valida cambio/no-op y coordina otras modificaciones de estado/contexto de esa UoW para no asignar dos tokens a una fase. T05/T06/T07 usarán la misma interfaz, sin receipts anidados. Library envía `[PRE]` sólo si cambian efectivamente título o reviewType normalizados; un cambio DB de documento vigente afecta `[PRE,P1,P2]` respetando NOT_STARTED.

**Coste/alternativas:** se duplica el seed como artefacto histórico y la lógica de inicialización SQL del upgrade/import; pruebas de paridad hacen explícito ese coste. Añadir callbacks al runner permitiría compartir Rust, pero ampliaría el mecanismo de migración y su identidad/checksum. Inicializar después del upgrade rompe atomicidad. Para tres definiciones y una migración, SQL autocontenido es la opción menor.

## 2. Persistence, proof, replay y propiedad del trabajo

**Decisión:** un `WorkflowService`, un adaptador SQLite y un puerto interno `application/workflow_ports.rs`, siguiendo el patrón Reader. Reutilizar DbActor, MaintenanceCoordinator, RequestRegistry y los handles de documentos T03. Workflow consume el puerto tipado adicional DocumentProofAccess definido abajo; DocumentReadAccess del lector conserva su firma y comportamiento. No crear otro actor ni scheduler.

Tipos internos mínimos:

```rust
struct WorkflowDocumentReference {
    paper_id: String,
    active_document_id: String,
    registered: RegisteredDocument, // API T03 existente
}
enum WorkflowDocumentProof {
    Available { reference: WorkflowDocumentReference, handle: VerifiedDocument },
    Unavailable { reference: WorkflowDocumentReference },
}
struct WorkflowAdmission {
    operation: OperationPermit,
    request: RequestPermit,
}
enum PreparedAdvance {
    Replay(WorkflowAdvancePhaseOutput),
    NeedsProof(WorkflowDocumentReference),
}
```

El repositorio Workflow consulta la referencia vigente sin filtro `status='ACTIVE'`. Conserva identidad, estado, hash, ruta administrada, tamaño y biblioteca para revalidación; sólo `{id,status,sha256,available}` pasa al snapshot. Archivo ausente/inaccesible con registro válido es Unavailable; registro Document ausente, ajeno al Paper o asociación rota es IntegrityFailure, porque Paper.documentId es obligatorio. Un estado MISSING/SUPERSEDED bien formado permite snapshot con available=false, sin intentar abrirlo como ACTIVE. No fabricar IDs/hashes/estados para completar un registro corrupto.

Firmas mínimas del puerto; `WorkflowFuture<'a,T>` tiene la misma forma boxed Send/Result que ReaderFuture. Se reutilizan los Args/Output DTO existentes; no hay tipos wire nuevos:

```rust
fn prepare_advance(&self, args: WorkflowAdvancePhaseArgs)
    -> WorkflowFuture<'static, PreparedAdvance>;
fn advance(&self, args: WorkflowAdvancePhaseArgs,
    proof: WorkflowDocumentProof, admission: WorkflowAdmission)
    -> WorkflowFuture<'static, WorkflowAdvancePhaseOutput>;
fn document_reference(&self, paper_id: String)
    -> WorkflowFuture<'static, WorkflowDocumentReference>;
fn evaluate(&self, args: WorkflowEvaluateGateArgs,
    proof: WorkflowDocumentProof, admission: WorkflowAdmission)
    -> WorkflowFuture<'static, GateEvaluationDto>;
```

`prepare_advance` hace lookup durable antes de cargar referencia o comprobar CAS/lifecycle/capacidad actual. Para `touchPhase`, que puede habilitar hacia delante, usar el mismo orden mediante `prepare_touch`/`touch` y un `PreparedTouch` análogo con `PhaseDto`; no forzar un framework genérico de comandos. save/goBack, que no necesitan proof para su operación, entran directamente en `with_receipt` y hacen replay antes CAS. Validación de forma/identidad del request y admisión siguen ocurriendo antes del lookup.

Extraer del código actual de receipts un lookup de sólo lectura compartido por prepare y `with_receipt`, con una única normalización/hash/verificación de comando y deserialización:

```rust
pub(crate) fn lookup_receipt<T: DeserializeOwned>(
    c: &Connection, request_id: &str, command: &str, payload: &Value,
) -> Result<Option<T>, AppError>;
```

El `Value` representa exactamente el mismo payload que recibe `with_receipt`. Mismo requestId con comando/hash distinto da Conflict incluso si el PDF no está disponible. No añadir tablas ni escribir receipts durante prepare. El lookup no sustituye la comprobación final dentro de `with_receipt`.

### ABI tipada de acceso documental: cerrar la pérdida de causa

**Hallazgo comprobado en código:** `LocalDocumentStore::open_verified` recibe sólo AppError de `managed_file`. Los mappings de `managed_files.rs` líneas580/700/740 conservan NotFound, pero convierten denegación de acceso y otros errores en StorageUnavailable; el join/panic de spawn_blocking también produce StorageUnavailable. Por tanto, Workflow no puede recuperar la causa desde el puerto actual ni deducirla por código/mensaje/details.

Añadir a `application/reader_ports.rs` estas abstracciones de documentos compartidas. No cambiar `DocumentReadAccess`, `DocumentReadHandle`, `VerifiedDocument`, DTOs ni AppError:

```rust
pub enum DocumentUnavailableReason { Missing, AccessDenied, SharingViolation }
pub enum DocumentAccessOutcome {
    Available(VerifiedDocument),
    Unavailable(DocumentUnavailableReason),
}
pub trait DocumentProofAccess: Send + Sync {
    fn prove_access(&self, registered: RegisteredDocument)
        -> ReaderFuture<'static, DocumentAccessOutcome>;
}
```

`LocalDocumentStore` implementa ambos traits sobre **una sola apertura verificada interna**. Esa rutina valida RegisteredDocument, adquiere el mismo ManagedFile/pins y comprueba tamaño. `prove_access` conserva el resultado tipado; `open_verified` adapta Available a su handle y Unavailable a los errores legacy (Missing→NotFound; AccessDenied/SharingViolation→StorageUnavailable). Los errores fatales AppError se propagan sin cambio en ambos. Join/panic ocurre fuera del resultado de apertura y sigue siendo `Err(StorageUnavailable)`, nunca `Ok(Unavailable(...))`. No añadir un default de trait que intente inferir la causa desde open_verified.

La causa debe conservarse donde todavía existe: `adapters/windows/managed_files.rs` introduce un error interno de apertura `ManagedOpenFailure::{Missing,AccessDenied,SharingViolation,Fatal(AppError)}`. Añadir variantes internas detalladas de `open_relative_directory`/`open_relative_file`; los wrappers actuales mantienen firmas AppError y mapean exactamente como hoy. Un único núcleo de `ManagedDirectory::open_child` y `open_file` conserva todas las validaciones/pins y ofrece al store una ruta `open_child_for_proof(name)` / `open_file_for_proof(name)` con el error tipado. Los métodos legacy delegan en el núcleo común: no duplicar la secuencia de adquisición o las guardas.

- En el resultado fallido de **NtCreateFile**, exclusivamente: STATUS_OBJECT_NAME_NOT_FOUND/STATUS_OBJECT_PATH_NOT_FOUND→Missing; STATUS_ACCESS_DENIED→AccessDenied; STATUS_SHARING_VIOLATION→SharingViolation. STATUS_OBJECT_NAME_COLLISION conserva Fatal(Conflict); demás estados conservan Fatal(StorageUnavailable), salvo guardas que ya produzcan otro AppError. No equiparar todos los NTSTATUS a indisponibilidad.
- En fallback no Windows, exclusivamente errores de apertura `std::io::ErrorKind::NotFound`/`PermissionDenied`→Missing/AccessDenied; AlreadyExists conserva Fatal(Conflict), demás Fatal(StorageUnavailable). No usar exists() como prueba ni reabrir por ruta absoluta para recuperar el error.
- Errores de validación, root binding, adquisición de identidad raíz, locks envenenados, metadata/final_path, regularidad, padre, descendencia, reparse o comparación de tamaño son **Fatal** con su AppError existente. No reclasificar un NotFound/StorageUnavailable que ya venga de esas guardas. Missing/AccessDenied/SharingViolation sólo nacen en la apertura OS detallada bajo la raíz retenida; el recorrido conserva todas las guardas T03 disponibles y sus pins. Una apertura que falla no acredita verificaciones del handle que no llegó a adquirir; sólo comunica indisponibilidad, jamás pertenencia o acceso válido.

En store, `managed_file_for_proof` sigue exactamente el recorrido administrado actual y traduce el error interno una sola vez al resultado de aplicación. `WorkflowDocumentProof::Unavailable` conserva siempre la referencia DB original, aunque el motivo de indisponibilidad no se serializa en snapshot/wire. No hay rutas, NTSTATUS ni errores OS en los DTOs. El servicio Workflow recibe `Arc<dyn DocumentProofAccess>` del mismo LocalDocumentStore de composición.

Secuencia obligatoria para advance:

1. Admitir con permisos existentes; moverlos a un trabajo propio con spawn/oneshot, como Reader. Cancelar el future IPC sólo abandona el receptor; el trabajo admitido conserva permisos durante prepare/proof y continúa hasta resultado durable o rollback.
2. `prepare_advance`: en actor, replay temprano o referencia DB. Replay devuelve el resultado confirmado sin volver a exigir PDF/CAS, aunque después cambiasen inputs o se archivase el Paper. Los permisos se conservan hasta terminar también este recorrido.
3. Llamar a `DocumentProofAccess::prove_access` **fuera** del actor. Con ACTIVE y Available, comparar `handle.registered()` con la referencia y mantener el handle. Sólo `Ok(Unavailable(...))` produce available=false; todo `Err(AppError)` se propaga, también NotFound/StorageUnavailable. No `read_all`, no hash masivo. La nueva ABI conserva la causa OS antes de que se pierda; no se infiere desde el puerto legacy. MISSING/SUPERSEDED registrado y coherente usa Unavailable sin abrir, conforme a la regla anterior.
4. Mover proof y ambos permisos a la closure final del actor. La closure los mantiene en variables propietarias **alrededor de toda la llamada `with_receipt`**, no sólo dentro de su `action`: deben seguir vivos cuando se ejecuta commit o rollback. Repetir lookup dentro de `with_receipt` antes CAS; si hay replay, no reevaluar.
5. Para ejecución nueva: revalidar dentro de la TX la referencia completa Paper/Document, tanto Available como Unavailable. Si cambió: Conflict sin usar una prueba anterior ni reinterpretar el resultado como PDF inaccesible. Referencia corrupta: IntegrityFailure. Luego CAS activo, guardas de lifecycle/capacidad, cadena aceptada vigente y gate. Unavailable bloquea con GateBlocked; no cambia el snapshot aceptado, clock, lifecycle ni crea receipt exitoso.
6. Si pasa, confirmar snapshot/state/contexto/clock/NEW→ACTIVE o archive/audit/receipt conjuntamente. Para P1 archive usar API existente `archive_paper_in_tx` con Transaction prestada. Soltar handle/permisos después del commit/rollback; responder por oneshot. Un replay nunca repite archive ni navegación.

`evaluate` usa referencia+proof y revalidación iguales, dentro de una lectura consistente explícita DEFERRED del actor, sin `with_receipt`, audit ni escrituras. Unavailable devuelve `GateEvaluationDto.complete=false` y snapshot/hash actual con available=false. La aceptación histórica queda intacta. No usar `with_transaction` actual si eso introduce su IMMEDIATE de escritura por defecto. Las consultas simples de definiciones/fases/respuestas tampoco necesitan proof ni mutan contexto.

`touch` sólo necesita proof si habilita/activa hacia delante; comprueba bajo TX la dirección real frente al contexto vigente y la cadena aceptada, no una suposición UI. Puede preparar proof conservadoramente antes; un fallo de disponibilidad no debe bloquear una navegación anterior que bajo TX no requiera esa cadena. `goBack` no inicia NOT_STARTED ni acepta prerequisitos. Las guardas backend de archivados siguen obligatorias para todas las mutaciones.

**Coste/alternativas:** hay una lectura corta adicional para replay/referencia. Evita que un PDF perdido o una respuesta IPC cancelada invalide un éxito durable. Introducir un recibo IN_PROGRESS, reservar requests en DB o mantener una TX mientras se abre el archivo añade estados/locks innecesarios; el RequestRegistry compartido y el receipt final ya cubren exclusión y reintento.

## 3. Entrada UI independiente del lector

**Decisión:** botón «Procesamiento» en cada tarjeta de `LibraryPage`, incluyendo filtro ARCHIVED. Nueva prop interna `onOpenWorkflow: (paperId: string) => void`. Invalida la intención de apertura Reader pendiente antes de navegar; así una respuesta tardía de openPaper no cambia la vista elegida. No invoca Reader, no comprueba PDF, no hace touch al abrir.

Ampliar internamente `ShellView.PaperWorkspace` con `section: 'reader' | 'workflow'`. App compone `PhaseWorkspace` mediante paperId/api.workflow y lectura `api.library.getPaper`; `opened: OpenPaperDto` sólo condiciona la sección reader. Una navegación interna a workflow no requiere `opened`. «Volver a biblioteca» funciona también tras fallo del PDF. Sin router nuevo ni modificación de CONTRACTS.

PhaseWorkspace hidrata Paper/fase/definición/respuestas por ID con protección ante respuestas tardías y borradores separados por paper/fase/key. Usar getPaper actual, no confiar permanentemente en la fila de la lista. ARCHIVED muestra datos y aviso con retorno a Biblioteca para restaurar explícitamente; edición/advance/touch/goBack quedan deshabilitados. Consultar una fase con getPhase/getPhaseAnswers no cambia activePhase. PDF inaccesible permite leer/guardar borradores cuando lifecycle lo permite, y el gate comunica el bloqueo al evaluar/avanzar. P2 lee definición/datos y muestra capacidad pendiente; P3/P4 sin controles activos.

**Coste/alternativas:** añade un callback y un discriminante local. Montar workflow únicamente después de openPaper excluye archivados/inaccesibles; inventar un segundo listado o forzar restauración para consultar duplica UI y cambia semántica. No se necesita tocar Reader ni PaperDetails para la entrada mínima.

## 4. Ownership y evidencia del despacho

Recomendación: **un único Luna T04** posee los archivos originales del brief y las transferencias acotadas del preflight. Sol conserva decisiones, revisión y merge. El worker no crea subagentes, no hace merges ni modifica configuración global; conserva cambios ajenos.

Además de la tabla completa del preflight, transferir explícitamente:

- `src-tauri/src/application/{workflow.rs,workflow_ports.rs}` nuevos y declaraciones en `application/mod.rs`: helpers de aplicación, repositorio transaccional, persistence y tipos internos de esta propuesta. `WorkflowRepository` recibe TX prestada en sus primitivas de lectura/escritura; nunca abre actor ni confirma. El dominio sólo recibe valores/modelos puros.
- `src-tauri/src/adapters/sqlite/receipts.rs`: extracción puntual de lookup compartido, preservando semántica de todos los consumidores; pruebas de regresión de receipts.
- `src-tauri/src/application/reader_ports.rs`: únicamente DocumentProofAccess y sus dos enums nuevos, conservando API Reader existente. `src-tauri/src/adapters/documents/store.rs`: núcleo único de apertura verificada, adaptadores Reader/proof y pruebas unitarias propias. `src-tauri/src/adapters/windows/managed_files.rs`: clasificación tipada en apertura OS, núcleos/wrappers descritos y pruebas unitarias de mappings/guardas; sin cambiar share flags, permisos pedidos, pins ni política de creación. `src-tauri/tests/{reader_integration.rs,document_protocol.rs}`: sólo regresión de apertura/propiedad/error legacy si la cobertura requiere ampliación. `lib.rs` ya transferido conecta el mismo store al nuevo trait; ningún otro cambio del lector.
- `src/features/library/LibraryPage.tsx`, `src/features/library/library.test.tsx`: sólo botón/callback/invalidación de openIntent y pruebas asociadas. `ShellView.ts`, App y App.test ya quedan transferidos para composición.
- `application/library.rs`: únicamente inicialización en confirm_in_tx e invalidación de inputs PRE tras cambio real, con los parámetros prestados indicados. `adapters/sqlite/library_repository.rs`: adaptar los callsites de `confirm_in_tx` (en commit_confirmation, línea457 del corte) y `update_metadata_in_tx` (línea1209), inyectando SqliteWorkflowRepository; no ampliar LibraryRepository con puentes. Transferir `src-tauri/tests/library_integration.rs`: las llamadas directas a update_metadata_in_tx de líneas3236/3256 requieren el nuevo argumento; mantener aserciones previas y añadir las de inicialización/invalidation. Estos son los callsites encontrados con `rg` en el corte; volver a comprobar por ambos símbolos antes de modificar, sin refactor ajeno.

Antes del merge, pruebas de comportamiento deben demostrar, además del brief:

1. Upgrade 0001→0002 con NEW/ARCHIVED/COMPLETED: preservación bibliográfica, paridad import/backfill y seed/hash; fallo al final revierte tablas/seed/backfill/ledger/user_version y conserva backup; reopen sin duplicados. Adaptar el test de migración futura que usa versión2 fija.
2. Replay después de éxito con PDF inaccesible y CAS/lifecycle posteriores: devuelve resultado exacto; payload/comando distinto Conflict. No exige proof ni repite efectos.
3. Cancelar IPC durante proof y con trabajo ya encolado conserva permisos; maintenance y request concurrente siguen excluidos hasta commit/rollback; reintento recupera resultado. Verificar lifetime del handle hasta después del commit.
4. MISSING/inaccesible válido produce snapshot false; ownership/ruta/registro corrupto produce error seguro. Cambio de referencia intercalado con proof exitoso **y fallido** produce Conflict sin efectos. Recuperación de acceso reevalúa sin clock ficticio.
5. «Procesamiento» en activo/archivado con reader.openPaper fallando abre e hidrata sin llamada Reader ni touch; consulta archivada sin controles de mutación; apertura Reader tardía no sustituye workflow; cambio de Paper no cruza borradores/respuestas.
6. ABI proof: tests deterministas del mapper para Missing/AccessDenied/SharingViolation frente a un NTSTATUS genérico; pruebas del store para ausencia real, denegación inyectada en el límite de apertura y fallo genérico/panic inyectado. Los tres primeros dan `Ok(Unavailable(reason))`; genérico/panic dan `Err(StorageUnavailable)`. UUID/biblioteca/ruta/reparse rechazados siguen Err(PathNotAllowed) cuando ésa sea la guarda que falla; tamaño divergente Err(IntegrityFailure). File ausente y directorio intermedio ausente prueban ambos recorridos. Guard failures artificiales con NotFound/StorageUnavailable prueban que no se degradan. Reader conserva los códigos legacy para idénticas causas; éxito produce el mismo handle retenido. El test nativo de archivo bloqueado verifica SharingViolation/guardas sobre NTFS; una denegación inyectada no se presenta como prueba ACL real. Workflow acepta false sólo desde el enum, y revalida la referencia también para denegación/acceso compartido fallidos.

Sol acepta estas decisiones después de contrastar el preflight, la propuesta del arquitecto y la pérdida real de causas IO en el código integrado. Se mantienen wire v1, migración atómica, replay antes CAS, handle/permisos hasta commit y entrada UI sin dependencia de PDF. Esta aceptación documental no demuestra implementación ni pruebas. El brief operativo concreta el ownership y el corte Git antes del despacho.

## Casos de uso de aplicación: precisión ADR-021

[TASK04_APPLICATION_CASES](TASK04_APPLICATION_CASES.md) fija las cuatro primitivas adicionales de WorkflowRepository, los cinco casos TX de aplicación y el plan puro mínimo. Sol contrastó LibraryRepository::paper/update_lifecycle/audit_change y archive_paper_in_tx en a051d88. No se cambian WorkflowPersistence, IPC, DTOs, esquema ni guards de propiedad documental. Los adaptadores conservan actor/receipts/proof/admission hasta commit/rollback; application decide CAS/pin/gate/contexto/lifecycle y eventos de cambio efectivo. Este anexo gobierna la corrección S3/S5/S9 de T04b.
