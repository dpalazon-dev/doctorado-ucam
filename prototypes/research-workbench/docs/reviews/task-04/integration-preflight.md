# T04 — preflight de integración y puertos existentes

Fecha: 2026-10-03. Investigación de `/root/task04_integration_ports`, sólo lectura de producto. Corte comprobado: `.worktrees/integration`, HEAD `f2d39a25feb81141b51ca6cf028777b38503818b`; `git status --short` vacío. Las rutas de producto siguientes son relativas a ese worktree. Este informe no implementa T04 ni acredita pruebas funcionales.

## Resultado para el despacho

El brief enumera los archivos nuevos de Workflow pero reserva a Sol varios archivos compartidos imprescindibles. El anexo del próximo worker debe transferir explícitamente los cambios acotados de esta lista; sin esa transferencia no puede entregar importación, upgrade, IPC y UI integrados. No es necesario cambiar el wire v1 ni ampliar los permisos Tauri.

| Archivo compartido | Intervención de T04 que debe autorizarse |
|---|---|
| `src-tauri/src/adapters/sqlite/migrations.rs` | Registrar 0002 y subir `SCHEMA_VERSION`; asegurar seed/backfill transaccional del upgrade y conservar backup previo. |
| `src-tauri/src/application/library.rs` | Inicialización de processing en la misma TX del nuevo import; invalidación del gate ante cambios efectivos de metadata que son inputs PRE. |
| `src-tauri/src/application/library_ports.rs` y `src-tauri/src/adapters/sqlite/library_repository.rs` | Transferencia acotada si el enganche anterior requiere ampliar primitivas de repositorio; firmas actuales y punto de inserción detallados abajo. No conceder ownership general de Library. |
| `src-tauri/src/domain/mod.rs`, `src-tauri/src/modules/mod.rs`, `src-tauri/src/adapters/sqlite/mod.rs` | Declarar los módulos nuevos que ya posee T04. |
| `src-tauri/src/application/mod.rs` | Sólo si el worker introduce el puerto interno de Workflow en `application`; el brief actual no asigna dicho archivo/puerto. |
| `src-tauri/src/desktop/lifecycle.rs` | Registrar/obtener el servicio Workflow y actualizar constructores de `DesktopState`. |
| `src-tauri/src/lib.rs` | Componer Workflow con el actor, documentos, maintenance, recovery y RequestRegistry compartidos. |
| `src-tauri/src/transport/commands/mod.rs` | Declarar `workflow` y conectar ocho ramas actualmente stub al nuevo adaptador. |
| `contracts/manifest.json` | Marcar implementados los ocho comandos PRE/P1; el cliente bloquea antes de invocar si siguen false. Candidatos P3 siguen false. |
| `src-tauri/src/modules/settings.rs` | Actualizar la capacidad Workflow actualmente false, sin atribuir soporte completo P2/P3/P4. |
| `src/app/App.tsx`, `src/app/App.test.tsx` | Montar y hacer accesible PhaseWorkspace desde PaperWorkspace; conectar `api.workflow`. |
| `src/app/ShellView.ts` | Sólo si la navegación elegida necesita extender la unión existente; actualmente PaperWorkspace contiene `paperId`, no fase ni vista interna. No hace falta inventar un router. |
| `src/shared/adapters/tauri/client.test.ts` | Cambiar la prueba que exige que `workflow.getPhase` sea unavailable sin invoke; probar el encaminamiento ahora disponible. |
| `src-tauri/tests/desktop_bootstrap.rs`, `src-tauri/tests/lifecycle_close.rs` | Constructores literales de DesktopState y fixture de migración futura que usa 2 fijo. |

Los estilos compartidos sólo necesitan transferencia si la UI usa reglas nuevas en ellos; no se han identificado como requisito obligatorio del puerto. `src/features/library/*` y `src/features/reader/*` pueden seguir bajo su ownership actual si la composición se hace en App. No modificar `0001_library.sql`: su checksum ya es histórico.

## 1. Migración, importación y upgrade

### API real de migración

`src-tauri/src/adapters/sqlite/migrations.rs:9`:

```rust
pub const SCHEMA_VERSION: i64 = 1;
pub struct Migration<'a> { pub version: i64, pub sql: &'a str }
pub const MIGRATIONS: &[Migration<'static>]; // sólo 0001 actualmente
pub fn migrate(c: &mut Connection, root: &LibraryRoot,
               steps: &[Migration<'_>]) -> Result<(), AppError>;
pub fn snapshot(c: &Connection, root: &LibraryRoot)
    -> Result<std::path::PathBuf, AppError>;
```

`migrate` verifica el ledger, selecciona versiones pendientes, hace snapshot fuera de TX si `current > 0` y ejecuta todos los SQL pendientes dentro de `with_transaction`. Registra checksum, `user_version` y verifica integridad antes de commit. No existe callback de aplicación ni invocación de `initialize_processing`; `Migration` sólo lleva SQL. El worker debe explicitar dónde se ejecutan el seed de las definiciones canónicas y el backfill, conservando esa atomicidad. El hecho de añadir el archivo 0002 no lo registra ni inicializa por sí solo.

`src-tauri/src/adapters/sqlite/actor.rs:85` ya llama `migrations::migrate(&mut c, &root, migrations::MIGRATIONS)`. El actor usa `SCHEMA_VERSION` para el modo futuro sólo lectura. No hace falta modificar actor para registrar otra migración. No mover upgrade a un paso tardío después de abrir servicios: el flujo actual decide writable/schema durante `prepare`.

`src-tauri/tests/desktop_bootstrap.rs:59` contiene `migration_failure_keeps_backup`: abre DB con el esquema actual, intenta un paso sintético `version: 2`, y espera rollback a `user_version=1`. Al subir a 2 esa prueba deja de intentar una migración pendiente y debe adaptar su versión futura/expectativa. Es un cambio compartido concreto, no un fallo de T04.

### Punto exacto del import

`src-tauri/src/application/library.rs:478`:

```rust
pub fn confirm_in_tx(tx: &Transaction<'_>, repository: &dyn LibraryRepository,
                     prepared: &PreparedImport, verified: &ResourceInspection)
    -> Result<PaperDto, AppError>;
```

Tras validar promoción/duplicados llama `repository.insert_paper_document(tx, NewPaperDocument { ... })`, vuelve a leer `repository.paper`, almacena ese DTO con `mark_import_committed`, audita y devuelve. La inicialización debe ocurrir antes de esa lectura/serialización final para que el resultado durable, el receipt y la DB reflejen `processingInitialized=true`, active PRE y fases fijadas juntos. Un segundo commit posterior es insuficiente. Las ramas COMMITTED/reuse existentes no deben reinicializar respuestas ni procesar dos veces el Paper.

`src-tauri/src/application/library_ports.rs:308`:

```rust
fn insert_paper_document(&self, tx: &Transaction<'_>, document: NewPaperDocument<'_>)
    -> Result<(), AppError>;
fn paper(&self, tx: &Transaction<'_>, paper_id: &str) -> Result<PaperDto, AppError>;
fn mark_import_committed(&self, tx: &Transaction<'_>, operation_id: &str,
                         paper: &PaperDto, updated_at: &str) -> Result<(), AppError>;
```

`NewPaperDocument` contiene `paper_id`, `doc_id`, `metadata`, `sha`, `size`, `filename`, `relative`, `now`. El SQL actual de `adapters/sqlite/library_repository.rs:350` inserta lifecycle NEW, revision 0, current_phase NULL, processing_initialized 0. El adaptador `commit_confirmation` de la línea 452 delega al helper de aplicación. La implementación pública del trait, línea 710, usa `actor.submit` + `with_receipt(..., "library_confirm_import", ..., |tx| ...)`. Recovery llega a ese mismo puerto desde `modules/library/service.rs:678`; por eso un enganche sólo en el flujo interactivo dejaría recovery sin inicialización.

No existen aún `initialize_processing` ni `invalidate_effective_change`: el brief fija sus nombres y TX prestada, no sus firmas concretas. No presentarlas al worker como API existente. Debe congelar sus firmas internas al entregar T04 para T05/T06/T07.

### Cambios bibliográficos que afectan al gate

`application/library.rs:399` expone:

```rust
pub fn update_metadata_in_tx(tx: &Transaction<'_>, repository: &dyn LibraryRepository,
    request_id: &str, paper_id: &str, expected_revision: i64,
    metadata: &PaperMetadataInput) -> Result<PaperDto, AppError>;
```

