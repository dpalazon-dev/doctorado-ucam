# T02 — revisión independiente de seguridad

## Spec Compliance

- **❌ Issues found / DONE_WITH_CONCERNS.** El corte no satisface aún las garantías de autoridad de tokens, propiedad de archivos, recuperación visible y lectura acotada requeridas por TASK02_PORTS, DATA y PDF_VALIDATION. Hallazgos S1–S7 abajo.
- **Task quality: Needs fixes.** No integrar con estos Important abiertos. No he identificado un Critical demostrado.
- **BASE:** `94f1796513383ae75486f0b429ab25bae0fb63c1`. **HEAD:** `45c4c8c48269495b74241004f92f68acf4770708`.
- **Revisor:** `/root/task02_security_review`. Paquete: `.superpowers/sdd/IMPLEMENTATION/review-94f1796..45c4c8c.diff`. Checkout: `.worktrees/task-02-library`. Lectura por porciones; no subagentes, suites, instalaciones, red, datos personales ni cambios Git/producto. Única escritura: este informe.

## Strengths

- `src-tauri/src/modules/library/service.rs:183` y `:235`: confirm/cancel exigen autoridad de token y exclusión por token; el permiso de mantenimiento abarca selección y trabajo filesystem, no solamente la transacción SQL. `desktop/maintenance.rs:58` y `desktop/lifecycle.rs:186` extienden el drenaje antes de parar el actor.
- `src-tauri/src/adapters/sqlite/library_repository.rs:517`: el resultado de confirmación pasa por `with_receipt`; Paper/authors/Document/COMMITTED quedan bajo el mismo commit. El payload original para idempotencia se separa de metadatos normalizados (`:227`). Los valores externos en consultas son parámetros; los fragmentos dinámicos de SQL forman solamente estructura e índices de placeholders (`:836`).
- `src-tauri/src/adapters/documents/pdf_probe.rs:5`: el probe usa parser real, resuelve la primera página, comprueba MediaBox finita y positiva y rechaza cifrado declarado. No ejecuta contenido gráfico ni JavaScript. `store.rs:118` limita la copia original y calcula SHA-256 en streaming. No se confunde este probe con render de todas las páginas ni con cuota total de RAM.
- `src-tauri/src/adapters/windows/pdf_picker.rs:27`: selección mediante callback Rust; el IPC solo recibe token/DTOs. `src-tauri/capabilities/main-local.json:9` conserva exclusivamente permisos propios. `transport/error.rs:17` añade solamente `currentRevision`, sin rutas ni errores crudos del OS/parser.
- `src-tauri/tests/library_integration.rs:985`: el destino ambiguo presente antes de recovery se conserva, sin crear Paper. `:1110` comprueba que la composición para esquema futuro no modifica el inventario de la raíz. Estas defensas existen, aunque no cubren las carreras de S1.

## Critical

Ninguno demostrado en este corte. Los hallazgos siguientes tienen impacto real en autorización, integridad o disponibilidad, pero no demuestran ejecución remota ni exfiltración.

## Important

### S1 — comprobaciones por pathname no vinculan la operación al archivo validado

**Referencias:** `src-tauri/src/adapters/documents/store.rs:67`, `:93`, `:224`, `:271`, `:278`, `:310`; limpiezas tras fallo `:174` y `:188`.

`operation_path` inspecciona componentes y devuelve un PathBuf. `matches_hash` abre por ese nombre y cierra el handle; más tarde `cleanup` vuelve a usar el nombre en `remove_file`. Promoción separa `target.exists()` de `fs::rename`. No se retienen handles de archivo/directorios, no se verifica identidad/final path del handle usado y no hay garantía atómica de no reemplazar destino. La exclusión por token y el lock SQLite no impiden cambios externos de archivos/directorios.

**Amenaza e impacto:** un proceso local o cambio externo sustituye staging después del hash, cambia un padre a un enlace/reparse point después de la inspección o crea el destino entre `exists` y `rename`. La operación puede borrar el archivo sustituto, acceder fuera de la raíz o reemplazar un destino ajeno. El hash posterior del destino no recupera un archivo ya reemplazado. Las limpiezas de staging inválido también borran por nombre después de cerrar el writer. Contradice DATA §5–6 y la conservación de archivos ambiguos.

