# T02 — revisión Rust independiente

## Conformidad

**❌ Issues found.** BASE `94f1796513383ae75486f0b429ab25bae0fb63c1`, HEAD `45c4c8c48269495b74241004f92f68acf4770708`. Los ocho comandos Library están conectados y los mínimos Rust pasan, pero hay incumplimientos verificables de PDF_VALIDATION, autoridad de tokens, diagnóstico de recovery, limpieza recuperable y propiedad de trabajos filesystem.

**Calidad: Needs fixes / Block.** No se recomienda integrar hasta cerrar los hallazgos Important. No se identificó un hallazgo Critical adicional. Revisión por `/root/task02_rust_review`, solo lectura del producto; única escritura este informe central.

## Fortalezas

- `src-tauri/src/modules/library/service.rs:162` coordina puertos asíncronos sin mover una Connection/Transaction al trabajo filesystem; `src-tauri/src/adapters/sqlite/library_repository.rs:524` confirma resultado/receipt mediante la UoW del actor. `src-tauri/tests/library_integration.rs:908` verifica rollback SQL sin Paper prematuro y recuperación tras reapertura.
- `src-tauri/src/adapters/documents/store.rs:114` copia/hash en streaming y verifica el límite durante la copia; `src-tauri/tests/library_integration.rs:816` verifica un PDF sintético de exactamente 500 MiB. El buffer de copia no se confunde aquí con la memoria total del parser.
- `src-tauri/src/desktop/maintenance.rs:61` y `src-tauri/src/desktop/lifecycle.rs:181` esperan operaciones/mantenimiento antes de detener el actor cuando sus permits siguen vivos. Las pruebas `src-tauri/tests/lifecycle_close.rs:101` y `:126` demuestran ese mecanismo.
- `src-tauri/src/lib.rs:40` condiciona recovery al esquema writable; `src-tauri/tests/library_integration.rs:1110` verifica inventario de raíz futura intacto y mutaciones `SchemaTooNew`.
- `src-tauri/src/transport/error.rs:18` conserva `details.currentRevision`; `src-tauri/tests/library_integration.rs:1020` comprueba el envelope. SQL modificado usa valores parametrizados; no se encontró `unsafe` propio nuevo.
- `FileOptions::uncached()` **sí configura strict** en la versión fijada `pdf=0.10.0`: fuente local `pdf-0.10.0/src/file.rs:712-717` asigna `ParseOptions::strict()`. Omitir un setter explícito no es un defecto.

## Critical

Ninguno identificado en el alcance leído.

## Important

### R1 — PDF válido con MediaBox heredada rechazado

**`src-tauri/src/adapters/documents/pdf_probe.rs:14-16`.** El probe exige `page.media_box` directamente. Ese campo contiene solo la entrada de la página; puede estar ausente cuando la caja se hereda de su padre `Pages`. La fuente fijada `pdf-0.10.0/src/object/types/page.rs:215-223` ofrece `Page::media_box()` para resolver esa herencia, y `src/file.rs:825-828` no rellena el campo al ejecutar `get_page`. Por tanto un documento con estructura y primera caja válidas recibe `InvalidPdf`. Las fixtures actuales ponen la caja tanto en padre como en hija.

**Corrección:** resolver la MediaBox efectiva, conservando protección frente a ciclos/profundidad adversa y validación de dimensiones finitas/positivas. Añadir una fixture xref correcta con caja solamente en el padre y verificar aceptación; conservar rechazo cuando no existe caja en ningún ancestro.

### R2 — Segundo read del staging no tiene límite

**`src-tauri/src/adapters/documents/store.rs:186`.** Tras cerrar el output y esperar el gate, `fs::read` carga el archivo completo sin el máximo `524288001` establecido en PDF_VALIDATION. La copia acotada anterior no limita esta lectura posterior: un staging que crece o es sustituido durante la espera puede producir asignación sin límite y validación de bytes distintos del hash/tamaño publicados. El test de crecimiento actual comprueba `copy_bounded`, no este segundo read.

**Corrección:** abrir el staging autorizado y leer como máximo MAX+1; rechazar exceso y contrastar tamaño/hash del mismo contenido validado antes de publicar preview. Añadir prueba determinista de cambio/crecimiento entre copia y probe, sin sleeps ni fixture gigante permanente.

### R3 — Replay de selectPdf vuelve a autorizar un token antiguo

**`src-tauri/src/modules/library/service.rs:124`, `:104-114`.** Al encontrar el receipt original de selección se llama `remember_token`, que establece sesión actual y otro plazo monotónico de 24 h. Así una nueva instancia del servicio puede recibir el receipt de la sesión anterior y usar su token todavía no expirado para una nueva confirmación/cancelación. La resolución vinculante de TASK02_PORTS #4 exige devolver el resultado anterior sin renovar su autoridad. En una misma sesión el replay también modifica artificialmente la autorización en memoria.

