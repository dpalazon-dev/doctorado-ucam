# T02 — corrección 3/5: informe de cierre

Estado: correcciones implementadas y comprobadas, listas para rerevisión independiente.

- Rama: `agent/task-02-library`
- Worktree: `.worktrees/task-02-library`
- BASE: `21fd14ed20ae391518a7dabaaece3dc551a92fa2`
- Commit de implementación: `bfb6e3eeda4019ef3dc758cd0a83350d6ad58ea8` (`fix(library): preserve cancellation recovery evidence`)
- HEAD al registrar la implementación: `bfb6e3eeda4019ef3dc758cd0a83350d6ad58ea8`
- Alcance: F3, C1 y F7 del brief fix3. Sin cambios de DTO/IPC, esquema, migraciones, permisos ni UI.

## Cambios y evidencia por hallazgo

**F3 — auditoría de la cancelación completada durante recovery.** `mark_recovered_cancelled` ahora valida la intención PENDING ligada al request ID y finaliza la fila FAILED→DONE, el receipt y el evento de entidad `import.cancelled` dentro de la misma transacción. Una discrepancia o un fallo de auditoría revierte el estado y el receipt en conjunto; el archivo ya retirado puede quedar ausente y la reconciliación repetirse. La regresión `durably_promoted_cancel_pending_is_recovered_after_database_reopen` fuerza un fallo del trigger de auditoría, comprueba rollback y luego una recuperación exitosa; replay conserva exactamente un receipt/evento.

- Rojo de comportamiento: `work/evidence/fix3-f3-audit-red2.log`, exit 101; recuperación cerraba sin evento de entidad.
- Verde focal: `work/evidence/fix3-f3-audit-green2.log`, exit 0; un test ejecutado.

**C1 — no convertir una cancelación previa desconocida en autoridad nueva.** El validador compartido acepta solo el sobre cancellation v1 completo, PENDING, de forma exacta y ligado al estado, token, request ID, comando y payload. `prepare_cancel` valida ese sobre dentro de la transacción antes de reescribir `error_json`; para FAILED preserva la evidencia de promoción únicamente cuando la cancelación previa supera la validación y existe el ConfirmIntent durable requerido. El recovery usa el mismo validador. La prueba end-to-end parte de una promoción y cancelación real PENDING, comprueba tanto versión 99 como `receipt.payload.importToken` discordante: ambos retries devuelven `ImportRecoveryRequired`, preservan bytes y wrapper, y no publican receipt.

- Rojo de comportamiento de versión 99: `work/evidence/fix3-c1-version-red.log`, exit 101.
- Verde de versión 99: `work/evidence/fix3-c1-version-green.log`, exit 0; un test.
- Rojo de token/payload discordante: no hubo un segundo rojo separado; se añadió como subcaso del mismo test focal para validar la ampliación del criterio.
- Verde final con ambos subcasos: `work/evidence/fix3-c1-payload-mismatch-green2.log`, exit 0; `unknown_cancellation_version_is_preserved_and_cannot_authorize_cleanup`, 1 passed.

**F7 — el fallo antes de abrir el PDF no prueba que staging esté vacío.** Los retornos tempranos que antes declaraban `Done` consultan ahora la ruta gestionada declarada solo después de validar el ID canónico. Se conserva el error original; `Done` requiere ausencia comprobada, y un archivo existente o una inspección fallida producen `Pending` sin borrar el recurso desconocido. El ID inválido devuelve `Pending` sin construir ni consultar un namespace. La prueba de servicio usa la barrera `GatedDocumentStore`, introduce bytes sintéticos después de `begin_import` y antes de `stage`, y verifica FAILED/PENDING, bytes intactos e incidencia tras reabrir. El control con staging vacío sigue finalizando tras probar ausencia.

- Rojo con residuo staging: `work/evidence/fix3-f7-early-stage-red.log`, exit 101.
- Verde con residuo y reapertura: `work/evidence/fix3-f7-early-stage-green.log`, exit 0; un test.
- Rojo de ID inválido: `work/evidence/fix3-f7-invalid-id-red.log`, exit 101.
- Verde de ID inválido: `work/evidence/fix3-f7-invalid-id-green.log`, exit 0; un test.
- Control de staging vacío: `work/evidence/fix3-f7-empty-stage-green.log`, exit 0; un test.

Los primeros intentos de compilación focal también se conservaron (`fix3-f3-audit-red.log`, `fix3-f3-audit-green.log` y `fix3-c1-payload-mismatch-green.log`); no se cuentan como evidencia de comportamiento. Los reruns `red2`/`green2` son las ejecuciones conductuales indicadas arriba. Todos los logs y archivos `.exitcode` están en `work/evidence` y no se sobrescribieron.

## Gate final

Ejecutado sobre el código final después de añadir el subcaso C1:

| Comando | Resultado | Evidencia |
|---|---:|---|
| `powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts\check.ps1` | exit 0 | `work/evidence/fix3-final-check2.log`, `fix3-final-check2.exitcode` |
| `cargo build --manifest-path src-tauri\Cargo.toml` con `. .\scripts\development-env.ps1` | exit 0 | `work/evidence/fix3-native-debug-build.log`, `fix3-native-debug-build.exitcode` |
| `git diff --check` | exit 0 | `work/evidence/fix3-diff-check.log`, `fix3-diff-check.exitcode` |
| Focal C1 `cargo test --manifest-path src-tauri\Cargo.toml --test library_integration unknown_cancellation_version_is_preserved_and_cannot_authorize_cleanup -- --exact --nocapture` | exit 0, 1 passed | `work/evidence/fix3-c1-payload-mismatch-green2.log`, `.exitcode` |

El gate incluye `tsc -b`, Vitest 14/14, build Vite, Clippy y Rust. Entradas Rust por runner: unit 12; actor 8; bootstrap 13; library 53; lifecycle 3; scaffold 6; schema diagnostic 4; settings 1. Total: 100 entradas de runner, equivalentes a 99 comprobaciones de comportamiento y el helper hijo `pending_wal_fixture_child` contado una vez. Suite de library: 53/53. El log final termina con éxito y exitcode 0.

## ABI, autorrevisión y límites

Las firmas públicas Rust, el contrato wire, DTOs, commands, esquema y migraciones no cambiaron. El helper `valid_pending_cancellation(&ImportRecord) -> Option<(String, bool)>` queda en la capa application para compartir la validación normativa entre persistencia y recovery; no emite efectos por sí mismo. `stage` conserva su API; la comprobación temprana es privada al adapter de documentos y no concede autoridad por ruta.

La revisión propia confirma que los negativos preservan la intención desconocida, el archivo y la ausencia de receipt; la prueba F3 inyecta el fallo después de la mutación dentro de la transacción y demuestra rollback; F7 comprueba recuperación tras reapertura y conserva el control positivo de ausencia demostrada. Los logs iniciales de compilación fallida se excluyeron de resultados verdes en vez de reemplazarlos.

Este gate no demuestra selector GUI, NSIS, instalación en un equipo limpio, acceso Reader/T03 ni resistencia ante administradores/drivers arbitrarios o pérdida de alimentación. No se accedió a bibliotecas personales. Las pruebas usan fixtures/datos sintéticos. No quedan tareas locales de implementación o validación asignadas en fix3; queda la rerevisión independiente del delta.