**Remedio mínimo concreto:** centralizar operaciones Windows en un helper privado del adaptador, sin nuevo IPC ni framework. Abrir y retener el archivo que se hashea/valida; comprobar su identidad, atributos reparse y destino final bajo la raíz autorizada. Retener guardas de la raíz y padres, con compartición que impida sustitución/renombrado y modificación durante la operación. Usar el mismo handle para lectura/hash y disposición de borrado, y promoción con no-clobber atómico. Las APIs propuestas para decisión del coordinador son `GetFinalPathNameByHandleW` y `SetFileInformationByHandle` con `FileDispositionInfo`/`FileRenameInfo`, `ReplaceIfExists=false`; la garantía exige también padres estables, no solamente otro `exists` o una canonicalización anterior. Un fallo al establecer estas garantías deja issue seguro y no elimina nada. El coordinador verificó esas APIs; no he implementado ni aprobado una nueva dependencia/API pública.

**Regresión necesaria:** hooks deterministas sobre raíz sintética para sustituir padre/archivo tras validación y crear destino inmediatamente antes de promoción; archivo ajeno íntegro, cero escritura fuera de root y cero Paper exitoso. No baso el hallazgo en afirmar que esta versión de Rust ignore todos los junctions: incluso si `is_symlink()` los detecta, la carrera por nombre permanece.

### S2 — un hash igual no demuestra propiedad de staging ni ausencia de referencias

**Referencias:** `src-tauri/src/adapters/documents/store.rs:286`, `:302`, `:310`; `src-tauri/src/adapters/sqlite/library_repository.rs:543`; `src-tauri/src/modules/library/service.rs:439`.

`cleanup_owned` acepta cualquier ruta relativa sin traversal y decide borrar por coincidencia SHA-256. No exige `staging/<operationId>/source.pdf`, no valida la relación entre IDs/rutas/biblioteca y no consulta si el recurso está referenciado por `documents` u otra operación. `prepare_cancel` construye el plan directamente desde la fila; su único rechazo de propiedad persistente es el estado COMMITTED de esa operación.

**Amenaza e impacto:** una intención incoherente o dañada puede señalar como staging `documents/<id>/original.pdf`, con su hash correcto y sin destino. Cancelación/recovery borra entonces un documento vigente de otra operación. No hace falta traversal ni vencer SHA-256: la ruta y hash existentes ya cumplen las comprobaciones actuales. Un hash prueba contenido, no dueño. TASK02_PORTS exige explícitamente propiedad inequívoca y ninguna referencia documental vigente antes del cleanup.

**Remedio:** validar IDs canónicos, libraryId efectivo y namespace exacto derivado del operationId; no confiar en nombres persistidos arbitrarios. Antes de conceder el plan de eliminación comprobar referencias vigentes y otras intenciones en la UoW, manteniendo las garantías de handles de S1. Ante discrepancia devolver issue y conservar el recurso. Probar intención sintética que apunta a un documento registrado y otra operación con mismo contenido: ambos sobreviven. Esto protege integridad frente a registros incoherentes; no es una promesa de proteger la DB frente a un administrador malicioso.

### S3 — replay de selectPdf concede autoridad nueva a un token de otra sesión

**Referencias:** `src-tauri/src/modules/library/service.rs:104`, `:119`, `:124`; `src-tauri/src/adapters/sqlite/library_repository.rs:431`.

La respuesta durable de `previous_selection` pasa por `remember_token`, que la vincula a la sesión actual y asigna otras 24 horas monotónicas. El receipt no se limita a devolver el preview original: renueva autoridad. Un nuevo LibraryService tras reinicio, con mapa de tokens vacío, puede repetir el requestId de selección anterior y después confirmar o cancelar su staging con una petición nueva. El expiry durable puede limitar la confirmación inicial, pero no impide la reautorización de una selección todavía vigente ni la cancelación con autoridad renovada.

**Impacto:** reanuda drafts de sesiones anteriores por un camino que el ruling de TASK02_PORTS §4 prohíbe expresamente. Recovery interno autorizado y replay de receipt no deben convertirse en renovación UI.

**Remedio:** devolver el preview original sin llamar a `remember_token` en replay. Registrar autoridad solamente al completar una selección nueva del picker. Conservar una autoridad ya existente en la misma sesión sin extender su vencimiento. Regresión: seleccionar, construir servicio nuevo, repetir select con mismo requestId, recibir mismo preview; confirm/cancel con requestIds nuevos deben dar OperationCancelled. Replay de receipts ya consumados sigue permitido y recovery de intención confirmada sigue separado.

