# T02 fix 4 — rerevisión focal de seguridad

Estado: **DONE / APPROVED** en el alcance revisado. SPEC focal: **APPROVED**. Calidad/seguridad: **APPROVED**. F3, C1 y F7 quedan ADDRESSED; 0 Critical, 0 Important y 0 Minor nuevos. No se identificaron regresiones directas del delta acumulado.

BASE de esta revisión `21fd14ed20ae391518a7dabaaece3dc551a92fa2`; HEAD congelado `c2b35e21f7bd2f785670b4544b741908d58fc472`, limpio según comunicación del orquestador. Checkout `.worktrees/task-02-library`. Commits de implementación: fix3 `bfb6e3eeda4019ef3dc758cd0a83350d6ad58ea8` y fix4 `1a691c2aa67bb287bb75ca7262e773cd577b8b85`. Revisor `/root/task02_security_review`.

Se leyó íntegramente `review-21fd14e..c2b35e2.diff` por porciones, el contexto fix4, informes fix3/fix4 y evidencia final existente. Se contrastaron las correcciones con la normativa ya revisada de MANAGED_FILES/C1 y los tres Important del informe de seguridad fix2. No se tomó el informe del autor como aprobación. F10/F11/F9/F13 y anteriores permanecen cerrados; T03 no forma parte de esta revisión. Única escritura: este informe; sin suites/probes nuevos, cambios de producto, Git ni subagentes.

## Veredictos por hallazgo

| Hallazgo | Veredicto | Evidencia de cierre |
|---|---|---|
| F3 — auditoría de cancellation recovery | **ADDRESSED** | `src-tauri/src/adapters/sqlite/library_repository.rs:1034` valida la intención PENDING y su requestId. La TX de with_receipt actualiza DONE solo si el error_json actual coincide con el snapshot y emite import.cancelled de entidad en `:1077`. Evento, estado y receipt comparten commit/rollback; replay no duplica el evento. |
| C1 — normalización de autoridad desconocida | **ADDRESSED** | `src-tauri/src/application/library.rs:143` acepta únicamente el wrapper conocido v1/PENDING, claves y tipos exactos, comando/payload ligados al token y requestId canónico. `library_repository.rs:729` valida el wrapper FAILED antes de reescribirlo; si promotionConfirmed es true exige además ConfirmIntent/promoción durable íntegros y ausencia de receipt de confirmación. El plan reutiliza el validador en `:834`. |
| F7 — DONE temprano sin ausencia comprobada | **ADDRESSED** | `src-tauri/src/adapters/documents/store.rs:192` deriva DONE solo de Ok(None) de inspección gestionada del staging declarado. Archivo presente o error de inspección dan PENDING; ID inválido da PENDING sin construir namespace. Los errores SourceUnreadable/InvalidInput originales se conservan y no conceden autoridad para borrar bytes desconocidos. |

**C1 y corrección fix4:** promotionConfirmed ausente se interpreta como false; si está presente debe ser booleano. No se infiere autoridad de un destino existente ni de su hash. Esto admite cancellation staging-only válida sin flag y mantiene el rechazo del destino promovido cuando falta autoridad. has_durable_promoted_confirmation usa el mismo validador para FAILED; una versión/tipo/payload desconocidos ya no pueden convertirse en un wrapper v1 autorizado mediante retry. No se debilita el control independiente de namespace, referencias y hash/tamaño del archivo.

## Regresiones y evidencia comprobadas

Se revisó la fuente de las regresiones del delta y su ejecución exitosa en el gate final:

- **F3:** `library_integration.rs:981`, durably_promoted_cancel_pending_is_recovered_after_database_reopen, inyecta un trigger que falla al auditar import.cancelled. Comprueba PENDING, cero receipts/eventos y una incidencia tras rollback; después retira el trigger, reconcilia y verifica DONE con exactamente un receipt y evento, también tras replay. La ausencia física tras el primer cleanup permite repetir la reconciliación sin inventar un éxito transaccional.
- **C1:** `library_integration.rs:1303`, unknown_cancellation_version_is_preserved_and_cannot_authorize_cleanup, parte de una promoción y cancellation real. Version99, token discordante del payload y promotionConfirmed de tipo string devuelven ImportRecoveryRequired, mantienen bytes/wrapper PENDING y no publican receipt exitoso. El código rechaza asimismo flag true sin ConfirmIntent durable válido antes de reescribir la fila.
- **C1 sin flag:** retry/replay staging-only (`:1136`) y reapertura real de DB (`:2197`) completan cleanup con un solo receipt/evento. El negativo con destino promovido y flag ausente (`:1217`) conserva bytes, PENDING y una incidencia, sin receipt. La ausencia no se transforma en permiso de eliminar destino.
- **F7:** `library_integration.rs:1857`, source_open_failure_with_unknown_stage_file_stays_pending_after_reopen, usa GatedDocumentStore para introducir bytes desconocidos después de begin_import y antes de stage. Verifica SourceUnreadable, FAILED/PENDING, conservación de bytes y una incidencia tras cerrar/reabrir SQLite. El control source_unreadable con staging realmente ausente sigue terminando en DONE. La regresión unitaria de ID inválido comprueba PENDING y ausencia de consulta/creación del namespace.

`work/evidence/fix4-final-check.log` y `.exitcode` 0 se leyeron directamente: typecheck, Vitest 14/14, build Vite, Clippy y Rust con 102 entradas de runner (101 comprobaciones de comportamiento más pending_wal_fixture_child), library 55/55, sin filtrados en las suites finales. Los casos anteriores aparecen ejecutados y verdes. Se leyeron `fix4-native-debug-build.log`/`.exitcode` 0, que muestran cargo build nativo debug completado, y `fix4-diff-check.exitcode` 0. Los informes registran el comando del gate `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts\check.ps1`; no se volvió a ejecutarlo.

## Límites y cierre

Esta aprobación cierra los tres hallazgos concretos y sus regresiones directas en el HEAD indicado. No acredita merge ni validación integrada posterior. No se revisaron WIP/futuro Reader, GUI del picker, bundle/NSIS, instalación limpia, otros filesystems/red, administradores/drivers arbitrarios ni pérdida de alimentación. No se consultaron bibliotecas personales ni memoria externa.

No quedan Important de esta revisión focal que impidan continuar al gate de integración, sujeto a los veredictos independientes SPEC/Rust y las comprobaciones del orquestador.