**Corrección:** no registrar/renovar autoridad al reproducir un receipt. Solo una selección nativa nueva autoriza el token nuevo. Verificar que replay tras crear un servicio nuevo devuelve el preview original, pero una nueva mutación con ese token da `OperationCancelled`; los receipts de confirmación/cancelación ya terminadas siguen siendo reproducibles.

### R4 — Recovery pierde sus errores e incidencias al componer runtime

**`src-tauri/src/lib.rs:49`, `src-tauri/src/adapters/sqlite/settings.rs:33`, `src-tauri/src/modules/library/service.rs:318-326`.** El background guarda `reconcile_imports().await` en `_report` y lo descarta, tanto `Err` como `RecoveryReport.issues`, antes de soltar exclusión. Settings reconstruye `recovery_required` solo con filas no COMMITTED. Un COMMITTED cuyo PDF falta/está cambiado genera issue en el servicio, pero puede quedar `recoveryRequired=false` en la aplicación. Un error global que impide enumerar imports tampoco conserva diagnóstico. Incumple la resolución de composición que exige conservar resultado/diagnóstico y hacer visibles issues reales.

**Corrección:** conservar resultado/estado seguro de recovery en el estado compartido y combinarlo con Settings; un fallo global debe dejar diagnóstico explícito y política de admisión segura. Verificar composición con documento COMMITTED ausente/alterado y fallo de enumeración, no solo inspección directa del reporte.

### R5 — Stage fallido se marca cleanup DONE sin prueba de eliminación

**`src-tauri/src/adapters/documents/store.rs:174-175`, `:188-189`; `src-tauri/src/modules/library/service.rs:146`; `src-tauri/src/adapters/sqlite/library_repository.rs:475-479`, `:560-564`.** Los errores de eliminación del archivo se silencian; hay además retornos de error de sync/read sin limpieza. El servicio llama luego a `cancel_unstaged`, también silenciando su fallo, y este registra incondicionalmente FAILED/cleanup DONE. Si el archivo quedó por fallo de sync, lectura, parser o eliminación, la consulta de recovery excluye esa operación y Settings la considera resuelta. Se puede acumular staging propio sin hash/diagnóstico ni recuperación, contradiciendo la garantía de intención recuperable.

**Corrección:** registrar DONE únicamente después de prueba de ausencia/limpieza propia; conservar PENDING/incidencia ante fallo y no perder el error de persistencia. El error primario puede seguir siendo seguro para UI, pero la saga necesita evidencia durable de limpieza. Inyectar fallo de limpieza o fallo después de sync y demostrar que reinicio conserva/diagnostica la operación, sin borrar recursos ambiguos.

### R6 — Recovery de reuseExisting omite limpiar staging

**`src-tauri/src/modules/library/service.rs:372-409`, `src-tauri/src/adapters/sqlite/library_repository.rs:367-370`, `:560-564`.** El camino normal de reuse hace `cleanup_owned` antes de COMMITTED. Recovery omite todo el bloque filesystem cuando `reuse_paper.is_some()` y confirma directamente. Un corte tras preparar reuse y antes de limpiar deja el PDF staging, después marcado COMMITTED con `destination_path=NULL`; `recoverable_imports` deja de enumerarlo. El reporte cuenta recuperación exitosa y ningún issue pese a la copia pendiente.

**Corrección:** usar el mismo plan/prueba de limpieza en recovery de reuse, o representar cleanup pendiente durable que siga enumerado y visible. Verificar corte tras preparación de reuse antes de cleanup, y un fallo de cleanup, comprobando conservación del Paper original y ausencia de staging huérfano o presencia explícita del issue.

### R7 — Cancelar el Future libera permit mientras continúa el filesystem

**`src-tauri/src/modules/library/service.rs:117`, `:169`, `:226`; `src-tauri/src/adapters/documents/store.rs:36-62`.** El OperationPermit y TokenPermit pertenecen al Future del servicio; los jobs `spawn_blocking` poseen solo raíz/IDs. Si se elimina/cancela ese Future mientras un job trabaja, se liberan ambos permits y se descarta el JoinHandle; Tokio mantiene ejecutándose el job. El cierre puede observar idle, cerrar actor y liberar el lock mientras ese job aún copia/promueve/elimina, y otra mutación del mismo token puede ser admitida. Las pruebas nuevas mueven manualmente permits a hilos auxiliares y por eso no ejercitan la propiedad real del adapter.

**Corrección:** la operación admitida debe conservar sus permits hasta terminar todo el trabajo, incluso cuando desaparece el caller (por ejemplo, job de aplicación propietario de toda la saga/permits y canal de resultado; no basta con abortar spawn_blocking). Añadir test determinista que inicia una operación real, bloquea el job, descarta su Future y solicita cierre/segunda mutación; lock y exclusión deben persistir hasta la liberación del job.

**Comprobación focalizada:** Tauri `async_runtime.rs:119-125` y `:209-215` delega a Tokio; `tokio/src/runtime/task/join.rs:18-36` documenta detach al soltar JoinHandle. No es un supuesto sobre el runtime.