### S4 — el arranque descarta incidencias y puede informar biblioteca sana con PDF COMMITTED ausente

**Referencias:** `src-tauri/src/lib.rs:49`; `src-tauri/src/modules/library/service.rs:316`; `src-tauri/src/adapters/sqlite/settings.rs:33`.

El task de arranque asigna `let _report = library.reconcile_imports().await` y libera el permiso incluso si devuelve error. No conserva Result, issues ni diagnóstico. Settings consulta filas no COMMITTED; no incluye los COMMITTED cuyo destino falte o cuyo hash haya cambiado, precisamente los que recovery marca como `integrityFailure`.

**Amenaza e impacto:** borrar/cambiar externamente un PDF confirmado y reiniciar produce una incidencia en memoria que se descarta; Settings puede devolver `recoveryRequired=false`. Un error global al leer/reconciliar también se pierde. Oculta una pérdida de integridad existente y permite que la UI siguiente presente ausencia de pendientes sin fundamento. El ruling de composición exige conservar issues reales y hacerlos visibles.

**Remedio:** mantener resultado/estado seguro de recovery en el estado ya compuesto y combinarlo con Settings; conservar errores globales e issues COMMITTED hasta reconciliación comprobada. No convertir pendientes aislados en readonly global. Regresión de composición: confirmar, cerrar, quitar/cambiar PDF sintético, arrancar y consultar Settings; debe señalar recuperación/integridad requerida y conservar la ficha. Inyectar error de recovery y verificar diagnóstico visible.

### S5 — la lectura para el parser vuelve a ser ilimitada y puede validar otros bytes

**Referencias:** `src-tauri/src/adapters/documents/store.rs:179`, `:181`, `:186`; `:118`; `src-tauri/tests/library_integration.rs:816`.

Tras copiar hasta 500 MiB, se cierra el writer y se reabre staging con `fs::read`, que no limita crecimiento ni comprueba longitud/hash frente a lo copiado. El mutex limita concurrencia del parser, no esa lectura. La prueba GrowingReader cubre `copy_bounded`; la fixture estática de 500 MiB no cubre la segunda lectura.

**Amenaza e impacto:** crecimiento/sustitución de staging mientras espera el turno de validación puede cargar mucho más de 500 MiB, agotar RAM o validar bytes distintos del hash y tamaño emitidos en preview. El límite de expansión del parser es un tradeoff aceptado; el input ilimitado adicional no lo es. PDF_VALIDATION manda leer hasta 524288001 bytes para detectar exceso/crecimiento.

**Remedio:** leer del handle retenido de S1 con `take(MAX_PDF_BYTES + 1)`, rechazar exceso y comprobar tamaño/hash del buffer contra los hechos que se van a publicar; mover el buffer al parser sin copia. Regresión determinista de crecimiento/sustitución entre copia y probe, con InvalidInput/issue seguro y cero preview. No reclamar con ello cuota total de RAM ni aislamiento del parser.

### S6 — cancel_unstaged marca cleanup DONE aunque existan recursos no limpiados

**Referencias:** `src-tauri/src/modules/library/service.rs:146`; `src-tauri/src/adapters/sqlite/library_repository.rs:475`, `:478`, `:563`; `src-tauri/src/adapters/documents/store.rs:174`, `:179`, `:186`, `:188`.

Todo error de `stage` lleva a `cancel_unstaged`, que escribe FAILED/cleanup DONE sin prueba filesystem. Un fallo `sync_all` o `fs::read` retorna dejando archivo; fallos de `remove_file`/`remove_dir` se ignoran. Luego `recoverable_imports` excluye FAILED/DONE y Settings tampoco anuncia ese pending.

**Impacto:** quedan copias completas/parciales conservadas pero declaradas limpiadas y excluidas de recuperación. Fallos de almacenamiento, antivirus o permisos producen fugas de recursos y pérdida de la intención recuperable; repetir selección puede acumular hasta 500 MiB por fallo sin diagnóstico. No es pérdida del original, pero sí estado durable falso y recursos abandonados.

**Remedio:** stage debe devolver o registrar resultado de cleanup inequívoco; DONE solamente tras demostrar ausencia/eliminación del recurso propio. Persistir fallo y PENDING si limpieza/sync/probe no garantiza disposición, sin borrar a ciegas staging cuyo hash aún no se conoce. Recovery muestra ese caso conservador. Regresiones con fallo sintético en sync/read/remove: archivo retenido e incidencia visible, sin marcar DONE ni éxito.

