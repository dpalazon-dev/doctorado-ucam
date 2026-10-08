# T04a — revisión final de seguridad

Fecha: 2026-10-03. Revisor: `/root/task04a_security_review`.

**PASS de seguridad para el delta revisado.** No se identificaron regresiones de seguridad críticas o importantes. Existe una observación no bloqueante sobre el alcance de las pruebas y su descripción. La conformidad estricta de cobertura con el brief corresponde también a la revisión SPEC del orquestador; este PASS no sustituye ese gate ni la revisión Rust.

## Corte y alcance

- Worktree: `C:/Users/david/Projects/Research-Workbench/.worktrees/task-04a-document-proof`.
- BASE: `9188ac5dc4f1cc5ad22347ef1422a2c51d795b1b`.
- HEAD congelado: `20c7da9f9c9cb1757bd9755aceb00906d6f56239`.
- Commit de producto: `883b38584027bc6fedb160ceb85d580cb2eb47f2`.
- HEAD y árbol limpio verificados mediante Git; se revisó el diff completo de los cuatro archivos Rust y el informe del autor. No se revisó WIP.
- Referencias: brief central, informe central del autor, revisión previa de diseño, TASK04_PORTS/ADR-018. Se mantiene la aplicación de security-review a la frontera filesystem y propagación de errores.

El delta sólo modifica reader_ports.rs, documents/store.rs, windows/managed_files.rs, tests/reader_integration.rs y docs/reports/task-04a-report.md. No cambia DTOs, dependencias, capabilities, permisos de Tauri, runner, schema, composición o UI.

## Hallazgos priorizados

### Baja — precisar el nivel de inyección demostrado

Ubicación: `store.rs:720-745`, `managed_files.rs:968-994` y sección de evidencia del informe del autor.

La prueba de denegación construye `ManagedOpenError::Os(AccessDenied)` y llama directamente al mapper de store. La prueba NTSTATUS llama directamente a `classify_ntstatus`. Los errores de guardas se construyen ya envueltos en Fatal; el panic ocurre en una closure bloqueante de prueba y se entrega al mismo helper de flattening. Estas pruebas verifican las conversiones, pero no inyectan un fallo en la apertura OS que ejecuta `prove_access`, ni fuerzan un fallo de metadata/root dentro de ese recorrido completo.

La inspección del código confirma que los callsites productivos usan estos helpers y propagan correctamente la procedencia. Por ello no es una vulnerabilidad demostrada ni bloquea el dictamen de seguridad. Sin embargo, la frase del informe «mapeo NTSTATUS inyectado» no debe leerse como inyección en NtCreateFile o ACL real. Para el gate SPEC, el orquestador debe contrastar esta cobertura con el requisito más específico del brief/TASK04_PORTS de inyectar en el límite de apertura; este informe no lo declara satisfecho de extremo a extremo.

La sharing violation sí se provoca mediante dos adquisiciones incompatibles de un archivo sintético NTFS y recorre `prove_access`/Reader. Ausencia, tamaño divergente y éxito también se ejercitan mediante el store real.

No hay hallazgos críticos/altos/medios de seguridad en este delta.

## Controles de diseño contrastados con implementación