### R8 — RequestId concurrente no se reserva antes de efectos filesystem

**`src-tauri/src/modules/library/service.rs:119-153`, `:226-248`; `src-tauri/src/adapters/sqlite/library_repository.rs:458-471`, `:543-556`.** La consulta de receipt previo y el commit con receipt están separados por picker/filesystem; la exclusión es por token, no por requestId. Dos `selectPdf` concurrentes con el mismo requestId pueden leer ausencia, crear dos intenciones/copias y recibir ambos el primer receipt. `with_receipt` devuelve el resultado previo antes de ejecutar el segundo `record_staged`, así la segunda copia no registra su hash y queda ambigua. Dos cancelaciones de tokens diferentes con un requestId compartido pueden eliminar ambos staging antes de que el segundo commit descubra `Conflict`. La serialización del actor no serializa la saga completa.

**Corrección:** coordinar/reservar identidad requestId/command/payload antes de comenzar efectos externos, sin mantener una TX durante filesystem ni modificar unilateralmente 0001. Una segunda petición compatible debe esperar/reproducir la primera; una incompatible debe fallar antes de limpiar/promover. Verificar ambos escenarios con barreras deterministas.

**Comprobación focalizada fuera del diff:** `src-tauri/src/adapters/sqlite/migrations/0001_library.sql:12` no impone unicidad a import_operations.request_id; `src-tauri/src/adapters/sqlite/receipts.rs:40-47` devuelve receipt previo sin ejecutar action. La migración se leyó solo para este riesgo; no se propone cambiarla.

## Minor

- **`src-tauri/src/adapters/sqlite/library_repository.rs:865-867`.** Dos unwrap de serialización de `PhaseCode` en código productivo. El enum unitario actual hace que no se observe un panic con entradas válidas, por lo que no se presenta como un crash demostrado; conviene un mapping exhaustivo/fallible que deje la invariante explícita.
- **`src-tauri/src/adapters/documents/store.rs:279`.** Se pierde el resultado del intento de sincronizar el directorio tras rename. Documentar la política Windows de esa barrera y manejar/clasificar el fallo; no presentar la llamada descartada como prueba de durabilidad de promoción. No se ejecutó fault injection de pérdida de energía.

## Verificación y límites

- Leídos AGENTS, INTENT, STATUS, brief/context/handoff/ruling/reporte centrales, TASK02_PORTS, PDF_VALIDATION y ADR-013/014. Aplicada plantilla Superpowers `task-reviewer-prompt.md` y `rust-patterns/SKILL.md`. El pase rápido de MEMORY.md no produjo coincidencias relevantes; no se usaron hechos de memoria ni OpenViking.
- Leído el paquete completo `review-94f1796..45c4c8c.diff` en porciones una vez; referencias calculadas desde sus hunks. No se volvieron a leer archivos modificados del producto. Fuera del diff solo se comprobaron los riesgos nombrados de defaults/herencia PDF, supervivencia de jobs y receipts/DDL. La consulta adicional de DATA fue normativa, no rastreo de código.
- Desde `.worktrees/task-02-library`, antes de cada Cargo: `. ./scripts/development-env.ps1`. Ejecutados secuencialmente `C:/Users/david/.cargo/bin/cargo.exe check --manifest-path src-tauri/Cargo.toml`, `clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`, `fmt --manifest-path src-tauri/Cargo.toml --check`, `test --manifest-path src-tauri/Cargo.toml`. **Los cuatro exit 0.** Sin warnings observados.
- Suite propia: 4 unit + 60 entradas integration runner; 64 entradas observadas, de las que `pending_wal_fixture_child` es helper (63 comprobaciones de comportamiento + 1 helper). Library 27/27, cierre 3/3. No se repitieron suites adicionales ni se escribió código/tests/configuración de producto.
- Ejecutado `git diff HEAD~1 -- '*.rs'` exigido por el rol: vacío, coherente con último commit documental. La revisión real usa el corte completo BASE..HEAD del paquete, no ese diff vacío.
- Leídas colas y exitcode=0 de `work/evidence/check-final.log` y `tauri-build-debug.log`. Evidencia previa completa de UI/contratos pertenece al autor/coordinador; el build nativo no prueba selector GUI, instalación NSIS, equipo limpio ni uso offline.
- No se ejecutaron pruebas nuevas para reproducir R1–R8; los escenarios y controles necesarios quedan identificados para el autor. No se afirma que los tests actuales los cubran. No se exige Reader/DocumentReadAccess T03 ni Workflow T04.
- La revisión de task no prueba estado de CI ni ausencia actual de conflictos de merge. Integración sigue requiriendo CI/gates aplicables verdes, árbol limpio y merge preparado verificado según AGENTS.

**Estado de entrega: DONE_WITH_CONCERNS.** Revisión terminada; integración bloqueada por R1–R8.