Actualmente normaliza, compara Paper.revision, actualiza metadata/autores y audita; no invalida ninguna fase. El SQL incrementa Paper.revision aun cuando los valores bibliográficos sean iguales. Para T04, al menos title/reviewType cambiados de forma efectiva deben renovar/invalidate los inputs PRE en la misma TX. No trasladar la obligación a T05: los tests stale preview/advance de T04 ya dependen de ella. No se propone redefinir aquí toda la política bibliográfica; distinguir el cambio efectivo de inputs del gate de una llamada metadata que no los cambia.

### Lifecycle atómico reutilizable

El nombre real es `archive_paper_in_tx`, no `archive_in_tx` como abrevia WORKFLOW_GATES:

```rust
pub fn archive_paper_in_tx(tx: &Transaction<'_>, repository: &dyn LibraryRepository,
    request_id: &str, paper_id: &str, expected_revision: i64)
    -> Result<PaperDto, AppError>;
```

Está en `application/library.rs:442`; `restore_paper_in_tx` tiene la misma firma. Usa repositorio prestado y no abre actor/receipt/commit. Es el puerto apto para P1 archive dentro de la TX Workflow. El repositorio expone también `current_revision`, `lifecycle_state`, `update_lifecycle` y `audit_change`. No llamar al wrapper público de Library desde advance. NEW→ACTIVE de PRE todavía no tiene helper específico; T04 debe realizarlo con esas primitivas dentro de su UoW.

## 2. Composición, IPC y ruta de frontend

`lib.rs:35–75` compone `Arc<LocalDocumentStore>`, Library y Reader con un `RequestRegistry` común, el mismo `MaintenanceCoordinator`, `DbActor` clonado y `RecoveryStatus`. Hoy consume `documents` y `request_registry` al construir Reader; será necesario clonarlos para compartirlos con Workflow. No crear un segundo actor ni registro de requests por módulo.

`desktop/lifecycle.rs:79` contiene `DesktopState` con `library: Mutex<Option<Arc<LibraryService>>>` y `reader: Mutex<Option<Arc<ReaderService>>>`; sus métodos públicos son `register_library`, `library`, `register_reader`, `reader`. Workflow no tiene slot. Los literales de ese struct también aparecen en tests desktop_bootstrap, lifecycle_close y en el módulo de pruebas del propio lifecycle.

`transport/commands/mod.rs:139`:

```rust
pub async fn dispatch(window: &str, command: &str, args: Value,
                      state: &DesktopState) -> Value;
```

Los diez nombres Workflow ya están en `COMMANDS`, `build.rs` y permisos read/write. Sus diez ramas deserializan DTOs existentes y devuelven UnsupportedCapability. Conectar ocho; mantener set/getP3Candidate indisponibles. El error y éxito ya se encapsulan por el dispatcher. Los DTOs exactos están en `transport/dto.rs:1725–1849`: `WorkflowGetPhaseArgs`, `WorkflowGetPhaseAnswersArgs`, `WorkflowGetPhaseDefinitionArgs`, `WorkflowSavePhaseAnswerArgs`, `WorkflowEvaluateGateArgs`, `WorkflowAdvancePhaseArgs/Output`, `WorkflowGoBackToPhaseArgs/Output`, `WorkflowTouchPhaseArgs`. No hace falta cambiar DTOs ni generar otros nombres. `version` es opcional no null; structuredValue/explanation son campos presentes nullable. Mantener esos requisitos.

Frontend dispone de `WorkflowApi` en `src/shared/contracts/ports.ts:27` y `api.workflow` ya construido en `src/shared/adapters/tauri/client.ts:38`. Sus métodos son exactamente getPhase/getPhaseAnswers/getPhaseDefinition/savePhaseAnswer/evaluateGate/advancePhase/goBackToPhase/touchPhase/setP3Candidate/getP3CandidateSummary.

**Bloqueo fácil de omitir:** `client.ts:14` comprueba `contracts/manifest.json` y devuelve UnsupportedCapability sin invoke cuando `implemented=false`. Los diez comandos Workflow están false actualmente. Cambiar sólo el dispatcher no hará accesible T04 desde UI. `src-tauri/src/modules/settings.rs:13` anuncia también workflow=false. El manifiesto es una lista por comando, de modo que habilitar ocho no elimina los guards internos P2 ni habilita candidatos.