### S7 — falta la comprobación normativa de denegación IPC con plugin registrado

**Referencias:** `src-tauri/src/lib.rs:16`; `src-tauri/tests/desktop_bootstrap.rs:139`; `src-tauri/src/adapters/windows/pdf_picker.rs:56`; TASK02_PORTS ruling §5 y NATIVE_PICKER.

El test existente de RuntimeAuthority construye `mock_builder` sin `tauri-plugin-dialog`; solo prueba Settings, otra ventana y arbitrary_sql. La prueba nueva del picker entrega directamente un oneshot y no invoca IPC. No existe en el diff/evidencia una prueba de `plugin:dialog|open` denegado cuando el plugin está efectivamente instalado, ni del comando propio conectado a Library real.

**Impacto:** requisito de seguridad explícito sin demostrar tras añadir un plugin que además incluye fs transitivo. La configuración estática no concede permisos genéricos; no afirmo un bypass existente. Bloquea la afirmación de denegación verificada requerida en esta tarea.

**Remedio:** añadir prueba de RuntimeAuthority con plugin registrado y capability real main-local: invocación directa de open y comando fs rechazada antes del picker; comando Library propio permitido con puerto sintético, envelope contractual y sin rutas libres. Ejecutarla de forma focalizada después del fix bajo coordinación Cargo. El selector GUI observado sigue siendo una comprobación distinta y pendiente.

## Minor

Ninguno adicional que cambie esta decisión. Los límites explícitos de render, memoria total del parser e instalación no se presentan como defectos implementativos de T02.

## Checks y límites de verificación

- Fuentes normativas leídas: AGENTS, INTENT, STATUS, brief/handoff/composition/review-context, TASK02_PORTS (ruling final), PDF_VALIDATION, NATIVE_PICKER, ADR-013/014 y secciones filesystem/import de DATA; QUALITY para límites de memoria. Skill aplicada: `security-review`; protocolo: `subagent-driven-development/task-reviewer-prompt.md`. Informe del autor tratado como afirmaciones, no aprobación.
- Diff completo leído por porciones. Salidas grandes truncaron partes del lockfile y una fixture: se recuperaron solamente esas partes; no se releerieron archivos de producto cambiados completos. Los números de línea se derivaron de hunks del paquete.
- Inspecciones externas al diff por riesgos concretos: `adapters/windows/paths.rs` para comprobar si LibraryRoot ya retenía handles/contención (no); `capabilities/main-local.json` y helper ACL de `tests/desktop_bootstrap.rs` para comprobar permisos y si registraba el plugin nuevo (solo permisos propios; helper sin plugin). No se amplió a revisión global del repositorio.
- Evidencia local legible: `work/evidence/check-final.log` y `check-final.exitcode=0`, `tauri-build-debug.log` y `tauri-build-debug.exitcode=0`. Gate: 14 UI; 4 unit Rust; integration 7 db_actor, 12 desktop_bootstrap, 27 library, 3 lifecycle, 6 contracts, 4 schema entries (una helper), 1 settings. Build debug MSVC terminado. Se consultó inicialmente el sufijo incorrecto `.log.exitcode`; se localizó y leyó el sidecar real, sin regenerar evidencia ni repetir pruebas.
- **Cannot verify:** no reproduje dinámicamente las carreras/errores S1–S6; se demuestran las ventanas y decisiones erróneas por flujo de código. Las regresiones indicadas todavía son necesarias. No se comprobó denegación dinámica plugin IPC, selector gráfico, NSIS, equipo limpio ni reader/render (fases posteriores). No ejecuté npm audit/cargo audit ni consulta de advisories por alcance expresamente sin red; no certifico ausencia de vulnerabilidades en dependencias. Pins/checksums nuevos sí están en el lockfile.
- No consulté OpenViking: el contexto local bastó y no se había verificado peer. La búsqueda ligera de MEMORY.md no produjo coincidencias relevantes; no se usaron decisiones de memoria.

## Assessment

**Security / spec: Needs fixes.** Las defensas de DTO/SQL/errores y la saga básica existen y sus pruebas verdes están documentadas. La autorización renovada por replay, las operaciones destructivas desacopladas del handle validado y la supresión de incidencias impiden aprobar la seguridad e integridad de T02 en este HEAD. Corregir en la rama del autor, probar las regresiones focalizadas y revisar el delta antes de integrar.