| Control | Evidencia revisada | Resultado |
|---|---|---|
| Sólo aperturas OS conocidas producen Unavailable | `classify_ntstatus` admite exactamente name/path-not-found, access-denied y sharing-violation; se invoca en el branch de fallo de NtCreateFile. Fallback clasifica únicamente errores de open. | Conservado |
| Guardas conservan Fatal, incluso NotFound/StorageUnavailable | `From<AppError> for ManagedOpenError` siempre envuelve Fatal; el mapper de store devuelve el AppError original. No inspecciona códigos o mensajes para inferir disponibilidad. | Conservado |
| Pin OS frente a verificaciones del pin | `open_child_inner`, líneas339-364: mantiene OpenOrCreate/ReadPin; la apertura conserva causa OS y regularidad/padre/descendencia siguen propagando AppError como Fatal. | Conservado |
| Identidad raíz y locks | `store.rs:267-268` usa root.managed_root y ManagedRoot::directory; sus errores pasan por la conversión Fatal. LibraryRoot conserva la identidad retenida, sin nuevo binding por ruta. | Conservado |
| Apertura relativa y protección de reparse | RootDirectory, nombre de un componente, OBJ_DONT_REPARSE y FILE_OPEN_REPARSE_POINT no cambian. Tampoco cambia el orden de las guardas tras adquirir el handle. | Conservado |
| Permisos y share flags | Las máscaras ReadPin/Managed, las de directorios y los flags share permanecen idénticos al baseline. | Conservado |
| Fallback create_dir legacy | `managed_files.rs:665-666` mantiene StorageUnavailable; se convierte en Fatal y después en el mismo AppError para consumidores legacy. No aplica el mapper de open a create_dir. | Conservado por lectura; no ejecutado en Linux |
| Retención de handles/pins | ManagedFile conserva parent; ManagedDirectory conserva root, handle, pin y ancestors. El resultado Available posee el ManagedDocumentReadHandle existente. | Conservado |
| Tamaño y registro | Validación previa sin cambios; tamaño registrado fuera de rango → PathNotAllowed; metadata fallida → su error; tamaño real distinto → IntegrityFailure. | Conservado |
| Panic/join | `flatten_blocking_result` recibe el resultado de spawn_blocking fuera del mapper de apertura y produce Err(StorageUnavailable). | Conservado |
| Reader legacy | open_verified delega al único prove_access y adapta sólo Unavailable: Missing→NotFound, Denied/Sharing→StorageUnavailable. Fatal se propaga con `?`. | Conservado |
| Cancelación | La closure bloqueante posee root/library_id/registered; no se sustituye el handle por una ruta ni se cambian ReaderService/permisos. Las regresiones existentes de cancelación hasta commit/rollback constan como PASS en el log. | Sin regresión identificada |

Los nombres internos difieren del bosquejo: `ManagedOpenError::{Os(ManagedOpenCause), Fatal}` sustituye `ManagedOpenFailure`, y `_for_document` sustituye `_for_proof`. La separación de causas es equivalente para seguridad; no modifica ABI wire. Los helpers privados OS cambian de Result<AppError> a Result<ManagedOpenError>, mientras los consumidores legacy convierten mediante From sin perder sus códigos existentes.

No se introduce exists() como prueba documental, reapertura absoluta de recuperación, lectura completa ni hash del PDF. La disponibilidad sigue siendo adquisición con la política Managed existente; no significa que una apertura menos restrictiva de sólo lectura fuera imposible.

## Verificación y límites

Este revisor realizó lectura/diff y comprobaciones Git; no ejecutó una nueva compilación o suite para evitar competir con la revisión Rust paralela. Leyó los logs centrales del corte y sus exit files:

- `work/task-04a/check-final2.exit`: 0. Log: 69 pruebas UI y 137 entradas Rust; Reader22, document protocol4, casos nuevos de mapper/panic/sharing y guardas Windows constan como PASS. Se conservan entradas helper/proceso hijo dentro del conteo, sin presentarlas como comportamientos independientes.
- `work/task-04a/build-debug-final2.exit`: 0. Log: compilación de `.worktrees/task-04a-document-proof/src-tauri`, build debug sin bundle terminado. Aviso Vite de chunk >500kB, no fallo.
- `git diff --check BASE..HEAD -- src-tauri`: sin salida/errores. El diff completo informa únicamente una línea vacía al final del reporte Markdown; no es un hallazgo de seguridad ni un fallo del producto.
- `git status --porcelain`: vacío antes y después de la lectura.

No se acreditan ACL reales, ejecución Linux, cancelación durante una inyección de apertura nueva, aplicación GUI, instalador o equipo limpio. Tampoco se acredita Workflow/T04b, revalidación de su referencia ni lifetime de proof hasta su commit; este corte prepara el puerto.

No se modificó producto, runner ni documentos normativos. Único archivo escrito por este revisor: este informe central. Sin commits, merges, cambios globales, aplicación ni subagentes.

**Entrega: DONE_WITH_CONCERNS — PASS de seguridad del HEAD indicado; observación de evidencia para el gate SPEC.**