`src/app/ShellView.ts` ya tiene `{kind:'PaperWorkspace';paperId:string}`. `App.tsx` conserva `OpenPaperDto` en `opened`, usa `showReader(OpenPaperDto)` para entrar y monta sólo `PaperReader` en PaperWorkspace, pasando `api.reader`; no monta una ruta Workflow ni ofrece entrada PRE/P1. `PhaseWorkspace` y los demás archivos del brief no quedan accesibles creando sólo sus ficheros: hay que transferir App y sus tests. Considerar acceso a lectura Workflow de archivados/inaccesibles; `reader.openPaper` es una mutación con PDF verificado, no un requisito contractual general para `workflow.getPhase/getPhaseAnswers`.

## 3. Proof documental previo a TX y revalidación

### Puerto disponible de T03

`src-tauri/src/application/reader_ports.rs:24`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct RegisteredDocument {
    pub library_id: UUID,
    pub document: DocumentDto,
    pub relative_path: String,
    pub size_bytes: i64,
}
pub type ReaderFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, AppError>> + Send + 'a>>;
pub type VerifiedDocument = Box<dyn DocumentReadHandle + Send>;
pub trait DocumentReadAccess: Send + Sync {
    fn open_verified(&self, registered: RegisteredDocument)
        -> ReaderFuture<'static, VerifiedDocument>;
}
pub trait DocumentReadHandle: Send {
    fn registered(&self) -> &RegisteredDocument;
    fn read_all(self: Box<Self>) -> ReaderFuture<'static, DocumentBody>;
}
```

`LocalDocumentStore` implementa ese trait en `adapters/documents/store.rs:168`. `open_verified` usa spawn_blocking, comprueba library UUID, document UUID/paper, status ACTIVE, ruta administrada exacta y tamaño, y devuelve handle retenido. No lee el PDF entero ni calcula hash: `sha256` aquí es el registrado. `read_all` carga bytes y no se necesita para el gate. No afirmar que el puerto verifica hash de contenido vivo.

El handle contiene `ManagedFile`, que retiene archivo y directorio padre; `ManagedFile` está en `adapters/windows/managed_files.rs:372`. `ManagedDirectory::open_file` usa adquisición relativa Managed (`FILE_SHARE_READ`, sin compartir escritura/borrado), comprobación de archivo regular/padre y mantiene la cadena. Este informe verifica código, no repite pruebas Windows.

### Revalidación transaccional reutilizable

`ReaderRepository` expone:

```rust
fn registered_document(&self, tx: &Transaction<'_>, document_id: &str)
    -> Result<RegisteredDocument, AppError>;
```

`SqliteReaderRepository` lo implementa en `adapters/sqlite/reader_repository.rs:35`: consulta Document con status ACTIVE y `papers.active_document_id = d.id`, comprueba ruta administrada/identidad biblioteca y devuelve datos comparables. `LibraryRepository::paper(tx, paper_id)` permite revalidar además `Paper.documentId`. El ejemplo real está en `application/reader.rs:14`, `open_paper_in_tx`: exige igualdad completa de RegisteredDocument y asociación con el Paper. **No reutilizar ese caso de uso para un gate**, porque modifica lastOpenedAt/app_session; reutilizar sus puertos/comprobaciones.

`ReaderPersistence::prepare_open(OpenPaperCommand)` también es un caso de uso Reader con semántica de receipt propia; no es un prepare genérico de Workflow. `current_document(connection,paper_id)` es privado en el adaptador Reader. La carga previa del documento de Workflow aún no tiene método público propio y cabe en su repositorio nuevo; no requiere exposición de rutas a UI.

La secuencia compatible con los puertos existentes es: lectura breve de referencia registrada del Paper; `DocumentReadAccess::open_verified` fuera del actor; envío del `VerifiedDocument` por ownership a la closure del actor; relectura de Paper/Document bajo la lectura consistente o TX de advance; comparación contra `verified.registered()`; mantener handle hasta finalizar comprobación/commit. `DbActor::submit` exige closure `FnOnce(&mut Connection) -> Result<T,AppError> + Send + 'static`, por lo que admite capturarlo por move. Conservar también OperationPermit y RequestPermit en el trabajo admitido, no sólo en el future IPC cancelable. `ReaderService::open_paper/open_owned` demuestra el patrón actual con spawn y oneshot.

### Dos huecos que no debe ocultar el handoff

1. El query Reader sólo devuelve ACTIVE/vigente o NotFound. No basta para construir un snapshot de gate actual de un Document MISSING/SUPERSEDED o ausente con `available=false`. Workflow necesita conservar/leer la identidad y el estado registrados de su input y representar la falta de acceso; no convertir indiscriminadamente errores de integridad/ownership en éxito ni llamar read_all. La revalidación también debe cubrir la referencia usada para una prueba fallida: si DB cambia durante el proof no usar un available=false de otra referencia.
2. `with_receipt` sólo hace replay al entrar a su TX. Un fallo de proof previo puede impedir llegar al replay durable si se copia literalmente el flujo Reader. El brief exige replay antes CAS; el diseño del worker debe resolver el lookup de receipt/reintento antes de exigir un proof nuevo para una mutación ya confirmada, sin confundir replay de éxito anterior con advance nuevo sobre PDF inaccesible. No existe un helper público de lookup genérico hoy; Reader tiene lógica ad hoc propia.

`application/unit_of_work.rs:2` ofrece `with_transaction(&mut Connection, FnOnce(&Transaction)->Result<T,AppError>)`, siempre IMMEDIATE y commit al final. `adapters/sqlite/receipts.rs:8` ofrece `with_receipt(connection,request_id,command,payload,action)`; hace replay antes de action/CAS, inserta receipt y audit genérico sólo tras éxito, dentro de esa UoW. Un error GateBlocked/Conflict revierte los efectos y no crea receipt exitoso. Evaluate debe usar lectura consistente sin with_receipt, para no escribir audit/receipt.

## Discrepancias cerrables antes de asignar

1. Ownership original impide tocar migración/import/composición; transferir explícitamente la lista acotada anterior. Las firmas wire ya existen; las internas de Workflow todavía no.
2. `archive_in_tx` documental es en código `archive_paper_in_tx`; usar nombre y firma reales.
3. No existe registro 0002 ni callback/backfill de procesamiento. `Migration` sólo tiene version/sql y el test sintético version2 asume esquema1.
4. Import normal y recovery comparten confirm_in_tx: inicializar antes de construir el resultado durable. Upgrade conserva revisiones/metadatos/lifecycle; no confundirlo con update_metadata.
5. Metadata PRE requiere invalidación en Library durante T04. El brief la exige semánticamente pero no transfiere hoy ese archivo.
6. Cliente y backend settings siguen anunciando Workflow indisponible; añadir archivos de manifiesto/capacidad y prueba del cliente al ownership.
7. App sólo muestra lector en PaperWorkspace; la ruta UI Workflow falta pese a existir api.workflow.
8. Puerto proof T03 reutilizable y suficiente para handle; query ACTIVE-only y replay temprano necesitan tratamiento explícito en Workflow. No prometer hashing vivo ni usar open_paper_in_tx como lectura pura.

## Evidencia y límites

Se leyeron AGENTS, INTENT, STATUS, brief T04, WORKFLOW_GATES, WORKFLOW y se consultó DATA para 0002; se inspeccionaron con `Get-Content`/`rg` las rutas anteriores. Comandos de identificación ejecutados en integration: `git rev-parse HEAD` y `git status --short`, ambos exit0, HEAD y árbol limpio indicados al inicio. No se han ejecutado tests/build, tocado producto, creado agentes, realizado commits o merges. Archivo nuevo único de esta investigación: este informe central ignorado. La lectura de memoria auxiliar sólo recordó las fronteras actor/UoW; todas las firmas y hechos de producto aquí documentados se verificaron en el corte actual.

Autorrevisión: las firmas reproducidas existen; las necesidades futuras están etiquetadas como pendientes y no como APIs disponibles. Sin rediseño de arquitectura ni ampliación funcional.

Estado: DONE_WITH_CONCERNS — preflight completo; requiere incorporar las transferencias y resolver los huecos identificados en el despacho T04.
